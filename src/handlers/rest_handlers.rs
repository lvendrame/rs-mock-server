use std::{path::PathBuf, str::FromStr, sync::Arc};

use axum::{
    extract::{Json, Path as AxumPath}, http::StatusCode, response::IntoResponse, routing::{delete, get, patch, post, put}
};
use fosk::{DbCollection, DbConfig};
use jgd_rs::generate_jgd_from_file;
use serde_json::{Map, Value};

use crate::{
    app::App, handlers::{is_jgd, SleepThread}, route_builder::{RouteRegistrator, RouteRest}
};

pub fn create_get_all(app: &mut App, route: &str, is_protected: bool, delay: Option<u16>, collection: &Arc<DbCollection>) {
    // GET /resource - list all
    let list_collection = Arc::clone(collection);
    let list_router = get(move || {
        async move {
            delay.sleep_thread();

            let items = list_collection.get_all();

            let mut data: Map<String, Value> = Map::new();
            data.insert("data".to_string(), Value::Array(items));

            Json(data).into_response()
        }
    });

    app.push_route(route, list_router, Some("GET"), is_protected, None);
}

pub fn create_insert(app: &mut App, route: &str, is_protected: bool, delay: Option<u16>, collection: &Arc<DbCollection>) {
    // POST /resource - create new
    let create_collection = Arc::clone(collection);
    let create_router = post(move |Json(payload): Json<Value>| {
        async move {
            delay.sleep_thread();

            match create_collection.add(payload) {
                Some(item) => (StatusCode::CREATED, Json(item)).into_response(),
                None => StatusCode::BAD_REQUEST.into_response(),
            }
        }
    });

    app.push_route(route, create_router, Some("POST"), is_protected, None);
}

pub fn create_get_item(app: &mut App, id_route: &str, is_protected: bool, delay: Option<u16>, collection: &Arc<DbCollection>) {
    // GET /resource/:id - get by id
    let get_collection = Arc::clone(collection);
    let get_router = get(move |AxumPath(id): AxumPath<String>| {
        async move {
            delay.sleep_thread();

            match get_collection.get(&id) {
                Some(item) => Json(item).into_response(),
                None => StatusCode::NOT_FOUND.into_response(),
            }
        }
    });

    app.push_route(id_route, get_router, Some("GET"), is_protected, None);
}

pub fn create_full_update(app: &mut App, id_route: &str, is_protected: bool, delay: Option<u16>, collection: &Arc<DbCollection>) {
    // PUT /resource/:id - update by id
    let update_collection = Arc::clone(collection);
    let put_router = put(move |AxumPath(id): AxumPath<String>, Json(payload): Json<Value>| {
        async move {
            delay.sleep_thread();

            match update_collection.update(&id, payload) {
                Some(item) => Json(item).into_response(),
                None => StatusCode::NOT_FOUND.into_response(),
            }
        }
    });

    app.push_route(id_route, put_router, Some("PUT"), is_protected, None);
}

pub fn create_partial_update(app: &mut App, id_route: &str, is_protected: bool, delay: Option<u16>, collection: &Arc<DbCollection>) {
    // PATCH /resource/:id - partial update by id
    let patch_collection = Arc::clone(collection);
    let patch_router = patch(move |AxumPath(id): AxumPath<String>, Json(payload): Json<Value>| {
        async move {
            delay.sleep_thread();

            match patch_collection.update_partial(&id, payload) {
                Some(item) => Json(item).into_response(),
                None => StatusCode::NOT_FOUND.into_response(),
            }
        }
    });

    app.push_route(id_route, patch_router, Some("PATCH"), is_protected, None);
}

pub fn create_delete(app: &mut App, id_route: &str, is_protected: bool, delay: Option<u16>, collection: &Arc<DbCollection>) {
    // DELETE /resource/:id - delete by id
    let delete_collection = Arc::clone(collection);
    let delete_router = delete(move |AxumPath(id): AxumPath<String>| {
        async move {
            delay.sleep_thread();

            match delete_collection.delete(&id) {
                Some(item) => Json(item).into_response(),
                None => StatusCode::NOT_FOUND.into_response(),
            }
        }
    });

    app.push_route(id_route, delete_router, Some("DELETE"), is_protected, None);
}

pub fn build_rest_routes(
    app: &mut App,
    config: &RouteRest,
) -> Arc<DbCollection> {
    let collection_name = config.collection_name.clone();
    let collection = app.db.create_with_config(
        &collection_name,
        DbConfig::from(config.id_type, &config.id_key));

    let result: Result<String, String> = if is_jgd(&config.path) {
        match generate_jgd_from_file(&PathBuf::from_str(config.path.to_str().unwrap()).unwrap()) {
            Ok(jgd_json) => {
                let value = collection.load_from_json(jgd_json, false);
                value
                    .map(|items| format!("✔️ Generated {} initial items from {}", items.len(), config.path.to_string_lossy()))
            },
            Err(error) => Err(format!("Error to generate JGD Json for file {}. Details: {}", config.path.to_string_lossy(), error)),
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
