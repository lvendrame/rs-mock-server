use std::{ffi::OsString, fs, io::Error, path::PathBuf, str::FromStr};

use async_graphql::{
    Error as GQLError, Request as GQLRequest, Response as GQLResponse, ServerError,
    Value as GValue,
    dynamic::{Field, FieldFuture, Object, Scalar, Schema, TypeRef},
    http::GraphiQLSource,
};
use axum::{
    extract::Json,
    routing::{get, post},
};
use fosk::{CollectionReadError, Db, IdType, JsonPrimitive};
use graphql_parser::query::{
    Definition, Document, OperationDefinition, Selection, Value as GqlValue, parse_query,
};
use serde_json;
use std::sync::Arc;

use jgd_rs::generate_jgd_from_file;

use crate::{
    app::App,
    handlers::{SleepThread, is_jgd, is_json},
    route_builder::{RouteRegistrator, route_graphql::RouteGraphQL},
};
use std::collections::{HashMap, HashSet};

/// Folder under a GraphQL route that seeds Fosk collections.
pub const COLLECTIONS_FOLDER: &str = "/collections";

/// Builds a dynamic Async-GraphQL schema from loaded Fosk collections.
pub fn build_dynamic_schema(db: &Db) -> Schema {
    struct CollectionMeta {
        raw: String,
        field: String,
        type_name: String,
    }

    fn sanitize(name: &str) -> String {
        name.chars()
            .map(|c| if c.is_alphanumeric() { c } else { '_' })
            .collect()
    }

    fn pascal_case(name: &str) -> String {
        sanitize(name)
            .split('_')
            .filter(|part| !part.is_empty())
            .map(|part| {
                let mut chars = part.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect::<String>()
    }

    fn output_field_type(primitive: &JsonPrimitive) -> TypeRef {
        match primitive {
            JsonPrimitive::String => TypeRef::named_nn("String"),
            JsonPrimitive::Int => TypeRef::named_nn("Int"),
            JsonPrimitive::Float => TypeRef::named_nn("Float"),
            JsonPrimitive::Bool => TypeRef::named_nn("Boolean"),
            _ => TypeRef::named_nn("JSON"),
        }
    }

    fn input_field_type(primitive: &JsonPrimitive) -> TypeRef {
        match primitive {
            JsonPrimitive::String => TypeRef::named("String"),
            JsonPrimitive::Int => TypeRef::named("Int"),
            JsonPrimitive::Float => TypeRef::named("Float"),
            JsonPrimitive::Bool => TypeRef::named("Boolean"),
            _ => TypeRef::named("JSON"),
        }
    }

    fn relation_fields(def: &fosk::SchemaWithRefs, raw: &str) -> HashMap<String, String> {
        let mut rel_map = HashMap::new();
        for outbound in def.outbound_refs.values() {
            if outbound.collection.as_str() == raw {
                continue;
            }
            let name = sanitize(&outbound.collection);
            let typ = pascal_case(&outbound.collection);
            rel_map.entry(name).or_insert(typ);
        }
        for inbound in def.inbound_refs.values() {
            if inbound.ref_collection.as_str() == raw {
                continue;
            }
            let name = sanitize(&inbound.ref_collection);
            let typ = pascal_case(&inbound.ref_collection);
            rel_map.entry(name).or_insert(typ);
        }
        rel_map
    }

    fn build_object(def: &fosk::SchemaWithRefs, meta: &CollectionMeta) -> Object {
        let mut obj = Object::new(meta.type_name.clone());

        for (field, info) in &def.fields {
            let field_name = field.clone();
            let gql_type = output_field_type(&info.ty);
            obj = obj.field(Field::new(field_name.clone(), gql_type, move |ctx| {
                let key = field_name.clone();
                FieldFuture::new(async move {
                    let parent = ctx
                        .parent_value
                        .try_downcast_ref::<serde_json::Value>()
                        .unwrap();
                    let val = parent.get(&key).cloned().unwrap_or(serde_json::Value::Null);
                    Ok(Some(GValue::from_json(val).unwrap_or(GValue::Null)))
                })
            }));
        }

        for (rel_name, rel_type) in relation_fields(def, &meta.raw) {
            if def.fields.contains_key(&rel_name) {
                continue;
            }
            obj = obj.field(Field::new(
                rel_name.clone(),
                TypeRef::named_nn_list_nn(&rel_type),
                move |_ctx| FieldFuture::new(async move { Ok(Some(GValue::List(Vec::new()))) }),
            ));
        }

        obj
    }

    fn build_query(collections: &[CollectionMeta]) -> Object {
        let mut query = Object::new("Query");
        let mut seen = HashSet::new();

        for meta in collections {
            if !seen.insert(meta.field.clone()) {
                continue;
            }

            let field_name = meta.field.clone();
            let coll_name = meta.raw.clone();
            let type_name = meta.type_name.clone();

            query = query.field(Field::new(
                field_name,
                TypeRef::named_nn_list_nn(&type_name),
                move |ctx| {
                    let db = ctx.data::<Arc<Db>>().unwrap().clone();
                    let coll_name = coll_name.clone();
                    FieldFuture::new(async move {
                        let coll = db.get(&coll_name).unwrap();
                        let items: Vec<GValue> = coll
                            .get_all()
                            .map_err(|err| GQLError::new(err.to_string()))?
                            .into_iter()
                            .map(|item| GValue::from_json(item).unwrap_or(GValue::Null))
                            .collect();
                        Ok(Some(GValue::List(items)))
                    })
                },
            ));
        }

        query
    }

    fn id_input_type(def: &fosk::SchemaWithRefs, id_key: &str) -> TypeRef {
        def.fields
            .get(id_key)
            .map(|info| match info.ty {
                JsonPrimitive::Int => TypeRef::named_nn("Int"),
                _ => TypeRef::named_nn("String"),
            })
            .unwrap_or_else(|| TypeRef::named_nn("String"))
    }

    fn build_create_field(
        type_name: &str,
        def: &fosk::SchemaWithRefs,
        id_key: &str,
        id_type: IdType,
    ) -> Field {
        let field_name = format!("create{}", type_name);
        let mut field = Field::new(field_name, TypeRef::named_nn(type_name), move |_ctx| {
            FieldFuture::new(async move { Ok::<_, GQLError>(Some(GValue::Null)) })
        });

        if id_type == IdType::None {
            field = field.argument(async_graphql::dynamic::InputValue::new(
                id_key,
                id_input_type(def, id_key),
            ));
        }

        for (f_name, info) in &def.fields {
            if f_name == id_key {
                continue;
            }
            field = field.argument(async_graphql::dynamic::InputValue::new(
                f_name,
                input_field_type(&info.ty),
            ));
        }

        field
    }

    fn build_update_field(type_name: &str, def: &fosk::SchemaWithRefs, id_key: &str) -> Field {
        let field_name = format!("update{}", type_name);
        let mut field = Field::new(field_name, TypeRef::named_nn(type_name), move |_ctx| {
            FieldFuture::new(async move { Ok::<_, GQLError>(Some(GValue::Null)) })
        });

        field = field.argument(async_graphql::dynamic::InputValue::new(
            id_key,
            id_input_type(def, id_key),
        ));

        for (f_name, info) in &def.fields {
            if f_name == id_key {
                continue;
            }
            field = field.argument(async_graphql::dynamic::InputValue::new(
                f_name,
                input_field_type(&info.ty),
            ));
        }

        field
    }

    fn build_delete_field(type_name: &str, def: &fosk::SchemaWithRefs, id_key: &str) -> Field {
        let field_name = format!("delete{}", type_name);
        Field::new(field_name, TypeRef::named_nn("Boolean"), move |_ctx| {
            FieldFuture::new(async move { Ok::<_, GQLError>(Some(GValue::Boolean(false))) })
        })
        .argument(async_graphql::dynamic::InputValue::new(
            id_key,
            id_input_type(def, id_key),
        ))
    }

    let mut schema = Schema::build("Query", Some("Mutation"), None);
    schema = schema.register(async_graphql::dynamic::Type::Scalar(Scalar::new("JSON")));

    let mut collections = Vec::new();
    for raw in db.list_collections() {
        if let Some(def) = db.schema_with_refs_of(&raw) {
            let meta = CollectionMeta {
                raw: raw.clone(),
                field: sanitize(&raw),
                type_name: pascal_case(&raw),
            };
            let object = build_object(&def, &meta);
            schema = schema.register(object);
            collections.push(meta);
        }
    }

    schema = schema.register(build_query(&collections));

    let mut mutation = Object::new("Mutation");
    for meta in &collections {
        if let Some(def) = db.schema_with_refs_of(&meta.raw)
            && let Some(coll) = db.get(&meta.raw)
            && let Ok(config) = coll.get_config()
        {
            let id_key = config.id_key.clone();
            mutation = mutation.field(build_create_field(
                &meta.type_name,
                &def,
                &id_key,
                config.id_type,
            ));
            mutation = mutation.field(build_update_field(&meta.type_name, &def, &id_key));
            mutation = mutation.field(build_delete_field(&meta.type_name, &def, &id_key));
        }
    }

    schema = schema.register(mutation);
    schema.finish().unwrap()
}

/// Registers the GraphiQL IDE route.
pub fn create_graphiql_route(app: &mut App) {
    // Serve GraphiQL IDE
    let router =
        get(async || axum::response::Html(GraphiQLSource::build().endpoint("/graphql").finish()));
    app.push_route("/graphiql", router, None, false, None);
}

/// Attempt to load static operation data from .json or .jgd file
fn load_static_data(base_path: &OsString, op_name: &str) -> Option<serde_json::Value> {
    let file_path = PathBuf::from(base_path);
    let json_file = file_path.join(format!("{}.json", op_name));
    if json_file.exists() {
        let data_str = fs::read_to_string(&json_file).unwrap_or_default();
        let data_json = serde_json::from_str(&data_str).unwrap_or(serde_json::Value::Null);
        return Some(data_json);
    }
    let jgd_file = file_path.join(format!("{}.jgd", op_name));
    if jgd_file.exists() {
        let data_json = generate_jgd_from_file(&jgd_file).unwrap_or(serde_json::Value::Null);
        return Some(data_json);
    }
    None
}

/// Build a GraphQL JSON response from serde_json::Value
fn response_from_json(data_json: serde_json::Value) -> Json<GQLResponse> {
    let mut response = GQLResponse::default();
    response.data =
        async_graphql::Value::from_json(data_json).unwrap_or(async_graphql::Value::Null);
    Json(response)
}

/// Parse the raw GraphQL request into an AST document
fn parse_request_ast(req: &GQLRequest) -> Result<Document<'_, String>, GQLError> {
    parse_query::<String>(&req.query)
        .map_err(|e| GQLError::new(format!("GraphQL syntax error: {}", e)))
}

/// Validate that all referenced collections exist in the Fosk DB
fn validate_request_ast(doc: &Document<String>, db: &Db) -> Result<(), GQLError> {
    for def in &doc.definitions {
        if let Definition::Operation(OperationDefinition::Query(q)) = def {
            for sel in &q.selection_set.items {
                if let Selection::Field(f) = sel {
                    let name = f.name.as_str();
                    // Skip introspection fields
                    if name.starts_with("__") {
                        continue;
                    }
                    // Check if collection exists
                    if db.get(name).is_none() {
                        return Err(GQLError::new(format!("Unknown collection '{}'", name)));
                    }
                }
            }
        } else if let Definition::Operation(OperationDefinition::Mutation(m)) = def {
            for sel in &m.selection_set.items {
                if let Selection::Field(f) = sel {
                    let name = f.name.as_str();
                    let coll = if let Some(c) = name.strip_prefix("create") {
                        c
                    } else if let Some(c) = name.strip_prefix("update") {
                        c
                    } else if let Some(c) = name.strip_prefix("delete") {
                        c
                    } else {
                        continue;
                    };
                    // Check if collection exists
                    if db.get(coll).is_none() {
                        return Err(GQLError::new(format!("Unknown collection '{}'", coll)));
                    }
                }
            }
        }
    }
    Ok(())
}

// Helper to collect expansion paths for nested selections
fn collect_expansion_paths(
    selection_set: &graphql_parser::query::SelectionSet<String>,
    prefix: &str,
    paths: &mut Vec<String>,
) {
    for sel in &selection_set.items {
        if let Selection::Field(f) = sel {
            // Only process if this field has nested selections
            if !f.selection_set.items.is_empty() {
                // Build base path for this field
                let base = if prefix.is_empty() {
                    f.name.clone()
                } else {
                    format!("{}.{}", prefix, f.name)
                };

                for child_sel in &f.selection_set.items {
                    if let Selection::Field(child_f) = child_sel {
                        // Only expand further if child has its own nested selections
                        if !child_f.selection_set.items.is_empty() {
                            let path = format!("{}.{}", base, child_f.name);
                            paths.push(path.clone());
                            // Recurse deeper
                            collect_expansion_paths(&child_f.selection_set, &path, paths);
                        }
                    }
                }
            }
        }
    }
}

fn expansion_paths(selection_set: &graphql_parser::query::SelectionSet<String>) -> Vec<String> {
    let mut paths = Vec::new();
    collect_expansion_paths(selection_set, "", &mut paths);
    paths
}

fn expand_list_with_selection(
    collection: &Arc<fosk::DbCollection>,
    items: Vec<serde_json::Value>,
    selection_set: &graphql_parser::query::SelectionSet<String>,
    db: &Db,
) -> Result<Vec<serde_json::Value>, CollectionReadError> {
    let mut expanded_items = items;
    for path in expansion_paths(selection_set) {
        expanded_items = collection.expand_list(expanded_items, &path, db)?;
    }

    Ok(expanded_items
        .into_iter()
        .map(|item| filter_value(item, selection_set))
        .collect())
}

fn expand_row_with_selection(
    collection: &Arc<fosk::DbCollection>,
    item: serde_json::Value,
    selection_set: &graphql_parser::query::SelectionSet<String>,
    db: &Db,
) -> Result<serde_json::Value, CollectionReadError> {
    let mut expanded_item = item;
    for path in expansion_paths(selection_set) {
        expanded_item = collection.expand_row(&expanded_item, &path, db)?;
    }

    Ok(filter_value(expanded_item, selection_set))
}

// Helper to filter JSON values based on selection set
fn filter_value(
    value: serde_json::Value,
    selection_set: &graphql_parser::query::SelectionSet<String>,
) -> serde_json::Value {
    match value {
        serde_json::Value::Object(mut map) => {
            let mut new_map = serde_json::Map::new();
            for sel in &selection_set.items {
                if let Selection::Field(f) = sel {
                    let key = f.name.as_str();
                    if let Some(val) = map.remove(key) {
                        let filtered_val = if !f.selection_set.items.is_empty() {
                            filter_value(val, &f.selection_set)
                        } else {
                            val
                        };
                        new_map.insert(key.to_string(), filtered_val);
                    }
                }
            }
            serde_json::Value::Object(new_map)
        }
        serde_json::Value::Array(arr) => serde_json::Value::Array(
            arr.into_iter()
                .map(|elem| filter_value(elem, selection_set))
                .collect(),
        ),
        _ => value,
    }
}

// Convert GraphQL parser values into serde_json via JSON parsing fallback
fn graphql_value_to_json(val: &GqlValue<String>) -> serde_json::Value {
    let s = val.to_string();
    serde_json::from_str(&s).unwrap_or_else(|_| serde_json::Value::String(s))
}

// Updated execute_query to respect GraphQL arguments for filtering
fn execute_query(
    db: &Db,
    result: &mut serde_json::Map<String, serde_json::Value>,
    query: &graphql_parser::query::Query<'_, String>,
) -> Result<(), CollectionReadError> {
    fn should_skip_field(name: &str) -> bool {
        name.starts_with("__")
    }

    fn fetch_collection_items(
        db: &Db,
        collection: &Arc<fosk::DbCollection>,
        field_name: &str,
        field: &graphql_parser::query::Field<'_, String>,
    ) -> Result<Vec<serde_json::Value>, CollectionReadError> {
        if field.arguments.is_empty() {
            return collection.get_all();
        }

        let id_key = collection.get_config()?.id_key;
        if field.arguments.len() == 1 && field.arguments[0].0 == id_key {
            let arg_val = graphql_value_to_json(&field.arguments[0].1);
            if let Some(item) = collection.get(arg_val.as_str().unwrap_or(""))? {
                return Ok(vec![item]);
            }
            return Ok(Vec::new());
        }

        let mut clauses = Vec::new();
        let mut args_json = Vec::new();
        for (name, val) in &field.arguments {
            clauses.push(format!("{} = ?", name));
            args_json.push(graphql_value_to_json(val));
        }

        let sql = format!(
            "SELECT * FROM {} WHERE {}",
            field_name,
            clauses.join(" AND ")
        );
        Ok(db
            .query_with_args(&sql, serde_json::Value::Array(args_json))
            .unwrap_or_default())
    }

    for sel in &query.selection_set.items {
        if let Selection::Field(field) = sel {
            if should_skip_field(field.name.as_str()) {
                continue;
            }

            let field_name = field.name.as_str();
            let value = match db.get(field_name) {
                Some(collection) => {
                    let items = fetch_collection_items(db, &collection, field_name, field)?;
                    let filtered =
                        expand_list_with_selection(&collection, items, &field.selection_set, db)?;
                    serde_json::Value::Array(filtered)
                }
                None => serde_json::Value::Null,
            };

            result.insert(field.name.clone(), value);
        }
    }

    Ok(())
}

fn execute_operation(
    db: &Db,
    result: &mut serde_json::Map<String, serde_json::Value>,
    errors: &mut Vec<String>,
    mutation: &graphql_parser::query::Mutation<'_, String>,
) {
    fn json_value_to_id(value: serde_json::Value) -> Option<String> {
        match value {
            serde_json::Value::Number(number) => number.as_u64().map(|n| n.to_string()),
            serde_json::Value::String(text) => Some(text),
            _ => None,
        }
    }

    fn handle_create(
        db: &Db,
        collection_name: &str,
        field: &graphql_parser::query::Field<'_, String>,
    ) -> Result<serde_json::Value, String> {
        if let Some(collection) = db.get(collection_name) {
            let mut new_map = serde_json::Map::new();
            for (arg_name, arg_val) in &field.arguments {
                new_map.insert(arg_name.clone(), graphql_value_to_json(arg_val));
            }
            let new_item = serde_json::Value::Object(new_map);
            let created = collection.add(new_item).map_err(|err| err.to_string())?;
            expand_row_with_selection(&collection, created, &field.selection_set, db)
                .map_err(|err| err.to_string())
        } else {
            Ok(serde_json::Value::Null)
        }
    }

    fn handle_update(
        db: &Db,
        collection_name: &str,
        field: &graphql_parser::query::Field<'_, String>,
    ) -> Result<serde_json::Value, String> {
        if let Some(collection) = db.get(collection_name) {
            let id_key = collection
                .get_config()
                .map_err(|err| err.to_string())?
                .id_key;
            let mut id_value = None;
            let mut update_map = serde_json::Map::new();
            for (arg_name, arg_val) in &field.arguments {
                let json_val = graphql_value_to_json(arg_val);
                if arg_name == &id_key {
                    id_value = json_value_to_id(json_val);
                } else {
                    update_map.insert(arg_name.clone(), json_val);
                }
            }

            if let Some(id) = id_value {
                let partial = serde_json::Value::Object(update_map);
                Ok(collection
                    .update_partial(&id, partial)
                    .map_err(|err| err.to_string())?
                    .unwrap_or(serde_json::Value::Null))
            } else {
                Ok(serde_json::Value::Null)
            }
        } else {
            Ok(serde_json::Value::Null)
        }
    }

    fn handle_delete(
        db: &Db,
        collection_name: &str,
        field: &graphql_parser::query::Field<'_, String>,
    ) -> Result<serde_json::Value, String> {
        if let Some(collection) = db.get(collection_name) {
            let id_key = collection
                .get_config()
                .map_err(|err| err.to_string())?
                .id_key;
            let id_value = field
                .arguments
                .iter()
                .find(|(name, _)| name == &id_key)
                .and_then(|(_, val)| json_value_to_id(graphql_value_to_json(val)));

            if let Some(id) = id_value {
                match collection.delete(&id).map_err(|err| err.to_string())? {
                    Some(deleted) => {
                        expand_row_with_selection(&collection, deleted, &field.selection_set, db)
                            .map_err(|err| err.to_string())
                    }
                    None => Ok(serde_json::Value::Null),
                }
            } else {
                Ok(serde_json::Value::Null)
            }
        } else {
            Ok(serde_json::Value::Null)
        }
    }

    for sel in &mutation.selection_set.items {
        if let Selection::Field(field) = sel {
            let field_name = field.name.as_str();
            let outcome = if let Some(collection_name) = field_name.strip_prefix("create") {
                handle_create(db, collection_name, field)
            } else if let Some(collection_name) = field_name.strip_prefix("update") {
                handle_update(db, collection_name, field)
            } else if let Some(collection_name) = field_name.strip_prefix("delete") {
                handle_delete(db, collection_name, field)
            } else {
                Ok(serde_json::Value::Null)
            };

            let value = match outcome {
                Ok(value) => value,
                Err(message) => {
                    errors.push(message);
                    serde_json::Value::Null
                }
            };

            result.insert(field.name.clone(), value);
        }
    }
}

/// Execute GraphQL operations directly on Fosk database
async fn execute_graphql_operations(
    doc: &Document<'_, String>,
    db: &Db,
) -> Result<(serde_json::Value, Vec<String>), String> {
    let mut result = serde_json::Map::new();
    let mut errors = Vec::new();

    for def in &doc.definitions {
        match def {
            Definition::Operation(OperationDefinition::Query(q)) => {
                execute_query(db, &mut result, q).map_err(|err| err.to_string())?;
            }
            Definition::Operation(OperationDefinition::Mutation(m)) => {
                execute_operation(db, &mut result, &mut errors, m);
            }
            _ => {}
        }
    }

    Ok((serde_json::Value::Object(result), errors))
}

// -------------------------------------------------------------------------------

/// Registers the GraphQL endpoint for dynamic collection queries and mutations.
pub fn create_graphql_route(
    app: &mut App,
    route: &str,
    path: OsString,
    is_protected: bool,
    delay: Option<u16>,
) {
    // Prepare dynamic schema for introspection
    let db = app.db.clone();
    // Build and store dynamic schema for GraphiQL introspection
    // build_dynamic_schema already returns a finished Schema
    let router = post(move |Json(req): Json<GQLRequest>| {
        let db = db.clone();
        async move {
            // Introspection queries (__schema or __type)
            let query_str = req.query.clone();
            if query_str.contains("__schema") || query_str.contains("__type") {
                // Build a fresh request for introspection and attach DB
                let int_req = async_graphql::Request::new(query_str).data(db.clone());
                let dyn_schema = build_dynamic_schema(&db);
                let resp = dyn_schema.execute(int_req).await;
                return Json(resp);
            }

            delay.sleep_thread();

            // 1) Parse request into AST
            let doc = match parse_request_ast(&req) {
                Err(err) => {
                    let mut response = GQLResponse::default();
                    response.errors = vec![ServerError::new(err.message, None)];
                    return Json(response);
                }
                Ok(d) => d,
            };
            // 2) Static operation override: return matching .json or .jgd file if present
            if let Some(op_name) = doc
                .definitions
                .iter()
                .filter_map(|def| {
                    if let Definition::Operation(OperationDefinition::Query(q)) = def {
                        q.name.clone()
                    } else if let Definition::Operation(OperationDefinition::Mutation(m)) = def {
                        m.name.clone()
                    } else {
                        None
                    }
                })
                .next()
                && let Some(data_json) = load_static_data(&path, &op_name)
            {
                return response_from_json(data_json);
            }

            // 3) Validate referenced collections exist in Fosk database
            if let Err(err) = validate_request_ast(&doc, &db) {
                let mut response = GQLResponse::default();
                response.errors = vec![ServerError::new(err.message, None)];
                return Json(response);
            }

            // Execute GraphQL operations directly on Fosk database
            let result = execute_graphql_operations(&doc, &db).await;

            // Return GraphQL response
            let mut response = GQLResponse::default();
            match result {
                Ok((data, mutation_errors)) => {
                    response.data =
                        async_graphql::Value::from_json(data).unwrap_or(async_graphql::Value::Null);
                    if !mutation_errors.is_empty() {
                        response.errors = mutation_errors
                            .into_iter()
                            .map(|message| ServerError::new(message, None))
                            .collect();
                    }
                }
                Err(err) => {
                    response.errors = vec![ServerError::new(err, None)];
                }
            }
            Json(response)
        }
    });
    app.push_route(route, router, Some("POST"), is_protected, None);
}

/// Loads JSON and JGD collection seed files from a GraphQL `collections` folder.
pub fn load_folder_collections(app: &mut App, path: OsString) -> Result<(), Error> {
    let mut path = path.clone();
    path.push(COLLECTIONS_FOLDER);

    if !fs::exists(&path)? {
        println!("Folder Collections doesn't exist for GraphQL routes");
        return Ok(());
    }

    fs::read_dir(path)?
        .filter_map(Result::ok)
        .filter(|file| is_jgd(&file.file_name()) || is_json(&file.file_name()))
        .for_each(|file| {
            let binding = file.path();
            let name = binding.file_stem().unwrap().to_string_lossy();
            let collection = app.db.create(&name);

            if is_jgd(&file.file_name()) {
                match generate_jgd_from_file(
                    &PathBuf::from_str(file.path().to_str().unwrap()).unwrap(),
                ) {
                    Ok(jgd_json) => {
                        let value = collection.load_from_json(jgd_json, false);
                        match value {
                            Ok(items) => {
                                println!(
                                    "✔️ Loaded collection {} with {} initial items from {}",
                                    name,
                                    items.len(),
                                    binding.to_string_lossy()
                                );
                            }
                            Err(error) => println!(
                                "Error to load JSON for file {}. Details: {}",
                                binding.to_string_lossy(),
                                error
                            ),
                        }
                    }
                    Err(error) => println!(
                        "Error to generate JGD JSON for file {}. Details: {}",
                        binding.to_string_lossy(),
                        error
                    ),
                }
            } else {
                let result = collection.load_from_file(&binding.as_os_str().to_os_string());
                match result {
                    Ok(value) => println!("{}", value),
                    Err(error) => println!("{}", error),
                }
            }
        });

    Ok(())
}

/// Loads GraphQL seed collections and registers GraphQL plus GraphiQL routes.
pub fn build_graphql_routes(app: &mut App, config: &RouteGraphQL) {
    let result = load_folder_collections(app, config.path.clone());
    if let Err(error) = result {
        println!("Error to load GraphQL collections. Details: {}", error);
    }

    let route = &config.route;
    let is_protected = config.is_protected;
    let delay = config.delay;
    let path = config.path.clone();

    create_graphiql_route(app);
    create_graphql_route(app, route, path, is_protected, delay);
}

// Unit tests for GraphQL helper functions
#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::{Method, Request, header::CONTENT_TYPE},
    };
    use fosk::DbConfig;
    use graphql_parser::parse_query;
    use graphql_parser::query::{Definition, OperationDefinition};
    use serde_json::{Value, json};
    use tower::ServiceExt;

    fn graphql_request(query: &str) -> Request<Body> {
        Request::builder()
            .method(Method::POST)
            .uri("/graphql")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(json!({ "query": query }).to_string()))
            .unwrap()
    }

    async fn response_json(response: axum::response::Response) -> Value {
        serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap()).unwrap()
    }

    #[test]
    fn test_collect_expansion_paths_only_full_paths() {
        // GraphQL with nested selections: order_items -> products
        let doc = parse_query::<String>("query { orders { order_items { products { id } } } }")
            .expect("Failed to parse query");
        // Extract the query definition
        let mut paths = Vec::new();
        if let Definition::Operation(OperationDefinition::Query(q)) = &doc.definitions[0] {
            // SelectionSet of 'orders'
            // First level under orders
            if let super::Selection::Field(f_orders) = &q.selection_set.items[0] {
                collect_expansion_paths(&f_orders.selection_set, "", &mut paths);
            }
        }
        assert_eq!(
            paths,
            vec!["order_items.products"],
            "collect_expansion_paths should only include full nested paths"
        );
    }

    #[tokio::test]
    async fn graphql_routes_load_collections_and_execute_queries() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let collections = temp_dir.path().join("collections");
        std::fs::create_dir(&collections).unwrap();
        std::fs::write(
            collections.join("Users.json"),
            r#"[{"id":"1","name":"Ada","active":true,"score":1.5,"age":42,"ignored":"x"}]"#,
        )
        .unwrap();

        let mut app = App::default();
        let config = RouteGraphQL::new(
            temp_dir.path().as_os_str().to_os_string(),
            "/graphql".to_string(),
            false,
            None,
        );
        build_graphql_routes(&mut app, &config);

        let router = app.take_router_for_test();
        let graphiql = router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/graphiql")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(graphiql.status(), http::StatusCode::OK);

        let query = router
            .clone()
            .oneshot(graphql_request(
                r#"query { Users { id name active score age } }"#,
            ))
            .await
            .unwrap();
        assert_eq!(query.status(), http::StatusCode::OK);
        let body = response_json(query).await;
        assert_eq!(body["data"]["Users"][0]["name"], "Ada");
        assert!(body["data"]["Users"][0].get("ignored").is_none());

        let by_id = router
            .clone()
            .oneshot(graphql_request(r#"query { Users(id: "1") { id name } }"#))
            .await
            .unwrap();
        assert_eq!(response_json(by_id).await["data"]["Users"][0]["id"], "1");

        let filtered = router
            .clone()
            .oneshot(graphql_request(
                r#"query { Users(name: "Ada") { id name } }"#,
            ))
            .await
            .unwrap();
        assert_eq!(
            response_json(filtered).await["data"]["Users"][0]["name"],
            "Ada"
        );

        let unknown = router
            .clone()
            .oneshot(graphql_request(r#"query { Missing { id } }"#))
            .await
            .unwrap();
        assert!(
            response_json(unknown).await["errors"][0]["message"]
                .as_str()
                .unwrap()
                .contains("Unknown collection")
        );

        let invalid = router
            .clone()
            .oneshot(graphql_request("query {"))
            .await
            .unwrap();
        assert!(
            response_json(invalid).await["errors"][0]["message"]
                .as_str()
                .unwrap()
                .contains("GraphQL syntax error")
        );

        let introspection = router
            .oneshot(graphql_request(
                r#"query { __schema { queryType { name } } }"#,
            ))
            .await
            .unwrap();
        assert_eq!(
            response_json(introspection).await["data"]["__schema"]["queryType"]["name"],
            "Query"
        );
    }

    #[tokio::test]
    async fn graphql_static_operations_and_mutations_are_supported() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let collections = temp_dir.path().join("collections");
        std::fs::create_dir(&collections).unwrap();
        std::fs::write(
            collections.join("Users.json"),
            r#"[{"id":"1","name":"Ada"}]"#,
        )
        .unwrap();
        std::fs::write(
            temp_dir.path().join("StaticUsers.json"),
            r#"{"users":[{"id":"static"}]}"#,
        )
        .unwrap();

        let mut app = App::default();
        let config = RouteGraphQL::new(
            temp_dir.path().as_os_str().to_os_string(),
            "/graphql".to_string(),
            false,
            None,
        );
        build_graphql_routes(&mut app, &config);
        let router = app.take_router_for_test();

        let static_response = router
            .clone()
            .oneshot(graphql_request(r#"query StaticUsers { Missing { id } }"#))
            .await
            .unwrap();
        assert_eq!(
            response_json(static_response).await["data"]["users"][0]["id"],
            "static"
        );

        let created = router
            .clone()
            .oneshot(graphql_request(
                r#"mutation { createUsers(id: "2", name: "Grace") { id name } }"#,
            ))
            .await
            .unwrap();
        let created_body = response_json(created).await;
        assert_eq!(created_body["data"]["createUsers"]["name"], "Grace");
        let created_id = created_body["data"]["createUsers"]["id"].as_str().unwrap();

        let updated = router
            .clone()
            .oneshot(graphql_request(&format!(
                r#"mutation {{ updateUsers(id: "{created_id}", name: "Hopper") {{ id name }} }}"#
            )))
            .await
            .unwrap();
        assert_eq!(
            response_json(updated).await["data"]["updateUsers"]["name"],
            "Hopper"
        );

        let deleted = router
            .clone()
            .oneshot(graphql_request(&format!(
                r#"mutation {{ deleteUsers(id: "{created_id}") {{ id name }} }}"#
            )))
            .await
            .unwrap();
        assert_eq!(
            response_json(deleted).await["data"]["deleteUsers"]["id"],
            created_id
        );

        let missing_update_id = router
            .clone()
            .oneshot(graphql_request(
                r#"mutation { updateUsers(name: "Nobody") { id name } }"#,
            ))
            .await
            .unwrap();
        assert!(response_json(missing_update_id).await["data"]["updateUsers"].is_null());

        let unknown_mutation = router
            .oneshot(graphql_request(
                r#"mutation { createMissing(id: "1") { id } }"#,
            ))
            .await
            .unwrap();
        assert!(
            response_json(unknown_mutation).await["errors"][0]["message"]
                .as_str()
                .unwrap()
                .contains("Unknown collection")
        );
    }

    #[tokio::test]
    async fn graphql_create_mutation_surfaces_duplicate_and_missing_id_errors() {
        let mut app = App::default();
        let collection = app.db.create_with_config("items", DbConfig::none("id"));
        collection.add(json!({"id": "1", "name": "First"})).unwrap();

        let temp_dir = tempfile::TempDir::new().unwrap();
        create_graphiql_route(&mut app);
        create_graphql_route(
            &mut app,
            "/graphql",
            temp_dir.path().as_os_str().to_os_string(),
            false,
            None,
        );
        let router = app.take_router_for_test();

        let duplicate = router
            .clone()
            .oneshot(graphql_request(
                r#"mutation { createItems(id: "1", name: "Second") { id } }"#,
            ))
            .await
            .unwrap();
        let duplicate_body = response_json(duplicate).await;
        assert!(duplicate_body["data"]["createItems"].is_null());
        assert!(
            duplicate_body["errors"][0]["message"]
                .as_str()
                .unwrap()
                .contains("duplicate collection id '1'")
        );

        let missing_id = router
            .oneshot(graphql_request(
                r#"mutation { createItems(name: "NoId") { id } }"#,
            ))
            .await
            .unwrap();
        let missing_id_body = response_json(missing_id).await;
        assert!(missing_id_body["data"]["createItems"].is_null());
        assert!(
            missing_id_body["errors"][0]["message"]
                .as_str()
                .unwrap()
                .contains("missing required id field 'id'")
        );
    }

    #[test]
    fn graphql_helpers_handle_static_data_and_value_conversion() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("Known.json"), r#"{"ok":true}"#).unwrap();

        assert_eq!(
            load_static_data(&temp_dir.path().as_os_str().to_os_string(), "Known").unwrap()["ok"],
            true
        );
        assert!(load_static_data(&temp_dir.path().as_os_str().to_os_string(), "Missing").is_none());

        let req = GQLRequest::new("query { __schema { queryType { name } } }");
        let doc = parse_request_ast(&req).unwrap();
        let db = Db::new_arc();
        assert!(validate_request_ast(&doc, &db).is_ok());

        let gql_value = GqlValue::String("text".to_string());
        assert_eq!(
            graphql_value_to_json(&gql_value),
            Value::String("text".to_string())
        );
    }
}
