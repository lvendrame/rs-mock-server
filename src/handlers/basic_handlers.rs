use std::{ffi::OsString, fs, sync::Arc};

use axum::{
    body::Body,
    extract::{FromRequestParts, Path as AxumPath, Request},
    http::StatusCode, response::IntoResponse,
    routing::{delete, get, options, patch, post, put, MethodRouter}
};
use http::{header::CONTENT_TYPE, HeaderMap, HeaderValue};
use jgd_rs::generate_jgd_from_file;
use mime_guess::from_path;
use serde_json::json;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

use crate::{app::App, handlers::{is_jgd, is_sql, is_text_file}};

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

pub fn content_handler(app: &mut App, file_path: OsString, method: &str) -> MethodRouter {
    let file_path = file_path.clone();
    let db = Arc::clone(&app.db);

    let handler = move |req: Request| {
        let file_path = file_path.clone();
        async move {
            if is_jgd(&file_path) {
                let json = generate_jgd_from_file(&file_path.into());
                serde_json::to_string_pretty(&json).unwrap().into_response()
            } else if is_sql(&file_path) {
                let sql = fs::read_to_string(file_path).unwrap();
                let (mut req_parts, _req_body) = req.into_parts();
                let response = match AxumPath::<String>::from_request_parts(&mut req_parts, &()).await {
                    Ok(AxumPath(id)) => db.query_with_args(&sql, json!(id)),
                    Err(_) => db.query(&sql),
                };
                match response {
                    Ok(response) => serde_json::to_string_pretty(&response).unwrap().into_response(),
                    Err(_) => StatusCode::BAD_REQUEST.into_response(),
                }

            } else {
                get_file_content(&file_path).into_response()
            }
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

// Helper to create a MethodRouter from a string
pub fn build_method_router(app: &mut App, file_path: &OsString, method: &str) -> MethodRouter {
    let file_path = file_path.clone();
    if is_text_file(&file_path) {
        content_handler(app, file_path, method)
    } else {
        build_stream_handler(file_path, method)
    }
}
