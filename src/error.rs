use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Unauthorized")]
    Unauthorized,

    #[error("Anthropic API error: {0}")]
    Anthropic(String),

    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Unauthorized          => (StatusCode::UNAUTHORIZED,           self.to_string()),
            AppError::Anthropic(msg)        => (StatusCode::BAD_GATEWAY,            msg.clone()),
            AppError::Request(_)            => (StatusCode::INTERNAL_SERVER_ERROR,  self.to_string()),
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}
