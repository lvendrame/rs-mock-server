use std::sync::Arc;

use axum::{
    extract::{Json, Multipart, Path as AxumPath},
    response::IntoResponse,
    routing::{get, post},
};
use fosk::{DbCollection, FieldInfo, JsonPrimitive, SchemaWithRefs};
use http::{
    HeaderMap, HeaderValue, StatusCode,
    header::{CONTENT_DISPOSITION, CONTENT_TYPE},
};
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

fn reference_to_json(s_ref: fosk::ReferenceColumn) -> Value {
    let mut j_ref: Map<String, Value> = Map::new();

    j_ref.insert("collection".to_string(), Value::String(s_ref.collection));
    j_ref.insert("column".to_string(), Value::String(s_ref.column));
    j_ref.insert(
        "ref_collection".to_string(),
        Value::String(s_ref.ref_collection),
    );
    j_ref.insert("ref_column".to_string(), Value::String(s_ref.ref_column));

    Value::Object(j_ref)
}

fn schema_to_json(schema: &SchemaWithRefs) -> Value {
    let mut j_map: Map<String, Value> = Map::new();
    let mut j_fields: Map<String, Value> = Map::new();
    let mut j_inbound_refs: Map<String, Value> = Map::new();
    let mut j_outbound_refs: Map<String, Value> = Map::new();

    for (name, field_info) in &schema.fields {
        j_fields.insert(name.clone(), field_info_to_json(field_info));
    }
    j_map.insert("fields".to_string(), Value::Object(j_fields));

    for in_ref in schema.inbound_refs.clone().into_values() {
        j_inbound_refs.insert(in_ref.ref_column.clone(), reference_to_json(in_ref));
    }
    j_map.insert("inbound_refs".to_string(), Value::Object(j_inbound_refs));

    for out_ref in schema.outbound_refs.clone().into_values() {
        j_outbound_refs.insert(out_ref.column.clone(), reference_to_json(out_ref));
    }
    j_map.insert("outbound_refs".to_string(), Value::Object(j_outbound_refs));

    Value::Object(j_map)
}

/// Registers the route that returns schemas for all loaded collections.
pub fn create_all_collections_info_route(app: &mut App) {
    let collection_route = format!("{}/collections", MOCK_SERVER_ROUTE);

    // POST /resource/login - auth
    let db = app.db.clone();

    let create_router = get(move || async move {
        let mut j_collections: Map<String, Value> = Map::new();
        let collections = db.list_collections();
        for collection in collections {
            let schema = db.schema_with_refs_of(&collection);
            if let Some(schema) = schema {
                j_collections.insert(collection, schema_to_json(&schema));
            }
        }
        Json(Value::Object(j_collections)).into_response()
    });
    app.route(&collection_route, create_router, Some("GET"), None);
}

fn create_collection_info_route(app: &mut App) {
    let collection_route = format!("{}/collections/{{name}}", MOCK_SERVER_ROUTE);

    let db = app.db.clone();

    let create_router = get(move |AxumPath(name): AxumPath<String>| async move {
        let schema = db.schema_with_refs_of(&name);
        if let Some(schema) = schema {
            Json(schema_to_json(&schema)).into_response()
        } else {
            StatusCode::NOT_FOUND.into_response()
        }
    });
    app.route(&collection_route, create_router, Some("GET"), None);
}

fn create_collection_load_from_file(app: &mut App) {
    let collection_route = format!("{}/collections/{{name}}", MOCK_SERVER_ROUTE);

    let db = app.db.clone();

    let create_router = post(
        async move |AxumPath(name): AxumPath<String>, mut multipart: Multipart| {
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

                let response = collection.load_from_json(json, false);

                if let Err(err) = response {
                    return (StatusCode::INTERNAL_SERVER_ERROR, err).into_response();
                }

                return StatusCode::OK.into_response();
            }

            StatusCode::BAD_REQUEST.into_response()
        },
    );
    app.route(
        &collection_route,
        create_router,
        Some("POST"),
        Some(&["upload".to_string()]),
    );
}

fn create_db_load_from_file(app: &mut App) {
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

            let response = db.load_from_json(json, false);

            if let Err(err) = response {
                return (StatusCode::INTERNAL_SERVER_ERROR, err).into_response();
            }

            return StatusCode::OK.into_response();
        }

        StatusCode::BAD_REQUEST.into_response()
    });
    app.route(
        &collection_route,
        create_router,
        Some("POST"),
        Some(&["upload".to_string()]),
    );
}

