use std::env;

#[derive(Clone)]
pub struct Config {
    pub anthropic_api_key: String,
    pub api_key: String,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            anthropic_api_key: require("ANTHROPIC_API_KEY"),
            api_key: require("API_KEY"),
            port: env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8080),
        }
    }
}

fn require(key: &str) -> String {
    env::var(key).unwrap_or_else(|_| panic!("Missing required env var: {key}"))
}
