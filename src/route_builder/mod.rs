pub mod route;
pub mod route_auth;
pub mod route_basic;
pub mod route_public;
pub mod route_rest;
pub mod route_upload;
pub mod route_params;
pub mod route_manager;

use axum::routing::MethodRouter;
use http::Method;
pub use route::*;
pub use route_auth::*;
pub use route_basic::*;
pub use route_public::*;
pub use route_rest::*;
pub use route_upload::*;
pub use route_params::*;

use crate::app::App;

pub trait PrintRoute {
    fn println(&self);
}

pub trait RouteRegistrator {
    fn push_route(&mut self, path: &str, router: MethodRouter, method: Option<&str>, is_protected: bool, options: Option<&[String]>);
}

pub trait RouteGenerator {
    fn make_routes(&self, app: &mut App);
}

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
        _ => Method::GET,
    }
}
