# basil-chat-api

Rust/Axum backend that powers the AI chat feature in [Basil](https://github.com/gavincdunne/basil) — a Type 1 Diabetes management app.

Accepts a conversation history, forwards it to Claude (Anthropic), and streams the response back to the mobile client via Server-Sent Events (SSE).

---

## Stack

| Concern | Crate |
|---|---|
| HTTP server | Axum 0.8 |
| Async runtime | Tokio 1 |
| Anthropic client | reqwest 0.12 (streaming) |
| Serialisation | serde / serde_json |
| CORS + tracing | tower-http |

---

## Endpoints

### `POST /chat`

Streams an AI response for a given conversation history.

**Auth:** `Authorization: Bearer <API_KEY>`

**Request body:**
```json
{
  "messages": [
    { "role": "user",      "content": "What is a safe BG range?" },
    { "role": "assistant", "content": "Generally 4–10 mmol/L..." },
    { "role": "user",      "content": "What about after meals?" }
  ]
}
```

**Response:** `200 OK` with `Content-Type: text/event-stream`

The raw SSE stream from Anthropic is forwarded directly. Clients parse `data:` lines and extract text from `content_block_delta` events.

**Error responses:**
| Status | Reason |
|---|---|
| `401` | Missing, malformed, or wrong bearer token |
| `502` | Anthropic returned a non-2xx response |
| `500` | Network-level failure reaching Anthropic |

---

## Running locally

**1. Install Rust**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

**2. Set environment variables**
```bash
cp .env.example .env
# fill in ANTHROPIC_API_KEY and API_KEY
```

**3. Run**
```bash
cargo run
```

Server starts on `http://localhost:8080` by default.

**Test a request:**
```bash
curl -N -X POST http://localhost:8080/chat \
  -H "Authorization: Bearer your-api-key" \
  -H "Content-Type: application/json" \
  -d '{"messages":[{"role":"user","content":"What is basal insulin?"}]}'
```

---

## Testing

```bash
cargo test
```

Tests use a `MockChatClient` that returns a fixed SSE chunk — no network calls, no API keys needed.

---

## Environment variables

| Variable | Required | Description |
|---|---|---|
| `ANTHROPIC_API_KEY` | Yes | Your Anthropic API key |
| `API_KEY` | Yes | Shared secret for client auth (`Authorization: Bearer`) |
| `PORT` | No | Port to listen on (default: `8080`) |

---

## Architecture

```
src/
  main.rs              server bootstrap, router wiring
  config.rs            env var loading
  error.rs             AppError → HTTP response mapping
  state.rs             AppState shared across handlers
  anthropic/
    mod.rs
    client.rs          ChatClient trait + AnthropicClient impl
    types.rs           Anthropic API request/response types
  routes/
    mod.rs
    chat.rs            POST /chat handler + tests
```

The `ChatClient` trait decouples handlers from the Anthropic implementation, making the request/auth logic fully testable without network calls or credentials.

---

## Deployment

Designed to deploy on [Fly.io](https://fly.io) or [Railway](https://railway.app). A `Dockerfile` and `fly.toml` will be added in a follow-up.

---

## Related

- [basil](https://github.com/gavincdunne/basil) — the KMP iOS/Android/Desktop app this API serves
