mod anthropic;
mod config;
mod error;
mod routes;

use axum::{routing::post, Router};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::info;

use anthropic::client::AnthropicClient;
use config::Config;
use routes::chat::chat;

#[tokio::main]
async fn main() {
    // Load .env if present (dev convenience — not required in production)
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "basil_chat_api=debug,tower_http=debug".into()),
        )
        .init();

    let config = Config::from_env();
    let client = AnthropicClient::new(config.anthropic_api_key.clone());

    let app = Router::new()
        .route("/chat", post(chat))
        .with_state(client)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    info!("basil-chat-api listening on {addr}");
    axum::serve(listener, app).await.unwrap();
}
