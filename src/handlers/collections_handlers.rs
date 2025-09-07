use std::sync::Arc;

use axum::{extract::{Json, Multipart, Path as AxumPath}, response::IntoResponse, routing::{get, post}};
use fosk::{database::SchemaProvider, DbCollection, FieldInfo, JsonPrimitive, SchemaDict};
use http::{header::{CONTENT_DISPOSITION, CONTENT_TYPE}, HeaderMap, HeaderValue, StatusCode};
use mime_guess::from_ext;
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

fn create_collection_load_from_file(
    app: &mut App,
) {
    let collection_route = format!("{}/collections/{{name}}", MOCK_SERVER_ROUTE);

    let db = app.db.clone();

    let create_router = post(async move |AxumPath(name): AxumPath<String>, mut multipart: Multipart| {
        if let Some(field) = multipart.next_field().await.unwrap() {
            // let field_name = field.name().unwrap_or("file").to_string();
            // let file_name = field.file_name()
            //     .map(|name| name.to_string())
            //     .unwrap_or_else(|| "collection.json".to_string());

            let data = field.bytes().await.unwrap();

            let result: Result<Value, serde_json::Error> = serde_json::from_slice(&data);

            let Ok(json) = result else {
                return StatusCode::BAD_REQUEST.into_response();
            };

            let collection = match db.get(&name) {
                Some(collection) => collection,
                None => db.create(&name),
            };

            let response = collection.load_from_json(json);

            if let Err(err) = response {
                return (StatusCode::INTERNAL_SERVER_ERROR, err).into_response()
            }

            return StatusCode::OK.into_response()
        }

        StatusCode::BAD_REQUEST.into_response()
    });
    app.route(&collection_route, create_router, Some("GET"), Some(&["upload".to_string()]));
}

fn create_db_load_from_file(
    app: &mut App,
) {
    let collection_route = format!("{}/collections", MOCK_SERVER_ROUTE);

    let db = app.db.clone();

    let create_router = post(async move |mut multipart: Multipart| {
        if let Some(field) = multipart.next_field().await.unwrap() {
            // let field_name = field.name().unwrap_or("file").to_string();
            // let file_name = field.file_name()
            //     .map(|name| name.to_string())
            //     .unwrap_or_else(|| "collection.json".to_string());

            let data = field.bytes().await.unwrap();

            let result: Result<Value, serde_json::Error> = serde_json::from_slice(&data);

            let Ok(json) = result else {
                return StatusCode::BAD_REQUEST.into_response();
            };

            let response = db.load_from_json(json);

            if let Err(err) = response {
                return (StatusCode::INTERNAL_SERVER_ERROR, err).into_response()
            }

            return StatusCode::OK.into_response()
        }

        StatusCode::BAD_REQUEST.into_response()
    });
    app.route(&collection_route, create_router, Some("GET"), Some(&["upload".to_string()]));
}

fn create_collection_download(
    app: &mut App,
) {
    let collection_route = format!("{}/collections/{{name}}/download", MOCK_SERVER_ROUTE);

    let db = app.db.clone();

    let create_router = get(async move |AxumPath(name): AxumPath<String>| {
        let collection: Option<Arc<DbCollection>> = db.get(&name);

        let Some(collection) = collection else {
            return StatusCode::NOT_FOUND.into_response();
        };

        let result: Result<Vec<u8>, serde_json::Error> = serde_json::to_vec(&collection.get_all());

        let Ok(contents) = result else {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        };

        let mime_type = from_ext("json")
                        .first_or_octet_stream()
                        .to_string();

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_str(&mime_type).unwrap());

        headers.insert(
            CONTENT_DISPOSITION,
            HeaderValue::from_str(&format!(
                "attachment; filename=\"{}.json\"",
                name
            ))
            .unwrap(),
        );

        (headers, contents).into_response()
    });
    app.route(&collection_route, create_router, Some("GET"), Some(&["download".to_string()]));
}

fn create_db_download(
    app: &mut App,
) {
    let collection_route = format!("{}/collections/download", MOCK_SERVER_ROUTE);

    // POST /resource/login - auth
    let db = app.db.clone();

    let create_router = get(async move || {
        let db_json = db.write_to_json();
        let result: Result<Vec<u8>, serde_json::Error> = serde_json::to_vec(&db_json);

        let Ok(contents) = result else {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        };

        let mime_type = from_ext("json")
                        .first_or_octet_stream()
                        .to_string();

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_str(&mime_type).unwrap());

        headers.insert(
            CONTENT_DISPOSITION,
            HeaderValue::from_str(
                "attachment; filename=\"collections.json\""
            )
            .unwrap(),
        );

        (headers, contents).into_response()
    });
    app.route(&collection_route, create_router, Some("GET"), Some(&["download".to_string()]));
}

pub fn create_collections_routes(
    app: &mut App,
) {
    create_all_collections_info_route(app);
    create_collection_info_route(app);
    create_collection_load_from_file(app);
    create_db_load_from_file(app);
    create_collection_download(app);
    create_db_download(app);
}
