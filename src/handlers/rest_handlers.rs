use std::{path::PathBuf, str::FromStr, sync::Arc};

use axum::{
    extract::{Json, Path as AxumPath},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, patch, post, put},
};
use fosk::{DbCollection, DbConfig};
use jgd_rs::generate_jgd_from_file;
use serde_json::{Map, Value};

use crate::{
    app::App,
    handlers::{SleepThread, is_jgd},
    route_builder::{RouteRegistrator, RouteRest},
};

pub fn create_get_all(
    app: &mut App,
    route: &str,
    is_protected: bool,
    delay: Option<u16>,
    collection: &Arc<DbCollection>,
) {
    // GET /resource - list all
    let list_collection = Arc::clone(collection);
    let list_router = get(move || async move {
        delay.sleep_thread();

        let items = list_collection.get_all();

        let mut data: Map<String, Value> = Map::new();
        data.insert("data".to_string(), Value::Array(items));

        Json(data).into_response()
    });

    app.push_route(route, list_router, Some("GET"), is_protected, None);
}

pub fn create_insert(
    app: &mut App,
    route: &str,
    is_protected: bool,
    delay: Option<u16>,
    collection: &Arc<DbCollection>,
) {
    // POST /resource - create new
    let create_collection = Arc::clone(collection);
    let create_router = post(move |Json(payload): Json<Value>| async move {
        delay.sleep_thread();

        match create_collection.add(payload) {
            Some(item) => (StatusCode::CREATED, Json(item)).into_response(),
            None => StatusCode::BAD_REQUEST.into_response(),
        }
    });

    app.push_route(route, create_router, Some("POST"), is_protected, None);
}

pub fn create_get_item(
    app: &mut App,
    id_route: &str,
    is_protected: bool,
    delay: Option<u16>,
    collection: &Arc<DbCollection>,
) {
    // GET /resource/:id - get by id
    let get_collection = Arc::clone(collection);
    let get_router = get(move |AxumPath(id): AxumPath<String>| async move {
        delay.sleep_thread();

        match get_collection.get(&id) {
            Some(item) => Json(item).into_response(),
            None => StatusCode::NOT_FOUND.into_response(),
        }
    });

    app.push_route(id_route, get_router, Some("GET"), is_protected, None);
}

pub fn create_full_update(
    app: &mut App,
    id_route: &str,
    is_protected: bool,
    delay: Option<u16>,
    collection: &Arc<DbCollection>,
) {
    // PUT /resource/:id - update by id
    let update_collection = Arc::clone(collection);
    let put_router = put(
        move |AxumPath(id): AxumPath<String>, Json(payload): Json<Value>| async move {
            delay.sleep_thread();

            match update_collection.update(&id, payload) {
                Some(item) => Json(item).into_response(),
                None => StatusCode::NOT_FOUND.into_response(),
            }
        },
    );

    app.push_route(id_route, put_router, Some("PUT"), is_protected, None);
}

pub fn create_partial_update(
    app: &mut App,
    id_route: &str,
    is_protected: bool,
    delay: Option<u16>,
    collection: &Arc<DbCollection>,
) {
    // PATCH /resource/:id - partial update by id
    let patch_collection = Arc::clone(collection);
    let patch_router = patch(
        move |AxumPath(id): AxumPath<String>, Json(payload): Json<Value>| async move {
            delay.sleep_thread();

            match patch_collection.update_partial(&id, payload) {
                Some(item) => Json(item).into_response(),
                None => StatusCode::NOT_FOUND.into_response(),
            }
        },
    );

    app.push_route(id_route, patch_router, Some("PATCH"), is_protected, None);
}

pub fn create_delete(
    app: &mut App,
    id_route: &str,
    is_protected: bool,
    delay: Option<u16>,
    collection: &Arc<DbCollection>,
) {
    // DELETE /resource/:id - delete by id
    let delete_collection = Arc::clone(collection);
    let delete_router = delete(move |AxumPath(id): AxumPath<String>| async move {
        delay.sleep_thread();

        match delete_collection.delete(&id) {
            Some(item) => Json(item).into_response(),
            None => StatusCode::NOT_FOUND.into_response(),
        }
    });

    app.push_route(id_route, delete_router, Some("DELETE"), is_protected, None);
}

