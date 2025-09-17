<!-- docs/11-graphql.md -->

# GraphQL Routes

This document describes how rs-mock-server handles GraphQL queries and mutations.

## Overview

When a `graphql` folder is detected in the mocks directory, the server:

-   Registers a `POST /graphql` endpoint to execute GraphQL operations.
-   Registers a `GET /graphiql` endpoint to serve the GraphiQL IDE.
-   Loads any files in a nested `/collections` subfolder into Fosk collections for query execution.

Static overrides (JSON or JGD files) take precedence over dynamic execution. If a query or mutation is named, the server will first check for a matching `<operationName>.json` or `<operationName>.jgd` file and return its content directly (for JGD files, it generates dynamic mock data based on the definition).

## GraphiQL Introspection

The GraphiQL IDE is fed by a dynamic schema constructed from the currently loaded collections. This means:

-   All collections under `mocks/graphql/collections` (and any other routes that populate the Fosk database) appear in the GraphiQL sidebar with the inferred fields and relations.
-   Relations inferred by rs-mock-server (for example `orders` → `order_items` → `products`) are surfaced as nested object lists, so you can explore available joins directly in the documentation panel.
-   CRUD mutations (`create<Collection>`, `update<Collection>`, `delete<Collection>`) are auto-generated per collection, and GraphiQL lists the expected arguments and return types for each of them.

Open `http://localhost:<port>/graphiql` and use the Docs panel to confirm which collections, relations, and mutations are currently available.

## Folder Layout

```
mocks/
├── graphql/
│   ├── collections/       # JSON or JGD files loaded as collections
│   │   └── users.json     # creates `users` collection
│   ├── getUsers.jgd       # GET /graphql?query=getUsers (static override)
│   └── ...
└── ...
```

## Static Overrides

If your GraphQL request includes an operation name:

```graphql
query showAllOrdersForProduct($product_id: Int) {
    showAllOrdersForProduct(product_id: 35) {
        id
        orderDate
        status
    }
}
```

The handler will look for:

-   `mocks/graphql/showAllOrdersForProduct.json`
-   `mocks/graphql/showAllOrdersForProduct.jgd`

If found:

-   For `.json`, it reads and returns the file as JSON.
-   For `.jgd`, it runs `generate_jgd_from_file` and returns the generated data.

Usage:

```bash
curl -X POST /graphql \
    -H 'Content-Type: application/json' \
    -d '{"query":"query { showAllOrdersForProduct(product_id:35){id}}","operationName":"showAllOrdersForProduct"}'
```

## Dynamic Execution

If no static file matches, the server parses the GraphQL AST and executes against the in-memory Fosk DB:

1. **Validate** referenced collections exist.
2. **Execute** queries by mapping root fields to collections:
    - No arguments → retrieve all records in the collection.
    - Single `id` argument → retrieve the record with that `id`.
    - Other arguments → treat them as filter conditions.
3. **Resolve** any nested relationships based on inferred foreign keys:
    - A foreign key is inferred when one collection’s primary field (e.g. `id` or `_id`) matches another collection’s field named `<collection>_id`.
    - This inference also applies across collections defined by REST routes or authentication handlers, so you can nest queries over any loaded collection (e.g., `users`, `sessions`).
4. **Filter** each JSON object to only include requested fields.

### Example Query

```graphql
query {
    orders(id: 5, status: "Shipped") {
        id
        orderDate
        status
        order_items {
            id
            quantity
            products {
                _id
                name
            }
        }
    }
}
```

Executes SQL:

```sql
SELECT * FROM orders WHERE id = 5 AND status = 'Shipped';
```

Then loads and nests `order_items` and `products`, returning only selected fields.

## Mutations

Root mutation fields map to CRUD operations on collections:

-   `create<CollectionName>` → create a new record in the collection.
-   `update<CollectionName>` → update the specified record.
-   `delete<CollectionName>` → remove the specified record.

Arguments are parsed from the AST and converted to JSON values.

### Create Example

```graphql
mutation {
    createUsers(firstName: "John", lastName: "Doe") {
        id
        firstName
        lastName
    }
}
```

A new user record is created with an auto-generated `id`, and the response includes only the requested fields.

### Update Example

```graphql
mutation {
    updateUsers(id: "1", email: "john.doe@example.com") {
        id
        email
    }
}
```

The existing user with `id = "1"` is updated, and the response includes only the requested fields.

### Delete Example

```graphql
mutation {
    deleteUsers(id: "2") {
        id
    }
}
```

A user with `id = "2"` is removed from the collection, and the response returns the fields specified in the request.

## Loading Collections

Files under `mocks/graphql/collections` are read at startup and loaded into Fosk:

```bash
mocks/graphql/collections/
├── users.json       # static array
└── products.jgd     # dynamic generator
```

Use standard JSON arrays or JGD definitions. See:

-   [02-rest-apis](02-rest-apis.md) for JSON
-   [06-jgd-files](06-jgd-files.md) for JGD
