//! Shared mapping from fosk collection errors to HTTP error responses.
//!
//! Every response produced here uses the same JSON shape:
//! `{"error": "<machine_code>", "message": "<human readable message>"}`.

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use fosk::{AddBatchError, AddError, CollectionReadError, CollectionWriteError, LoadCollectionError};
use serde_json::json;

/// Builds a JSON error response with the given status, machine-readable
/// error code, and human-readable message.
pub fn error_response(status: StatusCode, error: &str, message: impl Into<String>) -> Response {
    (
        status,
        Json(json!({ "error": error, "message": message.into() })),
    )
        .into_response()
}

fn internal_error() -> Response {
    error_response(
        StatusCode::INTERNAL_SERVER_ERROR,
        "internal_error",
        "The collection lock is poisoned and the operation could not complete",
    )
}

/// Maps a [`CollectionReadError`] to an HTTP error response.
pub fn read_error_response(err: CollectionReadError) -> Response {
    match err {
        CollectionReadError::LockPoisoned => internal_error(),
    }
}

/// Maps a [`CollectionWriteError`] to an HTTP error response.
pub fn write_error_response(err: CollectionWriteError) -> Response {
    match err {
        CollectionWriteError::LockPoisoned => internal_error(),
    }
}

/// Maps an [`AddError`] to an HTTP error response.
pub fn add_error_response(err: AddError) -> Response {
    match err {
        AddError::LockPoisoned => internal_error(),
        AddError::NonObjectItem => error_response(
            StatusCode::BAD_REQUEST,
            "invalid_payload",
            "The request body must be a JSON object",
        ),
        AddError::MissingId { id_key } => error_response(
            StatusCode::BAD_REQUEST,
            "missing_id",
            format!("The request body is missing the required id field '{id_key}'"),
        ),
        AddError::DuplicateId { id } => error_response(
            StatusCode::CONFLICT,
            "duplicate_id",
            format!("An item with id '{id}' already exists"),
        ),
    }
}

/// Maps an [`AddBatchError`] to an HTTP error response.
pub fn add_batch_error_response(err: AddBatchError) -> Response {
    match err {
        AddBatchError::LockPoisoned => internal_error(),
        AddBatchError::NonArrayInput => error_response(
            StatusCode::BAD_REQUEST,
            "invalid_payload",
            "The request body must be a JSON array",
        ),
        AddBatchError::NonObjectItem { index } => error_response(
            StatusCode::BAD_REQUEST,
            "invalid_payload",
            format!("Item at index {index} must be a JSON object"),
        ),
        AddBatchError::MissingId { index, id_key } => error_response(
            StatusCode::BAD_REQUEST,
            "missing_id",
            format!("Item at index {index} is missing the required id field '{id_key}'"),
        ),
        AddBatchError::DuplicateId { index, id } => error_response(
            StatusCode::CONFLICT,
            "duplicate_id",
            format!("Item at index {index} duplicates existing id '{id}'"),
        ),
        AddBatchError::InvalidIntId { index } => error_response(
            StatusCode::BAD_REQUEST,
            "invalid_id",
            format!("Item at index {index} has an invalid integer id"),
        ),
    }
}

