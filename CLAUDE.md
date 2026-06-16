# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`rs-mock-server` is a zero-configuration mock API server, shipped as both a CLI binary and an embeddable Rust library. It recursively scans a directory (default `./mocks`) and maps the folder/file structure onto Axum HTTP and GraphQL routes. Filenames and folder names encode HTTP method, route shape, and special behaviors (REST CRUD, JWT auth, file uploads, GraphQL, public static dirs, Fosk collections/schemas).

## Common Commands

- `cargo build` / `make build` — debug build
- `cargo build --release` / `make build-release`
- `cargo run` / `make run` — run with `./mocks` and port 4520
- `cargo test` / `make test` — run all tests (inline `#[cfg(test)] mod tests` blocks)
- `cargo test <module_name>` — run one module's tests, e.g. `cargo test route_builder`
- `cargo test -- --nocapture` — show `println!` output
- `make clippy` — `cargo clippy --all-targets --all-features -- -D warnings`
- `make fmt` / `make fmt-check` — format / check formatting
- `make check-all` — test + clippy + fmt-check; run this before committing (the pre-commit hook also runs `cargo test --quiet` and blocks the commit on failure)
- `make coverage` — `cargo llvm-cov` with a 95% function-coverage threshold, excluding `src/main.rs` and `src/handlers/graphql_handlers.rs`
- `cargo run -- --generate` (or `rs-mock-server --generate`) — launch the interactive TUI generator that scaffolds mock files and `rs-mock-server.toml`

## Architecture

### Startup flow (`src/main.rs` → `src/app.rs`)

- `main.rs` parses CLI args, optionally loads `./rs-mock-server.toml` (CLI SSL flags overlay it via `apply_cli_ssl_config`), then loops on `run_app_session`. A `notify` watcher on the mock folder triggers `SessionResult::Restart`, which tears down and rebuilds the entire `App` — this is the hot-reload mechanism. `{upload}` folders are excluded from triggering restarts (`is_upload_folder`) so file uploads/downloads don't restart the server.
- `App` (`src/app.rs`) holds an Axum `Router` in a `RefCell` (taken/replaced builder-style via `get_router`/`replace_router`), a `Pages` model for the home UI, a shared in-memory Fosk `Db` (`Arc<Db>`), upload cleanup configs, and the resolved `Config`.
- `App::build_router()` runs, in order: `build_dyn_routes` (filesystem route discovery via `RouteManager`) → `load_schema_files` → `load_collection_files` → `build_home_route` → `build_collections_route` → `build_schemas_route` → optional `build_fallback` (CLI mode only) → `build_middlewares` (trace/CORS/path-normalize) → `build_collections_references` (infers Fosk relations between every pair of loaded collections).
- Library embedding via `App::into_router()` skips the fallback handler and mounts the home UI at `/mock-server` (`MOCK_SERVER_ROUTE`) instead of `/`, leaving `/` and fallback behavior to the host app.
- `GLOBAL_SHARED_INFO` (a module-level `static RwLock`) carries the JWT secret, token-collection name, and auth-cookie name from the parsed `{auth}` route to `App::try_add_auth_middleware_layer`, which wraps any route with `is_protected = true` in the auth middleware.

### Route discovery (`src/route_builder/`)

This is the core of the file-to-route mapping; read here first before changing routing behavior.

- `RouteManager::from_dir` recursively walks the mock folder. For each directory it loads `ConfigStore::try_from_dir` (every `*.toml` in that dir, keyed by lowercase file stem) and merges that dir's `config.toml` into the inherited parent `Config` (child overrides parent via the `Mergeable` trait in `config.rs`).
- For each entry, `RouteParams::new` (`route_params.rs`) computes the route path, strips a leading `$` (which sets `route.protect = true`), merges any same-stem `*.toml` (e.g. `get.toml` next to `get.json`) into the effective config, and records `file_stem` / `file_extension` / `is_dir`.
- `Route::try_parse` (`route.rs`) dispatches `RouteParams` to per-kind parsers in priority order:
  - Directories: `RoutePublic` → `RouteUpload` → `RouteGraphQL` → `Route::None` (recurse as a plain folder)
  - Files: `RouteRest` → `RouteAuth` → `RouteBasic` → `Route::None`
  - Each `route_*.rs` owns a regex for its filename pattern, e.g. `RE_FILE_REST` (`rest` / `$rest{id:type}`), `RE_FILE_METHODS` (`get`/`post`/.../`{id}`/`{1-5}`/`{value}`), `RE_FILE_AUTH` (`{auth}`), `RE_DIR_UPLOAD` (`{upload}` / `{upload}{temp}` / `{upload}-name`), `RE_FOLDER_GRAPHQL` (`graphql`).
