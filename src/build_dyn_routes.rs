use std::{fs::{self, DirEntry}};
use once_cell::sync::Lazy;
use regex::Regex;

use crate::{
    app::App,
    handlers::{
        build_auth_routes, build_rest_routes, build_method_router, build_stream_handler, build_upload_routes
    },
    id_manager::IdType, utils::try_add_auth_middleware_layer
};

static RE_DIR_UPLOAD: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\{upload\}(\{temp\})?(-.+)?$").unwrap()
});

static RE_FILE_METHODS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(\$)?(get|post|put|patch|delete|options)(\{(.+)\})?$").unwrap()
});

static RE_FILE_REST: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(\$)?(rest)(\{(.+)\})?$").unwrap()
});

static RE_FILE_AUTH: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\{auth\}$").unwrap()
});

pub fn load_mock_dir(app: &mut App) {
    load_dir(app, "", &app.root_path.clone(), false);
}

fn load_dir(app: &mut App, parent_route: &str, entries_path: &str, is_protected: bool) {
    let entries = fs::read_dir(entries_path).unwrap();
    for entry in entries {
        let entry = entry.unwrap();
        load_entry(app, parent_route, &entry, is_protected);
    }
}

fn load_entry(app: &mut App, parent_route: &str, entry: &DirEntry, is_protected: bool) {
    let binding = entry.file_name();
    let binding = binding.to_string_lossy();
    let end_point = binding.replace("$", "");
    let full_route = format!("{}/{}", parent_route, end_point);

    let file_name = end_point;

    if entry.file_type().unwrap().is_dir() {
        let is_protected = is_protected || binding.starts_with("$");

        if file_name.starts_with("public") {
            return app.build_public_router(file_name,String::from(entry.path().to_string_lossy()));
        }

        if let Some(captures) = RE_DIR_UPLOAD.captures(&file_name) {
            let path = entry.path().as_os_str().to_str().unwrap().to_string();
            load_upload_folder(app, path, captures);
            return;
        }

        return load_dir(app, &full_route, entry.path().to_str().unwrap(), is_protected);
    }

    if entry.file_type().unwrap().is_file() &&  !file_name.starts_with(".") {
        // If it's a file, read its contents and register the route
        load_file_route(app, parent_route, entry, is_protected);
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
fn load_file_route(app: &mut App, parent_route: &str, entry: &DirEntry, is_protected: bool) {
    let binding = entry.file_name();
    let file_name = binding.to_string_lossy();
    let file_stem = file_name.split('.').next().unwrap_or("");

    let file_path = entry.path().into_os_string();

    if let Some(captures) = RE_FILE_METHODS.captures(file_stem) {
        let is_protected = is_protected || captures.get(1).is_some();
        let method = captures.get(2).unwrap().as_str();
        let pattern = captures.get(4);

        if let Some(pattern) = pattern {
            let pattern = pattern.as_str();

            // Pattern 1: get{id}.json -> Wildcard route /path/:param
            if pattern == "id" {
                let route_path = format!("{}/{}", parent_route, "{id}");
                let router = build_method_router(&file_path, method);
                let router = try_add_auth_middleware_layer(app, router, is_protected);
                println!("✔️ Mapped {} to {} {}", file_name, method.to_uppercase(), &route_path);
                app.route(&route_path, router, Some(method));
                return;
            }

            // Pattern 2: get{1-5}.json -> Range of static routes /path/1, /path/2, ...
            if pattern.contains('-') {
                if let Some((start_str, end_str)) = pattern.split_once('-') {
                    if let (Ok(start), Ok(end)) = (start_str.parse::<i32>(), end_str.parse::<i32>()) {
                        for i in start..=end {
                            let route_path = format!("{}/{}", parent_route, i);
                            let router = build_method_router(&file_path, method);
                            let router = try_add_auth_middleware_layer(app, router, is_protected);
                            app.route(&route_path, router, Some(method));
                        }
                        println!("✔️ Mapped {} to {} {}/[{}-{}]", file_name, method.to_uppercase(), parent_route, start, end);
                        return;
                    }
                }
            }

            // Pattern 3: get{123}.json -> A single static route /path/123
            let route_path = format!("{}/{}", parent_route, pattern);
            let router = build_method_router(&file_path, method);
            let router = try_add_auth_middleware_layer(app, router, is_protected);
            println!("✔️ Mapped {} to {} {}", file_name, method.to_uppercase(), &route_path);
            app.route(&route_path, router, Some(method));
            return;
        }

        // Default: get.json -> Route on the parent directory /path
        let method = captures.get(2).unwrap().as_str();
        let route_path = if parent_route.is_empty() { "/" } else { parent_route };
        let router = build_method_router(&file_path, method);
        let router = try_add_auth_middleware_layer(app, router, is_protected);
        println!("✔️ Mapped {} to {} {}", file_name, method.to_uppercase(), route_path);

        app.route(route_path, router, Some(method));

        return;
    }

    if let Some(captures) = RE_FILE_REST.captures(file_stem) {
        let is_protected = is_protected || captures.get(1).is_some();
        let descriptor = if let Some(pattern) = captures.get(4) {
            pattern.as_str()
        } else {
            "id:uuid"
        };

        let (id_key, id_type) = get_rest_options(descriptor);
        let route_path = if parent_route.is_empty() { "/" } else { parent_route };

        build_rest_routes(app, route_path, &file_path, id_key, id_type, is_protected);

        return;
    }

    if RE_FILE_AUTH.is_match(file_stem) {
        let route_path = if parent_route.is_empty() { "/" } else { parent_route };

        build_auth_routes(app, route_path, &file_path);
        return;
    }

    let route_path = if parent_route.is_empty() { "/" } else { parent_route };
    let route_path = format!("{}/{}", route_path, file_stem);
    let router = build_stream_handler(file_path, "GET");
    println!("✔️ Mapped {} to GET {}", file_name, route_path);

    app.route(&route_path, router, Some("GET"));
}


fn load_upload_folder(app: &mut App, path: String, captures: regex::Captures<'_>) {
    let is_temp = captures.get(1).is_some();
    let uploads_route = if let Some(route) = captures.get(2) {
        let mut ch = route.as_str().chars();
        ch.next();
        ch.as_str()
    } else {
        "upload"
    };

    app.push_uploads_config(path.clone(), is_temp);

    build_upload_routes(app, path.clone(), uploads_route);

    println!("✔️ Mapped uploads from folder {} to /{}", path, uploads_route);
}
