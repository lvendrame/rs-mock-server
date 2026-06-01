//! Axum handlers used by generated mock-server routes.

/// Static file and generated-content handlers.
pub mod basic_handlers;
pub use basic_handlers::*;

/// REST collection handlers.
pub mod rest_handlers;
pub use rest_handlers::*;

/// Upload and download handlers.
pub mod upload_handlers;
pub use upload_handlers::*;

/// Authentication handlers and middleware.
pub mod auth_handlers;
pub use auth_handlers::*;

/// Internal collection inspection handlers.
pub mod collections_handlers;
pub use collections_handlers::*;

/// GraphQL and GraphiQL handlers.
pub mod graphql_handlers;
pub use graphql_handlers::*;

/// Shared handler utilities.
pub mod utils;
pub use utils::*;
