use std::{ffi::OsString, fs::{self, DirEntry}, path::Path, sync::{Arc, Mutex}};

use axum::{
    body::Body, response::IntoResponse, routing::{delete, get, options, patch, post, put, MethodRouter}, extract::{Path as AxumPath, Json}, http::StatusCode
};
use http::{header::CONTENT_TYPE, HeaderMap, HeaderValue};
use mime_guess::from_path;
use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::Value;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

use crate::{app::App, id_manager::IdType, in_memory_collection::InMemoryCollection};

pub fn load_mock_dir(app: &mut App) {
    load_dir(app, "", &app.root_path.clone());
}

fn load_dir(app: &mut App, parent_route: &str, entries_path: &str) {
    let entries = fs::read_dir(entries_path).unwrap();
    for entry in entries {
        let entry = entry.unwrap();
        load_entry(app, parent_route, &entry);
    }
}

fn load_entry(app: &mut App, parent_route: &str, entry: &DirEntry) {
    let binding = entry.file_name();
    let end_point = binding.to_string_lossy();
    let full_route = format!("{}/{}", parent_route, end_point);

    let file_name = String::from(end_point);

    if entry.file_type().unwrap().is_dir() {
        if file_name.starts_with("public") {
            return app.build_public_router(file_name,String::from(entry.path().to_string_lossy()));
        }
        return load_dir(app, &full_route, entry.path().to_str().unwrap());

    }

    if entry.file_type().unwrap().is_file() &&  !file_name.starts_with(".") {
        // If it's a file, read its contents and register the route
        load_file_route(app, parent_route, entry);
    }
}

/// id:uuid, uuid, int, id, _id, _id:int
fn get_rest_options(descriptor: &str) -> (&str, IdType) {
    let parts: Vec<&str> = descriptor.split(':').collect();

    if parts.len() == 1 {
        // Single value like "uuid", "int", "id", "_id"
        let part = parts[0];
        match part {
            "uuid" => ("id", IdType::Uuid),
            "int" => ("id", IdType::Int),
            id_key => (id_key, IdType::Uuid), // Default fallback
        }
    } else if parts.len() == 2 {
        // Key:type format like "id:uuid", "_id:int"
        let id_key = parts[0];
        let type_str = parts[1];
        let id_type = match type_str {
            "uuid" => IdType::Uuid,
            "int" => IdType::Int,
            _ => IdType::Uuid, // Default to UUID
        };
        (id_key, id_type)
    } else {
        // Invalid format, return defaults
        ("id", IdType::Uuid)
    }
}

