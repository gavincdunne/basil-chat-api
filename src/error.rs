//! Application-level error type.
//!
//! `AppError` implements `IntoResponse` so Axum handlers can return
//! `Result<T, AppError>` and get a correctly shaped JSON error response
//! with the right HTTP status code automatically.

use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    /// The request is missing a valid `Authorization: Bearer` header.
    #[error("Unauthorized")]
    Unauthorized,

    /// The request is malformed or violates a server-side policy (e.g. the
    /// message list exceeds the maximum allowed count).
    #[error("Bad request: {0}")]
    BadRequest(String),

    /// The Anthropic API returned a non-2xx response.
    #[error("Anthropic API error: {0}")]
    Anthropic(String),

    /// A network-level error occurred when calling Anthropic.
    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Unauthorized      => (StatusCode::UNAUTHORIZED,          self.to_string()),
            AppError::BadRequest(msg)   => (StatusCode::UNPROCESSABLE_ENTITY,  msg.clone()),
            AppError::Anthropic(msg)    => (StatusCode::BAD_GATEWAY,           msg.clone()),
            AppError::Request(_)        => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}
