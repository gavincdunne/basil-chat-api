//! `POST /chat` — streams an AI response for a given conversation history.
//!
//! ## Request
//! ```json
//! {
//!   "messages": [
//!     { "role": "user", "content": "What is a safe BG range?" }
//!   ]
//! }
//! ```
//!
//! ## Response
//! `200 OK` with `Content-Type: text/event-stream`. The raw SSE byte stream
//! from Anthropic is forwarded directly — clients parse `data:` lines and
//! extract text deltas from `content_block_delta` events.
//!
//! ## Auth
//! Requires `Authorization: Bearer <API_KEY>`. Returns `401` if the header is
//! missing, malformed, or the token does not match the configured `API_KEY`.

use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Response,
    Json,
};
use serde::Deserialize;

use crate::{
    anthropic::{client::ChatClient, types::AnthropicMessage},
    error::AppError,
    state::AppState,
};

/// Request body for `POST /chat`.
#[derive(Deserialize)]
pub struct ChatRequest {
    /// Conversation history to send to the model. Must alternate user/assistant
    /// turns and end with a user message.
    pub messages: Vec<AnthropicMessage>,
}

/// Validates the bearer token, forwards the conversation to the AI client,
/// and streams the response back as SSE.
pub async fn chat(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<ChatRequest>,
) -> Result<Response, AppError> {
    // Extract and validate the bearer token against the configured API key.
    let token = bearer_token(&headers).ok_or(AppError::Unauthorized)?;
    if token != state.api_key {
        return Err(AppError::Unauthorized);
    }

    let stream = state.client.stream_chat(body.messages).await?;

    // Forward the raw SSE byte stream from Anthropic directly to the client.
    // No re-serialisation — the client parses `data:` lines itself.
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "text/event-stream")
        .header("cache-control", "no-cache")
        .header("x-accel-buffering", "no")  // disables proxy buffering (nginx/fly.io)
        .body(Body::from_stream(stream))
        .unwrap())
}

/// Extracts the token value from an `Authorization: Bearer <token>` header.
///
/// Returns `None` if the header is absent or does not use the Bearer scheme.
pub fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::to_bytes, http::Request, routing::post, Router};
    use bytes::Bytes;
    use futures_util::stream;
    use std::sync::Arc;
    use tower::ServiceExt;

    use crate::anthropic::client::ChatStream;

    // ── bearer_token ──────────────────────────────────────────────────────────

    #[test]
    fn bearer_token_returns_none_when_header_missing() {
        let headers = HeaderMap::new();
        assert!(bearer_token(&headers).is_none());
    }

    #[test]
    fn bearer_token_returns_none_for_non_bearer_scheme() {
        let mut headers = HeaderMap::new();
        headers.insert("authorization", "Basic abc123".parse().unwrap());
        assert!(bearer_token(&headers).is_none());
    }

    #[test]
    fn bearer_token_extracts_token_value() {
        let mut headers = HeaderMap::new();
        headers.insert("authorization", "Bearer my-secret-key".parse().unwrap());
        assert_eq!(bearer_token(&headers), Some("my-secret-key"));
    }

    // ── handler ───────────────────────────────────────────────────────────────

    /// Test double that returns a single SSE chunk without hitting the network.
    struct MockChatClient;

    #[async_trait::async_trait]
    impl ChatClient for MockChatClient {
        async fn stream_chat(&self, _: Vec<AnthropicMessage>) -> Result<ChatStream, AppError> {
            let chunk = Ok(Bytes::from("data: {\"type\":\"content_block_delta\"}\n\n"));
            Ok(Box::pin(stream::iter(vec![chunk])))
        }
    }

    fn test_app() -> Router {
        let state = AppState {
            client:  Arc::new(MockChatClient),
            api_key: "test-key".to_string(),
        };
        Router::new().route("/chat", post(chat)).with_state(state)
    }

    fn chat_body() -> &'static str {
        r#"{"messages":[{"role":"user","content":"What is a normal BG range?"}]}"#
    }

    #[tokio::test]
    async fn missing_auth_header_returns_401() {
        let response = test_app()
            .oneshot(
                Request::builder()
                    .method("POST").uri("/chat")
                    .header("content-type", "application/json")
                    .body(Body::from(chat_body()))
                    .unwrap(),
            )
            .await.unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn wrong_token_returns_401() {
        let response = test_app()
            .oneshot(
                Request::builder()
                    .method("POST").uri("/chat")
                    .header("content-type", "application/json")
                    .header("authorization", "Bearer wrong-key")
                    .body(Body::from(chat_body()))
                    .unwrap(),
            )
            .await.unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn valid_token_returns_200_with_sse_content_type() {
        let response = test_app()
            .oneshot(
                Request::builder()
                    .method("POST").uri("/chat")
                    .header("content-type", "application/json")
                    .header("authorization", "Bearer test-key")
                    .body(Body::from(chat_body()))
                    .unwrap(),
            )
            .await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "text/event-stream"
        );
    }

    #[tokio::test]
    async fn valid_token_streams_response_body() {
        let response = test_app()
            .oneshot(
                Request::builder()
                    .method("POST").uri("/chat")
                    .header("content-type", "application/json")
                    .header("authorization", "Bearer test-key")
                    .body(Body::from(chat_body()))
                    .unwrap(),
            )
            .await.unwrap();

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert!(body.starts_with(b"data:"));
    }
}
