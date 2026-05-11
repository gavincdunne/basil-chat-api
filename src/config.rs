//! Application configuration loaded from environment variables.
//!
//! All variables are required at startup except `PORT`, which defaults to 8080.
//! The app will panic with a clear message if a required variable is missing —
//! fail fast is preferable to a runtime error mid-request.

use std::env;

/// Holds all runtime configuration for the server.
#[derive(Clone)]
pub struct Config {
    /// Anthropic API key used to authenticate requests to Claude.
    pub anthropic_api_key: String,
    /// Shared secret that clients must include as `Authorization: Bearer <key>`.
    pub api_key: String,
    /// TCP port to bind the server on. Defaults to 8080.
    pub port: u16,
}

impl Config {
    /// Loads configuration from environment variables.
    ///
    /// # Panics
    /// Panics if `ANTHROPIC_API_KEY` or `API_KEY` are not set.
    pub fn from_env() -> Self {
        Self {
            anthropic_api_key: require("ANTHROPIC_API_KEY"),
            api_key:           require("API_KEY"),
            port: env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8080),
        }
    }
}

/// Reads an env var or panics with a helpful message.
fn require(key: &str) -> String {
    env::var(key).unwrap_or_else(|_| panic!("Missing required env var: {key}"))
}
