//! Library entry point for `rs-mock-server`.
//!
//! The library exposes the same route builder used by the CLI so other
//! applications can mount file-backed mock API routes in their own Axum
//! application.
#![cfg_attr(doc, warn(missing_docs))]

/// Default TCP port used by the server.
pub const DEFAULT_PORT: u16 = 4520;
/// Default folder scanned for mock definitions.
pub const DEFAULT_FOLDER: &str = "mocks";
/// Default folder, relative to the mock root, scanned for collection seed files.
pub const DEFAULT_COLLECTIONS_FOLDER: &str = "{collections}";
/// Default folder, relative to the mock root, scanned for schema files.
pub const DEFAULT_SCHEMAS_FOLDER: &str = "{schemas}";
/// Default file name for a complete compact database schema.
pub const DEFAULT_SCHEMAS_DB_FILE: &str = "db.schema";

/// Application bootstrap, router assembly, and shared server state.
pub mod app;
/// Startup collection seed file loading.
pub mod collection_files;
/// Interactive mock route and configuration generator.
pub mod generator;
/// HTTP handlers for generated mock routes.
pub mod handlers;
/// Link model used by the generated home page.
pub mod link;
/// Embedded home page renderer.
pub mod pages;
/// File and directory route discovery.
pub mod route_builder;
/// Compact Fosk schema file loading and serialization.
pub mod schema_files;
/// Local HTTPS configuration and certificate handling.
pub mod tls;
/// Upload cleanup configuration.
pub mod upload_configuration;

pub use app::App;
pub use route_builder::config::{Config, ServerConfig};
