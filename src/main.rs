use std::{ffi::OsString, fs::{self, DirEntry}};

use axum::{
    body::{Body},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse},
    routing::{delete, get, options, patch, post, put, MethodRouter},
    Router
};
use mime_guess::from_path;
use once_cell::sync::Lazy;
use regex::Regex;

use clap::Parser;
use tokio::fs::File;
use tokio_util::io::ReaderStream;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, normalize_path::NormalizePathLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::pages::Pages;

pub mod pages;

/// rs-mock-server is a simple mock server for testing APIs.
/// It serves static files as API responses based on their filenames and directory structure.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Port to run the server on
    #[arg(short, long, default_value_t = 4520,)]
    port: i32,

    /// Directory to load mock files from
    #[arg(short, long, default_value = "mocks")]
    folder: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_testing=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Args::parse();

    // build our application with a single route
    let app = Router::new();

    let mut pages = Pages::new();

    let app = load_mock_dir(app, &args.folder, &mut pages);

    let index = pages.render_index();

    let app = app
        .route("/", get(|| async {
            let body = index;
            let mut headers = HeaderMap::new();
            headers.insert(header::CONTENT_TYPE, HeaderValue::from_str("text/html").unwrap());

            (headers, body).into_response()
        }));

    let home = String::from(pages.home_template);

    let app = app
        .route("/home-api", get(|| async {
            let body = home;
            let mut headers = HeaderMap::new();
            headers.insert(header::CONTENT_TYPE, HeaderValue::from_str("text/html").unwrap());

            (headers, body).into_response()
        }));

    let app = app.fallback(handler_404);

    let app = app.layer(ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::very_permissive())
        .layer(NormalizePathLayer::trim_trailing_slash())
    );

    let address = format!("0.0.0.0:{}", args.port);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(address.clone()).await.unwrap();
    println!("🚀 RS-Mock Server listening on {}", address);
    axum::serve(listener, app).await.unwrap();
}

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "nothing to see here")
}

fn load_mock_dir(mut app: Router, mock_dir: &str, pages: &mut Pages) -> Router {
    let entries = fs::read_dir(mock_dir).unwrap();

    for entry in entries {
        let entry = entry.unwrap();
        app = load_entry(app, "", &entry, pages);
    }

    app
}

fn load_entry(mut app: Router, parent_route: &str, entry: &DirEntry, pages: &mut Pages) -> Router {
    let binding = entry.file_name();
    let path = binding.to_string_lossy();
    let full_route = format!("{}/{}", parent_route, path);

    if entry.file_type().unwrap().is_dir() {
        // If it's a directory, recursively load its contents
        let entries = fs::read_dir(entry.path()).unwrap();
        for entry in entries {
            let entry = entry.unwrap();
            app =load_entry(app, &full_route, &entry, pages);
        }
    } else if entry.file_type().unwrap().is_file() &&  !entry.file_name().to_string_lossy().starts_with(".") {
        // If it's a file, read its contents and register the route
        app = load_file_route(app, parent_route, entry, pages);
    }
    app
}

