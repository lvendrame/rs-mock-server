//! Route discovery, parsing, and registration primitives.

/// Configuration loading and merge behavior for route discovery.
pub mod config;
/// Shared route enum and collection metadata.
pub mod route;
/// Authentication route discovery.
pub mod route_auth;
/// Static file route discovery.
pub mod route_basic;
/// GraphQL route discovery.
pub mod route_graphql;
/// Directory traversal and route ordering.
pub mod route_manager;
/// Route parsing input assembled from filesystem entries.
pub mod route_params;
/// Public static directory route discovery.
pub mod route_public;
/// REST collection route discovery.
pub mod route_rest;
/// Upload directory route discovery.
pub mod route_upload;

use axum::routing::MethodRouter;
use http::Method;
pub use route::*;
pub use route_auth::*;
pub use route_basic::*;
pub use route_params::*;
pub use route_public::*;
pub use route_rest::*;
pub use route_upload::*;

use crate::app::App;

/// Prints a human-readable route registration message.
pub trait PrintRoute {
    /// Prints this route to standard output.
    fn println(&self);
}

/// Registers generated routes on an application router.
pub trait RouteRegistrator {
    /// Adds a route with optional auth protection and home-page options.
    fn push_route(
        &mut self,
        path: &str,
        router: MethodRouter,
        method: Option<&str>,
        is_protected: bool,
        options: Option<&[String]>,
    );
}

/// Builds Axum routes from a parsed route definition.
pub trait RouteGenerator {
    /// Registers all HTTP routes represented by this value.
    fn make_routes(&self, app: &mut App);
}

/// Converts a filename method token into an HTTP method, defaulting to GET.
pub fn method_from_str(value: &str) -> Method {
    match value.to_uppercase().as_str() {
        "GET" => Method::GET,
        "POST" => Method::POST,
        "PUT" => Method::PUT,
        "DELETE" => Method::DELETE,
        "HEAD" => Method::HEAD,
        "OPTIONS" => Method::OPTIONS,
        "CONNECT" => Method::CONNECT,
        "PATCH" => Method::PATCH,
        "TRACE" => Method::TRACE,
        "QUERY" => Method::from_bytes(b"QUERY").unwrap(),
        _ => Method::GET,
    }
}