fn create_collection_download(app: &mut App) {
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

        let mime_type = from_ext("json").first_or_octet_stream().to_string();

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_str(&mime_type).unwrap());

        headers.insert(
            CONTENT_DISPOSITION,
            HeaderValue::from_str(&format!("attachment; filename=\"{}.json\"", name)).unwrap(),
        );

        (headers, contents).into_response()
    });
    app.route(
        &collection_route,
        create_router,
        Some("GET"),
        Some(&["download".to_string()]),
    );
}

fn create_db_download(app: &mut App) {
    let collection_route = format!("{}/collections/download", MOCK_SERVER_ROUTE);

    // POST /resource/login - auth
    let db = app.db.clone();

    let create_router = get(async move || {
        let db_json = db.write_to_json();
        let result: Result<Vec<u8>, serde_json::Error> = serde_json::to_vec(&db_json);

        let Ok(contents) = result else {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        };

        let mime_type = from_ext("json").first_or_octet_stream().to_string();

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_str(&mime_type).unwrap());

        headers.insert(
            CONTENT_DISPOSITION,
            HeaderValue::from_str("attachment; filename=\"collections.json\"").unwrap(),
        );

        (headers, contents).into_response()
    });
    app.route(
        &collection_route,
        create_router,
        Some("GET"),
        Some(&["download".to_string()]),
    );
}

/// Registers internal collection metadata, upload, and download routes.
pub fn create_collections_routes(app: &mut App) {
    create_all_collections_info_route(app);
    create_collection_info_route(app);
    create_collection_load_from_file(app);
    create_db_load_from_file(app);
    create_collection_download(app);
    create_db_download(app);
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::{Method, Request, header::CONTENT_TYPE},
    };
    use serde_json::json;
    use tower::ServiceExt;

    fn multipart_json(body: &str) -> Request<Body> {
        let boundary = "BOUNDARY";
        let multipart = format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"data.json\"\r\nContent-Type: application/json\r\n\r\n{body}\r\n--{boundary}--\r\n"
        );
        Request::builder()
            .method(Method::POST)
            .header(
                CONTENT_TYPE,
                format!("multipart/form-data; boundary={boundary}"),
            )
            .body(Body::from(multipart))
            .unwrap()
    }

    #[tokio::test]
    async fn collection_routes_expose_schema_load_and_download() {
        let mut app = App::default();
        let users = app.db.create("users");
        users
            .load_from_json(json!([{"id":"1","name":"Ada"}]), false)
            .unwrap();
        create_collections_routes(&mut app);
        let router = app.take_router_for_test();

        let all = router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/mock-server/collections")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(all.status(), StatusCode::OK);
        let all_body: Value =
            serde_json::from_slice(&to_bytes(all.into_body(), usize::MAX).await.unwrap()).unwrap();
        assert!(all_body.get("users").is_some());

        let one = router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/mock-server/collections/users")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(one.status(), StatusCode::OK);

        let missing = router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/mock-server/collections/missing")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(missing.status(), StatusCode::NOT_FOUND);

        let download = router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/mock-server/collections/users/download")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(download.status(), StatusCode::OK);
        assert_eq!(
            download.headers().get(CONTENT_TYPE).unwrap(),
            "application/json"
        );

        let db_download = router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/mock-server/collections/download")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(db_download.status(), StatusCode::OK);

        let load_collection = router
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/mock-server/collections/projects")
                    .header(CONTENT_TYPE, "multipart/form-data; boundary=BOUNDARY")
                    .body(multipart_json(r#"[{"id":"p1","name":"Mock"}]"#).into_body())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(load_collection.status(), StatusCode::OK);

        let load_db = router
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/mock-server/collections")
                    .header(CONTENT_TYPE, "multipart/form-data; boundary=BOUNDARY")
                    .body(multipart_json(r#"{"teams":[{"id":"t1"}]}"#).into_body())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(load_db.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn collection_upload_routes_reject_bad_json_and_empty_multipart() {
        let mut app = App::default();
        create_collections_routes(&mut app);
        let router = app.take_router_for_test();

        let bad_json = router
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/mock-server/collections/users")
                    .header(CONTENT_TYPE, "multipart/form-data; boundary=BOUNDARY")
                    .body(multipart_json("not-json").into_body())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(bad_json.status(), StatusCode::BAD_REQUEST);

        let empty = "--BOUNDARY--\r\n";
        let empty_upload = router
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/mock-server/collections")
                    .header(CONTENT_TYPE, "multipart/form-data; boundary=BOUNDARY")
                    .body(Body::from(empty))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(empty_upload.status(), StatusCode::BAD_REQUEST);
    }
}