- `{collections}` and `{schemas}` folders (or their configured equivalents via `[collections].folder` / `[schemas].folder`) are skipped during traversal (`is_reserved_data_folder_entry`) and handled separately by `collection_files.rs` / `schema_files.rs`.
- Only one `{auth}` route is allowed per server — `RouteManager` panics if a second is found. It's stored separately (`auth_route`) and registered first so its collections/middleware exist before other protected routes build.
- `manager.sort()` orders routes by `Route` variant then path/method (`Route::partial_cmp`) before registration, which determines Axum route registration order and matching precedence.
- Each `Route*` struct implements `RouteGenerator::make_routes(&self, app: &mut App)` (registers Axum routes via `app.push_route` / `App::route`) and `PrintRoute::println` (the startup log line, e.g. "✔️ Mapped ... to GET /api/users").

### Config layering (`src/route_builder/config.rs`)

Three layers merge child-over-parent via the `Mergeable` trait: global `rs-mock-server.toml` (CWD) → per-directory `config.toml` (applies to all descendants) → per-route `<name>.toml` (e.g. `get.toml` beside `get.json`, `rest.toml`, `{auth}.toml`, `{upload}.toml`). `server`, `route`, `collections`, and `schemas` merge field-by-field across layers; `collection`, `auth`, and `upload` sub-configs do **not** merge — only the most specific layer that defines them wins. Full per-route-type TOML schema is in `docs/10-configurations.md`.

### Handlers (`src/handlers/`)

One `*_handlers.rs` per route kind (`basic_handlers`, `rest_handlers`, `auth_handlers`, `upload_handlers`, `graphql_handlers`, `collections_handlers`, `schema_handlers`), all re-exported from `handlers/mod.rs`. `basic_handlers::content_handler` dispatches on file extension: `.jgd` files generate data via `jgd-rs`, `.sql` files execute against the shared Fosk `Db` (`db.query` / `query_with_args`, using the URL `{id}` path param as the query argument), everything else is served as text or streamed binary with a guessed MIME type (`build_stream_handler`).

### In-memory data (Fosk `Db`)

A single `Arc<Db>` (from the `fosk` crate) is shared by REST routes, the `{auth}` user/token collections, GraphQL, and the `/mock-server` collection/schema inspection endpoints. `rest.json` / `rest.jgd` seed a collection named after the route's last path segment (or `[collection].name`); `{collections}` and `{schemas}` folders seed/define collections independently of routes. After everything is registered, `App::build_collections_references` infers relationships between every pair of loaded collections.

### Generator (`src/generator/`)

An interactive `ratatui`/`crossterm` TUI (entry point `generator::run_generator`, wired up via `--generate`) for scaffolding mock files and `rs-mock-server.toml` without hand-editing. `domain.rs` defines the selection model (`RouteKind`, `WritePlan`/`WriteOperation`); `paths.rs` is a pure function turning a `RouteSelection` into filesystem write operations (no I/O, easy to test); `content.rs` renders file contents; `writer.rs` executes the `WritePlan`; `tui/` holds the ratatui screens/components for each wizard step.

### Embedded home UI (`src/pages.rs`, `src/link.rs`, `src/home/`)

`src/home/{index.html,scripts.js,styles.css,mock-routes.js}` are embedded via `include_str!` and rendered by `Pages::render_index`, which injects a JSON array of `Link`s (method/route/options) accumulated as routes are registered.

### TLS (`src/tls.rs`)

Resolves `--ssl` / `--ssl-cert` / `--ssl-key` (or `[server]` TOML equivalents) into a `TlsMode`. `--ssl` alone generates and caches a self-signed localhost certificate via `rcgen` under `.rs-mock-server/ssl`.

## Testing Conventions

- Tests live inline in `#[cfg(test)] mod tests` next to the code under test — there is no separate `tests/` integration directory.
- Filesystem-dependent tests use `tempfile::TempDir`, never the real `mocks/` folder.
- When changing route-parsing behavior, add cases to the relevant `route_*.rs` test module (filename → `Route` variant mapping) and to `route.rs` / `route_manager.rs` for dispatch-priority and ordering interactions.
- `mocks/` at the repo root is a live example tree used for manual testing (`make run`) — treat it as documentation, not test fixtures.
