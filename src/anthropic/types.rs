//! Serde types for the Anthropic Messages API.
//!
//! Only the fields we actually use are modelled. Anthropic returns more
//! fields in streaming events but we forward the raw bytes to the client
//! rather than deserialising the full payload.

use serde::{Deserialize, Serialize};

// ── Outbound request ──────────────────────────────────────────────────────────

/// Top-level request body sent to `POST /v1/messages`.
#[derive(Serialize)]
pub struct AnthropicRequest {
    /// The Claude model to use (e.g. `claude-sonnet-4-6`).
    pub model: String,
    /// Maximum tokens in the response.
    pub max_tokens: u32,
    /// System prompt injected before the conversation.
    pub system: String,
    /// The conversation history, alternating user / assistant turns.
    pub messages: Vec<AnthropicMessage>,
    /// When `true`, Anthropic streams the response as SSE.
    pub stream: bool,
}

/// A single turn in the conversation.
#[derive(Serialize, Deserialize, Clone)]
pub struct AnthropicMessage {
    /// `"user"` or `"assistant"`.
    pub role: String,
    /// Plain text content of the message.
    pub content: String,
}

// ── Inbound streaming events ──────────────────────────────────────────────────

#[allow(dead_code)]
/// A single SSE event from the Anthropic streaming API.
///
/// The full event taxonomy is documented at
/// <https://docs.anthropic.com/en/api/messages-streaming>.
/// We only inspect `content_block_delta` events to extract text chunks;
/// all others are forwarded as-is to the client.
#[derive(Deserialize, Debug)]
pub struct StreamEvent {
    /// Event type — e.g. `"content_block_delta"`, `"message_stop"`.
    #[serde(rename = "type")]
    pub event_type: String,
    /// Present on `content_block_delta` events.
    pub delta: Option<Delta>,
}

#[allow(dead_code)]
/// The delta payload inside a `content_block_delta` event.
#[derive(Deserialize, Debug)]
pub struct Delta {
    /// Delta type — typically `"text_delta"`.
    #[serde(rename = "type")]
    pub delta_type: Option<String>,
    /// The incremental text chunk.
    pub text: Option<String>,
}
