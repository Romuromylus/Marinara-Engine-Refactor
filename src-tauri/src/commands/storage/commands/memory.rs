use super::memory;
use crate::state::AppState;
use marinara_core::AppError;
use serde_json::Value;
use tauri::State;

#[tauri::command]
pub fn memory_ensure_layout(state: State<'_, AppState>) -> Result<Value, AppError> {
    memory::ensure_layout(&state)
}

#[tauri::command]
pub fn memory_list_notes(
    state: State<'_, AppState>,
    options: Option<Value>,
) -> Result<Vec<Value>, AppError> {
    memory::list_notes(&state, options)
}

#[tauri::command]
pub fn memory_get_note(state: State<'_, AppState>, id: String) -> Result<Option<Value>, AppError> {
    memory::get_note(&state, &id)
}

#[tauri::command]
pub fn memory_create_note(state: State<'_, AppState>, value: Value) -> Result<Value, AppError> {
    memory::create_note(&state, value)
}

#[tauri::command]
pub fn memory_update_note(
    state: State<'_, AppState>,
    id: String,
    patch: Value,
) -> Result<Value, AppError> {
    memory::update_note(&state, &id, patch)
}

#[tauri::command]
pub fn memory_archive_note(state: State<'_, AppState>, id: String) -> Result<Value, AppError> {
    memory::archive_note(&state, &id)
}

#[tauri::command]
pub fn memory_append_event(state: State<'_, AppState>, value: Value) -> Result<Value, AppError> {
    memory::append_event(&state, value)
}

#[tauri::command]
pub fn memory_get_manifest(state: State<'_, AppState>) -> Result<Value, AppError> {
    memory::get_manifest(&state)
}

#[tauri::command]
pub fn memory_validate_vault(state: State<'_, AppState>) -> Result<Value, AppError> {
    memory::validate_vault(&state)
}

#[tauri::command]
pub fn memory_rebuild_indexes(
    state: State<'_, AppState>,
    request: Option<Value>,
) -> Result<Value, AppError> {
    memory::rebuild_indexes(&state, request)
}
