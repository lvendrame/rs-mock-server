use std::{ffi::OsString, fs, path::Path, sync::{Arc, Mutex}};

use axum::{
    body::Body, extract::{Json, Multipart, Path as AxumPath}, http::StatusCode, response::IntoResponse, routing::{delete, get, options, patch, post, put, MethodRouter}
};
use http::{header::{CONTENT_DISPOSITION, CONTENT_TYPE}, HeaderMap, HeaderValue};
use mime_guess::from_path;
use serde_json::Value;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

use crate::{app::App, id_manager::IdType, in_memory_collection::InMemoryCollection};

fn get_file_content(file_path: &OsString) -> String {
    fs::read_to_string(file_path).unwrap()
}

pub fn build_stream_handler(
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

pub fn content_handler(file_path: OsString, method: &str) -> MethodRouter {
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
pub fn build_method_router(file_path: &OsString, method: &str) -> MethodRouter {
    let file_path = file_path.clone();
    if is_text_file(&file_path) {
        content_handler(file_path, method)
    } else {
        build_stream_handler(file_path, method)
    }
}

fn load_initial_data(file_path: OsString, load_collection: &Arc<Mutex<InMemoryCollection>>) {
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
                eprintln!("⚠️ File {} does not contain a JSON array, skipping initial data load", file_path.to_string_lossy());
            }
        } else {
            eprintln!("⚠️ File {} does not contain valid JSON, skipping initial data load", file_path.to_string_lossy());
        }
    } else {
        eprintln!("⚠️ Could not read file {}, skipping initial data load", file_path.to_string_lossy());
    }
}

pub fn build_in_memory_routes(app: &mut App, route_path: &str, file_path: OsString, id_key: &str, id_type: IdType) {
    let in_memory_collection = InMemoryCollection::new(id_type, id_key.to_string());
    let collection = Arc::new(Mutex::new(in_memory_collection));

    load_initial_data(file_path, &collection);

    let id_route = format!("{}/{{{}}}", route_path, id_key);

    // Build REST routes for CRUD operations
    create_get_all(app, route_path, &collection);

    create_insert(app, route_path, &collection);

    create_get_item(app, &collection, &id_route);

    create_full_update(app, &collection, &id_route);

    create_partial_update(app, &collection, &id_route);

    create_delete(app, collection, id_route);

    println!("✔️ Built REST routes for {}", route_path);
}

fn create_get_all(app: &mut App, route_path: &str, collection: &Arc<Mutex<InMemoryCollection>>) {
    // GET /resource - list all
    let list_collection = Arc::clone(collection);
    let list_router = get(move || {
        async move {
            let list_collection = list_collection.lock().unwrap();
            let items = list_collection.get_all();
            Json(items).into_response()
        }
    });
    app.route(route_path, list_router, Some("GET"));
}

fn create_insert(app: &mut App, route_path: &str, collection: &Arc<Mutex<InMemoryCollection>>) {
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
    app.route(route_path, create_router, Some("POST"));
}

fn create_get_item(app: &mut App, collection: &Arc<Mutex<InMemoryCollection>>, id_route: &String) {
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
    app.route(id_route, get_router, Some("GET"));
}

fn create_full_update(app: &mut App, collection: &Arc<Mutex<InMemoryCollection>>, id_route: &String) {
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
    app.route(id_route, put_router, Some("PUT"));
}

fn create_partial_update(app: &mut App, collection: &Arc<Mutex<InMemoryCollection>>, id_route: &String) {
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
    app.route(id_route, patch_router, Some("PATCH"));
}

fn create_delete(app: &mut App, collection: Arc<Mutex<InMemoryCollection>>, id_route: String) {
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
    app.route(&id_route, delete_router, Some("DELETE"));
}

pub fn build_upload_routes(app: &mut App, path: String, route: &str) {
    create_upload_route(app, path.clone(), route);

    create_download_route(app, path.clone(), route);
}

fn create_upload_route(app: &mut App, upload_path: String, route: &str) {
    let uploads_route = format!("/{}", route);

    // POST /uploads - create new
    let uploads_router = post(async move |mut multipart: Multipart| {
        while let Some(field) = multipart.next_field().await.unwrap() {
            let field_name = field.name().unwrap_or("file").to_string();
            let file_name = field.file_name()
                .map(|name| name.to_string())
                .unwrap_or_else(|| "uploaded_file.bin".to_string());

            let data = field.bytes().await.unwrap();

            println!("Received file '{}' in field '{}' with {} bytes", file_name, field_name, data.len());

            // Save the file with its original name
            let file_path = format!("{}/{}", upload_path, file_name);
            tokio::fs::write(&file_path, &data).await.unwrap();
        }

        "File uploaded successfully"
    });

    app.route(&uploads_route, uploads_router, Some("POST"));
}

fn create_download_route(app: &mut App, download_path: String, route: &str) {
    let download_route = format!("/{}/{{file_name}}", route);

    // GET /uploads/{filename} - download file
    let download_router = get(move |AxumPath(file_name): AxumPath<String>| {
        async move {
            let file_path = Path::new(&download_path).join(&file_name);

            // Check if file exists
            if !file_path.exists() {
                return StatusCode::NOT_FOUND.into_response();
            }

            // Read file content
            match tokio::fs::read(&file_path).await {
                Ok(contents) => {
                    // Guess MIME type
                    let mime_type = from_path(&file_path)
                        .first_or_octet_stream()
                        .to_string();

                    // Set headers
                    let mut headers = HeaderMap::new();
                    headers.insert(CONTENT_TYPE, HeaderValue::from_str(&mime_type).unwrap());

                    headers.insert(
                        CONTENT_DISPOSITION,
                        HeaderValue::from_str(&format!(
                            "attachment; filename=\"{}\"",
                            file_name
                        ))
                        .unwrap(),
                    );

                    (headers, contents).into_response()
                },
                Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            }
        }
    });

    app.route(&download_route, download_router, Some("GET"));
}

