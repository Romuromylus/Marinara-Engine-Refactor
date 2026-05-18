# Rust Crates And Tauri Plugins

This file lists recommended Rust crates and Tauri plugins. Final choices should be validated during implementation.

Prefer fewer dependencies and simpler Rust code when a small local implementation is easier for a JavaScript-heavy team to review. Add crates when they remove real risk or complexity, not by default.

## Core Runtime

| Need | Candidate | Notes |
| --- | --- | --- |
| Async runtime | `tokio` | Required for async IO, process control, downloads, streams. |
| Serialization | `serde`, `serde_json` | Primary DTO format. |
| Validation | `validator` or custom validators | Keep complex domain validation close to DTOs/services. |
| Errors | `thiserror`, `anyhow` | `thiserror` for library errors, `anyhow` only at app edges if needed. |
| Logging | `tracing`, `tracing-subscriber` | Prefer structured logs. |
| IDs | `nanoid`, `uuid`, or `ulid` | Pick one and standardize. |
| Time | `time` or `chrono` | `time` is lean; `chrono` has broader ecosystem familiarity. |

## Tauri And Type Bindings

| Need | Candidate | Notes |
| --- | --- | --- |
| Tauri app | `tauri` v2 | Desktop shell and command system. |
| Type export | `specta`, `tauri-specta` | Generate TS types and typed command wrappers. |
| Dialogs | `tauri-plugin-dialog` | Native folder/file pickers. |
| Opener | `tauri-plugin-opener` | External links and files. |
| Filesystem | `tauri-plugin-fs` | Use sparingly with tight scopes. Rust should own most file access. |
| Shell | `tauri-plugin-shell` | Avoid for sidecar internals; use Rust `tokio::process`. |
| Stronghold | `tauri-plugin-stronghold` | Optional encrypted vault. Consider OS keychain first. |

## Storage

| Need | Candidate | Notes |
| --- | --- | --- |
| Atomic file writes | custom temp-file-and-rename helper | Keep simple and auditable. |
| JSON snapshots | `serde_json` | Required. Current app stores live data as file-native JSON table snapshots. |
| Storage manifest | custom serde structs | Track storage version, saved timestamp, table counts, and file-storage compatibility metadata. |
| Full-text search | `tantivy` | Useful for chat/lorebook/knowledge search later. |
| File walking | `walkdir` or `ignore` | Imports and asset scans. |
| Object storage | `object_store` or S3-compatible SDK | Sync server blob abstraction. |

## Security

| Need | Candidate | Notes |
| --- | --- | --- |
| OS secret storage | `keyring` | Good default for provider keys and OAuth tokens. |
| Encryption | `aes-gcm`, `rand` | Use only if maintaining app-managed encrypted files. |
| Constant-time compare | `constant_time_eq` | Admin secret/path token comparisons. |
| URL parsing | `url` | Provider and outbound validation. |
| IP networks | `ipnet` | Private/reserved network checks. |
| DNS lookup | `hickory-resolver` or `tokio::net` | Needed for SSRF defenses. |
| MIME sniffing | `infer` | Validate uploaded/downloaded file content. |

## HTTP And Downloads

| Need | Candidate | Notes |
| --- | --- | --- |
| HTTP client | `reqwest` | Provider calls, downloads, OAuth, bot-browser proxy. |
| Streaming | `futures`, `tokio-stream`, `async-stream` | Generation streams and download progress. |
| Multipart | `reqwest` multipart | Image providers and uploads to external services. |
| Rate limiting | `governor` | For local abuse controls if remote mode remains. |

## LLM And Providers

| Need | Candidate | Notes |
| --- | --- | --- |
| Unified LLM API | `genai` | Promising broad provider abstraction. Wrap behind Marinara traits. |
| OpenAI-compatible | `async-openai` | Mature option for OpenAI and compatible providers. |
| Agent framework | `rig` | Evaluate for future agent abstractions, but do not make it core initially. |
| Tokenization | provider-specific later, `tiktoken-rs` for OpenAI-ish estimates | Current app uses rough estimates in places. |

Recommendation: define Marinara's own `ChatProvider` trait first. Use `genai` or `async-openai` inside provider implementations, not throughout the app. That prevents a crate API from becoming your architecture.

## Images And Media

| Need | Candidate | Notes |
| --- | --- | --- |
| Image decoding/resizing | `image` | Avatar/background/gallery validation and thumbnails. |
| PNG chunks | `png`, `crc32fast` | Character card metadata if existing JS PNG parser is ported. |
| ZIP packages | `zip` | Current profile packages and workflow imports. |
| Audio metadata | evaluate per need | Frontend still plays audio. Rust may only manage files/cache. |

## Sidecar And Processes

| Need | Candidate | Notes |
| --- | --- | --- |
| Process control | `tokio::process::Command` | Sidecar runtime start/stop. |
| Free port | `tokio::net::TcpListener` | Bind port 0 on loopback. |
| Log tailing | custom file tailer | Emit `sidecar://log` events. |
| Archives | `zip`, `tar`, `flate2` | Runtime installs if needed. |
| Local inference runtime | Crane, llama.cpp, MLX | Prefer an existing runtime/package over porting old custom sidecar internals line-for-line. Evaluate Crane as a Rust/Candle candidate with OpenAI-compatible server support. |

## Integrations

| Need | Candidate | Notes |
| --- | --- | --- |
| Spotify OAuth | `reqwest`, `url`, custom PKCE | Keep tokens in Rust secrets. |
| Haptic | evaluate Buttplug Rust ecosystem | Current Node dependency uses Buttplug. Validate Rust support and packaging. |
| TTS | `reqwest`, provider-specific clients | Keep cache keys stable. |
| Translation | `reqwest` | DeepLX and other provider calls. |
| Discord webhook | `reqwest` | Simple HTTP client is enough. |
| Home Assistant | `reqwest` or local webhook handler | Depends whether Tauri app must expose a local listener. |

## Testing

| Need | Candidate | Notes |
| --- | --- | --- |
| Async tests | `tokio::test` | Services and repositories. |
| Temp dirs | `tempfile` | Storage and import tests. |
| HTTP mocking | `wiremock` or `httpmock` | Provider tests. |
| Snapshot tests | `insta` | Prompt assembly and game state outputs. |
| Property tests | `proptest` | Path safety, parsers, dice/mechanics. |

## Sync Server

| Need | Candidate | Notes |
| --- | --- | --- |
| Web server | `axum` | REST and WebSocket sync API. |
| Middleware | `tower-http` | Tracing, CORS, compression, request limits. |
| OpenAPI | `utoipa` | Generated API docs for Docker users and client tests. |
| PostgreSQL | `sqlx` | Sync server metadata for hosted production mode. Not used by the desktop app. |
| Password hashing | `argon2` | Account passwords and invite secrets. |
| Tokens | `jsonwebtoken` or `paseto` | Device access and refresh tokens. |
| CRDT exploration | `automerge` | Candidate for conflict-free metadata sync. |
| Blob backend | `object_store` or S3 SDK | Local filesystem and MinIO/S3-compatible storage. |
