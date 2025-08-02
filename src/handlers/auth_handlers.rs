use std::{ffi::OsString, fs, sync::{Arc}};

use axum::{response::{IntoResponse, Response}, routing::post, Json};
use http::{StatusCode, HeaderValue};
use serde_json::Value;
use jsonwebtoken::{encode, Header, EncodingKey};
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};

use crate::{
    app::App,
    handlers::in_memory_collection::{InMemoryCollection, ProtectedMemCollection}, id_manager::IdType
};

static USERNAME_FIELD: &str = "username";
static PASSWORD_FIELD: &str = "password";
static JWT_SECRET: &str = "your-secret-key"; // In production, use environment variable

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,      // Subject (user identifier)
    username: String, // Username
    exp: i64,         // Expiration time
    iat: i64,         // Issued at
}

#[derive(Serialize)]
struct AuthResponse {
    token: String,
    user: Value,
}

fn try_load_users(file_path: OsString, load_collection: &ProtectedMemCollection) -> bool {
    // Try to read the file content
    if let Ok(file_content) = fs::read_to_string(&file_path) {
        // Try to parse the content as JSON
        if let Ok(json_value) = serde_json::from_str::<Value>(&file_content) {
            // Check if it's a JSON Array
            if let Value::Array(_) = json_value {
                // Load the array into the collection using add_batch
                let mut collection = load_collection.lock().unwrap();
                let added_items = collection.add_batch(json_value);
                println!("✔️ Loaded {} credentials from {}", added_items.len(), file_path.to_string_lossy());
                return true;
            }

            eprintln!("⚠️ File {} does not contain a JSON array, skipping initial data load", file_path.to_string_lossy());
            return false;
        }
        eprintln!("⚠️ File {} does not contain valid JSON, skipping initial data load", file_path.to_string_lossy());
        return false;
    }

    eprintln!("⚠️ Could not read file {}, skipping initial data load", file_path.to_string_lossy());
    false
}

fn try_get_auth_info(payload: Value) -> Option<(String, String)> {
    if let Some(Value::String(username)) = payload.get(USERNAME_FIELD.to_string()) {
        if let Some(Value::String(password)) = payload.get(PASSWORD_FIELD.to_string()) {
            return Some((username.clone(), password.clone()));
        }
    }
    None
}

fn check_password(item: &Value, password: String) -> bool {
    if let Some(Value::String(user_pass)) = item.get(PASSWORD_FIELD) {
        return password == *user_pass;
    }
    false
}

fn generate_token(item: Value) -> Response<axum::body::Body> {
    // Extract username from the user data
    let username = item.get(USERNAME_FIELD)
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    // Extract user ID (could be from 'id' or '_id' field)
    let user_id = item.get("id")
        .or_else(|| item.get("_id"))
        .and_then(|v| v.as_str())
        .unwrap_or(username); // Fallback to username if no ID found

    // Create JWT claims
    let now = Utc::now();
    let expiration = now + Duration::hours(24); // Token expires in 24 hours

    let claims = Claims {
        sub: user_id.to_string(),
        username: username.to_string(),
        exp: expiration.timestamp(),
        iat: now.timestamp(),
    };

    // Generate JWT token
    let token = match encode(&Header::default(), &claims, &EncodingKey::from_secret(JWT_SECRET.as_ref())) {
        Ok(token) => token,
        Err(err) => {
            eprintln!("⚠️ Failed to generate JWT token: {}", err);
            return Json(serde_json::json!({
                "error": "Failed to generate authentication token"
            })).into_response();
        }
    };

    // Create response with token and user info (excluding password)
    let mut user_data = item.clone();
    if let Some(obj) = user_data.as_object_mut() {
        obj.remove(PASSWORD_FIELD); // Remove password from response
    }

    let response = AuthResponse {
        token: token.clone(),
        user: user_data,
    };

    // Create cookie with JWT token
    let cookie_value = format!(
        "auth_token={}; HttpOnly; Secure; SameSite=Strict; Max-Age=86400; Path=/",
        token
    );

    // Build response with cookie header
    let json_response = Json(serde_json::to_value(response).unwrap_or_else(|_| serde_json::json!({
        "error": "Failed to serialize response"
    })));

    let mut response = json_response.into_response();

    // Add Set-Cookie header
    if let Ok(cookie_header) = HeaderValue::from_str(&cookie_value) {
        response.headers_mut().insert("Set-Cookie", cookie_header);
    } else {
        eprintln!("⚠️ Failed to create cookie header");
    }

    response
}

fn create_login_route(app: &mut App, route_path: &str, collection: &ProtectedMemCollection) {
    // POST /resource - auth
    let auth_collection = Arc::clone(collection);
    let create_router = post(move |Json(payload): Json<Value>| {
        async move {
            if let Some((username, password)) = try_get_auth_info(payload) {
                let auth_collection = auth_collection.lock().unwrap();

                return match auth_collection.get(&username) {
                    Some(item) => if check_password(&item, password) {
                        (StatusCode::OK, generate_token(item)).into_response()
                    } else {
                        StatusCode::UNAUTHORIZED.into_response()
                    },
                    None => StatusCode::BAD_REQUEST.into_response(),
                }
            }

            StatusCode::BAD_REQUEST.into_response()
        }
    });
    app.route(route_path, create_router, Some("POST"));
}

pub fn build_auth_routes(app: &mut App, route_path: &str, file_path: OsString) {
    let in_memory_collection = InMemoryCollection::new(IdType::Uuid, USERNAME_FIELD.to_string());
    let collection = in_memory_collection.into_protected();

    if !try_load_users(file_path, &collection) {
        return;
    }

    create_login_route(app, route_path, &collection);

    println!("✔️ Built AUTH routes for {}", route_path);
}