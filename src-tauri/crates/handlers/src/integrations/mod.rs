// Transport-agnostic integration handlers (TTS, future Spotify) shared
// between the Tauri desktop binary and the Axum server. Currently houses
// just `tts` from Phase 4d; Phase 4e/4f will add `spotify` alongside.

pub mod tts;
