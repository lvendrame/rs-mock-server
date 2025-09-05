# JWT Authentication

Create a complete JWT-based authentication system with login/logout endpoints and route protection using special `{auth}` files.

## Overview

When the server detects a file named `{auth}.json`, it automatically:

1. **Loads user credentials** from the JSON array in the file
2. **Creates authentication endpoints** for login and logout
3. **Creates REST endpoints** for user
4. **Generates JWT tokens** with secure cookies
5. **Provides middleware** for protecting routes with authentication

## Authentication File Detection

Only **one authentication route is allowed** per server instance. The `{auth}` file creates authentication endpoints based on its folder location:

| File Location                  | Generated Routes                                                            | Description                           |
| :----------------------------- | :-------------------------------------------------------------------------- | :------------------------------------ |
| `./mocks/account/{auth}.json`  | `POST /account/login`<br>`POST /account/logout`<br>`REST /account/users`    | Authentication for account management |
| `./mocks/api/auth/{auth}.json` | `POST /api/auth/login`<br>`POST /api/auth/logout`<br>`REST /api/auth/users` | API authentication endpoints          |
| `./mocks/{auth}.json`          | `POST /login`<br>`POST /logout`<br>`REST /users`                            | Root-level authentication             |

## Credentials File Format

The `{auth}.json` file should contain an array of user objects with `username` and `password` as required fields:

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
    },
    {
        "id": "550e8400-e29b-41d4-a716-446655440003",
        "username": "john.doe",
        "password": "password123",
        "email": "john.doe@example.com",
        "role": "user"
    }
]
```

## Authentication Endpoints

### Login Endpoint - `POST /{folder}/login`

**Request:**

```bash
curl -X POST http://localhost:4520/account/login \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "password": "admin123"}'
```

**Response:**

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

**Features:**

-   **Request**: JSON with `username` and `password`
-   **Response**: JWT token and user info (password excluded)
-   **Cookie**: Sets HTTP-only `auth_token` cookie for 24 hours

### Logout Endpoint - `POST /{folder}/logout`

**Request:**

```bash
curl -X POST http://localhost:4520/account/logout \
  -H "Authorization: Bearer <jwt_token>"
```

**Response:**

```json
{
    "message": "Successfully logged out"
}
```

**Features:**

-   **Request**: JWT token via Authorization header or cookie
-   **Response**: Success message
-   **Action**: Revokes the token from valid tokens list

### Users REST Endpoint

The authentication system also creates a full REST API for user management:

-   **GET** `/{folder}/users` - List all users (protected)
-   **POST** `/{folder}/users` - Create new user (protected)
-   **GET** `/{folder}/users/{username}` - Get user by username (protected)
-   **PUT** `/{folder}/users/{username}` - Update user (protected)
-   **PATCH** `/{folder}/users/{username}` - Partially update user (protected)
-   **DELETE** `/{folder}/users/{username}` - Delete user (protected)

**Configuration:**

-   **IdType**: None
-   **IdKey**: `username`
-   **Protected**: All REST endpoints require authentication token

## Route Protection

To protect routes with authentication, prefix folder names or filenames with `$`:

### Protected Files

```
mocks/
├── api/
│   ├── cities/
│   │   └── $get.json        # Protected: GET /api/cities
│   └── companies/
│       └── get.json         # Public: GET /api/companies
```

### Protected Folders (protects all children)

```
mocks/
├── $admin/                  # All routes under /admin are protected
│   ├── repositories/
│   │   └── rest.json        # Protected: Full CRUD at /admin/repositories/*
│   └── settings/
│       └── get.json         # Protected: GET /admin/settings
└── open/
    └── info.json            # Public: GET /open/info
```

**Note:** It is not possible to protect a `public` folder.

## Authentication Methods

### Option A: Authorization Header

```bash
curl -H "Authorization: Bearer <jwt_token>" \
  http://localhost:4520/admin/repositories
```

### Option B: Cookie (automatic)

```bash
curl -b "auth_token=<jwt_token>" \
  http://localhost:4520/admin/repositories
```

**Note:** When using the web interface, cookies are handled automatically.

## Complete Authentication Flow

### 1. Login

**Request:**

```bash
curl -X POST http://localhost:4520/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "admin",
    "password": "admin123"
  }'
