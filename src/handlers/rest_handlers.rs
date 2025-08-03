use std::{ffi::OsString, fs, sync::{Arc}};

use axum::{
    extract::{Json, Path as AxumPath}, http::StatusCode, response::IntoResponse, routing::{delete, get, patch, post, put}
};
use serde_json::Value;

use crate::{
    app::App, id_manager::IdType, in_memory_collection::{InMemoryCollection, ProtectedMemCollection}, route_builder::RouteRegistrator
};

pub fn load_initial_data(file_path: &OsString, load_collection: &ProtectedMemCollection) {
    // Try to read the file content
    if let Ok(file_content) = fs::read_to_string(file_path) {
        // Try to parse the content as JSON
        if let Ok(json_value) = serde_json::from_str::<Value>(&file_content) {
            // Check if it's a JSON Array
            if let Value::Array(_) = json_value {
                // Load the array into the collection using add_batch
                let mut collection = load_collection.lock().unwrap();
                let added_items = collection.add_batch(json_value);
                return println!("✔️ Loaded {} initial items from {}", added_items.len(), file_path.to_string_lossy());
            }
            return eprintln!("⚠️ File {} does not contain a JSON array, skipping initial data load", file_path.to_string_lossy());
        }
        return eprintln!("⚠️ File {} does not contain valid JSON, skipping initial data load", file_path.to_string_lossy());
    }
    eprintln!("⚠️ Could not read file {}, skipping initial data load", file_path.to_string_lossy());
}

pub fn create_get_all(app: &mut App, route_path: &str, collection: &ProtectedMemCollection, is_protected: bool) {
    // GET /resource - list all
    let list_collection = Arc::clone(collection);
    let list_router = get(move || {
        async move {
            let list_collection = list_collection.lock().unwrap();
            let items = list_collection.get_all();
            Json(items).into_response()
        }
    });

    app.push_route(route_path, list_router, Some("GET"), is_protected);
}

pub fn create_insert(app: &mut App, route_path: &str, collection: &ProtectedMemCollection, is_protected: bool) {
    // POST /resource - create new
    let create_collection = Arc::clone(collection);
    let create_router = post(move |Json(payload): Json<Value>| {
        async move {
            let mut create_collection = create_collection.lock().unwrap();
            match create_collection.add(payload) {
                Some(item) => (StatusCode::CREATED, Json(item)).into_response(),
                None => StatusCode::BAD_REQUEST.into_response(),
            }
        }
    });

    app.push_route(route_path, create_router, Some("POST"), is_protected);
}

pub fn create_get_item(app: &mut App, collection: &ProtectedMemCollection, id_route: &str, is_protected: bool) {
    // GET /resource/:id - get by id
    let get_collection = Arc::clone(collection);
    let get_router = get(move |AxumPath(id): AxumPath<String>| {
        async move {
            let get_collection = get_collection.lock().unwrap();
            match get_collection.get(&id) {
                Some(item) => Json(item).into_response(),
                None => StatusCode::NOT_FOUND.into_response(),
            }
        }
    });

    app.push_route(id_route, get_router, Some("GET"), is_protected);
}

pub fn create_full_update(app: &mut App, collection: &ProtectedMemCollection, id_route: &str, is_protected: bool) {
    // PUT /resource/:id - update by id
    let update_collection = Arc::clone(collection);
    let put_router = put(move |AxumPath(id): AxumPath<String>, Json(payload): Json<Value>| {
        async move {
            let mut update_collection = update_collection.lock().unwrap();
            match update_collection.update(&id, payload) {
                Some(item) => Json(item).into_response(),
                None => StatusCode::NOT_FOUND.into_response(),
            }
        }
    });

    app.push_route(id_route, put_router, Some("PUT"), is_protected);
}

pub fn create_partial_update(app: &mut App, collection: &ProtectedMemCollection, id_route: &str, is_protected: bool) {
    // PATCH /resource/:id - partial update by id
    let patch_collection = Arc::clone(collection);
    let patch_router = patch(move |AxumPath(id): AxumPath<String>, Json(payload): Json<Value>| {
        async move {
            let mut patch_collection = patch_collection.lock().unwrap();
            match patch_collection.update_partial(&id, payload) {
                Some(item) => Json(item).into_response(),
                None => StatusCode::NOT_FOUND.into_response(),
            }
        }
    });

    app.push_route(id_route, patch_router, Some("PATCH"), is_protected);
}

pub fn create_delete(app: &mut App, collection: ProtectedMemCollection, id_route: &str, is_protected: bool) {
    // DELETE /resource/:id - delete by id
    let delete_collection = Arc::clone(&collection);
    let delete_router = delete(move |AxumPath(id): AxumPath<String>| {
        async move {
            let mut delete_collection = delete_collection.lock().unwrap();
            match delete_collection.delete(&id) {
                Some(item) => Json(item).into_response(),
                None => StatusCode::NOT_FOUND.into_response(),
            }
        }
    });

    app.push_route(id_route, delete_router, Some("DELETE"), is_protected);
}

pub fn build_rest_routes(app: &mut App, route_path: &str, file_path: &OsString, id_key: &str, id_type: IdType, is_protected: bool) {
    let in_memory_collection = InMemoryCollection::new(id_type, id_key.to_string());
    let collection = in_memory_collection.into_protected();

    load_initial_data(file_path, &collection);

    let id_route = format!("{}/{{{}}}", route_path, id_key);

    // Build REST routes for CRUD operations
    create_get_all(app, route_path, &collection, is_protected);

    create_insert(app, route_path, &collection, is_protected);

    create_get_item(app, &collection, &id_route, is_protected);

    create_full_update(app, &collection, &id_route, is_protected);

    create_partial_update(app, &collection, &id_route, is_protected);

    create_delete(app, collection, &id_route, is_protected);

    println!("✔️ Built REST routes for {}", route_path);
}
