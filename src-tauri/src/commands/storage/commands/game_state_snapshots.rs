use crate::state::AppState;
use marinara_core::AppError;
use serde_json::Value;
use tauri::State;

#[tauri::command]
pub fn tracker_snapshot_latest(
    state: State<'_, AppState>,
    chat_id: String,
) -> Result<Value, AppError> {
    Ok(marinara_handlers::tracker::latest_snapshot(&state.storage, &chat_id)?
        .unwrap_or(Value::Null))
}

#[tauri::command]
pub fn tracker_snapshot_get(
    state: State<'_, AppState>,
    chat_id: String,
    message_id: String,
    swipe_index: i64,
) -> Result<Value, AppError> {
    Ok(marinara_handlers::tracker::snapshot_for_target(
        &state.storage,
        &chat_id,
        &message_id,
        swipe_index,
    )?
    .unwrap_or(Value::Null))
}

#[tauri::command]
pub fn tracker_snapshot_save(
    state: State<'_, AppState>,
    chat_id: String,
    snapshot: Value,
) -> Result<Value, AppError> {
    marinara_handlers::tracker::save_snapshot(&state.storage, &chat_id, snapshot)
}
