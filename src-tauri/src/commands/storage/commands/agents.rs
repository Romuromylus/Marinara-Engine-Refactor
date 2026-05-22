use super::custom_tools;
use crate::state::AppState;
use marinara_core::{AppError, AppResult};
use serde_json::{json, Value};
use std::fs;
use tauri::State;

#[tauri::command]
pub async fn custom_tool_execute(
    state: State<'_, AppState>,
    body: Value,
) -> Result<Value, AppError> {
    custom_tools::execute_custom_tool(&state, body).await
}

#[tauri::command]
pub fn custom_tool_capabilities() -> Result<Value, AppError> {
    Ok(custom_tools::custom_tool_capabilities())
}

#[tauri::command]
pub fn agent_patch_by_type(
    state: State<'_, AppState>,
    agent_type: String,
    patch: Value,
) -> Result<Value, AppError> {
    marinara_handlers::agents::patch_agent_type(&state.storage, &agent_type, patch)
}

#[tauri::command]
pub fn agent_toggle_by_type(
    state: State<'_, AppState>,
    agent_type: String,
) -> Result<Value, AppError> {
    marinara_handlers::agents::toggle_agent_type(&state.storage, &agent_type)
}

#[tauri::command]
pub fn agent_cadence_status(
    state: State<'_, AppState>,
    agent_type: String,
    chat_id: String,
) -> Result<Value, AppError> {
    marinara_handlers::agents::agent_cadence_status(&state.storage, &agent_type, &chat_id)
}

#[tauri::command]
pub fn admin_expunge_command(
    state: State<'_, AppState>,
    scopes: Vec<String>,
) -> Result<Value, AppError> {
    let outcome = marinara_handlers::admin::expunge(
        &state.storage,
        json!({ "confirm": true, "scopes": scopes }),
    )?;
    if outcome.clear_runtime_media {
        clear_runtime_media(&state)?;
    }
    Ok(marinara_handlers::admin::expunge_response(&outcome))
}

#[tauri::command]
pub fn admin_clear_all_command(state: State<'_, AppState>) -> Result<Value, AppError> {
    let outcome = marinara_handlers::admin::clear_all_storage(&state.storage)?;
    if outcome.clear_runtime_media {
        clear_runtime_media(&state)?;
    }
    Ok(marinara_handlers::admin::clear_all_response())
}

#[tauri::command]
pub fn agent_memory_get(
    state: State<'_, AppState>,
    agent_type: String,
    chat_id: String,
) -> Result<Value, AppError> {
    marinara_handlers::agents::agent_memory(
        &state.storage,
        "GET",
        &agent_type,
        &chat_id,
        Value::Null,
    )
}

#[tauri::command]
pub fn agent_memory_patch(
    state: State<'_, AppState>,
    agent_type: String,
    chat_id: String,
    patch: Value,
) -> Result<Value, AppError> {
    marinara_handlers::agents::agent_memory(
        &state.storage,
        "PATCH",
        &agent_type,
        &chat_id,
        json!({ "patch": patch }),
    )
}

#[tauri::command]
pub fn agent_memory_clear(
    state: State<'_, AppState>,
    agent_type: String,
    chat_id: String,
) -> Result<Value, AppError> {
    marinara_handlers::agents::agent_memory(
        &state.storage,
        "DELETE",
        &agent_type,
        &chat_id,
        Value::Null,
    )
}

#[tauri::command]
pub fn agent_runs_clear_for_chat(
    state: State<'_, AppState>,
    chat_id: String,
) -> Result<Value, AppError> {
    marinara_handlers::agents::clear_agent_runs_and_memory_for_chat(&state.storage, &chat_id)
}

#[tauri::command]
pub fn agent_echo_messages_clear(
    state: State<'_, AppState>,
    chat_id: String,
) -> Result<Value, AppError> {
    marinara_handlers::agents::echo_messages(&state.storage, "DELETE", &chat_id)
}

fn clear_runtime_media(state: &AppState) -> AppResult<()> {
    for path in [
        state.data_dir.join("avatars"),
        state.data_dir.join("fonts"),
        state.data_dir.join("knowledge-sources"),
        state.game_assets.root().to_path_buf(),
        state.backgrounds.root().to_path_buf(),
    ] {
        if path.exists() {
            fs::remove_dir_all(&path)?;
        }
        fs::create_dir_all(&path)?;
    }
    Ok(())
}

