use std::{pin::Pin, sync::Arc};

use axum::{body::Body, extract::Request, middleware::Next, response::{IntoResponse, Response}, routing::post, Json};
use fosk::{DbConfig, DbCollection};
use http::{StatusCode, HeaderValue};
use serde_json::{json, Value};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};

use crate::{
    app::{App, GLOBAL_SHARED_INFO}, handlers::{build_rest_routes, SleepThread}, route_builder::{RouteAuth, RouteRest}
};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,      // Subject (user identifier)
    username: String,
    roles: String,
    exp: i64,         // Expiration time
    iat: i64,         // Issued at
}

#[derive(Serialize)]
struct AuthResponse {
    token: String,
    user: Value,
}

fn try_get_auth_info(payload: Value, username_field: &str, password_field: &str) -> Option<(String, String)> {
    if let Some(Value::String(username)) = payload.get(username_field) {
        if let Some(Value::String(password)) = payload.get(password_field) {
            return Some((username.clone(), password.clone()));
        }
    }
    None
}

fn check_password(item: &Value, password: String, password_field: &str) -> bool {
    if let Some(Value::String(user_pass)) = item.get(password_field) {
        return password == *user_pass;
    }
    false
}

fn generate_token(
    token_collection: Arc<DbCollection>,
    item: &Value,
    auth_def: &RouteAuth,
) -> Response<axum::body::Body> {
    let id_key = &auth_def.token_collection.id_key;
    let username_field = &auth_def.username_field;
    let roles_field = &auth_def.roles_field;
    let jwt_secret = &auth_def.jwt_secret;

    // Extract username from the user data
    let username = item.get(username_field)
        .and_then(|v| v.as_str())
        .unwrap_or("unknown").to_string();

    // Extract user ID (could be from 'id' or '_id' field)
    let user_id = item.get(id_key)
        .or_else(|| item.get("id"))
        .or_else(|| item.get("_id"))
        .and_then(|v| v.as_str() )
        .unwrap_or(&username).to_string(); // Fallback to username if no ID found

    // Extract roles from the user data
    let roles = item.get(roles_field)
        .and_then(|v| v.as_str())
        .unwrap_or("unknown").to_string();

    // Create JWT claims
    let now = Utc::now();
    let expiration = now + Duration::hours(24); // Token expires in 24 hours

    let claims = Claims {
        sub: user_id,
        username,
        roles,
        exp: expiration.timestamp(),
        iat: now.timestamp(),
    };

    // Generate JWT token
    let token = match encode(&Header::default(), &claims, &EncodingKey::from_secret(jwt_secret.as_ref())) {
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
        obj.remove(&auth_def.password_field); // Remove password from response
    }

    let response = AuthResponse {
        token: token.clone(),
        user: user_data.clone(),
    };

    {
        let mut user_data = user_data.clone();
        if let Some(obj) = user_data.as_object_mut() {
            obj.insert(auth_def.token_collection.id_key.to_string(), Value::String(token.clone())); // add token
        }

        token_collection.add(user_data);
    }

    // Create cookie with JWT token
    let cookie_value = format!(
        "{}={}; HttpOnly; Secure; SameSite=Strict; Max-Age=86400; Path=/",
        auth_def.cookie_name, token
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
    auth_def: &RouteAuth
) {
    let login_route = format!("{}{}", auth_def.route, auth_def.login_endpoint);
    let token_collection = auth_def.token_collection.name.clone();
    let user_collection = auth_def.user_collection.name.clone();
    let username_field = auth_def.username_field.clone();
    let password_field = auth_def.password_field.clone();
    let delay = auth_def.delay;

    // POST /resource/login - auth
    let db = app.db.clone();

    let auth_def_clone = auth_def.clone();
    let create_router = post(move |Json(payload): Json<Value>| {
        async move {
            delay.sleep_thread();

            if let Some((username, password)) = try_get_auth_info(payload, &username_field, &password_field) {

                let sql = format!(r#"
                    SELECT * FROM {user_collection}
                    WHERE {username_field} = ? AND {password_field} = ?
                "#);

                let users = db.query_with_args(&sql, json!([username, password]));
                if users.is_err() {
                    return StatusCode::UNAUTHORIZED.into_response();
                }

                let users = users.unwrap();
                if users.is_empty() {
                    return StatusCode::UNAUTHORIZED.into_response();
                }

                return match users.first() {
                    Some(item) => if check_password(item, password, &auth_def_clone.password_field) {
                        let token_collection = db.get(&token_collection).unwrap();
                        (StatusCode::OK, generate_token(token_collection, item, &auth_def_clone)).into_response()
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

fn decode_jwt(jwt_token: &str, jwt_secret: &str) -> Result<TokenData<Claims>, StatusCode> {
    let result: Result<TokenData<Claims>, StatusCode> = decode(
        jwt_token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);
    result
}

fn extract_token_from_request(req: &Request, cookie_name: &str) -> Option<String> {
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
                    if name.trim() == cookie_name {
                        return Some(value.trim().to_string());
                    }
                }
            }
        }
    }

    None
}

type AuthMiddlewareReturn = Pin<Box<dyn std::future::Future<Output = Result<Response<Body>, StatusCode>> + Send + 'static>>;

// For when you need access to the token collection (token revocation)
pub fn make_auth_middleware(
    token_collection: &Arc<DbCollection>,
    jwt_secret: &str,
    cookie_name: &str,
) -> impl Clone + Send + Sync + 'static + Fn(Request, Next) -> AuthMiddlewareReturn {
    let token_collection = Arc::clone(token_collection);
    let jwt_secret = jwt_secret.to_string();
    let cookie_name = cookie_name.to_string();
    move |req: Request, next: Next| {
        let jwt_secret = jwt_secret.to_string();
        let token_collection = Arc::clone(&token_collection);
        let cookie_name = cookie_name.clone();
        Box::pin(async move {
            let token = match extract_token_from_request(&req, &cookie_name) {
                Some(token) => token,
                None => return Err(StatusCode::UNAUTHORIZED),
            };

            let _token_data = match decode_jwt(&token, &jwt_secret) {
                Ok(data) => data,
                Err(status) => return Err(status),
            };

            if !token_collection.exists(&token) {
                return Err(StatusCode::UNAUTHORIZED);
            }

            let response = next.run(req).await;
            Ok(response)
        })
    }
}

pub fn create_logout_route(
    app: &mut App,
    auth_def: &RouteAuth
) {
    let logout_route = format!("{}{}", auth_def.route, auth_def.logout_endpoint);

    let token_collection = app.db.get(&auth_def.token_collection.name).unwrap();
    let cookie_name = auth_def.cookie_name.clone();
    let delay = auth_def.delay;

    let logout_router = post(move |req: Request| {
        async move {
            delay.sleep_thread();

            // Extract token from request
            let token = match extract_token_from_request(&req, &cookie_name) {
                Some(token) => token,
                None => return StatusCode::UNAUTHORIZED.into_response(),
            };

            // Remove token from auth collection (logout/revoke)
            token_collection.delete(&token);

            Json(serde_json::json!({
                "message": "Successfully logged out"
            })).into_response()
        }
    });

    app.route(&logout_route, logout_router, Some("POST"), None);
}

pub fn build_auth_routes(app: &mut App, auth_def: &RouteAuth) {
    println!("Starting loading Auth route");

    let mut shared_info = GLOBAL_SHARED_INFO.write().unwrap();
    shared_info.jwt_secret = auth_def.jwt_secret.clone();
    shared_info.token_collection = auth_def.token_collection.name.clone();
    shared_info.auth_cookie_name = auth_def.cookie_name.clone();
    drop(shared_info);

    // !the Auth collection should be created before the rest endpoints
    app.db.create_with_config(
        &auth_def.token_collection.name,
        DbConfig::from(auth_def.token_collection.id_type, &auth_def.token_collection.id_key));

    let users_routes = auth_def.users_route.clone();

    let rest_config = RouteRest::new(
        users_routes.clone(),
        auth_def.path.clone(),
        auth_def.user_collection.id_key.clone(),
        auth_def.user_collection.id_type,
        true,
        auth_def.user_collection.name.clone(),
        auth_def.delay
    );

    let users_collection = build_rest_routes(
        app,
        &rest_config,
    );

    println!("✔️ Built REST routes for {}", users_routes);

    if users_collection.count() == 0 {
        return eprintln!("⚠️ Authentication routes were not created")
    }

    create_login_route(app, auth_def);
    create_logout_route(app, auth_def);

}
