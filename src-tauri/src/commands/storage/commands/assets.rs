use super::{backgrounds, fonts, http, lorebook_images, shared};
use crate::state::AppState;
use marinara_core::AppError;
use serde_json::{json, Value};
use std::collections::HashMap;
use tauri::State;

#[tauri::command]
pub fn backgrounds_list(state: State<'_, AppState>) -> Result<Value, AppError> {
    backgrounds::backgrounds_call(&state, "GET", &[], Value::Null)
}

#[tauri::command]
pub fn backgrounds_tags(state: State<'_, AppState>) -> Result<Value, AppError> {
    backgrounds::backgrounds_call(&state, "GET", &["tags"], Value::Null)
}

#[tauri::command]
pub fn background_upload(state: State<'_, AppState>, body: Value) -> Result<Value, AppError> {
    backgrounds::backgrounds_call(&state, "POST", &["upload"], body)
}

#[tauri::command]
pub fn background_delete(state: State<'_, AppState>, filename: String) -> Result<Value, AppError> {
    backgrounds::backgrounds_call(&state, "DELETE", &[filename.as_str()], Value::Null)
}

#[tauri::command]
pub fn background_tags_update(
    state: State<'_, AppState>,
    filename: String,
    tags: Vec<String>,
) -> Result<Value, AppError> {
    backgrounds::backgrounds_call(
        &state,
        "PATCH",
        &[filename.as_str(), "tags"],
        json!({ "tags": tags }),
    )
}

#[tauri::command]
pub fn background_rename(
    state: State<'_, AppState>,
    filename: String,
    name: String,
) -> Result<Value, AppError> {
    backgrounds::backgrounds_call(
        &state,
        "PATCH",
        &[filename.as_str(), "rename"],
        json!({ "name": name }),
    )
}

#[tauri::command]
pub async fn fonts_list(state: State<'_, AppState>) -> Result<Value, AppError> {
    fonts::fonts_call(&state, "GET", &[], Value::Null).await
}

#[tauri::command]
pub async fn fonts_google_download(
    state: State<'_, AppState>,
    family: String,
) -> Result<Value, AppError> {
    fonts::fonts_call(
        &state,
        "POST",
        &["google", "download"],
        json!({ "family": family }),
    )
    .await
}

#[tauri::command]
pub async fn fonts_open_folder(state: State<'_, AppState>) -> Result<Value, AppError> {
    fonts::fonts_call(&state, "POST", &["open-folder"], Value::Null).await
}

#[tauri::command]
pub fn game_assets_list(
    state: State<'_, AppState>,
    path: Option<String>,
) -> Result<Value, AppError> {
    marinara_handlers::game_assets::list(&state.game_assets, path.as_deref())
}

#[tauri::command]
pub fn game_assets_manifest(state: State<'_, AppState>) -> Result<Value, AppError> {
    marinara_handlers::game_assets::manifest(&state.game_assets)
}

#[tauri::command]
pub fn game_assets_tree(state: State<'_, AppState>) -> Result<Value, AppError> {
    marinara_handlers::game_assets::tree(&state.game_assets)
}

#[tauri::command]
pub fn game_assets_rescan(state: State<'_, AppState>) -> Result<Value, AppError> {
    marinara_handlers::game_assets::rescan(&state.game_assets)
}

#[tauri::command]
pub fn game_assets_create_folder(
    state: State<'_, AppState>,
    path: String,
) -> Result<Value, AppError> {
    marinara_handlers::game_assets::create_folder(&state.game_assets, &path)
}

#[tauri::command]
pub fn game_assets_delete_folder(
    state: State<'_, AppState>,
    path: String,
    recursive: Option<bool>,
) -> Result<Value, AppError> {
    marinara_handlers::game_assets::delete_folder(
        &state.game_assets,
        &path,
        recursive.unwrap_or(false),
    )
}

#[tauri::command]
pub fn game_assets_delete_file(
    state: State<'_, AppState>,
    path: String,
) -> Result<Value, AppError> {
    marinara_handlers::game_assets::delete_file(&state.game_assets, &path)
}

#[tauri::command]
pub fn game_assets_file_path(state: State<'_, AppState>, path: String) -> Result<Value, AppError> {
    marinara_handlers::game_assets::file_path(&state.game_assets, &path)
}

#[tauri::command]
pub fn game_assets_read_text(state: State<'_, AppState>, path: String) -> Result<Value, AppError> {
    marinara_handlers::game_assets::read_text(&state.game_assets, &path)
}