// Routes examples:
// /mocks
// /mocks/login/get.json,post.json,delete.json,put.json,patch.json
//
// get.json => /
// get{id}.json => /asd,/123,/456
// get{123}.json => /123
// get{1-5}.json => /1, /2, /3, /4, /5
fn load_file_route(mut app: Router, parent_route: &str, entry: &DirEntry, pages: &mut Pages) -> Router {
    let binding = entry.file_name();
    let file_name = binding.to_string_lossy();
    let file_stem = file_name.split('.').next().unwrap_or("");

    static RE_PARAM: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^(get|post|put|patch|delete|options)\{(.+)\}$").unwrap()
    });

    static RE_METHOD: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^(get|post|put|patch|delete|options)$").unwrap()
    });

    let file_path = entry.path().into_os_string();

    if let Some(captures) = RE_PARAM.captures(file_stem) {
        let method = captures.get(1).unwrap().as_str();
        let pattern = captures.get(2).unwrap().as_str();

        // Pattern 1: get{id}.json -> Wildcard route /path/:param
        if pattern == "id" {
            let route_path = format!("{}/{}", parent_route, "{id}");
            let router = build_method_router(method, &file_path);
            println!("✔️ Mapped {} to {} {}", file_name, method.to_uppercase(), &route_path);
            pages.links.push(route_path.clone());
            return app.route(&route_path, router);
        }

        // Pattern 2: get{1-5}.json -> Range of static routes /path/1, /path/2, ...
        if pattern.contains('-') {
            if let Some((start_str, end_str)) = pattern.split_once('-') {
                if let (Ok(start), Ok(end)) = (start_str.parse::<i32>(), end_str.parse::<i32>()) {
                    for i in start..=end {
                        let route_path = format!("{}/{}", parent_route, i);
                        let router = build_method_router(method, &file_path);
                        pages.links.push(route_path.clone());
                        app = app.route(&route_path, router);
                    }
                    println!("✔️ Mapped {} to {} {}/[{}-{}]", file_name, method.to_uppercase(), parent_route, start, end);
                    return app;
                }
            }
        }

        // Pattern 3: get{123}.json -> A single static route /path/123
        let route_path = format!("{}/{}", parent_route, pattern);
        let router = build_method_router(method, &file_path);
        println!("✔️ Mapped {} to {} {}", file_name, method.to_uppercase(), &route_path);

        pages.links.push(route_path.clone());
        app.route(&route_path, router)

    } else if let Some(captures) = RE_METHOD.captures(file_stem) {
        // Default: get.json -> Route on the parent directory /path
        let method = captures.get(1).unwrap().as_str();
        let route_path = if parent_route.is_empty() { "/" } else { parent_route };
        let router = build_method_router(method, &file_path);
        println!("✔️ Mapped {} to {} {}", file_name, method.to_uppercase(), route_path);

        pages.links.push(route_path.into());
        app.route(route_path, router)
    } else {
        let route_path = if parent_route.is_empty() { "/" } else { parent_route };
        let route_path = format!("{}/{}", route_path, file_stem);
        let router = build_handler(file_path);
        println!("✔️ Mapped {} to GET {}", file_name, route_path);

        pages.links.push(route_path.clone());
        app.route(&route_path, router)
    }
}

fn get_file_content(file_path: &OsString) -> String {
    fs::read_to_string(file_path).unwrap()
}

pub fn build_handler(
    file_path: OsString,
) -> MethodRouter {
    get(move || {
        let file_path = file_path.clone();
        async move {
            // Open the file
            let file = match File::open(&file_path).await {
                Ok(file) => file,
                Err(_) => {
                    return (
                        StatusCode::NOT_FOUND,
                        format!("File not found: {}", file_path.display()),
                    )
                        .into_response();
                }
            };

            // Guess MIME type
            let mime_type = from_path(&file_path)
                .first_or_octet_stream()
                .to_string();

            // Stream the file
            let stream = ReaderStream::new(file);
            let body = Body::from_stream(stream);

            // Set headers
            let mut headers = HeaderMap::new();
            headers.insert(header::CONTENT_TYPE, HeaderValue::from_str(&mime_type).unwrap());
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
    })
}

fn get_file_extension(file_path: &OsString) -> String {
    std::path::Path::new(file_path)
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or_default()
        .to_string()
}

fn is_text_file(file_path: &OsString) -> bool {
    let extension = get_file_extension(file_path);
    extension == "txt" || extension == "md" || extension == "json"
}

// Helper to create a MethodRouter from a string
fn build_method_router(method: &str, file_path: &OsString) -> MethodRouter {
    let file_path = file_path.clone();
    let file_path_clone = file_path.clone();
    let handler = move || {
        async move { get_file_content(&file_path) }
    };
    match method.to_lowercase().as_str() {
        "get" => if is_text_file(&file_path_clone) { get(handler) } else{ build_handler(file_path_clone) },
        "post" => post(handler),
        "put" => put(handler),
        "patch" => patch(handler),
        "delete" => delete(handler),
        "options" => options(handler),
        // Fallback for an unknown method string
        _ => get(|| async { "Unknown method in filename" }),
    }
}