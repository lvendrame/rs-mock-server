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

/// Application bootstrap, router assembly, and shared server state.
pub mod app;
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
/// Local HTTPS configuration and certificate handling.
pub mod tls;
/// Upload cleanup configuration.
pub mod upload_configuration;

pub use app::App;
pub use route_builder::config::{Config, ServerConfig};
