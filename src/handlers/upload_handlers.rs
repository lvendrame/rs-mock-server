use std::{ffi::OsStr, fs, path::Path};

use axum::{
    extract::{Json, Multipart, Path as AxumPath},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use http::{
    HeaderMap, HeaderValue,
    header::{CONTENT_DISPOSITION, CONTENT_TYPE},
};
use mime_guess::from_path;
use serde_json::Value;

use crate::{
    app::App,
    route_builder::{FILE_NAME_PARAM, RouteUpload},
};

fn create_upload_route(app: &mut App, upload_def: &RouteUpload) {
    let route = upload_def.get_upload_route();
    let download_route = upload_def.get_download_route();
    let upload_path = upload_def.path.to_string_lossy().to_string();

    // POST /uploads - create new
    let uploads_router = post(async move |mut multipart: Multipart| {
        let mut file_name = "".to_string();

        while let Some(field) = multipart.next_field().await.unwrap() {
            let field_name = field.name().unwrap_or("file").to_string();
            file_name = field
                .file_name()
                .map(|name| name.to_string())
                .unwrap_or_else(|| "uploaded_file.bin".to_string());

            let data = field.bytes().await.unwrap();

            println!(
                "Received file '{}' in field '{}' with {} bytes",
                file_name,
                field_name,
                data.len()
            );

            // Save the file with its original name
            let file_path = format!("{}/{}", upload_path, file_name);
            tokio::fs::write(&file_path, &data).await.unwrap();
        }
        let response = Value::Object({
            let mut map = serde_json::Map::new();
            map.insert("status".to_string(), Value::String("success".to_string()));
            map.insert(
                "message".to_string(),
                Value::String("File uploaded successfully".to_string()),
            );
            map.insert("filename".to_string(), Value::String(file_name.clone()));
            map.insert(
                "filepath".to_string(),
                Value::String(download_route.replace(FILE_NAME_PARAM, &file_name)),
            );
            map
        });

        Json(response).into_response()
    });

    app.route(
        &route,
        uploads_router,
        Some("POST"),
        Some(&["upload".to_string()]),
    );
}

fn create_download_route(app: &mut App, upload_def: &RouteUpload) {
    let download_route = upload_def.get_download_route();
    let download_path = upload_def.path.to_string_lossy().to_string();

    // GET /uploads/{filename} - download file
    let download_router = get(move |AxumPath(file_name): AxumPath<String>| {
        async move {
            let file_path = Path::new(&download_path).join(&file_name);

            // Check if file exists
            if !file_path.exists() {
                return StatusCode::NOT_FOUND.into_response();
            }

            // Read file content
            match tokio::fs::read(&file_path).await {
                Ok(contents) => {
                    // Guess MIME type
                    let mime_type = from_path(&file_path).first_or_octet_stream().to_string();

                    // Set headers
                    let mut headers = HeaderMap::new();
                    headers.insert(CONTENT_TYPE, HeaderValue::from_str(&mime_type).unwrap());

                    headers.insert(
                        CONTENT_DISPOSITION,
                        HeaderValue::from_str(&format!("attachment; filename=\"{}\"", file_name))
                            .unwrap(),
                    );

                    (headers, contents).into_response()
                }
                Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            }
        }
    });

    app.route(
        &download_route,
        download_router,
        Some("GET"),
        Some(&["download".to_string()]),
    );
}

fn create_uploaded_list_route(app: &mut App, upload_def: &RouteUpload) {
    let route = upload_def.get_list_files_route();
    let download_route = upload_def.get_download_route();
    let upload_path = upload_def.path.to_string_lossy().to_string();

    // GET /uploads - download file
    let upload_list_router = get(move || {
        async move {
            let upload_path = Path::new(&upload_path);

            // Check if file exists
            if !upload_path.exists() {
                return StatusCode::NOT_FOUND.into_response();
            }

            let entries = fs::read_dir(upload_path).unwrap();
            let array = entries
                .filter_map(Result::ok)
                .filter(|entry| {
                    !entry
                        .path()
                        .extension()
                        .and_then(OsStr::to_str)
                        .unwrap_or_default()
                        .eq_ignore_ascii_case("toml")
                })
                .map(|entry| {
                    let value = download_route
                        .replace(FILE_NAME_PARAM, entry.file_name().to_str().unwrap());

                    Value::String(value)
                })
                .collect();

            let body = Value::Array(array);

            Json(body).into_response()
        }
    });

    app.route(&route, upload_list_router, Some("GET"), None);
}

/// Registers upload, download, and list-file routes for an upload directory.
pub fn build_upload_routes(app: &mut App, upload_def: &RouteUpload) {
    create_upload_route(app, upload_def);

    create_download_route(app, upload_def);

    create_uploaded_list_route(app, upload_def);
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::{Method, Request, header::CONTENT_TYPE},
    };
    use tower::ServiceExt;

    fn upload_def(path: &std::path::Path) -> RouteUpload {
        RouteUpload {
            path: path.as_os_str().to_os_string(),
            route: "/uploads".to_string(),
            is_temporary: false,
            is_protected: false,
            delay: None,
            upload_endpoint: None,
            download_endpoint: None,
            list_files_endpoint: None,
        }
    }

    #[tokio::test]
    async fn upload_list_and_download_routes_work() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("existing.txt"), "existing").unwrap();
        std::fs::write(temp_dir.path().join("config.toml"), "ignored = true").unwrap();

        let mut app = App::default();
        build_upload_routes(&mut app, &upload_def(temp_dir.path()));
        let router = app.take_router_for_test();

        let list = router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/uploads")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(list.status(), StatusCode::OK);
        let body: Value =
            serde_json::from_slice(&to_bytes(list.into_body(), usize::MAX).await.unwrap()).unwrap();
        assert_eq!(body.as_array().unwrap().len(), 1);
        assert_eq!(body[0], "/uploads/existing.txt");

        let download = router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/uploads/existing.txt")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(download.status(), StatusCode::OK);
        assert_eq!(download.headers().get(CONTENT_TYPE).unwrap(), "text/plain");
        assert_eq!(
            to_bytes(download.into_body(), usize::MAX).await.unwrap(),
            "existing"
        );

        let missing = router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/uploads/missing.txt")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(missing.status(), StatusCode::NOT_FOUND);

        let boundary = "BOUNDARY";
        let multipart = concat!(
            "--BOUNDARY\r\n",
            "Content-Disposition: form-data; name=\"file\"; filename=\"new.txt\"\r\n",
            "Content-Type: text/plain\r\n\r\n",
            "uploaded\r\n",
            "--BOUNDARY--\r\n"
        );
        let uploaded = router
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/uploads")
                    .header(
                        CONTENT_TYPE,
                        format!("multipart/form-data; boundary={boundary}"),
                    )
                    .body(Body::from(multipart))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(uploaded.status(), StatusCode::OK);
        assert_eq!(
            std::fs::read_to_string(temp_dir.path().join("new.txt")).unwrap(),
            "uploaded"
        );
    }

    #[tokio::test]
    async fn upload_list_reports_missing_folder() {
        let mut app = App::default();
        build_upload_routes(
            &mut app,
            &upload_def(std::path::Path::new("missing-uploads")),
        );

        let response = app
            .take_router_for_test()
            .oneshot(
                Request::builder()
                    .uri("/uploads")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
