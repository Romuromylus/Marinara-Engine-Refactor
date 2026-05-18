use super::{imports, knowledge};
use crate::state::AppState;
use marinara_core::AppError;
use serde_json::{json, Value};
use tauri::State;

#[tauri::command]
pub fn knowledge_sources_list(state: State<'_, AppState>) -> Result<Value, AppError> {
    knowledge::knowledge_sources_call(&state, "GET", &[], Value::Null)
}

#[tauri::command]
pub fn knowledge_source_upload(state: State<'_, AppState>, body: Value) -> Result<Value, AppError> {
    knowledge::knowledge_sources_call(&state, "POST", &["upload"], body)
}

#[tauri::command]
pub fn knowledge_source_delete(state: State<'_, AppState>, id: String) -> Result<Value, AppError> {
    knowledge::knowledge_sources_call(&state, "DELETE", &[&id], Value::Null)
}

#[tauri::command]
pub fn knowledge_source_text(state: State<'_, AppState>, id: String) -> Result<Value, AppError> {
    knowledge::knowledge_sources_call(&state, "GET", &[&id, "text"], Value::Null)
}

#[tauri::command]
pub fn import_marinara(state: State<'_, AppState>, envelope: Value) -> Result<Value, AppError> {
    imports::import_call(&state, &["marinara"], envelope)
}

#[tauri::command]
pub fn import_marinara_file(state: State<'_, AppState>, body: Value) -> Result<Value, AppError> {
    imports::import_call(&state, &["marinara-file"], body)
}

#[tauri::command]
pub fn import_st_character(state: State<'_, AppState>, body: Value) -> Result<Value, AppError> {
    imports::import_call(&state, &["st-character"], body)
}

#[tauri::command]
pub fn import_st_character_batch(
    state: State<'_, AppState>,
    body: Value,
) -> Result<Value, AppError> {
    imports::import_call(&state, &["st-character", "batch"], body)
}

#[tauri::command]
pub fn import_st_character_inspect(
    state: State<'_, AppState>,
    body: Value,
) -> Result<Value, AppError> {
    imports::import_call(&state, &["st-character", "inspect"], body)
}

#[tauri::command]
pub fn import_st_chat(state: State<'_, AppState>, body: Value) -> Result<Value, AppError> {
    imports::import_call(&state, &["st-chat"], body)
}

#[tauri::command]
pub fn import_st_chat_into_group(
    state: State<'_, AppState>,
    body: Value,
) -> Result<Value, AppError> {
    imports::import_call(&state, &["st-chat-into-group"], body)
}

#[tauri::command]
pub fn import_st_preset(state: State<'_, AppState>, payload: Value) -> Result<Value, AppError> {
    imports::import_call(&state, &["st-preset"], payload)
}

#[tauri::command]
pub fn import_st_lorebook(state: State<'_, AppState>, payload: Value) -> Result<Value, AppError> {
    imports::import_call(&state, &["st-lorebook"], payload)
}

#[tauri::command]
pub fn import_list_directory(
    state: State<'_, AppState>,
    path: String,
    picker_selected: Option<bool>,
) -> Result<Value, AppError> {
    imports::import_call(
        &state,
        &["list-directory"],
        json!({ "path": path, "pickerSelected": picker_selected.unwrap_or(false) }),
    )
}

#[tauri::command]
pub fn import_st_bulk_scan(state: State<'_, AppState>, payload: Value) -> Result<Value, AppError> {
    imports::import_call(&state, &["st-bulk", "scan"], payload)
}

#[tauri::command]
pub fn import_st_bulk_run(state: State<'_, AppState>, payload: Value) -> Result<Value, AppError> {
    imports::import_call(&state, &["st-bulk", "run"], payload)
}

#[tauri::command]
pub fn import_st_bulk_run_events(
    state: State<'_, AppState>,
    payload: Value,
    on_event: tauri::ipc::Channel<Value>,
) -> Result<(), AppError> {
    imports::import_stream_channel(&state, &["st-bulk", "run"], payload, on_event)
}
