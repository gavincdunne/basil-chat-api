//! Anthropic API client.
//!
//! [`AnthropicClient`] is the production implementation of [`ChatClient`].
//! The trait exists so handlers can be tested with a [`MockChatClient`] that
//! never touches the network.

use async_trait::async_trait;
use bytes::Bytes;
use futures_util::Stream;
use reqwest::Client;
use std::pin::Pin;

use crate::error::AppError;
use super::types::{AnthropicMessage, AnthropicRequest};

const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";
/// Model used for all chat completions.
const MODEL: &str = "claude-sonnet-4-6";
/// Hard cap on response length. Keeps costs predictable for a mobile use case.
const MAX_TOKENS: u32 = 1024;

/// Basil's system prompt. Establishes the assistant's persona, scope, and
/// safety guardrails before any user message is processed.
pub const SYSTEM_PROMPT: &str = "\
You are Basil, a knowledgeable and empathetic companion for people living with Type 1 Diabetes. \
You help users understand their BG readings, insulin doses, carb counts, and general T1D management. \
You are not a medical professional and always encourage users to consult their endocrinologist for \
medical decisions. Keep responses concise and practical. Never store or repeat specific health \
values back to the user in a way that could feel intrusive.";

/// A pinned, heap-allocated byte stream returned from [`ChatClient::stream_chat`].
///
/// Using a type alias keeps handler signatures readable and allows the
/// underlying stream type to change without touching call sites.
pub type ChatStream = Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>>;

/// Abstraction over the AI chat backend.
///
/// Implement this trait to swap in a different provider or a test double
/// without changing any handler code.
#[async_trait]
pub trait ChatClient: Send + Sync {
    /// Sends `messages` to the AI backend and returns a byte stream of SSE
    /// data that can be forwarded directly to the HTTP client.
    async fn stream_chat(&self, messages: Vec<AnthropicMessage>) -> Result<ChatStream, AppError>;
}

/// Production [`ChatClient`] backed by the Anthropic Messages API.
pub struct AnthropicClient {
    client: Client,
    api_key: String,
}

impl AnthropicClient {
    /// Creates a new client authenticated with `api_key`.
    pub fn new(api_key: String) -> Self {
        Self { client: Client::new(), api_key }
    }
}

#[async_trait]
impl ChatClient for AnthropicClient {
    async fn stream_chat(&self, messages: Vec<AnthropicMessage>) -> Result<ChatStream, AppError> {
        let request = AnthropicRequest {
            model:      MODEL.to_string(),
            max_tokens: MAX_TOKENS,
            system:     SYSTEM_PROMPT.to_string(),
            messages,
            stream:     true,
        };

        let response = self
            .client
            .post(ANTHROPIC_API_URL)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body   = response.text().await.unwrap_or_default();
            return Err(AppError::Anthropic(format!("{status}: {body}")));
        }

        // Wrap in a Box<Pin<...>> so the concrete stream type is erased and
        // the return type matches the trait signature.
        Ok(Box::pin(response.bytes_stream()))
    }
}
