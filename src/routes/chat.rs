use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;

use crate::{anthropic::{client::AnthropicClient, types::AnthropicMessage}, error::AppError};

#[derive(Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<AnthropicMessage>,
}

pub async fn chat(
    State(client): State<AnthropicClient>,
    headers: HeaderMap,
    Json(body): Json<ChatRequest>,
) -> Result<Response, AppError> {
    // Validate bearer token
    let auth = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if !auth.starts_with("Bearer ") {
        return Err(AppError::Unauthorized);
    }

    let stream = client.stream_chat(body.messages).await?;

    // Forward the raw SSE byte stream from Anthropic directly to the client.
    // The client parses `data:` lines and extracts text deltas.
    let body = Body::from_stream(stream);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "text/event-stream")
        .header("cache-control", "no-cache")
        .header("x-accel-buffering", "no")
        .body(body)
        .unwrap())
}
