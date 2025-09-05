use axum::{extract::{Json, Path as AxumPath}, response::IntoResponse, routing::get};
use fosk::{database::SchemaProvider, FieldInfo, JsonPrimitive, SchemaDict};
use http::StatusCode;
use serde_json::{Map, Value};

use crate::app::{App, MOCK_SERVER_ROUTE};

fn field_info_to_json(field_info: &FieldInfo) -> Value {
    let mut j_fi: Map<String, Value> = Map::new();

    let f_type = match field_info.ty {
        JsonPrimitive::Null => "Null",
        JsonPrimitive::Bool => "Bool",
        JsonPrimitive::Int => "Int",
        JsonPrimitive::Float => "Float",
        JsonPrimitive::String => "String",
        JsonPrimitive::Object => "Object",
        JsonPrimitive::Array => "Array",
    };
    j_fi.insert("type".to_string(), Value::String(f_type.to_string()));
    j_fi.insert("nullable".to_string(), Value::Bool(field_info.nullable));

    Value::Object(j_fi)
}

fn schema_to_json(schema: &SchemaDict) -> Value {
    let mut j_map: Map<String, Value> = Map::new();

    for (name, field_info) in &schema.fields {
        j_map.insert(name.clone(), field_info_to_json(field_info));
    }

    Value::Object(j_map)
}

pub fn create_all_collections_info_route(
    app: &mut App,
) {
    let collection_route = format!("{}/collections", MOCK_SERVER_ROUTE);

    // POST /resource/login - auth
    let db = app.db.clone();

    let create_router = get(move || {
        async move {
            let mut j_collections: Map<String, Value> = Map::new();
            let collections = db.list_collections();
            for collection in collections {
                let schema = db.schema_of(&collection);
                if let Some(schema) = schema {
                    j_collections.insert(collection, schema_to_json(&schema));
                }
            }
            Json(Value::Object(j_collections)).into_response()
        }

    });
    app.route(&collection_route, create_router, Some("GET"), None);
}

fn create_collection_info_route(
    app: &mut App,
) {
    let collection_route = format!("{}/collections/{{name}}", MOCK_SERVER_ROUTE);

    // POST /resource/login - auth
    let db = app.db.clone();

    let create_router = get(move |AxumPath(name): AxumPath<String>| {
        async move {
            let schema = db.schema_of(&name);
            if let Some(schema) = schema {
                Json(schema_to_json(&schema)).into_response()
            } else {
                StatusCode::NOT_FOUND.into_response()
            }
        }

    });
    app.route(&collection_route, create_router, Some("GET"), None);
}

pub fn create_collections_routes(
    app: &mut App,
) {
    create_all_collections_info_route(app);
    create_collection_info_route(app);
}
