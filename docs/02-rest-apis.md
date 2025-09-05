# In-Memory REST APIs

Create fully functional CRUD APIs with automatic ID generation and data persistence using special `rest.json` or `rest.jgd` files.

## Overview

When the server detects a file named `rest.json`, `rest.jgd`, or `rest{params}.json/jgd`, it automatically:

1. **Loads initial data** from the JSON array in the file (for `.json`) or generates fake data using JGD (for `.jgd`)
2. **Creates a complete REST API** with all CRUD operations
3. **Maintains data in memory** during the server's lifetime
4. **Handles ID generation** automatically for new items (except for None ID Type)

## REST File Naming Convention

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

## Generated Endpoints

For a `rest.json` or `rest.jgd` file in `./mocks/api/products/`, the following endpoints are automatically created:

| Method     | Route                | Description                                    |
| :--------- | :------------------- | :--------------------------------------------- |
| **GET**    | `/api/products`      | List all products                              |
| **POST**   | `/api/products`      | Create a new product (auto-generates ID)       |
| **GET**    | `/api/products/{id}` | Get a specific product by ID                   |
| **PUT**    | `/api/products/{id}` | Update an entire product (replaces all fields) |
| **PATCH**  | `/api/products/{id}` | Partially update a product (merges fields)     |
| **DELETE** | `/api/products/{id}` | Delete a product by ID                         |

## Initial Data Format

### JSON Files

The JSON file should contain an array of objects, where each object represents an item with the configured ID field:

**Example: `rest.json` (default UUID)**

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

**Example: `rest{_id:int}.json` (integer IDs)**

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

### JGD REST Files

When using `rest.jgd` files, the server generates dynamic fake data using JGD (JSON Generation Definition) and uses it as initial data for the REST API.

**Example: `rest{_id:int}.jgd`**

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

**Example: `rest{uuid}.jgd`**

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

## Usage Examples

### Creating Items

**Request:**

```bash
curl -X POST http://localhost:4520/api/products \
  -H "Content-Type: application/json" \
  -d '{
    "name": "New Product",
    "price": 29.99,
    "category": "Tools"
  }'
```

**Response:**

```json
{
    "id": "550e8400-e29b-41d4-a716-446655440003",
    "name": "New Product",
    "price": 29.99,
    "category": "Tools"
}
```

### Listing All Items

**Request:**

```bash
curl http://localhost:4520/api/products
```

**Response:**

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
    },
    {
        "id": "550e8400-e29b-41d4-a716-446655440003",
        "name": "New Product",
        "price": 29.99,
        "category": "Tools"
    }
]
```

### Getting Single Item

**Request:**

```bash
curl http://localhost:4520/api/products/550e8400-e29b-41d4-a716-446655440001
```

**Response:**

```json
{
    "id": "550e8400-e29b-41d4-a716-446655440001",
    "name": "Wireless Headphones",
    "price": 199.99,
    "category": "Electronics"
}
```

### Updating Item (PUT)

**Request:**

```bash
curl -X PUT http://localhost:4520/api/products/550e8400-e29b-41d4-a716-446655440001 \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Premium Wireless Headphones",
    "price": 249.99,
    "category": "Electronics",
    "features": ["noise-canceling", "bluetooth"]
  }'
```

### Partial Update (PATCH)

**Request:**

```bash
curl -X PATCH http://localhost:4520/api/products/550e8400-e29b-41d4-a716-446655440001 \
  -H "Content-Type: application/json" \
  -d '{
    "price": 179.99
  }'
```

### Deleting Item

**Request:**

```bash
curl -X DELETE http://localhost:4520/api/products/550e8400-e29b-41d4-a716-446655440001
```

## ID Types Explained

### UUID (Default)

-   Generates cryptographically secure UUIDs
-   Format: `550e8400-e29b-41d4-a716-446655440001`
-   Best for: Production-like scenarios, avoiding conflicts

### Integer

-   Auto-increments starting from 1
-   Format: `1`, `2`, `3`, etc.
-   Best for: Simple testing, ordered data

### None

-   No automatic ID generation
-   IDs must be provided in requests
-   Best for: Custom ID schemes, composite keys

## Data Persistence

-   **Runtime Persistence**: All changes persist in memory during server lifetime
-   **Initial Load**: Data reloads from files on server restart
-   **CRUD Operations**: Full create, read, update, delete functionality
-   **Validation**: Automatic ID validation and conflict prevention

## Error Handling

The REST API provides appropriate HTTP status codes:

-   `200 OK` - Successful GET, PUT, PATCH
-   `201 Created` - Successful POST
-   `204 No Content` - Successful DELETE
-   `400 Bad Request` - Invalid JSON or missing required fields
-   `404 Not Found` - Item with specified ID doesn't exist
-   `409 Conflict` - ID already exists (for None ID type with manual IDs)

## Combining with Other Features

REST APIs work seamlessly with other rs-mock-server features:

-   **Authentication**: Protect REST endpoints with `$` prefix
-   **JGD Files**: Generate realistic initial data
-   **Hot Reload**: Changes to REST files restart the server
-   **Web Interface**: Test all CRUD operations in the browser

## Next Steps

-   Learn about [JWT Authentication](authentication.md) to protect your REST APIs
-   Explore [JGD Files](jgd-files.md) for generating realistic test data
-   See [Static File Serving](static-files.md) for serving assets alongside APIs
