# Interactive Generator

The interactive generator helps create mock routes and the main server configuration
without manually writing every file name, folder, and seed payload.

Run it with:

```bash
rs-mock-server --generate
```

To generate files in a different mocks folder, pass `--folder` together with
`--generate`:

```bash
rs-mock-server --folder ./my-api-mocks --generate
```

## Wizard Controls

| Key       | Action                                      |
| --------- | ------------------------------------------- |
| `Enter`   | Continue to the next step or write files    |
| `Left`    | Go back to the previous step                |
| `b`       | Go back outside text input screens          |
| `Esc`     | Quit the generator                          |
| `Space`   | Change the selected route option            |
| `f`       | Toggle force overwrite on the review screen |
| `w`       | Write files from the review screen          |

When generation succeeds, the wizard returns to the top menu so you can create
another route or generate the main configuration.

## Top Menu

The generator starts with three choices:

-   **Generate Route** creates one mock route, route directory, or route seed.
-   **Generate Main Configuration** creates `rs-mock-server.toml`.
-   **Exit** closes the wizard.

The review screen always shows the planned directories, files, and file content
before writing. Use `f` to allow overwriting existing files.

## Route Generation

Each route kind asks for the information needed by that route type.

| Route kind  | What the wizard asks for                               | Generated output                                      |
| ----------- | ------------------------------------------------------ | ----------------------------------------------------- |
| Basic       | Route path, method, protection, response type, fields  | Method file such as `get.json`, `post.jgd`, or `$put.txt` |
| REST        | Resource route, ID strategy, protection, JSON/JGD seed | `rest.json` or `rest.jgd` CRUD route seed             |
| Auth        | Auth base route and user fields                        | `{auth}.json` user credential seed                    |
| Upload      | Upload route name, temporary storage, protection       | `{upload}` directory variants                         |
| Public      | Public route alias                                     | `public` or `public-<alias>` directory                |
| GraphQL     | Service or collection name, protection, JSON/JGD seed  | `graphql/collections/<service>.json` or `.jgd`        |
| SQL         | GET route, collection/table name, protection           | `.sql` GET route with a basic `select` query          |

### Basic Routes

Use Basic routes for file-backed method endpoints such as:

```text
mocks/api/users/get.json
mocks/api/users/post.jgd
mocks/api/users/$put.txt
```

The wizard lets you choose the HTTP method, public or protected access, and the
response format. JSON and JGD routes continue to the field editor so the seed
content can be shaped before review.

### REST Routes

Use REST routes for in-memory CRUD APIs backed by `rest.json` or `rest.jgd`.
The wizard asks for the resource route, ID strategy, protection, and data source.

Supported ID strategies are:

-   UUID ID
-   Integer ID
-   No generated ID

See [In-Memory REST APIs](02-rest-apis.md) for endpoint behavior and descriptor
rules.

### Auth Routes

Use Auth routes to create a `{auth}.json` credentials file. The generated users
include the fields needed by the authentication handler, including username,
password, roles, and audit-style fields.

See [JWT Authentication](03-authentication.md) for login, logout, user routes,
and route protection behavior.

### Upload Routes

Use Upload routes to create upload directories. The wizard can generate:

-   persistent uploads
-   temporary uploads
-   public upload routes
-   protected upload routes

Examples:

```text
mocks/{upload}
mocks/{upload}{temp}
mocks/${upload}-documents
mocks/${upload}{temp}-documents
```

See [File Uploads & Downloads](04-file-uploads.md) for upload, list, and download
endpoints.

### Public Routes

Use Public routes for directories served as static assets. The default alias
creates `public`; a custom alias such as `assets` creates `public-assets`.

See [Static Files](05-static-files.md) for public directory behavior.

### GraphQL Routes

GraphQL routes use the fixed GraphQL endpoint created by the `graphql` folder.
The wizard asks for the service or collection name, not an endpoint path.

For example, entering `users` creates a seed file like:

```text
mocks/graphql/collections/users.json
```

Protected GraphQL generation uses the `$graphql` folder convention.

See [GraphQL Routes](11-graphql.md) for GraphiQL, static overrides, dynamic
queries, mutations, and collection loading.

### SQL Routes

Use SQL routes to create GET endpoints backed by `.sql` files. The wizard asks
for the route path and the collection/table name used in the generated query.

For example, `/reports/companies/{id}` can generate:

```text
mocks/reports/companies{id}.sql
```

with content similar to:

```sql
select * from companies where id = ?;
```

See [SQL Routes](09-sql-routes.md) for SQL route behavior and parameter binding.

## Main Configuration

The main configuration flow generates `rs-mock-server.toml` in the current
working directory. It starts from the same server defaults used by the CLI:

-   port `4520`
-   folder `mocks`
-   CORS enabled

The review screen shows the final TOML before writing it.

See [Configuration Guide](10-configurations.md) for all configuration tables and
override behavior.

## Field Editor

JSON and JGD route flows open the field editor before review. The field editor
lets you shape the generated object fields and ID metadata used by route seeds.

Common field editor actions:

-   `a` adds a field.
-   `e` edits the selected field name and type.
-   `r` removes the selected field.
-   `i` cycles the selected field type and updates ID metadata when the ID field
    is selected.
-   `Enter` continues to the review screen.

The review screen renders the full generated file content, so you can verify the
JSON, JGD, or SQL before writing.

## Next Steps

-   Learn route filename rules in [Basic Routing](01-basic-routing.md).
-   Build CRUD mocks with [In-Memory REST APIs](02-rest-apis.md).
-   Generate dynamic seed data with [JGD Files](06-jgd-files.md).
-   Configure the server with [Configuration Guide](10-configurations.md).
