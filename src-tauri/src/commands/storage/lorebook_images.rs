// Thin Tauri adapter over `marinara_handlers::lorebook_images`. The lift
// (Phase 3e2) replaced `&AppState` with the explicit (`storage`, `data_dir`)
// pair so the Axum server can share the same logic.

use crate::state::AppState;
use marinara_core::AppResult;
use marinara_handlers::lorebook_images as handlers;
use serde_json::Value;

pub(crate) fn update_lorebook_image(
    state: &AppState,
    lorebook_id: &str,
    body: Value,
) -> AppResult<Value> {
    handlers::update_lorebook_image(&state.storage, &state.data_dir, lorebook_id, body)
}

pub(crate) fn remove_lorebook_image_file(state: &AppState, record: &Value) {
    handlers::remove_lorebook_image_file(&state.data_dir, record)
}

pub(crate) fn lorebook_image_file_path(
    state: &AppState,
    encoded_filename: &str,
) -> AppResult<Value> {
    handlers::lorebook_image_file_path(&state.data_dir, encoded_filename)
}
