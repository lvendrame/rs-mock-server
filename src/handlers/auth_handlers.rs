use std::{ffi::OsString, pin::Pin, sync::Arc};

use axum::{body::Body, extract::Request, middleware::Next, response::{IntoResponse, Response}, routing::post, Json};
use http::{StatusCode, HeaderValue};
use serde_json::Value;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};

use crate::{
    app::App, handlers::build_rest_routes, memory_db::{constraint::{Comparer, Constraint}, id_manager::IdType, memory_collection::ProtectedMemCollection, CollectionConfig, DbCollection, DbProtectedExt}
};

static ID_FIELD: &str = "id";
static USERNAME_FIELD: &str = "username";
static PASSWORD_FIELD: &str = "password";
static TOKEN_FIELD: &str = "token";
static AUTH_TOKEN_FIELD: &str = "auth_token";
static JWT_SECRET: &str = "1!2@3#4$5%6â7&8*9(0)-_=+±§";

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

fn generate_token(item: &Value, auth_collection: &ProtectedMemCollection) -> Response<axum::body::Body> {
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
        user: user_data.clone(),
    };

    {
        let mut user_data = user_data.clone();
        if let Some(obj) = user_data.as_object_mut() {
            obj.insert(TOKEN_FIELD.to_string(), Value::String(token.clone())); // add token
        }
        let mut auth_collection = auth_collection.write().unwrap();
        auth_collection.add(user_data);
    }

    // Create cookie with JWT token
    let cookie_value = format!(
        "{}={}; HttpOnly; Secure; SameSite=Strict; Max-Age=86400; Path=/",
        AUTH_TOKEN_FIELD, token
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

pub fn create_login_route(
    app: &mut App,
    route_path: &str,
    users_collection: &ProtectedMemCollection,
    auth_collection: &ProtectedMemCollection,
) {
    let login_route = format!("{}/login", route_path);

    // POST /resource/login - auth
    let user_collection = Arc::clone(users_collection);
    let auth_collection = Arc::clone(auth_collection);
    let create_router = post(move |Json(payload): Json<Value>| {
        async move {
            if let Some((username, password)) = try_get_auth_info(payload) {
                let user_collection = user_collection.read().unwrap();

                let criteria = Constraint::try_new(USERNAME_FIELD.to_string(), Comparer::Equal, Some(Value::String(username.clone())));
                if criteria.is_err() {
                    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                }

                let users = user_collection.get_from_criteria(criteria.unwrap());
                if users.is_empty() {
                    return StatusCode::UNAUTHORIZED.into_response();
                }

                return match users.first() {
                    Some(item) => if check_password(item, password) {
                        (StatusCode::OK, generate_token(item, &auth_collection)).into_response()
                    } else {
                        StatusCode::UNAUTHORIZED.into_response()
                    },
                    None => StatusCode::UNAUTHORIZED.into_response(),
                }
            }

            StatusCode::BAD_REQUEST.into_response()
        }
    });
    app.route(&login_route, create_router, Some("POST"), None);
}

pub fn build_auth_routes(app: &mut App, route_path: &str, file_path: &OsString) {
    println!("Starting loading Auth route");

    let auth_collection = app.db.create(CollectionConfig::none(TOKEN_FIELD, "{{auth}}-tokens"));

    // The app.auth_collection should be set before to load the rest routes
    app.auth_collection = Some(Arc::clone( &auth_collection));

    let users_routes = format!("{}/users", route_path);
    let users_collection = build_rest_routes(app, &users_routes, file_path, ID_FIELD, IdType::None, true);
    println!("✔️ Built REST routes for {}", users_routes);

    if users_collection.count() == 0 {
        app.auth_collection = None;
        return eprintln!("⚠️ Authentication routes were not created")
    }

    create_login_route(app, route_path, &users_collection, &auth_collection);
    create_logout_route(app, route_path, &auth_collection);

}

fn decode_jwt(jwt_token: String) -> Result<TokenData<Claims>, StatusCode> {
    let result: Result<TokenData<Claims>, StatusCode> = decode(
        &jwt_token,
        &DecodingKey::from_secret(JWT_SECRET.as_ref()),
        &Validation::default(),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);
    result
}

fn extract_token_from_request(req: &Request) -> Option<String> {
    // Try to get token from Authorization header first
    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                return Some(token.to_string());
            }
        }
    }

    // Try to get token from cookies if not found in header
    if let Some(cookie_header) = req.headers().get("Cookie") {
        if let Ok(cookie_str) = cookie_header.to_str() {
            for cookie in cookie_str.split(';') {
                let cookie = cookie.trim();
                if let Some((name, value)) = cookie.split_once('=') {
                    if name.trim() == AUTH_TOKEN_FIELD {
                        return Some(value.trim().to_string());
                    }
                }
            }
        }
    }

    None
}

// Simple authorization middleware function that can be used with axum::middleware::from_fn
pub async fn authorization_middleware(
    req: Request,
    next: Next,
) -> Result<Response<Body>, StatusCode> {
    // This is a simplified version that only validates JWT tokens
    // For token revocation, you'd need to use the factory function below

    // Extract token from request
    let token = match extract_token_from_request(&req) {
        Some(token) => token,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    // Decode and validate JWT token
    let _token_data = match decode_jwt(token) {
        Ok(data) => data,
        Err(status) => return Err(status),
    };

    // Token is valid, continue with the request
    let response = next.run(req).await;
    Ok(response)
}

type AuthMiddlewareReturn = Pin<Box<dyn std::future::Future<Output = Result<Response<Body>, StatusCode>> + Send + 'static>>;

// For when you need access to the auth collection (token revocation)
pub fn make_auth_middleware(
    auth_collection: &ProtectedMemCollection,
) -> impl Clone + Send + Sync + 'static + Fn(Request, Next) -> AuthMiddlewareReturn {
    let auth_collection = Arc::clone(auth_collection);
    move |req: Request, next: Next| {
        let auth_collection = Arc::clone(&auth_collection);
        Box::pin(async move {
            // Extract token from request
            let token = match extract_token_from_request(&req) {
                Some(token) => token,
                None => return Err(StatusCode::UNAUTHORIZED),
            };

            // Decode and validate JWT token
            let _token_data = match decode_jwt(token.clone()) {
                Ok(data) => data,
                Err(status) => return Err(status),
            };

            // Check if token exists in auth_collection (for token revocation/blacklisting)
            {
                let auth_collection = auth_collection.read().unwrap();
                if !auth_collection.exists(&token) {
                    return Err(StatusCode::UNAUTHORIZED);
                }
            }

            // Token is valid, continue with the request
            let response = next.run(req).await;
            Ok(response)
        })
    }
}

pub fn create_logout_route(
    app: &mut App,
    route_path: &str,
    auth_collection: &ProtectedMemCollection,
) {
    let logout_route = format!("{}/logout", route_path);

    let mut auth_collection = Arc::clone(auth_collection);
    let logout_router = post(move |req: Request| {
        async move {
            // Extract token from request
            let token = match extract_token_from_request(&req) {
                Some(token) => token,
                None => return StatusCode::UNAUTHORIZED.into_response(),
            };

            // Remove token from auth collection (logout/revoke)
            auth_collection.delete(&token);

            Json(serde_json::json!({
                "message": "Successfully logged out"
            })).into_response()
        }
    });

    app.route(&logout_route, logout_router, Some("POST"), None);
}