pub fn build_rest_routes(app: &mut App, config: &RouteRest) -> Arc<DbCollection> {
    let collection_name = config.collection_name.clone();
    let collection = app.db.create_with_config(
        &collection_name,
        DbConfig::from(config.id_type, &config.id_key),
    );

    let result: Result<String, String> = if is_jgd(&config.path) {
        match generate_jgd_from_file(&PathBuf::from_str(config.path.to_str().unwrap()).unwrap()) {
            Ok(jgd_json) => {
                let value = collection.load_from_json(jgd_json, false);
                value.map(|items| {
                    format!(
                        "✔️ Generated {} initial items from {}",
                        items.len(),
                        config.path.to_string_lossy()
                    )
                })
            }
            Err(error) => Err(format!(
                "Error to generate JGD Json for file {}. Details: {}",
                config.path.to_string_lossy(),
                error
            )),
        }
    } else {
        collection.load_from_file(&config.path)
    };

    // load_initial_data(file_path, &collection);
    match result {
        Ok(msg) => println!("{}", msg),
        Err(msg) => eprintln!("{}", msg),
    }

    let route = &config.route;
    let id_route = &format!("{}/{{{}}}", route, config.id_key);
    let is_protected = config.is_protected;
    let delay = config.delay;

    // Build REST routes for CRUD operations
    create_get_all(app, route, is_protected, delay, &collection);

    create_insert(app, route, is_protected, delay, &collection);

    create_get_item(app, id_route, is_protected, delay, &collection);

    create_full_update(app, id_route, is_protected, delay, &collection);

    create_partial_update(app, id_route, is_protected, delay, &collection);

    create_delete(app, id_route, is_protected, delay, &collection);

    collection
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::{Method, Request, header::CONTENT_TYPE},
    };
    use fosk::IdType;
    use serde_json::json;
    use tower::ServiceExt;

    fn json_request(method: Method, uri: &str, body: Value) -> Request<Body> {
        Request::builder()
            .method(method)
            .uri(uri)
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
            .unwrap()
    }

    async fn body_json(response: axum::response::Response) -> Value {
        serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap()).unwrap()
    }

    #[tokio::test]
    async fn rest_routes_support_crud_and_missing_items() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let file_path = temp_dir.path().join("rest.json");
        std::fs::write(&file_path, r#"[{"id":"1","name":"Ada"}]"#).unwrap();

        let mut app = App::default();
        let config = RouteRest::new(
            "/users".to_string(),
            file_path.into_os_string(),
            "id".to_string(),
            IdType::None,
            false,
            "users".to_string(),
            None,
        );
        let collection = build_rest_routes(&mut app, &config);
        assert_eq!(collection.count(), 1);

        let router = app.take_router_for_test();
        let list = router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/users")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(list.status(), StatusCode::OK);
        assert_eq!(body_json(list).await["data"][0]["name"], "Ada");

        let created = router
            .clone()
            .oneshot(json_request(
                Method::POST,
                "/users",
                json!({"id":"2","name":"Grace"}),
            ))
            .await
            .unwrap();
        assert_eq!(created.status(), StatusCode::CREATED);

        let item = router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/users/2")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(item.status(), StatusCode::OK);
        assert_eq!(body_json(item).await["name"], "Grace");

        let put = router
            .clone()
            .oneshot(json_request(
                Method::PUT,
                "/users/2",
                json!({"id":"2","name":"Hopper"}),
            ))
            .await
            .unwrap();
        assert_eq!(put.status(), StatusCode::OK);
        assert_eq!(body_json(put).await["name"], "Hopper");

        let patch = router
            .clone()
            .oneshot(json_request(
                Method::PATCH,
                "/users/2",
                json!({"role":"admin"}),
            ))
            .await
            .unwrap();
        assert_eq!(patch.status(), StatusCode::OK);
        assert_eq!(body_json(patch).await["role"], "admin");

        let delete = router
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::DELETE)
                    .uri("/users/2")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(delete.status(), StatusCode::OK);

        let missing = router
            .oneshot(
                Request::builder()
                    .uri("/users/2")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(missing.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn rest_routes_report_bad_initial_data_but_still_register_routes() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let file_path = temp_dir.path().join("rest.json");
        std::fs::write(&file_path, r#"{"not":"an array"}"#).unwrap();

        let mut app = App::default();
        let config = RouteRest::new(
            "/items".to_string(),
            file_path.into_os_string(),
            "id".to_string(),
            IdType::None,
            false,
            "items".to_string(),
            None,
        );
        build_rest_routes(&mut app, &config);

        let response = app
            .take_router_for_test()
            .oneshot(
                Request::builder()
                    .uri("/items/missing")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