// Routes examples:
// /mocks
// /mocks/login/get.json,post.json,delete.json,put.json,patch.json
//
// get.json => /
// get{id}.json => /asd,/123,/456
// get{123}.json => /123
// get{1-5}.json => /1, /2, /3, /4, /5
fn load_file_route(app: &mut App, parent_route: &str, entry: &DirEntry) {
    let binding = entry.file_name();
    let file_name = binding.to_string_lossy();
    let file_stem = file_name.split('.').next().unwrap_or("");

    static RE_METHODS: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^(get|post|put|patch|delete|options)(\{(.+)\})?$").unwrap()
    });

    static RE_REST: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^(rest)(\{(.+)\})?$").unwrap()
    });

    let file_path = entry.path().into_os_string();

    if let Some(captures) = RE_METHODS.captures(file_stem) {
        let method = captures.get(1).unwrap().as_str();
        let pattern = captures.get(3);

        if let Some(pattern) = pattern {
            let pattern = pattern.as_str();

            // Pattern 1: get{id}.json -> Wildcard route /path/:param
            if pattern == "id" {
                let route_path = format!("{}/{}", parent_route, "{id}");
                let router = build_method_router(&file_path, method);
                println!("✔️ Mapped {} to {} {}", file_name, method.to_uppercase(), &route_path);
                app.route(&route_path, router, Some(method.to_string()));
                return;
            }

            // Pattern 2: get{1-5}.json -> Range of static routes /path/1, /path/2, ...
            if pattern.contains('-') {
                if let Some((start_str, end_str)) = pattern.split_once('-') {
                    if let (Ok(start), Ok(end)) = (start_str.parse::<i32>(), end_str.parse::<i32>()) {
                        for i in start..=end {
                            let route_path = format!("{}/{}", parent_route, i);
                            let router = build_method_router(&file_path, method);
                            app.route(&route_path, router, Some(method.to_string()));
                        }
                        println!("✔️ Mapped {} to {} {}/[{}-{}]", file_name, method.to_uppercase(), parent_route, start, end);
                        return;
                    }
                }
            }

            // Pattern 3: get{123}.json -> A single static route /path/123
            let route_path = format!("{}/{}", parent_route, pattern);
            let router = build_method_router(&file_path, method);
            println!("✔️ Mapped {} to {} {}", file_name, method.to_uppercase(), &route_path);

            app.route(&route_path, router, Some(method.to_string()));
            return;
        }

        // Default: get.json -> Route on the parent directory /path
        let method = captures.get(1).unwrap().as_str();
        let route_path = if parent_route.is_empty() { "/" } else { parent_route };
        let router = build_method_router(&file_path, method);
        println!("✔️ Mapped {} to {} {}", file_name, method.to_uppercase(), route_path);

        app.route(route_path, router, Some(method.to_string()));

        return;
    }

    if let Some(captures) = RE_REST.captures(file_stem) {
        let descriptor = if let Some(pattern) = captures.get(3) {
            pattern.as_str()
        } else {
            "id:uuid"
        };

        let (id_key, id_type) = get_rest_options(descriptor);
        let route_path = if parent_route.is_empty() { "/" } else { parent_route };

        build_in_memory_routes(app, route_path, file_path, id_key, id_type);

        return;
    }

    let route_path = if parent_route.is_empty() { "/" } else { parent_route };
    let route_path = format!("{}/{}", route_path, file_stem);
    let router = build_stream_handler(file_path, "GET");
    println!("✔️ Mapped {} to GET {}", file_name, route_path);

    app.route(&route_path, router, Some(String::from("GET")));
}

fn get_file_content(file_path: &OsString) -> String {
    fs::read_to_string(file_path).unwrap()
}

fn build_stream_handler(
    file_path: OsString,
    method: &str
) -> MethodRouter {
    let handler = move || {
        let file_path = file_path.clone();
        async move {
            // Open the file
            let file = File::open(&file_path).await;

            if file.is_err() {
                return (
                    StatusCode::NOT_FOUND,
                    format!("File not found: {}", file_path.display()),
                ).into_response();
            }

            let file = file.unwrap();

            // Guess MIME type
            let mime_type = from_path(&file_path)
                .first_or_octet_stream()
                .to_string();

            // Stream the file
            let stream = ReaderStream::new(file);
            let body = Body::from_stream(stream);

            // Set headers
            let mut headers = HeaderMap::new();
            headers.insert(CONTENT_TYPE, HeaderValue::from_str(&mime_type).unwrap());
            // headers.insert(
            //     header::CONTENT_DISPOSITION,
            //     HeaderValue::from_str(&format!(
            //         "attachment; filename=\"{}\"",
            //         file_path.to_string_lossy()
            //     ))
            //     .unwrap(),
            // );

            (headers, body).into_response()
        }
    };

    match method.to_uppercase().as_str() {
        "GET" => get(handler),
        "POST" => post(handler),
        "PUT" => put(handler),
        "PATCH" => patch(handler),
        "DELETE" => delete(handler),
        "OPTIONS" => options(handler),
        // Fallback for an unknown method string
        _ => get(|| async { "Unknown method in filename" }),
    }
}

fn get_file_extension(file_path: &OsString) -> String {
    Path::new(file_path)
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or_default()
        .to_string()
}

fn is_text_file(file_path: &OsString) -> bool {
    let extension = get_file_extension(file_path);
    extension == "txt" || extension == "md" || extension == "json"
}

fn content_handler(file_path: OsString, method: &str) -> MethodRouter {
    let file_path = file_path.clone();
    let handler = move || async move { get_file_content(&file_path) };

    match method.to_uppercase().as_str() {
        "GET" => get(handler),
        "POST" => post(handler),
        "PUT" => put(handler),
        "PATCH" => patch(handler),
        "DELETE" => delete(handler),
        "OPTIONS" => options(handler),
        // Fallback for an unknown method string
        _ => get(|| async { "Unknown method in filename" }),
    }
}

