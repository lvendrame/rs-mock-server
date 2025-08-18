use std::{ffi::OsString, fs};

use axum::{
    body::Body,
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, options, patch, post, put, MethodRouter}
};
use http::{header::{CONTENT_TYPE}, HeaderMap, HeaderValue};
use jgd_rs::generate_jgd_from_file;
use mime_guess::{from_path};
use tokio::fs::File;
use tokio_util::io::ReaderStream;

use crate::handlers::{is_jgd, is_text_file};

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

pub fn content_handler(file_path: OsString, method: &str) -> MethodRouter {
    let file_path = file_path.clone();
    let handler = move || {
        let file_path = file_path.clone();
        async move {
            if is_jgd(&file_path) {
                let json = generate_jgd_from_file(&file_path.into());
                serde_json::to_string_pretty(&json).unwrap()
            } else {
                get_file_content(&file_path)
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
pub fn build_method_router(file_path: &OsString, method: &str) -> MethodRouter {
    let file_path = file_path.clone();
    if is_text_file(&file_path) {
        content_handler(file_path, method)
    } else {
        build_stream_handler(file_path, method)
    }
}
