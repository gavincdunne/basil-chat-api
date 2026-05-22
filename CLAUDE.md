# basil-chat-api — Claude Code Rules

## Git workflow
- Branch from `develop`, never from `main`
- Branch naming: `MMDDYYYY-description` (e.g. `05222026-rate-limiting`) — date first so branches sort chronologically
- PRs always target `develop` — pass `--base develop` explicitly with `gh pr create`
- `main` is production-only; only merged from `develop` at release time
- No co-author lines, no AI attribution in commit messages

## Development standards
- TDD: write tests before or alongside implementation
- `cargo test` must pass before committing
- Run with: `~/.cargo/bin/cargo run`
- Needs `.env` with `ANTHROPIC_API_KEY`, `API_KEY`, `PORT=8080`

## Architecture
- Axum 0.8, Tokio, Reqwest 0.12 (streaming), async-trait
- `ChatClient` trait for testability — handlers depend on the trait, not `AnthropicClient` directly
- `AppState { client: Arc<dyn ChatClient>, api_key: String }`
- SSE stream forwarded raw from Anthropic to client

## HIPAA
- Never log request or response bodies — they may contain PHI
- `TraceLayer` stays at `info` level
- `MAX_MESSAGES = 40` guard on `/chat` — reject oversized histories with 422
- Request body limit: 32 KB via `RequestBodyLimitLayer`
- CORS locked down: `CorsLayer::new()` (no allowed browser origins)
- System prompt encodes minimum-necessary and no-PHI-relay rules

## companion repo
- KMP app lives at `~/Desktop/Projects/basil`
- Follows identical branching, TDD, and commit standards
