// Transport-agnostic integration handlers (TTS, Spotify) shared between
// the Tauri desktop binary and the Axum server. Phase 4d brought TTS in;
// Phase 4e adds Spotify (with the loopback OAuth callback listener kept
// Tauri-only — Phase 4f adds the browser-friendly HTTPS callback route).

pub mod spotify;
pub mod tts;
