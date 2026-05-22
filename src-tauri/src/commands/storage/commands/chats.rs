use crate::state::AppState;
use marinara_core::AppError;
use serde_json::{json, Value};
use tauri::State;

#[tauri::command]
pub fn chat_memories_list(state: State<'_, AppState>, chat_id: String) -> Result<Value, AppError> {
    marinara_handlers::chats::chat_array_field(&state.storage, &chat_id, "memories")
}

#[tauri::command]
pub fn chat_memory_delete(
    state: State<'_, AppState>,
    chat_id: String,
    memory_id: String,
) -> Result<Value, AppError> {
    marinara_handlers::chats::delete_chat_array_item(
        &state.storage,
        &chat_id,
        "memories",
        &memory_id,
    )
}

#[tauri::command]
pub fn chat_memories_clear(state: State<'_, AppState>, chat_id: String) -> Result<Value, AppError> {
    marinara_handlers::chats::set_chat_array_field(&state.storage, &chat_id, "memories", Vec::new())
}

#[tauri::command]
pub fn chat_memories_refresh(
    state: State<'_, AppState>,
    chat_id: String,
) -> Result<Value, AppError> {
    marinara_handlers::chats::refresh_chat_memories(&state.storage, &chat_id)
}

#[tauri::command]
pub fn chat_memories_export(
    state: State<'_, AppState>,
    chat_id: String,
) -> Result<Value, AppError> {
    marinara_handlers::chats::export_chat_memories(&state.storage, &chat_id)
}

#[tauri::command]
pub fn chat_memories_import(
    state: State<'_, AppState>,
    chat_id: String,
    body: Value,
) -> Result<Value, AppError> {
    marinara_handlers::chats::import_chat_memories(&state.storage, &chat_id, body)
}

#[tauri::command]
pub fn chat_notes_list(state: State<'_, AppState>, chat_id: String) -> Result<Value, AppError> {
    marinara_handlers::chats::chat_array_field(&state.storage, &chat_id, "notes")
}

#[tauri::command]
pub fn chat_note_delete(
    state: State<'_, AppState>,
    chat_id: String,
    note_id: String,
) -> Result<Value, AppError> {
    marinara_handlers::chats::delete_chat_array_item(&state.storage, &chat_id, "notes", &note_id)
}

#[tauri::command]
pub fn chat_notes_clear(state: State<'_, AppState>, chat_id: String) -> Result<Value, AppError> {
    marinara_handlers::chats::set_chat_array_field(&state.storage, &chat_id, "notes", Vec::new())
}

#[tauri::command]
pub fn chat_group_delete(state: State<'_, AppState>, group_id: String) -> Result<Value, AppError> {
    marinara_handlers::chats::delete_chat_group(&state.storage, &group_id)
}

#[tauri::command]
pub fn chat_autonomous_unread_mark(
    state: State<'_, AppState>,
    chat_id: String,
    body: Value,
) -> Result<Value, AppError> {
    marinara_handlers::chats::mark_autonomous_unread(&state.storage, &chat_id, body)
}

#[tauri::command]
pub fn chat_autonomous_unread_clear(
    state: State<'_, AppState>,
    chat_id: String,
) -> Result<Value, AppError> {
    marinara_handlers::chats::clear_autonomous_unread(&state.storage, &chat_id)
}

#[tauri::command]
pub fn chat_messages_bulk_delete(
    state: State<'_, AppState>,
    chat_id: String,
    message_ids: Vec<String>,
) -> Result<Value, AppError> {
    marinara_handlers::chats::bulk_delete_messages(
        &state.storage,
        &chat_id,
        json!({ "messageIds": message_ids }),
    )
}

#[tauri::command]
pub fn chat_branch(
    state: State<'_, AppState>,
    chat_id: String,
    up_to_message_id: Option<String>,
) -> Result<Value, AppError> {
    marinara_handlers::chats::branch_chat(
        &state.storage,
        &chat_id,
        json!({ "upToMessageId": up_to_message_id }),
    )
}

#[tauri::command]
pub fn chat_message_swipes(
    state: State<'_, AppState>,
    chat_id: String,
    message_id: String,
) -> Result<Value, AppError> {
    marinara_handlers::chats::message_swipes(
        &state.storage,
        "GET",
        &chat_id,
        &message_id,
        Value::Null,
    )
}

#[tauri::command]
pub fn chat_message_add_swipe(
    state: State<'_, AppState>,
    chat_id: String,
    message_id: String,
    body: Value,
) -> Result<Value, AppError> {
    marinara_handlers::chats::message_swipes(&state.storage, "POST", &chat_id, &message_id, body)
}

#[tauri::command]
pub fn chat_message_set_active_swipe(
    state: State<'_, AppState>,
    chat_id: String,
    message_id: String,
    index: i64,
) -> Result<Value, AppError> {
    marinara_handlers::chats::set_active_swipe(
        &state.storage,
        &chat_id,
        &message_id,
        json!({ "index": index }),
    )
}

#[tauri::command]
pub fn chat_message_delete_swipe(
    state: State<'_, AppState>,
    chat_id: String,
    message_id: String,
    index: String,
) -> Result<Value, AppError> {
    marinara_handlers::chats::delete_swipe(&state.storage, &chat_id, &message_id, &index)
}

#[tauri::command]
pub fn chat_connect(
    state: State<'_, AppState>,
    chat_id: String,
    target_chat_id: String,
) -> Result<Value, AppError> {
    marinara_handlers::chats::connect(&state.storage, &chat_id, &target_chat_id)
}

#[tauri::command]
pub fn chat_disconnect(state: State<'_, AppState>, chat_id: String) -> Result<Value, AppError> {
    marinara_handlers::chats::disconnect(&state.storage, &chat_id)
}
