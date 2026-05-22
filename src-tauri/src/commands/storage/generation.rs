// Thin Tauri-side wrappers. The non-image connection-test surface lives in the
// shared `marinara_handlers::llm` module so the Axum server can call the same
// code path; image-generation testing is still desktop-only (Phase 4c will
// lift it).
use super::*;

pub(crate) async fn test_connection(state: &AppState, id: &str) -> AppResult<Value> {
    marinara_handlers::llm::connection_test(&state.storage, id).await
}

pub(crate) async fn test_message(state: &AppState, id: &str) -> AppResult<Value> {
    marinara_handlers::llm::connection_test_message(&state.storage, id).await
}
