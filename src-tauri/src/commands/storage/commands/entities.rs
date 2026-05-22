use crate::state::AppState;
use marinara_core::AppError;
use serde_json::Value;
use tauri::State;

#[tauri::command]
pub fn storage_list(
    state: State<'_, AppState>,
    entity: String,
    options: Option<Value>,
) -> Result<Value, AppError> {
    marinara_handlers::entities::list(&state.storage, &entity, options)
}

#[tauri::command]
pub fn storage_get(
    state: State<'_, AppState>,
    entity: String,
    id: String,
) -> Result<Value, AppError> {
    marinara_handlers::entities::get(&state.storage, &entity, &id)
}

#[tauri::command]
pub fn storage_create(
    state: State<'_, AppState>,
    entity: String,
    value: Value,
) -> Result<Value, AppError> {
    marinara_handlers::entities::create(&state.storage, &entity, value)
}

#[tauri::command]
pub fn storage_update(
    state: State<'_, AppState>,
    entity: String,
    id: String,
    patch: Value,
) -> Result<Value, AppError> {
    marinara_handlers::entities::update(&state.storage, &entity, &id, patch)
}

#[tauri::command]
pub fn storage_delete(
    state: State<'_, AppState>,
    entity: String,
    id: String,
) -> Result<Value, AppError> {
    use super::{avatars, lorebook_images};
    let existing = match entity.as_str() {
        "characters" | "personas" | "lorebooks" => state.storage.get(&entity, &id)?,
        _ => None,
    };
    let result = marinara_handlers::entities::delete(&state.storage, &entity, &id)?;
    let deleted = result
        .get("deleted")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    if deleted {
        if let Some(record) = existing.as_ref() {
            match entity.as_str() {
                "characters" | "personas" => avatars::remove_avatar_file(&state, &entity, record),
                "lorebooks" => lorebook_images::remove_lorebook_image_file(&state, record),
                _ => {}
            }
        }
    }
    Ok(result)
}

#[tauri::command]
pub fn storage_duplicate(
    state: State<'_, AppState>,
    entity: String,
    id: String,
) -> Result<Value, AppError> {
    marinara_handlers::entities::duplicate(&state.storage, &entity, &id)
}
