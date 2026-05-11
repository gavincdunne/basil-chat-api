use bytes::Bytes;
use futures_util::Stream;
use reqwest::Client;

use crate::error::AppError;
use super::types::{AnthropicMessage, AnthropicRequest};

const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";
const MODEL: &str = "claude-sonnet-4-6";
const MAX_TOKENS: u32 = 1024;

pub const SYSTEM_PROMPT: &str = "\
You are Basil, a knowledgeable and empathetic companion for people living with Type 1 Diabetes. \
You help users understand their BG readings, insulin doses, carb counts, and general T1D management. \
You are not a medical professional and always encourage users to consult their endocrinologist for \
medical decisions. Keep responses concise and practical. Never store or repeat specific health \
values back to the user in a way that could feel intrusive.";

pub struct AnthropicClient {
    client: Client,
    api_key: String,
}

impl AnthropicClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    pub async fn stream_chat(
        &self,
        messages: Vec<AnthropicMessage>,
    ) -> Result<impl Stream<Item = Result<Bytes, reqwest::Error>>, AppError> {
        let request = AnthropicRequest {
            model: MODEL.to_string(),
            max_tokens: MAX_TOKENS,
            system: SYSTEM_PROMPT.to_string(),
            messages,
            stream: true,
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
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::Anthropic(format!("{status}: {body}")));
        }

        Ok(response.bytes_stream())
    }
}
