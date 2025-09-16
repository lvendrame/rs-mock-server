use std::{ffi::OsString, fs, io::Error, path::PathBuf, str::FromStr};

use async_graphql::{
    Error as GQLError, Request as GQLRequest, Response as GQLResponse, ServerError,
    http::GraphiQLSource,
};
use axum::{
    extract::Json,
    routing::{get, post},
};
use fosk::Db;
use graphql_parser::query::{Definition, Document, OperationDefinition, Selection, parse_query, Value as GqlValue};
use serde_json;

use jgd_rs::generate_jgd_from_file;

use crate::{
    app::App,
    handlers::{SleepThread, is_jgd, is_json},
    route_builder::{RouteRegistrator, route_graphql::RouteGraphQL},
};

pub fn create_graphiql_route(app: &mut App) {
    let router =
        get(async || axum::response::Html(GraphiQLSource::build().endpoint("/graphql").finish()));
    app.push_route("/graphiql", router, None, false, None);
}
// -- GraphQL handler helpers -------------------------------------------------

/// Parse the raw GraphQL request into an AST document
fn parse_request_ast(req: &GQLRequest) -> Result<Document<String>, GQLError> {
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
fn collect_expansion_paths(selection_set: &graphql_parser::query::SelectionSet<String>, prefix: &str, paths: &mut Vec<String>) {
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

// Helper to filter JSON values based on selection set
fn filter_value(value: serde_json::Value, selection_set: &graphql_parser::query::SelectionSet<String>) -> serde_json::Value {
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
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(
                arr.into_iter().map(|elem| filter_value(elem, selection_set)).collect()
            )
        }
        _ => value,
    }
}

// Convert GraphQL parser values into serde_json via JSON parsing fallback
fn graphql_value_to_json(val: &GqlValue<String>) -> serde_json::Value {
    let s = val.to_string();
    serde_json::from_str(&s).unwrap_or_else(|_| serde_json::Value::String(s))
}

// Updated execute_query to respect GraphQL arguments for filtering
 fn execute_query(db: &Db, result: &mut serde_json::Map<String, serde_json::Value>, q: &graphql_parser::query::Query<'_, String>) {
     for sel in &q.selection_set.items {
         if let Selection::Field(f) = sel {
             let field_name = f.name.as_str();

             // Skip introspection fields
             if field_name.starts_with("__") {
                 continue;
             }

             // Execute query on collection
             if let Some(collection) = db.get(field_name) {
                 // Retrieve items based on GraphQL arguments
                 // Default id key, adjust if custom
                 let id_key = collection.get_config().id_key;
                 let mut items = if f.arguments.is_empty() {
                     collection.get_all()
                } else if f.arguments.len() == 1 && f.arguments[0].0 == id_key {
                    // Single ID lookup
                    let arg_val = graphql_value_to_json(&f.arguments[0].1);
                    if let Some(item) = collection.get(arg_val.as_str().unwrap_or("")) {
                         vec![item]
                     } else {
                         vec![]
                     }
                 } else {
                     // Build simple SQL for multiple args or non-id key arg
                     let mut clauses = Vec::new();
                     let mut args_json = Vec::new();
                     for (name, val) in &f.arguments {
                         clauses.push(format!("{} = ?", name));
                         args_json.push(graphql_value_to_json(val));
                     }
                     let sql = format!("SELECT * FROM {} WHERE {}", field_name, clauses.join(" AND "));
                     db.query_with_args(&sql, serde_json::Value::Array(args_json))
                         .unwrap_or_default()
                 };

                 // Expand nested lists
                 let mut paths = Vec::new();
                 collect_expansion_paths(&f.selection_set, "", &mut paths);
                 for path in paths {
                     items = collection.expand_list(items, &path, db);
                 }

                 // Filter fields based on selection
                 let filtered: Vec<serde_json::Value> = items.into_iter().map(|item| {
                     filter_value(item, &f.selection_set)
                 }).collect();

                 result.insert(field_name.to_string(), serde_json::Value::Array(filtered));
             } else {
                 result.insert(field_name.to_string(), serde_json::Value::Null);
             }
         }
     }
}

