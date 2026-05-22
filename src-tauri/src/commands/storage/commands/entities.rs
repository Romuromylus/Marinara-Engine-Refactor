use super::{avatars, lorebook_images, shared};
use crate::builtins::is_protected_record;
use crate::state::AppState;
use marinara_core::AppError;
use serde_json::{json, Value};
use tauri::State;

#[tauri::command]
pub fn storage_list(
    state: State<'_, AppState>,
    entity: String,
    options: Option<Value>,
) -> Result<Value, AppError> {
    marinara_handlers::storage::list(&state.storage, &entity, options)
}

#[tauri::command]
pub fn storage_get(
    state: State<'_, AppState>,
    entity: String,
    id: String,
) -> Result<Value, AppError> {
    let mut value = state.storage.get(&entity, &id)?.unwrap_or(Value::Null);
    if entity == "messages" {
        shared::materialize_message_swipe_fields(&mut value);
    }
    Ok(value)
}

#[tauri::command]
pub fn storage_create(
    state: State<'_, AppState>,
    entity: String,
    value: Value,
) -> Result<Value, AppError> {
    state
        .storage
        .create(&entity, shared::with_entity_defaults(&entity, value))
}

#[tauri::command]
pub fn storage_update(
    state: State<'_, AppState>,
    entity: String,
    id: String,
    patch: Value,
) -> Result<Value, AppError> {
    state.storage.patch(
        &entity,
        &id,
        shared::normalize_update_patch(&entity, patch)?,
    )
}

#[tauri::command]
pub fn storage_delete(
    state: State<'_, AppState>,
    entity: String,
    id: String,
) -> Result<Value, AppError> {
    if is_protected_record(&entity, &id) {
        return Err(AppError::invalid_input(
            "Built-in Professor Mari cannot be deleted",
        ));
    }
    let existing = media_owned_record(&state, &entity, &id)?;
    let deleted = state.storage.delete(&entity, &id)?;
    if deleted {
        if let Some(record) = existing.as_ref() {
            remove_owned_media(&state, &entity, record);
        }
    }
    Ok(json!({ "deleted": deleted }))
}

fn media_owned_record(state: &AppState, entity: &str, id: &str) -> Result<Option<Value>, AppError> {
    match entity {
        "characters" | "personas" | "lorebooks" => state.storage.get(entity, id),
        _ => Ok(None),
    }
}

fn remove_owned_media(state: &AppState, entity: &str, record: &Value) {
    match entity {
        "characters" | "personas" => avatars::remove_avatar_file(state, entity, record),
        "lorebooks" => lorebook_images::remove_lorebook_image_file(state, record),
        _ => {}
    }
}

#[tauri::command]
pub fn storage_duplicate(
    state: State<'_, AppState>,
    entity: String,
    id: String,
) -> Result<Value, AppError> {
    shared::duplicate_record(&state, &entity, &id)
}