/// Maps a [`LoadCollectionError`] to an HTTP error response.
pub fn load_collection_error_response(err: LoadCollectionError) -> Response {
    match err {
        LoadCollectionError::LockPoisoned => internal_error(),
        LoadCollectionError::NonArrayInput => error_response(
            StatusCode::BAD_REQUEST,
            "invalid_payload",
            "The loaded JSON must contain an array at the root",
        ),
        LoadCollectionError::FileRead { path } => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "file_read_error",
            format!("Could not read file '{path}'"),
        ),
        LoadCollectionError::InvalidJson { path } => error_response(
            StatusCode::BAD_REQUEST,
            "invalid_payload",
            format!("File '{path}' does not contain valid JSON"),
        ),
        LoadCollectionError::Batch(inner) => add_batch_error_response(inner),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;

    async fn body_json(response: Response) -> serde_json::Value {
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test]
    async fn maps_collection_read_lock_poisoned_to_internal_error() {
        let response = read_error_response(CollectionReadError::LockPoisoned);
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let body = body_json(response).await;
        assert_eq!(body["error"], "internal_error");
    }

    #[tokio::test]
    async fn maps_collection_write_lock_poisoned_to_internal_error() {
        let response = write_error_response(CollectionWriteError::LockPoisoned);
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let body = body_json(response).await;
        assert_eq!(body["error"], "internal_error");
    }

    #[tokio::test]
    async fn maps_add_error_variants() {
        let response = add_error_response(AddError::LockPoisoned);
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(body_json(response).await["error"], "internal_error");

        let response = add_error_response(AddError::NonObjectItem);
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = body_json(response).await;
        assert_eq!(body["error"], "invalid_payload");
        assert_eq!(body["message"], "The request body must be a JSON object");

        let response = add_error_response(AddError::MissingId {
            id_key: "id".to_string(),
        });
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = body_json(response).await;
        assert_eq!(body["error"], "missing_id");
        assert_eq!(
            body["message"],
            "The request body is missing the required id field 'id'"
        );

        let response = add_error_response(AddError::DuplicateId { id: "1".to_string() });
        assert_eq!(response.status(), StatusCode::CONFLICT);
        let body = body_json(response).await;
        assert_eq!(body["error"], "duplicate_id");
        assert_eq!(body["message"], "An item with id '1' already exists");
    }

    #[tokio::test]
    async fn maps_add_batch_error_variants() {
        let response = add_batch_error_response(AddBatchError::LockPoisoned);
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(body_json(response).await["error"], "internal_error");

        let response = add_batch_error_response(AddBatchError::NonArrayInput);
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(body_json(response).await["error"], "invalid_payload");

        let response = add_batch_error_response(AddBatchError::NonObjectItem { index: 2 });
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = body_json(response).await;
        assert_eq!(body["error"], "invalid_payload");
        assert_eq!(body["message"], "Item at index 2 must be a JSON object");

        let response = add_batch_error_response(AddBatchError::MissingId {
            index: 1,
            id_key: "id".to_string(),
        });
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = body_json(response).await;
        assert_eq!(body["error"], "missing_id");
        assert_eq!(
            body["message"],
            "Item at index 1 is missing the required id field 'id'"
        );

        let response = add_batch_error_response(AddBatchError::DuplicateId {
            index: 3,
            id: "7".to_string(),
        });
        assert_eq!(response.status(), StatusCode::CONFLICT);
        let body = body_json(response).await;
        assert_eq!(body["error"], "duplicate_id");
        assert_eq!(body["message"], "Item at index 3 duplicates existing id '7'");

        let response = add_batch_error_response(AddBatchError::InvalidIntId { index: 4 });
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = body_json(response).await;
        assert_eq!(body["error"], "invalid_id");
        assert_eq!(body["message"], "Item at index 4 has an invalid integer id");
    }

    #[tokio::test]
    async fn maps_load_collection_error_variants() {
        let response = load_collection_error_response(LoadCollectionError::LockPoisoned);
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(body_json(response).await["error"], "internal_error");

        let response = load_collection_error_response(LoadCollectionError::NonArrayInput);
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = body_json(response).await;
        assert_eq!(body["error"], "invalid_payload");
        assert_eq!(
            body["message"],
            "The loaded JSON must contain an array at the root"
        );

        let response = load_collection_error_response(LoadCollectionError::FileRead {
            path: "data.json".to_string(),
        });
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let body = body_json(response).await;
        assert_eq!(body["error"], "file_read_error");
        assert_eq!(body["message"], "Could not read file 'data.json'");

        let response = load_collection_error_response(LoadCollectionError::InvalidJson {
            path: "data.json".to_string(),
        });
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = body_json(response).await;
        assert_eq!(body["error"], "invalid_payload");
        assert_eq!(
            body["message"],
            "File 'data.json' does not contain valid JSON"
        );

        // Batch(...) delegates to add_batch_error_response
        let response = load_collection_error_response(LoadCollectionError::Batch(
            AddBatchError::DuplicateId {
                index: 0,
                id: "1".to_string(),
            },
        ));
        assert_eq!(response.status(), StatusCode::CONFLICT);
        let body = body_json(response).await;
        assert_eq!(body["error"], "duplicate_id");
        assert_eq!(body["message"], "Item at index 0 duplicates existing id '1'");
    }
}
