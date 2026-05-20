use crate::state::AppState;
use marinara_core::AppResult;

pub(crate) fn ensure_layout(state: &AppState) -> AppResult<serde_json::Value> {
    state.memory.ensure_layout()
}

pub(crate) fn list_notes(
    state: &AppState,
    options: Option<serde_json::Value>,
) -> AppResult<Vec<serde_json::Value>> {
    state.memory.list_notes(options)
}

pub(crate) fn get_note(state: &AppState, id: &str) -> AppResult<Option<serde_json::Value>> {
    state.memory.get_note(id)
}

pub(crate) fn create_note(
    state: &AppState,
    value: serde_json::Value,
) -> AppResult<serde_json::Value> {
    state.memory.create_note(value)
}

pub(crate) fn update_note(
    state: &AppState,
    id: &str,
    patch: serde_json::Value,
) -> AppResult<serde_json::Value> {
    state.memory.update_note(id, patch)
}

pub(crate) fn archive_note(state: &AppState, id: &str) -> AppResult<serde_json::Value> {
    state.memory.archive_note(id)
}

pub(crate) fn append_event(
    state: &AppState,
    value: serde_json::Value,
) -> AppResult<serde_json::Value> {
    state.memory.append_event(value)
}

pub(crate) fn get_manifest(state: &AppState) -> AppResult<serde_json::Value> {
    state.memory.manifest()
}

pub(crate) fn validate_vault(state: &AppState) -> AppResult<serde_json::Value> {
    state.memory.validate_vault()
}

pub(crate) fn rebuild_indexes(
    state: &AppState,
    request: Option<serde_json::Value>,
) -> AppResult<serde_json::Value> {
    state.memory.rebuild_indexes(request)
}