#[tauri::command]
pub fn game_assets_write_text(
    state: State<'_, AppState>,
    path: String,
    content: String,
) -> Result<Value, AppError> {
    marinara_handlers::game_assets::write_text(&state.game_assets, &path, &content)
}

#[tauri::command]
pub fn game_assets_rename(
    state: State<'_, AppState>,
    path: String,
    new_name: String,
) -> Result<Value, AppError> {
    marinara_handlers::game_assets::rename(&state.game_assets, &path, &new_name)
}

#[tauri::command]
pub fn game_assets_move(
    state: State<'_, AppState>,
    path: String,
    target_folder: Option<String>,
) -> Result<Value, AppError> {
    marinara_handlers::game_assets::move_one(
        &state.game_assets,
        &path,
        target_folder.as_deref().unwrap_or(""),
    )
}

#[tauri::command]
pub fn game_assets_copy(
    state: State<'_, AppState>,
    path: String,
    target_folder: Option<String>,
) -> Result<Value, AppError> {
    marinara_handlers::game_assets::copy_one(
        &state.game_assets,
        &path,
        target_folder.as_deref().unwrap_or(""),
    )
}

#[tauri::command]
pub fn game_assets_move_bulk(
    state: State<'_, AppState>,
    paths: Vec<String>,
    target_folder: Option<String>,
) -> Result<Value, AppError> {
    marinara_handlers::game_assets::move_bulk(
        &state.game_assets,
        &paths,
        target_folder.as_deref().unwrap_or(""),
    )
}

#[tauri::command]
pub fn game_assets_copy_bulk(
    state: State<'_, AppState>,
    paths: Vec<String>,
    target_folder: Option<String>,
) -> Result<Value, AppError> {
    marinara_handlers::game_assets::copy_bulk(
        &state.game_assets,
        &paths,
        target_folder.as_deref().unwrap_or(""),
    )
}

#[tauri::command]
pub fn game_assets_delete_bulk(
    state: State<'_, AppState>,
    paths: Vec<String>,
) -> Result<Value, AppError> {
    marinara_handlers::game_assets::delete_bulk(&state.game_assets, &paths)
}

#[tauri::command]
pub fn game_assets_file_info(state: State<'_, AppState>, path: String) -> Result<Value, AppError> {
    marinara_handlers::game_assets::file_info(&state.game_assets, &path)
}

#[tauri::command]
pub fn game_assets_folder_description(
    state: State<'_, AppState>,
    path: String,
    description: String,
) -> Result<Value, AppError> {
    marinara_handlers::game_assets::folder_description(&state.game_assets, &path, &description)
}

#[tauri::command]
pub fn game_assets_upload(state: State<'_, AppState>, body: Value) -> Result<Value, AppError> {
    marinara_handlers::game_assets::upload(&state.game_assets, body)
}

/// Tauri-only post-processing: the shared handler resolves the path and
/// creates the folder; this shim layers `tauri_plugin_opener::open_path` on
/// top so the desktop binary actually opens the system file manager.
#[tauri::command]
pub fn game_assets_open_folder(
    state: State<'_, AppState>,
    subfolder: Option<String>,
) -> Result<Value, AppError> {
    let mut response =
        marinara_handlers::game_assets::open_folder(&state.game_assets, subfolder.as_deref())?;
    if let Some(path) = response.get("path").and_then(Value::as_str) {
        tauri_plugin_opener::open_path(path, None::<&str>)
            .map_err(|error| AppError::new("open_folder_failed", error.to_string()))?;
    }
    if let Some(map) = response.as_object_mut() {
        map.insert("opened".into(), json!(true));
    }
    Ok(response)
}

#[tauri::command]
pub fn background_file_path(
    state: State<'_, AppState>,
    filename: String,
) -> Result<Value, AppError> {
    Ok(json!({ "path": state.backgrounds.absolute_path_string(&filename)? }))
}

#[tauri::command]
pub fn lorebook_image_file_path(
    state: State<'_, AppState>,
    filename: String,
) -> Result<Value, AppError> {
    lorebook_images::lorebook_image_file_path(&state, &filename)
}

#[tauri::command]
pub async fn gif_search(
    q: Option<String>,
    limit: Option<u32>,
    pos: Option<String>,
) -> Result<Value, AppError> {
    let mut query = HashMap::new();
    if let Some(q) = q {
        query.insert("q".to_string(), q);
    }
    if let Some(limit) = limit {
        query.insert("limit".to_string(), limit.to_string());
    }
    if let Some(pos) = pos {
        query.insert("pos".to_string(), pos);
    }
    http::gifs_search(&shared::ParsedPath {
        parts: Vec::new(),
        query,
    })
    .await
}
