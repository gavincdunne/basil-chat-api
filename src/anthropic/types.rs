use serde::{Deserialize, Serialize};

// ── Outbound request ─────────────────────────────────────────

#[derive(Serialize)]
pub struct AnthropicRequest {
    pub model: String,
    pub max_tokens: u32,
    pub system: String,
    pub messages: Vec<AnthropicMessage>,
    pub stream: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AnthropicMessage {
    pub role: String,
    pub content: String,
}

// ── Inbound streaming events ──────────────────────────────────

#[derive(Deserialize, Debug)]
pub struct StreamEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub delta: Option<Delta>,
}

#[derive(Deserialize, Debug)]
pub struct Delta {
    #[serde(rename = "type")]
    pub delta_type: Option<String>,
    pub text: Option<String>,
}