// Helper to create a MethodRouter from a string
fn build_method_router(file_path: &OsString, method: &str) -> MethodRouter {
    let file_path = file_path.clone();
    if is_text_file(&file_path) {
        content_handler(file_path, method)
    } else {
        build_stream_handler(file_path, method)
    }
}

fn build_in_memory_routes(app: &mut App, route_path: &str, file_path: OsString, id_key: &str, id_type: IdType) {
    let in_memory_collection = InMemoryCollection::new(id_type, id_key.to_string());
    let collection = Arc::new(Mutex::new(in_memory_collection));

    let load_collection = Arc::clone(&collection);
    load_initial_data(file_path, load_collection);

    // Build REST routes for CRUD operations
    // GET /resource - list all
    let list_collection = Arc::clone(&collection);
    let list_router = get(move || {
        async move {
            let list_collection = list_collection.lock().unwrap();
            let items = list_collection.get_all();
            Json(items).into_response()
        }
    });
    app.route(route_path, list_router, Some("GET".to_string()));

    // POST /resource - create new
    let create_collection = Arc::clone(&collection);
    let create_router = post(move |Json(payload): Json<Value>| {
        async move {
            let mut create_collection = create_collection.lock().unwrap();
            match create_collection.add(payload) {
                Some(item) => (StatusCode::CREATED, Json(item)).into_response(),
                None => StatusCode::BAD_REQUEST.into_response(),
            }
        }
    });
    app.route(route_path, create_router, Some("POST".to_string()));

    // GET /resource/:id - get by id
    let id_route = format!("{}/{{{}}}", route_path, id_key);
    let get_collection = Arc::clone(&collection);
    let get_router = get(move |AxumPath(id): AxumPath<String>| {
        async move {
            let get_collection = get_collection.lock().unwrap();
            match get_collection.get(&id) {
                Some(item) => Json(item).into_response(),
                None => StatusCode::NOT_FOUND.into_response(),
            }
        }
    });
    app.route(&id_route, get_router, Some("GET".to_string()));

    // PUT /resource/:id - update by id
    let update_collection = Arc::clone(&collection);
    let put_router = put(move |AxumPath(id): AxumPath<String>, Json(payload): Json<Value>| {
        async move {
            let mut update_collection = update_collection.lock().unwrap();
            match update_collection.update(&id, payload) {
                Some(item) => Json(item).into_response(),
                None => StatusCode::NOT_FOUND.into_response(),
            }
        }
    });
    app.route(&id_route, put_router, Some("PUT".to_string()));

    // PATCH /resource/:id - partial update by id
    let patch_collection = Arc::clone(&collection);
    let patch_router = patch(move |AxumPath(id): AxumPath<String>, Json(payload): Json<Value>| {
        async move {
            let mut patch_collection = patch_collection.lock().unwrap();
            match patch_collection.update_partial(&id, payload) {
                Some(item) => Json(item).into_response(),
                None => StatusCode::NOT_FOUND.into_response(),
            }
        }
    });
    app.route(&id_route, patch_router, Some("PATCH".to_string()));

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
    app.route(&id_route, delete_router, Some("DELETE".to_string()));

    println!("✔️ Built REST routes for {}", route_path);
}

fn load_initial_data(file_path: OsString, load_collection: Arc<Mutex<InMemoryCollection>>) {
    // Try to read the file content
    if let Ok(file_content) = fs::read_to_string(&file_path) {
        // Try to parse the content as JSON
        if let Ok(json_value) = serde_json::from_str::<Value>(&file_content) {
            // Check if it's a JSON Array
            if let Value::Array(_) = json_value {
                // Load the array into the collection using add_batch
                let mut collection = load_collection.lock().unwrap();
                let added_items = collection.add_batch(json_value);
                println!("✔️ Loaded {} initial items from {}", added_items.len(), file_path.to_string_lossy());
            } else {
                println!("⚠️ File {} does not contain a JSON array, skipping initial data load", file_path.to_string_lossy());
            }
        } else {
            println!("⚠️ File {} does not contain valid JSON, skipping initial data load", file_path.to_string_lossy());
        }
    } else {
        println!("⚠️ Could not read file {}, skipping initial data load", file_path.to_string_lossy());
    }
}
