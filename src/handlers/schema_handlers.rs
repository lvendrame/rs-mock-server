//! Internal schema upload and download handlers.

use axum::{
    extract::{Multipart, Path as AxumPath},
    response::IntoResponse,
    routing::{get, post},
};
use http::{
    HeaderMap, HeaderValue, StatusCode,
    header::{CONTENT_DISPOSITION, CONTENT_TYPE},
};
use mime_guess::from_ext;
use serde_json::Value;

use crate::{
    app::{App, MOCK_SERVER_ROUTE},
    handlers::read_error_response,
    schema_files::{collection_schema_to_compact_json, db_schemas_to_compact_json},
};

async fn multipart_json(mut multipart: Multipart) -> Result<Value, StatusCode> {
    let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
    else {
        return Err(StatusCode::BAD_REQUEST);
    };

    let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
    serde_json::from_slice(&data).map_err(|_| StatusCode::BAD_REQUEST)
}

fn json_download_response(file_name: &str, value: &Value) -> axum::response::Response {
    let Ok(contents) = serde_json::to_vec(value) else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    let mime_type = from_ext("json").first_or_octet_stream().to_string();
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_str(&mime_type).unwrap());
    headers.insert(
        CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("attachment; filename=\"{file_name}\"")).unwrap(),
    );

    (headers, contents).into_response()
}

fn create_collection_schema_load_route(app: &mut App) {
    let schema_route = format!("{}/schemas/{{name}}", MOCK_SERVER_ROUTE);
    let db = app.db.clone();

    let create_router = post(
        async move |AxumPath(name): AxumPath<String>, multipart: Multipart| {
            let json = match multipart_json(multipart).await {
                Ok(json) => json,
                Err(status) => return status.into_response(),
            };

            match db.load_collection_schema_from_json(&name, json) {
                Ok(()) => StatusCode::OK.into_response(),
                Err(err) => (StatusCode::BAD_REQUEST, err).into_response(),
            }
        },
    );

    app.route(
        &schema_route,
        create_router,
        Some("POST"),
        Some(&["upload".to_string()]),
    );
}

fn create_db_schema_load_route(app: &mut App) {
    let schema_route = format!("{}/schemas", MOCK_SERVER_ROUTE);
    let db = app.db.clone();

    let create_router = post(async move |multipart: Multipart| {
        let json = match multipart_json(multipart).await {
            Ok(json) => json,
            Err(status) => return status.into_response(),
        };

        match db.load_schemas_from_json(json) {
            Ok(_) => StatusCode::OK.into_response(),
            Err(err) => (StatusCode::BAD_REQUEST, err).into_response(),
        }
    });

    app.route(
        &schema_route,
        create_router,
        Some("POST"),
        Some(&["upload".to_string()]),
    );
}

fn create_collection_schema_download_route(app: &mut App) {
    let schema_route = format!("{}/schemas/{{name}}/download", MOCK_SERVER_ROUTE);
    let db = app.db.clone();

    let create_router = get(async move |AxumPath(name): AxumPath<String>| {
        let Some(collection) = db.get(&name) else {
            return StatusCode::NOT_FOUND.into_response();
        };
        let schema = match collection_schema_to_compact_json(&collection) {
            Ok(Some(schema)) => schema,
            Ok(None) => return StatusCode::NOT_FOUND.into_response(),
            Err(err) => return read_error_response(err),
        };

        json_download_response(&format!("{name}.schema"), &schema)
    });

    app.route(
        &schema_route,
        create_router,
        Some("GET"),
        Some(&["download".to_string()]),
    );
}

fn create_db_schema_download_route(app: &mut App) {
    let schema_route = format!("{}/schemas/download", MOCK_SERVER_ROUTE);
    let db = app.db.clone();

    let create_router = get(async move || {
        let schema = db_schemas_to_compact_json(&db);
        json_download_response("db.schema", &schema)
    });

    app.route(
        &schema_route,
        create_router,
        Some("GET"),
        Some(&["download".to_string()]),
    );
}

/// Registers internal schema upload and download routes.
pub fn create_schema_routes(app: &mut App) {
    create_collection_schema_load_route(app);
    create_db_schema_load_route(app);
    create_collection_schema_download_route(app);
    create_db_schema_download_route(app);
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::{Method, Request, header::CONTENT_TYPE},
    };
    use http::StatusCode;
    use serde_json::json;
    use tower::ServiceExt;

    fn multipart_json_request(body: &str) -> Body {
        let boundary = "BOUNDARY";
        let multipart = format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"schema.json\"\r\nContent-Type: application/json\r\n\r\n{body}\r\n--{boundary}--\r\n"
        );
        Body::from(multipart)
    }

    #[tokio::test]
    async fn schema_routes_load_and_download_compact_schemas() {
        let mut app = App::default();
        create_schema_routes(&mut app);
        let router = app.take_router_for_test();

        let load_db = router
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/mock-server/schemas")
                    .header(CONTENT_TYPE, "multipart/form-data; boundary=BOUNDARY")
                    .body(multipart_json_request(
                        r#"{"users":{"user_id":"Id","name":"String!"}}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(load_db.status(), StatusCode::OK);

        let load_collection = router
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/mock-server/schemas/orders")
                    .header(CONTENT_TYPE, "multipart/form-data; boundary=BOUNDARY")
                    .body(multipart_json_request(
                        r#"{"order_id":"Uuid","user_id":"Int!","total":"Float!"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(load_collection.status(), StatusCode::OK);

        let one = router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/mock-server/schemas/users/download")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(one.status(), StatusCode::OK);
        assert_eq!(one.headers().get(CONTENT_TYPE).unwrap(), "application/json");
        let one_body: Value =
            serde_json::from_slice(&to_bytes(one.into_body(), usize::MAX).await.unwrap()).unwrap();
        assert_eq!(one_body, json!({"user_id":"Id","name":"String!"}));

        let all = router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/mock-server/schemas/download")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(all.status(), StatusCode::OK);
        let all_body: Value =
            serde_json::from_slice(&to_bytes(all.into_body(), usize::MAX).await.unwrap()).unwrap();
        assert!(all_body.get("users").is_some());
        assert!(all_body.get("orders").is_some());
    }

    #[tokio::test]
    async fn schema_routes_reject_bad_uploads_and_missing_downloads() {
        let mut app = App::default();
        create_schema_routes(&mut app);
        let router = app.take_router_for_test();

        let bad_json = router
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/mock-server/schemas/users")
                    .header(CONTENT_TYPE, "multipart/form-data; boundary=BOUNDARY")
                    .body(multipart_json_request("not-json"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(bad_json.status(), StatusCode::BAD_REQUEST);

        let invalid_schema = router
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/mock-server/schemas/users")
                    .header(CONTENT_TYPE, "multipart/form-data; boundary=BOUNDARY")
                    .body(multipart_json_request(r#"{"id":"Unknown"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(invalid_schema.status(), StatusCode::BAD_REQUEST);

        let missing = router
            .oneshot(
                Request::builder()
                    .uri("/mock-server/schemas/missing/download")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(missing.status(), StatusCode::NOT_FOUND);
    }
}
