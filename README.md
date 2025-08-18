# rs-mock-server 🦀

A simple, zero-configuration mock server built in Rust. Spin up a realistic REST API for local development or testing just by creating folders and files.

It works by scanning a directory and mapping its structure directly to API routes, with clever filename conventions for handling HTTP methods, dynamic parameters, and static assets.

---

## Features

-   🚀 **File-System Routing**: Your folder structure defines your API routes. No config files needed.
-   🧩 **Dynamic Path Generation**: Create routes with parameters (`{id}`), specific values (`{admin}`), and even numeric ranges (`{1-10}`) right from the filename.
-   ⚙️ **Full HTTP Method Support**: Define `GET`, `POST`, `PUT`, `DELETE`, `PATCH`, and `OPTIONS` endpoints.
-   🔗 **In-Memory REST API**: Create fully functional CRUD APIs with automatic ID generation and data persistence during runtime using special `rest.json` or `rest.jgd` files.
-   🔐 **JWT Authentication**: Automatic authentication system with login/logout endpoints and route protection using special `{auth}` files.
-   📤 **File Upload & Download**: Create upload endpoints with automatic file handling and download capabilities using special `{upload}` folders.
-   🖼️ **Static File Serving**: Automatically serves any file (like images, CSS, or JS) with its correct `Content-Type` if the filename doesn't match a method pattern.
-   📊 **JGD Support**: Generate dynamic JSON responses using JGD (JSON Generation Definition) files with the [JGD-rs library](https://github.com/lvendrame/jgd-rs/tree/main/jgd-rs) for realistic test data.
-   🌐 **Public Directory Serving**: Serve a directory of static files (e.g., a frontend build) from a root public folder, or map a folder like public-assets to a custom /assets route.
-   🔧 **Configurable**: Easily change the port and mock directory via command-line arguments.
-   ⚡ **Lightweight & Fast**: Built with Rust for minimal resource usage and maximum performance.

---

## How It Works

The server recursively scans a root directory (defaults to `./mocks`) and translates the file and folder paths into API endpoints.

### Folder Structure → URL Path

The path of each folder becomes the base URL for the routes within it.

-   A folder at `./mocks/api/users` creates the base route `/api/users`.
-   A nested folder at `./mocks/api/users/profiles` creates the base route `/api/users/profiles`.

### Filename Conventions → Endpoints

The name of a file determines the **HTTP method** and the **final URL segment**. The content of the file is served as the response body.

The following table shows how different filename patterns are mapped to routes, assuming they are inside a `./mocks/api/users` directory:

| Filename Pattern      | Example File      | Generated Route(s)                                                                                                                             | Description                                                                                                                                                                       |
| :-------------------- | :---------------- | :--------------------------------------------------------------------------------------------------------------------------------------------- | :-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `[method]`            | `get.json`        | `GET /api/users`                                                                                                                               | Creates a route for a standard HTTP method.                                                                                                                                       |
| `[method]{id}`        | `get{id}.json`    | `GET /api/users/{id}`                                                                                                                          | A dynamic segment that accepts any value in that position.                                                                                                                        |
| `[method]{value}`     | `get{admin}.json` | `GET /api/users/admin`                                                                                                                         | Matches a specific, hardcoded value.                                                                                                                                              |
| `[method]{start-end}` | `get{1-5}.json`   | `GET /api/users/1`<br>`GET /api/users/2`<br>...<br>`GET /api/users/5`                                                                          | A numeric range that generates multiple distinct routes.                                                                                                                          |
| `rest[{params}]`      | `rest.json`       | `GET /api/users`<br>`POST /api/users`<br>`GET /api/users/{id}`<br>`PUT /api/users/{id}`<br>`PATCH /api/users/{id}`<br>`DELETE /api/users/{id}` | **In-Memory REST API**.<br>Creates a full CRUD API with automatic ID generation, data persistence,<br>and initial data loading from the JSON array in the file.                   |
| `rest[{params}]`      | `rest.jgd`        | `GET /api/users`<br>`POST /api/users`<br>`GET /api/users/{id}`<br>`PUT /api/users/{id}`<br>`PATCH /api/users/{id}`<br>`DELETE /api/users/{id}` | **In-Memory REST API with JGD**.<br>Creates a full CRUD API with dynamic fake data generation using JGD<br>as initial data, then maintains persistence during runtime.            |
| `{auth}`              | `{auth}.json`     | `POST /api/login`<br>`POST /api/logout`                                                                                                        | **JWT Authentication**. Creates login and logout endpoints with JWT token generation<br>and validation middleware for route protection.                                           |
| `[filename].[ext]`    | `avatar.png`      | `GET /api/users/avatar`                                                                                                                        | **Static File**. Any filename that doesn't match the patterns above is served as a static asset.<br>The `Content-Type` header is automatically set based on the file's extension. |
| `[filename].jgd`      | `users.jgd`       | `GET /api/users/users`                                                                                                                         | **JGD File**. JSON Generation Definition files that dynamically generate realistic JSON data<br>using the [JGD-rs library](https://github.com/lvendrame/jgd-rs/tree/main/jgd-rs). |

### In-Memory REST API

For rapid prototyping and testing, you can create fully functional CRUD APIs using special `rest.json` or `rest.jgd` files. When the server detects a file named `rest.json`, `rest.jgd`, or `rest{params}.json/jgd`, it automatically:

1. **Loads initial data** from the JSON array in the file (for `.json`) or generates fake data using JGD (for `.jgd`)
2. **Creates a complete REST API** with all CRUD operations
3. **Maintains data in memory** during the server's lifetime
4. **Handles ID generation** automatically for new items (except for None ID Type)

#### REST File Naming Convention

The `{params}` in the filename configures the ID field behavior, and the file extension determines the initial data source:

| Filename Pattern      | ID Key | ID Type | Initial Data Source    | Example Usage                                                   |
| :-------------------- | :----- | :------ | :--------------------- | :-------------------------------------------------------------- |
| `rest.json`           | `id`   | UUID    | Static JSON array      | Default configuration with static data                          |
| `rest.jgd`            | `id`   | UUID    | Dynamic JGD generation | Default configuration with generated data                       |
| `rest{none}.json`     | `id`   | None    | Static JSON array      | Explicit None type with static data                             |
| `rest{none}.jgd`      | `id`   | None    | Dynamic JGD generation | Explicit None type with generated data                          |
| `rest{uuid}.json`     | `id`   | UUID    | Static JSON array      | Explicit UUID type with static data                             |
| `rest{uuid}.jgd`      | `id`   | UUID    | Dynamic JGD generation | Explicit UUID type with generated data                          |
| `rest{int}.json`      | `id`   | Integer | Static JSON array      | Integer IDs starting from 1 with static data                    |
| `rest{int}.jgd`       | `id`   | Integer | Dynamic JGD generation | Integer IDs starting from 1 with generated data                 |
| `rest{_id}.json`      | `_id`  | UUID    | Static JSON array      | Custom ID field name with UUID and static data                  |
| `rest{_id}.jgd`       | `_id`  | UUID    | Dynamic JGD generation | Custom ID field name with UUID and generated data               |
| `rest{_id:none}.json` | `_id`  | None    | Static JSON array      | Custom ID field name with explicit None type and static data    |
| `rest{_id:none}.jgd`  | `_id`  | None    | Dynamic JGD generation | Custom ID field name with explicit None type and generated data |
| `rest{_id:uuid}.json` | `_id`  | UUID    | Static JSON array      | Custom ID field name with explicit UUID type and static data    |
| `rest{_id:uuid}.jgd`  | `_id`  | UUID    | Dynamic JGD generation | Custom ID field name with explicit UUID type and generated data |
| `rest{_id:int}.json`  | `_id`  | Integer | Static JSON array      | Custom ID field name with integer type and static data          |
| `rest{_id:int}.jgd`   | `_id`  | Integer | Dynamic JGD generation | Custom ID field name with integer type and generated data       |

#### Generated Endpoints

For a `rest.json` or `rest.jgd` file in `./mocks/api/products/`, the following endpoints are automatically created:

| Method     | Route                | Description                                    |
| :--------- | :------------------- | :--------------------------------------------- |
| **GET**    | `/api/products`      | List all products                              |
| **POST**   | `/api/products`      | Create a new product (auto-generates ID)       |
| **GET**    | `/api/products/{id}` | Get a specific product by ID                   |
| **PUT**    | `/api/products/{id}` | Update an entire product (replaces all fields) |
| **PATCH**  | `/api/products/{id}` | Partially update a product (merges fields)     |
| **DELETE** | `/api/products/{id}` | Delete a product by ID                         |

#### Initial Data Format

The JSON file should contain an array of objects, where each object represents an item with the configured ID field:

```json
[
    {
        "id": "550e8400-e29b-41d4-a716-446655440001",
        "name": "Wireless Headphones",
        "price": 199.99,
        "category": "Electronics"
    },
    {
        "id": "550e8400-e29b-41d4-a716-446655440002",
        "name": "Coffee Mug",
        "price": 15.99,
        "category": "Kitchen"
    }
]
```

For integer IDs using `rest{_id:int}.json`:

```json
[
    {
        "_id": 1,
        "name": "Product One",
        "description": "First product"
    },
    {
        "_id": 2,
        "name": "Product Two",
        "description": "Second product"
    }
]
```

#### JGD REST Files

When using `rest.jgd` files, the server generates dynamic fake data using JGD (JSON Generation Definition) and uses it as initial data for the REST API. This is perfect for creating realistic test data without manually writing JSON arrays.

**Example `rest{_id:int}.jgd` file:**

```jgd
{
  "$format": "jgd/v1",
  "version": "1.0.0",
  "root": {
    "count": 25,
    "fields": {
      "_id": "${index}",
      "name": "${lorem.words(2,3)}",
      "description": "${lorem.sentence(5,12)}",
      "price": {
        "number": {
          "min": 10.99,
          "max": 999.99,
          "integer": false
        }
      },
      "category": "${lorem.word}",
      "in_stock": "${boolean.boolean(80)}",
      "created_at": "${chrono.dateTime}"
    }
  }
}
```

**Example `rest{uuid}.jgd` file:**

```jgd
{
  "$format": "jgd/v1",
  "version": "1.0.0",
  "root": {
    "count": 50,
    "fields": {
      "id": "${uuid.v4}",
      "name": "${name.name}",
      "email": "${internet.safeEmail}",
      "age": {
        "number": {
          "min": 18,
          "max": 80,
          "integer": true
        }
      },
      "address": {
        "fields": {
          "street": "${address.streetName}",
          "city": "${address.cityName}",
          "zipcode": "${address.zipCode}"
        }
      },
      "created_at": "${chrono.dateTime}"
    }
  }
}
```

The generated data becomes the initial dataset for the REST API, and all CRUD operations work normally with persistent in-memory storage during the server's lifetime. For more information about JGD syntax, visit the [JGD-rs documentation](https://github.com/lvendrame/jgd-rs/tree/main/jgd-rs).

### JWT Authentication

For applications requiring user authentication, you can create a complete JWT-based authentication system using special `{auth}` files. When the server detects a file named `{auth}.json`, it automatically:

1. **Loads user credentials** from the JSON array in the file
2. **Creates authentication endpoints** for login and logout
3. **Creates REST endpoints** for user
4. **Generates JWT tokens** with secure cookies
5. **Provides middleware** for protecting routes with authentication

#### Authentication File Detection

Only **one authentication route is allowed** per server instance. The `{auth}` file creates authentication endpoints based on its folder location:

| File Location                  | Generated Routes                                                            | Description                           |
| :----------------------------- | :-------------------------------------------------------------------------- | :------------------------------------ |
| `./mocks/account/{auth}.json`  | `POST /account/login`<br>`POST /account/logout`<br>`REST /account/users`    | Authentication for account management |
| `./mocks/api/auth/{auth}.json` | `POST /api/auth/login`<br>`POST /api/auth/logout`<br>`REST /api/auth/users` | API authentication endpoints          |
| `./mocks/{auth}.json`          | `POST /login`<br>`POST /logout`<br>`REST /users`                            | Root-level authentication             |

#### Credentials File Format

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

#### Authentication Endpoints

**Login Endpoint** - `POST /{folder}/login`

-   **Request**: JSON with `username` and `password`
-   **Response**: JWT token and user info (password excluded)
-   **Cookie**: Sets HTTP-only `auth_token` cookie for 24 hours

**Logout Endpoint** - `POST /{folder}/logout`

-   **Request**: JWT token via Authorization header or cookie
-   **Response**: Success message
-   **Action**: Revokes the token from valid tokens list

**Users REST Endpoint** - `GET,POST,PUT,PATCH,DELETE /{folder}/users[/{username}]`

-   **IdType**: None
-   **IdKey**: `username`
-   **Protected**: All REST end-points are protected and only accessible with authentication token

#### Route Protection

To protect routes with authentication, prefix folder names or filenames with `$`:

**Protected Files**

```
mocks/
├── api/
│   ├── cities/
│   │   └── $get.json        # Protected: GET /api/cities
│   └── companies/
│       └── get.json         # Public: GET /api/companies
```

**Protected Folders** (protects all children)

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

#### Authentication Examples

**Login Request**

```bash
curl -X POST http://localhost:4520/account/login \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "password": "admin123"}'
```

**Login Response**

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

**Accessing Protected Routes**

Option A: Authorization Header

```bash
curl -H "Authorization: Bearer <jwt_token>" \
  http://localhost:4520/admin/repositories
```

Option B: Cookie (automatic)

```bash
curl -b "auth_token=<jwt_token>" \
  http://localhost:4520/admin/repositories
```

**Logout Request**

```bash
curl -X POST http://localhost:4520/account/logout \
  -H "Authorization: Bearer <jwt_token>"
```

-   It is not possible to protect a `public` folder

#### Security Features

-   **JWT Tokens**: 24-hour expiration with HS256 signing
-   **HTTP-Only Cookies**: Prevents XSS attacks
-   **Token Revocation**: Logout invalidates tokens
-   **Dual Authentication**: Supports Authorization header and cookies
-   **Password Protection**: Login responses exclude password fields
-   **Route-Level Protection**: Granular control over protected endpoints

### JGD (JSON Generation Definition) Files

For generating dynamic and realistic test data, rs-mock-server supports JGD files using the [JGD-rs library](https://github.com/lvendrame/jgd-rs/tree/main/jgd-rs). JGD is a powerful JSON generation definition language that allows you to create realistic mock data with minimal configuration.

#### How JGD Works

When the server detects a file with the `.jgd` extension, it automatically processes it using the JGD-rs library to generate JSON responses dynamically. This is perfect for:

-   **Realistic Test Data**: Generate users, products, orders with proper relationships
-   **Dynamic Content**: Data changes on each request for more realistic testing
-   **Complex Structures**: Nested objects, arrays, and references
-   **Localization**: Generate data in different languages and formats

#### JGD File Examples

**Basic User Generation** (`users.jgd`):

```jgd
{
  "$format": "jgd/v1",
  "version": "1.0.0",
  "root": {
    "count": 10,
    "fields": {
      "id": "${uuid.v4}",
      "name": "${name.name}",
      "email": "${internet.safeEmail}",
      "age": {
        "number": {
          "min": 18,
          "max": 80,
          "integer": true
        }
      },
      "created_at": "${chrono.dateTime}"
    }
  }
}
```

**Complex Product Catalog** (`products.jgd`):

```jgd
{
  "$format": "jgd/v1",
  "version": "1.0.0",
  "root": {
    "count": 25,
    "fields": {
      "id": "${uuid.v4}",
      "name": "${lorem.words(2,4)}",
      "description": "${lorem.sentence(8,15)}",
      "price": {
        "number": {
          "min": 10.99,
          "max": 999.99,
          "integer": false
        }
      },
      "category": "${lorem.word}",
      "in_stock": "${boolean.boolean(85)}",
      "tags": {
        "array": {
          "count": [1, 5],
          "of": "${lorem.word}"
        }
      },
      "created_at": "${chrono.dateTime}",
      "updated_at": "${chrono.dateTime}"
    }
  }
}
```

#### JGD Route Generation

JGD files follow the same naming conventions as other files:

| JGD File Pattern     | Generated Route(s)              | Description                    |
| :------------------- | :------------------------------ | :----------------------------- |
| `get.jgd`            | `GET /api/data`                 | Dynamic JSON for GET method    |
| `post{products}.jgd` | `POST /api/data/products`       | Dynamic JSON for POST method   |
| `users{1-5}.jgd`     | `GET /api/data/users/1` to `/5` | Range-based dynamic generation |
| `sample.jgd`         | `GET /api/data/sample`          | Static file route with JGD     |

#### JGD vs Static JSON

-   **Static JSON** (`.json`): Same response every time, perfect for consistent test scenarios
-   **JGD Files** (`.jgd`): Dynamic responses on each request, ideal for realistic testing and development
-   **REST with Static JSON** (`rest.json`): Fixed initial data for REST API, CRUD operations work on persistent in-memory data
-   **REST with JGD** (`rest.jgd`): Generated initial data for REST API, CRUD operations work on persistent in-memory data derived from JGD generation

For more information about JGD syntax and capabilities, visit the [JGD-rs documentation](https://github.com/lvendrame/jgd-rs/tree/main/jgd-rs).

### Special "Public" Folder for Static Serving

To serve a directory of static assets (like a frontend app), you can use a specially named `public` folder in your mock directory root.

-   **`public` folder**: If you create a folder named `public`, all its contents will be served from the `/public` route.

    -   `./mocks/public/home.html` → `GET /public/home.html`

-   **`public-<alias>` folder**: You can customize the URL path by adding a dash. A folder named `public-static` will serve its files from the `/static` route.

    -   `./mocks/public-static/style.css` → `GET /static/style.css`

### Special "{upload}" Folder for File Handling

For file upload and download functionality, you can create a specially named `{upload}` folder. When detected, the server automatically creates endpoints for uploading and downloading files.

#### Basic Upload Folder

-   **`{upload}` folder**: Creates upload and download endpoints at `/upload`.

    -   **POST** `/upload` - Upload files (multipart/form-data)
    -   **GET** `/upload` - List all uploaded files
    -   **GET** `/upload/{filename}` - Download files by name

#### Upload Folder Configuration

The `{upload}` folder supports additional configuration through special naming patterns:

| Folder Pattern        | Upload Route   | List Route    | Download Route           | Temporary Files | Description                  |
| :-------------------- | :------------- | :------------ | :----------------------- | :-------------- | :--------------------------- |
| `{upload}`            | `POST /upload` | `GET /upload` | `GET /upload/{filename}` | No              | Basic upload/download        |
| `{upload}{temp}`      | `POST /upload` | `GET /upload` | `GET /upload/{filename}` | **Yes**         | Files deleted on server stop |
| `{upload}-files`      | `POST /files`  | `GET /files`  | `GET /files/{filename}`  | No              | Custom route name            |
| `{upload}{temp}-docs` | `POST /docs`   | `GET /docs`   | `GET /docs/{filename}`   | **Yes**         | Custom route + temporary     |

#### Examples

-   **Basic**: `./mocks/{upload}/` creates `POST /upload`, `GET /upload`, and `GET /upload/{filename}`
-   **Temporary**: `./mocks/{upload}{temp}/` - same endpoints, but files are cleaned up when server stops
-   **Custom Route**: `./mocks/{upload}-files/` creates `POST /files`, `GET /files`, and `GET /files/{filename}`
-   **Combined**: `./mocks/{upload}{temp}-upfiles/` creates `POST /upfiles`, `GET /upfiles`, and `GET /upfiles/{filename}` with automatic cleanup

All uploaded files are saved in the detected folder with their original filenames, and downloads include proper `Content-Type` and `Content-Disposition` headers.

---

## Installation

### With Cargo

You can install it directly:

```sh
cargo install rs-mock-server
```

### From Source

Alternatively, you can clone the repository and build it yourself:

```sh
# Clone the repository
git clone https://github.com/lvendrame/rs-mock-server.git

# Navigate into the project directory
cd rs-mock-server

# Build the project for release
cargo build --release

# The executable will be at ./target/release/rs-mock-server
./target/release/rs-mock-server --help
```

---

## Usage

You can run the server using the `rs-mock-server` executable.

**To start the server with default settings:**
(This will use the `./mocks` folder and run on port `4520`)

```sh
rs-mock-server
```

**To specify a custom port and mock directory:**

```sh
rs-mock-server --port 8080 --folder ./my-api-mocks
```

### Command-Line Options

```sh
Usage: rs-mock-server [OPTIONS]

Options:
  -p, --port <PORT>      Port to run the server on [default: 4520]
  -f, --folder <FOLDER>  Directory to load mock files from [default: mocks]
  -h, --help             Print help
  -V, --version          Print version
```

---

## Example Walkthrough

Imagine you have the following directory structure:

```
mocks/
├── api/
│   ├── users/
│   │   ├── get.json         # Contains a JSON array of all users
│   │   ├── post.json        # Contains a success message for user creation
│   │   ├── get{id}.json     # Contains a single user object template
│   │   └── sample.jgd       # Dynamic JSON generation using JGD
│   ├── products/
│   │   ├── rest{_id:int}.json # In-memory REST API with integer IDs (static data)
│   │   ├── get{1-3}.json    # Contains a product template for IDs 1, 2, 3
│   │   ├── get{special}.json # Contains a specific "special" product
│   │   └── catalog.jgd      # Dynamic product catalog generation
│   ├── inventory/
│   │   └── rest{uuid}.jgd   # In-memory REST API with UUID IDs (generated data)
│   ├── companies/
│   │   └── rest.json        # In-memory REST API with UUID IDs (static data)
│   ├── auth/
│   │   └── {auth}.json      # JWT authentication with user credentials
│   └── status.txt           # Contains the plain text "API is running"
├── $admin/                  # Protected folder - requires authentication
│   └── settings/
│       └── get.json         # Protected admin settings
├── assets/
│   └── logo.svg             # An SVG image file
├── {upload}/                # Upload folder for file handling
├── {upload}{temp}-docs/     # Temporary upload folder with custom route
└── public-static/
    ├── image.jpg            # An JPG image file
    └── css/
        └── style.css        # A stylesheet
```

Running `rs-mock-server` in the same directory will create the following endpoints:

| Method     | Route                   | Response Body From...                    | `Content-Type`     | Description                                    |
| :--------- | :---------------------- | :--------------------------------------- | :----------------- | :--------------------------------------------- |
| **GET**    | `/api/users`            | `mocks/api/users/get.json`               | `application/json` | Static response                                |
| **POST**   | `/api/users`            | `mocks/api/users/post.json`              | `application/json` | Static response                                |
| **GET**    | `/api/users/{id}`       | `mocks/api/users/get{id}.json`           | `application/json` | Static response                                |
| **GET**    | `/api/users/sample`     | Dynamic JGD from `sample.jgd`            | `application/json` | **JGD** - Dynamic JSON generation              |
| **GET**    | `/api/products`         | In-memory data from `rest{_id:int}.json` | `application/json` | **REST API** - List all products               |
| **POST**   | `/api/products`         | In-memory database                       | `application/json` | **REST API** - Create new product              |
| **GET**    | `/api/products/{_id}`   | In-memory database                       | `application/json` | **REST API** - Get product by ID               |
| **PUT**    | `/api/products/{_id}`   | In-memory database                       | `application/json` | **REST API** - Update product                  |
| **PATCH**  | `/api/products/{_id}`   | In-memory database                       | `application/json` | **REST API** - Partial update                  |
| **DELETE** | `/api/products/{_id}`   | In-memory database                       | `application/json` | **REST API** - Delete product                  |
| **GET**    | `/api/products/1`       | `mocks/api/products/get{1-3}.json`       | `application/json` | Static response                                |
| **GET**    | `/api/products/2`       | `mocks/api/products/get{1-3}.json`       | `application/json` | Static response                                |
| **GET**    | `/api/products/3`       | `mocks/api/products/get{1-3}.json`       | `application/json` | Static response                                |
| **GET**    | `/api/products/special` | `mocks/api/products/get{special}.json`   | `application/json` | Static response                                |
| **GET**    | `/api/products/catalog` | Dynamic JGD from `catalog.jgd`           | `application/json` | **JGD** - Dynamic product catalog              |
| **GET**    | `/api/inventory`        | In-memory data from `rest{uuid}.jgd`     | `application/json` | **REST API + JGD** - List all inventory items  |
| **POST**   | `/api/inventory`        | In-memory database                       | `application/json` | **REST API + JGD** - Create new inventory item |
| **GET**    | `/api/inventory/{id}`   | In-memory database                       | `application/json` | **REST API + JGD** - Get inventory item by ID  |
| **PUT**    | `/api/inventory/{id}`   | In-memory database                       | `application/json` | **REST API + JGD** - Update inventory item     |
| **PATCH**  | `/api/inventory/{id}`   | In-memory database                       | `application/json` | **REST API + JGD** - Partial update            |
| **DELETE** | `/api/inventory/{id}`   | In-memory database                       | `application/json` | **REST API + JGD** - Delete inventory item     |
| **GET**    | `/api/companies`        | In-memory data from `rest.json`          | `application/json` | **REST API** - List all companies              |
| **POST**   | `/api/companies`        | In-memory database                       | `application/json` | **REST API** - Create new company              |
| **GET**    | `/api/companies/{id}`   | In-memory database                       | `application/json` | **REST API** - Get company by ID               |
| **PUT**    | `/api/companies/{id}`   | In-memory database                       | `application/json` | **REST API** - Update company                  |
| **PATCH**  | `/api/companies/{id}`   | In-memory database                       | `application/json` | **REST API** - Partial update                  |
| **DELETE** | `/api/companies/{id}`   | In-memory database                       | `application/json` | **REST API** - Delete company                  |
| **GET**    | `/api/status`           | `mocks/api/status.txt`                   | `text/plain`       | Static file                                    |
| **POST**   | `/api/auth/login`       | JWT authentication                       | `application/json` | **Auth** - Login with credentials              |
| **POST**   | `/api/auth/logout`      | JWT token revocation                     | `application/json` | **Auth** - Logout and revoke token             |
| **GET**    | `/admin/settings`       | `mocks/$admin/settings/get.json`         | `application/json` | **Protected** - Requires authentication        |
| **GET**    | `/assets/logo`          | `mocks/assets/logo.svg`                  | `image/svg+xml`    | Static file                                    |
| **POST**   | `/upload`               | File upload handling                     | `text/plain`       | **Upload** - Upload files                      |
| **GET**    | `/upload`               | List of uploaded files                   | `application/json` | **Upload** - List uploaded files               |
| **GET**    | `/upload/{filename}`    | Files from `{upload}/` folder            | _varies_           | **Download** - Download files                  |
| **POST**   | `/docs`                 | File upload handling (temporary)         | `text/plain`       | **Upload** - Upload files (temp)               |
| **GET**    | `/docs`                 | List of uploaded files (temporary)       | `application/json` | **Upload** - List uploaded files               |
| **GET**    | `/docs/{filename}`      | Files from `{upload}{temp}-docs/` folder | _varies_           | **Download** - Download files (temp)           |
| **GET**    | `/static/image.jpg`     | `mocks/public-static/image.svg`          | `image/jpg`        | Static file                                    |
| **GET**    | `/static/css/style.css` | `mocks/public-static/css/style.css`      | `text/css`         | Static file                                    |

**Note**:

-   The REST API endpoints provide full CRUD functionality with automatic ID generation, data persistence during runtime, and initial data loading from JSON files or JGD generation.
-   REST files with `.json` extension load static initial data from the JSON array in the file.
-   REST files with `.jgd` extension generate dynamic fake data using JGD and use it as initial data for the REST API.
-   Authentication endpoints provide JWT-based login/logout with secure token management and route protection capabilities.
-   Protected routes (prefixed with `$`) require valid JWT tokens via Authorization header or auth_token cookie.
-   Upload endpoints handle multipart/form-data file uploads and preserve original filenames.
-   Download endpoints serve files with proper Content-Type detection and Content-Disposition headers.
-   Temporary upload folders (`{temp}`) automatically clean up all files when the server stops.
-   JGD files (`.jgd`) generate dynamic JSON responses using the JGD-rs library for realistic test data.
-   You can interact with all REST endpoints using any HTTP client, and data will persist until the server is restarted.
-   Temporary upload folders (`{temp}`) automatically clean up all files when the server stops.
-   JGD files (`.jgd`) generate dynamic JSON responses using the JGD-rs library for realistic test data.
-   You can interact with all endpoints using any HTTP client, and data will persist until the server is restarted.

---

## Development

### Getting Started

1. **Clone the repository**:

    ```bash
    git clone https://github.com/lvendrame/rs-mock-server.git
    cd rs-mock-server
    ```

2. **Install dependencies**:

    ```bash
    cargo build
    ```

3. **Set up development environment**:
    ```bash
    make dev-setup
    ```

### Development Commands

The project includes a Makefile with convenient development commands:

```bash
# Run tests
make test

# Run tests in watch mode (requires cargo-watch)
make test-watch

# Run all quality checks (tests, clippy, formatting)
make check-all

# Format code
make fmt

# Run Clippy linter
make clippy

# Build the project
make build

# Run the application
make run

# Set up Git hooks
make setup-hooks
```

### Pre-commit Hooks

This project uses Git pre-commit hooks to ensure code quality. The hooks automatically:

-   ✅ Run all tests before each commit
-   ❌ Block commits if tests fail
-   🎯 Ensure consistent code quality

**Automatic setup**: Pre-commit hooks are installed automatically when you run `make dev-setup` or `make setup-hooks`.

**Manual bypass** (not recommended): If you need to commit without running tests:

```bash
git commit --no-verify -m "Your message"
```

**Configuration**: You can customize the pre-commit checks by editing `.git/hooks/pre-commit`. The hook supports:

-   Running tests (enabled by default)
-   Running Clippy checks (disabled by default)
-   Checking code formatting (disabled by default)

### Testing

The project includes comprehensive test coverage:

```bash
# Run all tests
cargo test

# Run specific test module
cargo test id_manager
cargo test in_memory_collection
cargo test route_builder

# Run tests with output
cargo test -- --nocapture
```

### Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature-name`
3. Make your changes and ensure tests pass
4. Commit your changes (pre-commit hooks will run automatically)
5. Push to your fork and submit a pull request

**Code Quality**: All contributions must:

-   Pass existing tests
-   Include tests for new functionality
-   Follow Rust formatting standards (`cargo fmt`)
-   Pass Clippy linting (`cargo clippy`)
