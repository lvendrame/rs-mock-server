use std::{ffi::OsString, path::PathBuf, str::FromStr, sync::Arc};

use axum::{
    extract::{Json, Path as AxumPath}, http::StatusCode, response::IntoResponse, routing::{delete, get, patch, post, put}
};
use jgd_rs::generate_jgd_from_file;
use serde_json::Value;

use crate::{
    app::App, handlers::is_jgd, memory_db::{id_manager::IdType, memory_collection::MemoryCollection, CollectionConfig, DbCollection, DbCommon}, route_builder::RouteRegistrator
};

pub fn create_get_all(app: &mut App, route: &str, is_protected: bool, collection: &MemoryCollection) {
    // GET /resource - list all
    let list_collection = Arc::clone(collection);
    let list_router = get(move || {
        async move {
            let items = list_collection.get_all();
            Json(items).into_response()
        }
    });

    app.push_route(route, list_router, Some("GET"), is_protected, None);
}

pub fn create_insert(app: &mut App, route: &str, is_protected: bool, collection: &MemoryCollection) {
    // POST /resource - create new
    let mut create_collection = Arc::clone(collection);
    let create_router = post(move |Json(payload): Json<Value>| {
        async move {
            match create_collection.add(payload) {
                Some(item) => (StatusCode::CREATED, Json(item)).into_response(),
                None => StatusCode::BAD_REQUEST.into_response(),
            }
        }
    });

    app.push_route(route, create_router, Some("POST"), is_protected, None);
}

pub fn create_get_item(app: &mut App, id_route: &str, is_protected: bool, collection: &MemoryCollection) {
    // GET /resource/:id - get by id
    let get_collection = Arc::clone(collection);
    let get_router = get(move |AxumPath(id): AxumPath<String>| {
        async move {
            match get_collection.get(&id) {
                Some(item) => Json(item).into_response(),
                None => StatusCode::NOT_FOUND.into_response(),
            }
        }
    });

    app.push_route(id_route, get_router, Some("GET"), is_protected, None);
}

pub fn create_full_update(app: &mut App, id_route: &str, is_protected: bool, collection: &MemoryCollection) {
    // PUT /resource/:id - update by id
    let mut update_collection = Arc::clone(collection);
    let put_router = put(move |AxumPath(id): AxumPath<String>, Json(payload): Json<Value>| {
        async move {
            match update_collection.update(&id, payload) {
                Some(item) => Json(item).into_response(),
                None => StatusCode::NOT_FOUND.into_response(),
            }
        }
    });

    app.push_route(id_route, put_router, Some("PUT"), is_protected, None);
}

pub fn create_partial_update(app: &mut App, id_route: &str, is_protected: bool, collection: &MemoryCollection) {
    // PATCH /resource/:id - partial update by id
    let mut patch_collection = Arc::clone(collection);
    let patch_router = patch(move |AxumPath(id): AxumPath<String>, Json(payload): Json<Value>| {
        async move {
            match patch_collection.update_partial(&id, payload) {
                Some(item) => Json(item).into_response(),
                None => StatusCode::NOT_FOUND.into_response(),
            }
        }
    });

    app.push_route(id_route, patch_router, Some("PATCH"), is_protected, None);
}

pub fn create_delete(app: &mut App, id_route: &str, is_protected: bool, collection: &MemoryCollection) {
    // DELETE /resource/:id - delete by id
    let mut delete_collection = Arc::clone(collection);
    let delete_router = delete(move |AxumPath(id): AxumPath<String>| {
        async move {
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
    route: &str,
    file_path: &OsString,
    id_key: &str,
    id_type: IdType,
    is_protected: bool,
) -> MemoryCollection {
    let mut collection = app.db.create(CollectionConfig::from(id_type, id_key, route));

    let result: Result<String, String> = if is_jgd(file_path) {
        match generate_jgd_from_file(&PathBuf::from_str(file_path.to_str().unwrap()).unwrap()) {
            Ok(jgd_json) => {
                let value = collection.load_from_json(jgd_json);
                value
                    .map(|items| format!("✔️ Generated {} initial items from {}", items.len(), file_path.to_string_lossy()))
            },
            Err(error) => Err(format!("Error to generate JGD Json for file {}. Details: {}", file_path.to_string_lossy(), error)),
        }
    } else {
        collection.load_from_file(file_path)
    };

    // load_initial_data(file_path, &collection);
    match result {
        Ok(msg) => println!("{}", msg),
        Err(msg) => eprintln!("{}", msg),
    }

    let id_route = &format!("{}/{{{}}}", route, id_key);

    // Build REST routes for CRUD operations
    create_get_all(app, route, is_protected, &collection);

    create_insert(app, route, is_protected, &collection);

    create_get_item(app, id_route, is_protected, &collection);

    create_full_update(app, id_route, is_protected, &collection);

    create_partial_update(app, id_route, is_protected, &collection);

    create_delete(app, id_route, is_protected, &collection);

    collection
}
