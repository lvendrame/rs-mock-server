use std::{ffi::OsString, fs::{self, DirEntry}, path::Path};

use axum::{
    body::Body, response::IntoResponse, routing::{delete, get, options, patch, post, put, MethodRouter}
};
use http::{header::CONTENT_TYPE, HeaderMap, HeaderValue, StatusCode};
use mime_guess::from_path;
use once_cell::sync::Lazy;
use regex::Regex;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

use crate::app::App;

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

    static RE_PARAM: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^(get|post|put|patch|delete|options)(\{(.+)\})?$").unwrap()
    });

    let file_path = entry.path().into_os_string();

    if let Some(captures) = RE_PARAM.captures(file_stem) {
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
