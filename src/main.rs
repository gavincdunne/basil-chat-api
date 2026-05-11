//! # basil-chat-api
//!
//! Axum-based HTTP API that powers the Basil AI chat feature.
//!
//! ## Endpoints
//! - `POST /chat` — accepts a conversation history and streams a response from
//!   Claude via Server-Sent Events (SSE).
//!
//! ## Auth
//! Every request must include `Authorization: Bearer <API_KEY>` where `API_KEY`
//! matches the `API_KEY` environment variable. This is a simple shared-secret
//! model suitable for a mobile app hitting its own backend.
//!
//! ## Environment variables
//! See `.env.example` for required variables.

mod anthropic;
mod config;
mod error;
mod routes;
mod state;

use std::sync::Arc;

use axum::{routing::post, Router};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::info;

use anthropic::client::AnthropicClient;
use config::Config;
use routes::chat::chat;
use state::AppState;

#[tokio::main]
async fn main() {
    // Load .env if present — dev convenience, not required in production.
    dotenvy::dotenv().ok();

    // Structured logging. Set RUST_LOG=debug for verbose output.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "basil_chat_api=debug,tower_http=debug".into()),
        )
        .init();

    let config = Config::from_env();

    let state = AppState {
        client: Arc::new(AnthropicClient::new(config.anthropic_api_key)),
        api_key: config.api_key,
    };

    let app = Router::new()
        .route("/chat", post(chat))
        .with_state(state)
        .layer(CorsLayer::permissive())   // tighten origins before production
        .layer(TraceLayer::new_for_http());

    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    info!("basil-chat-api listening on {addr}");
    axum::serve(listener, app).await.unwrap();
}
