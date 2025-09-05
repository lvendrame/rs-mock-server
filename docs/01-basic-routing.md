# Basic Routing

This document covers the fundamental file-system routing capabilities of rs-mock-server.

## How It Works

The server recursively scans a root directory (defaults to `./mocks`) and translates the file and folder paths into API endpoints.

### Folder Structure → URL Path

The path of each folder becomes the base URL for the routes within it.

-   A folder at `./mocks/api/users` creates the base route `/api/users`.
-   A nested folder at `./mocks/api/users/profiles` creates the base route `/api/users/profiles`.

### Filename Conventions → Endpoints

The name of a file determines the **HTTP method** and the **final URL segment**. The content of the file is served as the response body.

## Basic Filename Patterns

The following table shows how different filename patterns are mapped to routes, assuming they are inside a `./mocks/api/users` directory:

| Filename Pattern      | Example File      | Generated Route(s)                                                    | Description                                                |
| :-------------------- | :---------------- | :-------------------------------------------------------------------- | :--------------------------------------------------------- |
| `[method]`            | `get.json`        | `GET /api/users`                                                      | Creates a route for a standard HTTP method.                |
| `[method]{id}`        | `get{id}.json`    | `GET /api/users/{id}`                                                 | A dynamic segment that accepts any value in that position. |
| `[method]{value}`     | `get{admin}.json` | `GET /api/users/admin`                                                | Matches a specific, hardcoded value.                       |
| `[method]{start-end}` | `get{1-5}.json`   | `GET /api/users/1`<br>`GET /api/users/2`<br>...<br>`GET /api/users/5` | A numeric range that generates multiple distinct routes.   |

## HTTP Methods

rs-mock-server supports all standard HTTP methods:

-   `GET` - Retrieve data
-   `POST` - Create new resources
-   `PUT` - Update entire resources
-   `PATCH` - Partially update resources
-   `DELETE` - Remove resources
-   `OPTIONS` - Handle preflight requests

## Examples

### Basic Method Files

```
mocks/
├── api/
│   ├── users/
│   │   ├── get.json        # GET /api/users
│   │   ├── post.json       # POST /api/users
│   │   ├── put.json        # PUT /api/users
│   │   └── delete.json     # DELETE /api/users
```

### Dynamic Parameters

```
mocks/
├── api/
│   ├── users/
│   │   ├── get{id}.json    # GET /api/users/{id} - accepts any value
│   │   ├── put{id}.json    # PUT /api/users/{id}
│   │   └── delete{id}.json # DELETE /api/users/{id}
```

### Specific Values

```
mocks/
├── api/
│   ├── users/
│   │   ├── get{admin}.json    # GET /api/users/admin
│   │   ├── get{profile}.json  # GET /api/users/profile
│   │   └── post{register}.json # POST /api/users/register
```

### Numeric Ranges

```
mocks/
├── api/
│   ├── products/
│   │   ├── get{1-10}.json    # GET /api/products/1, /api/products/2, ..., /api/products/10
│   │   └── get{100-200}.json # GET /api/products/100, /api/products/101, ..., /api/products/200
```

## File Content Examples

### JSON Response

**File:** `mocks/api/users/get.json`

```json
[
    {
        "id": 1,
        "name": "John Doe",
        "email": "john@example.com"
    },
    {
        "id": 2,
        "name": "Jane Smith",
        "email": "jane@example.com"
    }
]
```

**Route:** `GET /api/users`
**Content-Type:** `application/json`

### Dynamic Parameter Response

**File:** `mocks/api/users/get{id}.json`

```json
{
    "id": "{{id}}",
    "name": "User {{id}}",
    "email": "user{{id}}@example.com",
    "active": true
}
```

**Route:** `GET /api/users/123`
**Response:** The `{{id}}` placeholder gets the actual value from the URL

### Text Response

**File:** `mocks/api/status.txt`

```
API is running successfully
```

**Route:** `GET /api/status`
**Content-Type:** `text/plain`

## Route Priority

When multiple patterns could match the same route, rs-mock-server follows this priority order:

1. **Exact matches** - `get{admin}.json`
2. **Numeric ranges** - `get{1-5}.json`
3. **Dynamic parameters** - `get{id}.json`

For example, with these files:

```
mocks/api/users/
├── get{id}.json     # Priority 3
├── get{admin}.json  # Priority 1
└── get{1-10}.json   # Priority 2
```

-   `GET /api/users/admin` → uses `get{admin}.json`
-   `GET /api/users/5` → uses `get{1-10}.json`
-   `GET /api/users/anything-else` → uses `get{id}.json`

## Content-Type Detection

rs-mock-server automatically sets the `Content-Type` header based on the file extension:

| Extension       | Content-Type             |
| --------------- | ------------------------ |
| `.json`         | `application/json`       |
| `.xml`          | `application/xml`        |
| `.html`         | `text/html`              |
| `.txt`          | `text/plain`             |
| `.css`          | `text/css`               |
| `.js`           | `application/javascript` |
| `.png`          | `image/png`              |
| `.jpg`, `.jpeg` | `image/jpeg`             |
| `.svg`          | `image/svg+xml`          |
| `.pdf`          | `application/pdf`        |

## Next Steps

-   Learn about [In-Memory REST APIs](02-rest-apis.md) for full CRUD functionality
-   Explore [JWT Authentication](03-authentication.md) for protected routes
-   Discover [File Uploads](04-file-uploads.md) for handling file operations
-   Try [JGD Files](06-jgd-files.md) for dynamic data generation
