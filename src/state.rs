//! Shared application state threaded through all Axum handlers via `State<T>`.
//!
//! `AppState` is cheap to clone — the client is behind an `Arc` and the
//! API key is a small string.

use std::sync::Arc;

use crate::anthropic::client::ChatClient;

/// State shared across all request handlers.
#[derive(Clone)]
pub struct AppState {
    /// The chat client used to stream responses from the AI backend.
    /// Stored as a trait object so tests can inject a mock implementation.
    pub client: Arc<dyn ChatClient>,

    /// The expected bearer token. Incoming requests are validated against this.
    pub api_key: String,
}