```

**Response:**

```json
{
    "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJhZG1pbiIsImV4cCI6MTY5NzIyNzIwMH0.abc123",
    "user": {
        "id": "550e8400-e29b-41d4-a716-446655440001",
        "username": "admin",
        "email": "admin@example.com",
        "role": "administrator"
    }
}
```

### 2. Access Protected Route

**Request:**

```bash
curl -H "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..." \
  http://localhost:4520/admin/settings
```

**Response:**

```json
{
    "maintenance_mode": false,
    "api_version": "v1.0",
    "features": ["uploads", "auth", "rest"]
}
```

### 3. Logout

**Request:**

```bash
curl -X POST http://localhost:4520/api/auth/logout \
  -H "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..."
```

**Response:**

```json
{
    "message": "Successfully logged out"
}
```

## Security Features

### JWT Tokens

-   **Expiration**: 24-hour automatic expiration
-   **Algorithm**: HS256 signing for security
-   **Claims**: Includes username and expiration time

### HTTP-Only Cookies

-   **XSS Protection**: Prevents client-side JavaScript access
-   **Automatic**: Set on successful login
-   **Secure**: Can be configured for HTTPS environments

### Token Revocation

-   **Active Tracking**: Server maintains list of valid tokens
-   **Logout**: Immediately invalidates tokens
-   **Security**: Prevents token reuse after logout

### Password Protection

-   **Response Filtering**: Login responses exclude password fields
-   **Storage**: Passwords stored in plain text in JSON (for development/testing)
-   **Validation**: Username and password required for login

## Error Responses

### Invalid Credentials

```json
{
    "error": "Invalid credentials"
}
```

**Status:** `401 Unauthorized`

### Missing Token

```json
{
    "error": "Authentication required"
}
```

**Status:** `401 Unauthorized`

### Invalid Token

```json
{
    "error": "Invalid or expired token"
}
```

**Status:** `401 Unauthorized`

### Token Expired

```json
{
    "error": "Token has expired"
}
```

**Status:** `401 Unauthorized`

## User Management Examples

### List Users (Protected)

```bash
curl -H "Authorization: Bearer <token>" \
  http://localhost:4520/api/auth/users
```

### Create User (Protected)

```bash
curl -X POST http://localhost:4520/api/auth/users \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "username": "newuser",
    "password": "newpass123",
    "email": "newuser@example.com",
    "role": "user"
  }'
```

### Update User Password (Protected)

```bash
curl -X PATCH http://localhost:4520/api/auth/users/john.doe \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "password": "newpassword456"
  }'
```

## Integration with Other Features

### REST APIs

Protect your REST APIs by using the `$` prefix:

```
mocks/
├── api/
│   ├── auth/
│   │   └── {auth}.json      # Authentication system
│   ├── $products/
│   │   └── rest.json        # Protected REST API
│   └── public-data/
│       └── rest.json        # Public REST API
```

### File Uploads

Protect upload endpoints:

```
mocks/
├── auth/
│   └── {auth}.json          # Authentication
├── ${upload}/               # Protected uploads
└── {upload}-public/         # Public uploads
```

### Static Files

Protect static content:

```
mocks/
├── auth/
│   └── {auth}.json          # Authentication
├── $admin/
│   └── dashboard.html       # Protected admin dashboard
└── public-assets/
    └── logo.png             # Public assets
```

## Best Practices

1. **Single Auth System**: Use only one `{auth}` file per server instance
2. **Consistent Location**: Place auth files in logical locations (e.g., `/auth`, `/account`)
3. **Strong Passwords**: Use complex passwords even for development
4. **Token Management**: Always logout when done to revoke tokens
5. **Route Organization**: Group protected routes under `$` prefixed folders
6. **Error Handling**: Implement proper error handling for auth failures

## Next Steps

-   Learn about [Route Protection](01-basic-routing.md#route-priority) strategies
-   Explore [File Uploads](04-file-uploads.md) with authentication
-   See [REST APIs](02-rest-apis.md) for protected CRUD operations
-   Try the [Web Interface](07-web-interface.md) for interactive authentication testing
