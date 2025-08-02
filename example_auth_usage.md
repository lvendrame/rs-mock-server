# JWT Authentication Usage

## Overview

The auth_handlers module provides JWT-based authentication with the following features:

1. **Login**: POST endpoint that validates credentials and returns JWT token + cookie
2. **Logout**: POST endpoint that revokes the token
3. **Authorization Middleware**: Validates JWT tokens from Authorization header or cookies

## Setup

Create an auth file at `/mocks/api/auth/login/{auth}.json`:

```json
[
    {
        "id": "550e8400-e29b-41d4-a716-446655440001",
        "username": "admin",
        "password": "admin123",
        "email": "admin@example.com",
        "role": "administrator"
    },
    {
        "id": "550e8400-e29b-41d4-a716-446655440002",
        "username": "user",
        "password": "user123",
        "email": "user@example.com",
        "role": "user"
    }
]
```

## Usage

### 1. Login

```bash
curl -X POST http://localhost:4520/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "password": "admin123"}'
```

Response:

```json
{
    "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
    "user": {
        "id": "550e8400-e29b-41d4-a716-446655440001",
        "username": "admin",
        "email": "admin@example.com",
        "role": "administrator"
    }
}
```

The response also sets an HTTP-only cookie: `auth_token=<jwt_token>`

### 2. Using the Token

**Option A: Authorization Header**

```bash
curl -H "Authorization: Bearer <jwt_token>" http://localhost:4520/protected-route
```

**Option B: Cookie (automatic)**

```bash
curl -b "auth_token=<jwt_token>" http://localhost:4520/protected-route
```

### 3. Logout

```bash
curl -X POST http://localhost:4520/api/auth/logout \
  -H "Authorization: Bearer <jwt_token>"
```

## Middleware Functions

### Simple Middleware (JWT validation only)

```rust
use axum::middleware;

// For routes that just need JWT validation
let protected_routes = Router::new()
    .route("/protected", get(protected_handler))
    .layer(middleware::from_fn(authorization_middleware));
```

### Advanced Middleware (with token revocation)

```rust
use axum::middleware;

// For routes that need token revocation checking
let auth_collection = Arc::new(Mutex::new(HashMap::new()));
let auth_middleware = make_auth_middleware(auth_collection);

let protected_routes = Router::new()
    .route("/protected", get(protected_handler))
    .layer(middleware::from_fn_with_state(state, auth_middleware));
```

## Security Features

-   **JWT Token**: 24-hour expiration with HS256 signing
-   **HTTP-Only Cookie**: Prevents XSS attacks
-   **Token Revocation**: Logout removes token from valid tokens list
-   **Dual Authentication**: Supports both Authorization header and cookies
-   **Password Removal**: Login response excludes password field
