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
//! ## HIPAA notes
//! This API handles Protected Health Information (PHI) in transit. The
//! following hardening measures are in place:
//! - TLS termination is the responsibility of the reverse proxy (Fly.io, nginx).
//! - Request bodies are capped at [`REQUEST_BODY_LIMIT_BYTES`] to limit PHI
//!   surface area per request.
//! - CORS is locked to deny all cross-origin browser requests; the mobile app
//!   never sends a preflight and does not need permissive CORS.
//! - Security headers (`X-Content-Type-Options`, `X-Frame-Options`,
//!   `Cache-Control`) are set on every response.
//! - The `TraceLayer` logs request metadata only — never request or response
//!   bodies, which may contain PHI.
//! - The Anthropic client prepends a HIPAA-aligned system prompt to every
//!   request instructing the model not to ask for unnecessary PHI or echo it
//!   back to the user.
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
use axum::http::{HeaderName, HeaderValue};
use tower_http::{
    cors::CorsLayer,
    limit::RequestBodyLimitLayer,
    set_header::SetResponseHeaderLayer,
    trace::TraceLayer,
};
use tracing::info;

use anthropic::client::AnthropicClient;
use config::Config;
use routes::chat::chat;
use state::AppState;

/// Maximum allowed request body size in bytes.
///
/// 32 KB is generous for a chat message list while bounding the PHI that can
/// be transmitted in a single request. A conversation that exceeds this is
/// almost certainly a client bug or an abuse attempt.
const REQUEST_BODY_LIMIT_BYTES: usize = 32 * 1024; // 32 KB

#[tokio::main]
async fn main() {
    // Load .env if present — dev convenience, not required in production.
    dotenvy::dotenv().ok();

    // Structured logging. Set RUST_LOG=debug for verbose output.
    // Note: never log request or response bodies — they may contain PHI.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "basil_chat_api=info,tower_http=info".into()),
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
        // ── Security layers ──────────────────────────────────────
        //
        // Body size limit: caps PHI surface area per request and prevents
        // memory exhaustion from oversized payloads.
        .layer(RequestBodyLimitLayer::new(REQUEST_BODY_LIMIT_BYTES))
        // CORS: mobile clients do not require CORS. Denying all cross-origin
        // browser requests prevents the API from being called from arbitrary
        // web pages. CorsLayer::new() with no allowed origins is the most
        // restrictive posture.
        .layer(CorsLayer::new())
        // Security headers on every response.
        .layer(SetResponseHeaderLayer::if_not_present(
            HeaderName::from_static("x-content-type-options"),
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            HeaderName::from_static("x-frame-options"),
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            HeaderName::from_static("cache-control"),
            HeaderValue::from_static("no-store"),
        ))
        // Request/response metadata logging. Bodies are never logged.
        .layer(TraceLayer::new_for_http());

    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    info!("basil-chat-api listening on {addr}");
    axum::serve(listener, app).await.unwrap();
}