fn execute_operation(db: &Db, result: &mut serde_json::Map<String, serde_json::Value>, m: &graphql_parser::query::Mutation<'_, String>) {
    for sel in &m.selection_set.items {
        if let Selection::Field(f) = sel {
            let field_name = f.name.as_str();

            // Handle create mutations
            if let Some(collection_name) = field_name.strip_prefix("create") {
                if let Some(collection) = db.get(collection_name) {
                    // Build new item from GraphQL arguments
                    let mut new_map = serde_json::Map::new();
                    for (arg_name, arg_val) in &f.arguments {
                        new_map.insert(arg_name.clone(), graphql_value_to_json(arg_val));
                    }
                    let new_item = serde_json::Value::Object(new_map.clone());
                    // Add to collection (auto-generates id if configured)
                    let created = collection.add(new_item.clone()).unwrap_or(new_item.clone());
                    // Expand nested selections
                    let mut item = created;
                    let mut paths = Vec::new();
                    collect_expansion_paths(&f.selection_set, "", &mut paths);
                    for path in paths {
                        item = collection.expand_row(&item, &path, db);
                    }
                    // Filter fields
                    let filtered = filter_value(item, &f.selection_set);
                    result.insert(field_name.to_string(), filtered);
                } else {
                    result.insert(field_name.to_string(), serde_json::Value::Null);
                }
            }
            // Handle update mutations
            else if let Some(collection_name) = field_name.strip_prefix("update") {
                if let Some(collection) = db.get(collection_name) {
                    // Determine id key and parse arguments
                    let id_key = collection.get_config().id_key;
                    let mut id_val = None;
                    let mut update_map = serde_json::Map::new();
                    for (arg_name, arg_val) in &f.arguments {
                        let json_val = graphql_value_to_json(arg_val);
                        if arg_name == &id_key {
                            id_val = json_val.as_str().map(String::from);
                        } else {
                            update_map.insert(arg_name.clone(), json_val);
                        }
                    }
                    // Perform update if id provided
                    if let Some(id) = id_val {
                        let partial = serde_json::Value::Object(update_map.clone());
                        let updated = collection.update_partial(&id, partial).unwrap_or(serde_json::Value::Null);
                        // Expand nested selections
                        let mut item = updated;
                        let mut paths = Vec::new();
                        collect_expansion_paths(&f.selection_set, "", &mut paths);
                        for path in paths {
                            item = collection.expand_row(&item, &path, db);
                        }
                        // Filter fields
                        let filtered = filter_value(item, &f.selection_set);
                        result.insert(field_name.to_string(), filtered);
                    } else {
                        result.insert(field_name.to_string(), serde_json::Value::Null);
                    }
                }
            }
            // Handle delete mutations
            else if let Some(collection_name) = field_name.strip_prefix("delete") {
                if let Some(collection) = db.get(collection_name) {
                    // Determine id key and parse id argument
                    let id_key = collection.get_config().id_key;
                    let id_val = f.arguments.iter()
                        .find(|(name, _)| name == &id_key)
                        .and_then(|(_, val)| graphql_value_to_json(val).as_str().map(String::from));
                    // Perform delete if id provided
                    if let Some(id) = id_val {
                        let deleted = collection.delete(&id).unwrap_or(serde_json::Value::Null);
                        // Expand nested selections
                        let mut item = deleted;
                        let mut paths = Vec::new();
                        collect_expansion_paths(&f.selection_set, "", &mut paths);
                        for path in paths {
                            item = collection.expand_row(&item, &path, db);
                        }
                        // Filter fields
                        let filtered = filter_value(item, &f.selection_set);
                        result.insert(field_name.to_string(), filtered);
                    } else {
                        result.insert(field_name.to_string(), serde_json::Value::Null);
                    }
                }
            }
        }
    }
}

/// Execute GraphQL operations directly on Fosk database
async fn execute_graphql_operations(
    doc: &Document<'_, String>,
    db: &Db,
) -> Result<serde_json::Value, String> {
    let mut result = serde_json::Map::new();

    for def in &doc.definitions {
        match def {
            Definition::Operation(OperationDefinition::Query(q)) => {
                execute_query(db, &mut result, q);
            }
            Definition::Operation(OperationDefinition::Mutation(m)) => {
                execute_operation(db, &mut result, m);
            }
            _ => {}
        }
    }

    Ok(serde_json::Value::Object(result))
}

// -------------------------------------------------------------------------------

pub fn create_graphql_route(app: &mut App, route: &str, is_protected: bool, delay: Option<u16>) {
    // POST /graphql handler: parse, validate, and execute GraphQL manually
    let db = app.db.clone();
    let router = post(move |Json(req): Json<GQLRequest>| {
        let db = db.clone();
        async move {
            delay.sleep_thread();

            // Parse request into AST
            let doc = match parse_request_ast(&req) {
                Err(err) => {
                    let mut response = GQLResponse::default();
                    response.errors = vec![ServerError::new(err.message, None)];
                    return Json(response);
                }
                Ok(d) => d,
            };

            // Validate referenced collections exist in Fosk database
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
                Ok(data) => {
                    response.data =
                        async_graphql::Value::from_json(data).unwrap_or(async_graphql::Value::Null);
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

pub fn load_folder_collections(app: &mut App, path: OsString) -> Result<(), Error> {
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

pub fn build_graphql_routes(app: &mut App, config: &RouteGraphQL) {
    let result = load_folder_collections(app, config.path.clone());
    if let Err(error) = result {
        println!("Error to load GraphQL collections. Details: {}", error);
    }

    let route = &config.route;
    let is_protected = config.is_protected;
    let delay = config.delay;

    create_graphiql_route(app);
    create_graphql_route(app, route, is_protected, delay);
}

// Unit tests for GraphQL helper functions
#[cfg(test)]
mod tests {
    use super::*;
    use graphql_parser::parse_query;
    use graphql_parser::query::{Definition, OperationDefinition};

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
        assert_eq!(paths, vec!["order_items.products"], "collect_expansion_paths should only include full nested paths");
    }
}
