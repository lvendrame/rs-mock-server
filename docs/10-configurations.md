# Configuration Guide

This document explains how to customize the mock server and its routes using TOML files. Configurations are loaded in three layers (from widest to most specific):

1. **Server-level**: `rs-mock-server.toml` in the execution path applies globally.
2. **Directory-level**: `config.toml` in any folder applies to all child routes.
3. **Route-level**: method- or route-specific TOML files alongside mocks.

---

## 1. Server-Level Configuration

Place a file named `rs-mock-server.toml` in the current working directory before starting the server. Only the `[server]` and `[route]` tables are supported here; omitted settings use defaults.

Example `rs-mock-server.toml`:

```toml
 [server]
 port = 8080           # listening port
 folder = "./mocks"    # mocks directory
 enable_cors = true    # allow CORS requests
 allowed_origin = "*"  # CORS origin

 [route]
 delay = 50            # artificial delay (ms)
 remap = "/v1"         # route prefix
 protect = false       # require auth by default
```

Omitted sections fall back to default behavior documented elsewhere.

---

## 2. Directory-Level Configuration

To override defaults for all routes under a given folder, add a file named `config.toml` inside that directory. Any settings in this file will apply to child routes, unless overridden further by route-level configs.
Only protect and delay configurations were inherited

Example folder structure:

```
mocks/
└── api/
    ├── config.toml      # applies to /api/*
    └── users/
        ├── get.json
        └── get.toml     # route-level override
```

Example `mocks/api/config.toml`:

```toml
[route]
protect = true  # require auth on all /api/* routes
delay = 300     # 300 milliseconds of response delay
```

---

## 3. Route-Level Configuration

For fine-grained control of a single endpoint, place a TOML file next to the mock file using the same base name. Valid tables vary by route type:

### Generic Routes

For standard endpoints (e.g., `get.json`, `post.json`), only the `[route]` table are supported.

Example `get.toml` with all fields:

```toml
[route]
delay = 100                  # artificial delay in milliseconds
remap = "/api/new-path".     # rewrite path. It will rewrite the whole path, so be aware about collision names and use it carefully
protect = true               # require authentication for this route
```

### Authentication Routes

For `{auth}.json`, only the `[route]` and `[auth]` tables are supported.

Example `{auth}.toml`:

```toml
[route]
delay = 0                    # no artificial delay
remap = "/accounts"          # no path rewrite
protect = true               # always protected

[auth]
username_field = "username"  # field name for login
password_field = "password"  # field name for password
roles_field = "roles"        # field name for user roles
cookie_name = "auth_token"   # name of the auth cookie
encrypt_password = false     # store passwords as plain text
jwt_secret = "super-secret"  # secret for signing JWTs
# Routes for login/logout and user management
login_endpoint = "/signin"     # login endpoint path suffix
logout_endpoint = "/signout"   # logout endpoint path suffix
users_route = "/users"         # users REST route
# Nested collection settings (optional)
[auth.token_collection]
name = "tokens"              # collection name for tokens
id_key = "token"             # identifier field for tokens
id_type = "Uuid"             # token ID generation
[auth.user_collection]
name = "users"               # collection name for users
id_key = "id"                # identifier field for users
id_type = "Uuid"             # user ID generation
```

### Upload Routes

For upload folders (`{upload}`), only the `[route]` and `[upload]` tables are supported.

Example `{upload}.toml`:

```toml
[route]
delay = 0                    # no delay on upload endpoints
protect = false              # public by default
remap = "/manage-files"

[upload]
upload_endpoint = "/upload"        # endpoint for upload a file
download_endpoint = "/download"    # endpoint for download a file
list_files_endpoint = "/files"     # endpoint to list uploads
temporary = true                   # delete files on server shutdown
```

### REST API Routes

For `rest.json` or `rest.jgd`, only the `[route]` and `[collection]` tables are supported.

Example `rest.toml`:

```toml
[route]
delay = 200            # extra delay for CRUD operations
remap = "/v1/product"  # no prefix
protect = false        # public REST API

[collection]
name = "products"      # collection name
id_key = "_id"         # custom id field
id_type = "Uuid"       # use UUIDs for new items
```

---

### Loading Order and Overrides

1. Global `rs-mock-server.toml` (lowest priority)
2. `config.toml` in each directory (applied recursively)
3. Route-level `{method}.toml` files (highest priority)

Each layer merges with the previous one, so you only need to specify the fields you want to change.

---

For more details on individual settings, see `src/route_builder/config.rs` and its struct documentation.
