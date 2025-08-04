use std::{fs, path::Path};

use axum::{
    extract::{Json, Multipart, Path as AxumPath}, http::StatusCode, response::IntoResponse, routing::{get, post}
};
use http::{header::{CONTENT_DISPOSITION, CONTENT_TYPE}, HeaderMap, HeaderValue};
use mime_guess::{from_path};
use serde_json::Value;

use crate::{app::App};

fn create_upload_route(app: &mut App, upload_path: String, route: &str) {
    let uploads_route = route.to_string();

    // POST /uploads - create new
    let uploads_router = post(async move |mut multipart: Multipart| {
        let mut file_name = "".to_string();

        while let Some(field) = multipart.next_field().await.unwrap() {
            let field_name = field.name().unwrap_or("file").to_string();
            file_name = field.file_name()
                .map(|name| name.to_string())
                .unwrap_or_else(|| "uploaded_file.bin".to_string());

            let data = field.bytes().await.unwrap();

            println!("Received file '{}' in field '{}' with {} bytes", file_name, field_name, data.len());

            // Save the file with its original name
            let file_path = format!("{}/{}", upload_path, file_name);
            tokio::fs::write(&file_path, &data).await.unwrap();
        }
        let response = Value::Object({
            let mut map = serde_json::Map::new();
            map.insert("status".to_string(), Value::String("success".to_string()));
            map.insert("message".to_string(), Value::String("File uploaded successfully".to_string()));
            map.insert("filename".to_string(), Value::String(file_name.clone()));
            map.insert("filepath".to_string(), Value::String(format!("{}/{}", uploads_route, file_name) ));
            map
        });

        Json(response).into_response()
    });

    app.route(route, uploads_router, Some("POST"));
}

fn create_download_route(app: &mut App, download_path: String, route: &str) {
    let download_route = format!("{}/{{file_name}}", route);

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

fn create_uploaded_list_route(app: &mut App, upload_path: String, route: &str) {
    let upload_list_route = route.to_string();

    // GET /uploads/{filename} - download file
    let upload_list_router = get(move || {
        async move {
            let upload_path = Path::new(&upload_path);

            // Check if file exists
            if !upload_path.exists() {
                return StatusCode::NOT_FOUND.into_response();
            }

            let entries = fs::read_dir(upload_path).unwrap();
            let array = entries.map(|entry| {
                let value = format!("{}/{}",
                    upload_list_route,
                    entry.unwrap().file_name().to_string_lossy()
                );

                Value::String(value)
            }).collect();

            let body = Value::Array(array);

            Json(body).into_response()
        }
    });

    app.route(route, upload_list_router, Some("GET"));
}

pub fn build_upload_routes(app: &mut App, path: String, route: &str) {
    create_upload_route(app, path.clone(), route);

    create_download_route(app, path.clone(), route);

    create_uploaded_list_route(app, path.clone(), route);
}
