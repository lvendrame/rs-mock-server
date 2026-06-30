//! Utility helpers shared by route handlers.

use std::{ffi::OsString, path::Path};

use axum::{
    extract::Request,
    handler::Handler,
    http::{HeaderName, HeaderValue, StatusCode, header::CONTENT_TYPE},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{MethodRouter, any},
};

/// Returns the lowercase-sensitive file extension for a path, or an empty string.
pub fn get_file_extension(file_path: &OsString) -> String {
    Path::new(file_path)
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or_default()
        .to_string()
}

/// Returns true for file types that can be served as textual responses.
pub fn is_text_file(file_path: &OsString) -> bool {
    let extension = get_file_extension(file_path);
    extension == "txt"
        || extension == "md"
        || extension == "json"
        || extension == "jgd"
        || extension == "sql"
}

/// Returns true when the path has a JSON extension.
pub fn is_json(file_path: &OsString) -> bool {
    let extension = get_file_extension(file_path);
    extension == "json"
}

/// Returns true when the path has a JGD extension.
pub fn is_jgd(file_path: &OsString) -> bool {
    let extension = get_file_extension(file_path);
    extension == "jgd"
}

/// Returns true when the path has a SQL extension.
pub fn is_sql(file_path: &OsString) -> bool {
    let extension = get_file_extension(file_path);
    extension == "sql"
}

/// Returns true when the path has a TOML extension.
pub fn is_toml(file_path: &OsString) -> bool {
    let extension = get_file_extension(file_path);
    extension == "toml"
}

/// Extension trait for applying optional route response delays.
pub trait SleepThread {
    /// Sleeps the current thread when the option contains a delay in milliseconds.
    fn sleep_thread(self);
}

impl SleepThread for Option<u16> {
    fn sleep_thread(self) {
        if let Some(delay) = self {
            let millis = std::time::Duration::from_millis(delay.into());
            std::thread::sleep(millis);
        }
    }
}

/// `Accept-Query` response header value, advertised per RFC 10008 ยง7. This
/// server doesn't parse or validate request content against any particular
/// format — mock routes return their configured response regardless of
/// body — so `*/*` is the accurate claim, not a specific media type.
const ACCEPT_QUERY_MEDIA_TYPE: &str = "*/*";

/// Routes requests with the HTTP `QUERY` verb (RFC 10008), a safe,
/// idempotent, body-bearing read.
///
/// Axum's `MethodFilter` has no bit for non-standard methods, so the only
/// way to claim a path for `QUERY` is `any()`; this wraps it with the
/// RFC's normative server requirements: requests with any other method get
/// 405, requests with content but no `Content-Type` get 400, and every
/// response carries an `Accept-Query` header advertising the accepted
/// content type.
pub fn query<H, T, S>(handler: H) -> MethodRouter<S>
where
    H: Handler<T, S>,
    T: 'static,
    S: Clone + Send + Sync + 'static,
{
    any(handler).layer(middleware::from_fn(enforce_query_semantics))
}

async fn enforce_query_semantics(req: Request, next: Next) -> Response {
    let mut response = if req.method().as_str() != "QUERY" {
        StatusCode::METHOD_NOT_ALLOWED.into_response()
    } else if !req.headers().contains_key(CONTENT_TYPE) {
        // RFC 10008 ยง2: "Servers MUST fail the request if the Content-Type
        // request field is missing" — unconditionally, even with no body.
        StatusCode::BAD_REQUEST.into_response()
    } else {
        next.run(req).await
    };

    response.headers_mut().insert(
        HeaderName::from_static("accept-query"),
        HeaderValue::from_static(ACCEPT_QUERY_MEDIA_TYPE),
    );
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        Router,
        body::{Body, to_bytes},
        http::Request,
    };
    use std::ffi::OsString;
    use std::time::Instant;
    use tower::ServiceExt;

    #[tokio::test]
    async fn query_routes_requests_with_query_method() {
        let app = Router::new().route("/search", query(|| async { "found" }));

        let response = app
            .oneshot(
                Request::builder()
                    .method("QUERY")
                    .uri("/search")
                    .header(CONTENT_TYPE, "application/json")
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get("accept-query").unwrap(),
            ACCEPT_QUERY_MEDIA_TYPE
        );
        assert_eq!(
            to_bytes(response.into_body(), usize::MAX).await.unwrap(),
            "found"
        );
    }

    #[tokio::test]
    async fn query_rejects_empty_body_without_content_type() {
        let app = Router::new().route("/search", query(|| async { "found" }));

        let response = app
            .oneshot(
                Request::builder()
                    .method("QUERY")
                    .uri("/search")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // RFC 10008: Content-Type is mandatory even for an empty body.
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn query_rejects_non_query_methods() {
        let app = Router::new().route("/search", query(|| async { "found" }));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/search")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
        assert_eq!(
            response.headers().get("accept-query").unwrap(),
            ACCEPT_QUERY_MEDIA_TYPE
        );
    }

    #[tokio::test]
    async fn query_rejects_body_without_content_type() {
        let app = Router::new().route("/search", query(|| async { "found" }));

        let response = app
            .oneshot(
                Request::builder()
                    .method("QUERY")
                    .uri("/search")
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            response.headers().get("accept-query").unwrap(),
            ACCEPT_QUERY_MEDIA_TYPE
        );
    }

    #[test]
    fn file_type_helpers_detect_supported_extensions() {
        assert_eq!(get_file_extension(&OsString::from("data.json")), "json");
        assert_eq!(get_file_extension(&OsString::from("README")), "");
        assert!(is_text_file(&OsString::from("data.json")));
        assert!(is_text_file(&OsString::from("data.jgd")));
        assert!(is_text_file(&OsString::from("query.sql")));
        assert!(is_json(&OsString::from("data.json")));
        assert!(is_jgd(&OsString::from("data.jgd")));
        assert!(is_sql(&OsString::from("query.sql")));
        assert!(is_toml(&OsString::from("config.toml")));
        assert!(!is_text_file(&OsString::from("image.png")));
    }

    #[test]
    fn sleep_thread_handles_none_and_some() {
        let start = Instant::now();
        None::<u16>.sleep_thread();
        Some(1).sleep_thread();
        assert!(start.elapsed().as_millis() >= 1);
    }
}
