# Schema Loading

rs-mock-server can initialize Fosk collection schemas from compact JSON schema
files. Schemas define field names, primitive types, nullability, and optional ID
behavior before or alongside collection data.

## Startup Folder

By default, schemas are loaded from:

```text
mocks/{schemas}
```

The complete database schema file is:

```text
mocks/{schemas}/db.schema
```

Every other non-TOML file in `mocks/{schemas}` is loaded as one collection schema.
The collection name comes from the file stem, so `users` or `users.schema` loads
the `users` or `users.schema` collection respectively.

`{schemas}` is reserved for schema loading and is not exposed as normal mock
routes.

## Compact Schema Format

A single collection schema is a JSON object where keys are field names and values
are compact type strings:

```json
{
  "user_id": "Id",
  "name": "String!",
  "age": "Int",
  "active": "Bool!"
}
```

Supported field types are:

| Type     | Meaning                  |
| -------- | ------------------------ |
| `Null`   | JSON null                |
| `Bool`   | JSON boolean             |
| `Int`    | integer number           |
| `Float`  | floating-point number    |
| `String` | JSON string              |
| `Object` | JSON object              |
| `Array`  | JSON array               |

Nullability is controlled with `!`:

| Spec      | Meaning             |
| --------- | ------------------- |
| `String`  | nullable string     |
| `String!` | non-nullable string |

## ID Markers

One field can declare collection ID behavior:

| Spec          | Behavior                                             |
| ------------- | ---------------------------------------------------- |
| `Id`          | auto-increment integer ID, stored as `Int!`          |
| `Uuid`        | generated UUID ID, stored as `String!`               |
| `None:String` | caller-provided string ID, stored as `String!`       |
| `None:Int`    | caller-provided integer ID, stored as `Int!`         |
| `None:Float`  | caller-provided floating-point ID, stored as `Float!` |

Fosk validates schema files when they are loaded. Invalid type strings,
duplicate ID markers, missing configured ID fields, and conflicting collection
ID configs are rejected.

## Complete Database Schema

`db.schema` contains a JSON object keyed by collection name:

```json
{
  "users": {
    "user_id": "Id",
    "name": "String!"
  },
  "orders": {
    "order_id": "Uuid",
    "user_id": "Int!",
    "total": "Float!"
  }
}
```

The database schema is loaded before single collection schema files. References
are inferred by Fosk after successful schema loads.

## Configuration

Override the schema folder or complete database schema filename in
`rs-mock-server.toml`:

```toml
[server]
folder = "./mocks"

[schemas]
folder = "{schemas}"
db_schema = "db.schema"
```

Relative schema folders are resolved under `[server].folder`. Absolute schema
folders are used as provided.

## HTTP Endpoints

Schema files can also be loaded and downloaded at runtime.

| Method | Route                                   | Description                    |
| ------ | --------------------------------------- | ------------------------------ |
| `POST` | `/mock-server/schemas`                  | Upload a complete DB schema    |
| `POST` | `/mock-server/schemas/{name}`           | Upload one collection schema   |
| `GET`  | `/mock-server/schemas/download`         | Download all schemas           |
| `GET`  | `/mock-server/schemas/{name}/download`  | Download one collection schema |

Uploads use multipart form data with a file part:

```bash
curl -F 'file=@mocks/{schemas}/db.schema' \
  http://localhost:4520/mock-server/schemas

curl -F 'file=@mocks/{schemas}/users' \
  http://localhost:4520/mock-server/schemas/users
```

Downloads return compact JSON that can be uploaded again:

```bash
curl -OJ http://localhost:4520/mock-server/schemas/download
curl -OJ http://localhost:4520/mock-server/schemas/users/download
```
