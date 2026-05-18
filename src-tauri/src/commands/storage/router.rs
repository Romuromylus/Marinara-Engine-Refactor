use super::*;
use super::agents::*;
use super::backup::*;
use super::backgrounds::*;
use super::bot_browser::*;
use super::chat_presets::*;
use super::chats::*;
use super::encounter::*;
use super::game::*;
use super::game_assets::*;
use super::generation::*;
use super::http::*;
use super::images::*;
use super::imports::*;
use super::integrations::*;
use super::knowledge::*;
use super::llm::*;
use super::makers::*;
use super::prompts::*;
use super::scene::*;
use super::shared::*;
use super::sprites::*;

pub(crate) async fn stream_events(
    state: &AppState,
    path: String,
    body: Option<Value>,
) -> Result<Vec<Value>, AppError> {
    let route = ParsedPath::new(&path);
    let parts: Vec<&str> = route.parts.iter().map(String::as_str).collect();
    match parts.as_slice() {
        ["generate"] => generate_events(state, body.unwrap_or(Value::Null)).await,
        ["llm", "stream"] => llm_stream_events(state, body.unwrap_or(Value::Null)).await,
        ["prompt-reviewer", "review"] => prompt_reviewer_events(state, body.unwrap_or(Value::Null)).await,
        ["character-maker", "generate"] => maker_events("character").await,
        ["persona-maker", "generate"] => maker_events("persona").await,
        ["lorebook-maker", "generate"] => Ok(vec![
            json!({ "type": "token", "data": "{\"lorebook_name\":\"Generated Lorebook\",\"lorebook_description\":\"\",\"category\":\"world\",\"entries\":[]}" }),
            json!({ "type": "done", "data": "{\"lorebook_name\":\"Generated Lorebook\",\"lorebook_description\":\"\",\"category\":\"world\",\"entries\":[]}" }),
        ]),
        _ => Err(AppError::new(
            "stream_not_supported",
            format!("Streaming is not supported for {path}"),
        )),
    }
}

pub(crate) async fn route_request(state: &AppState, method: &str, path: &str, body: Value) -> AppResult<Value> {
    let route = ParsedPath::new(path);
    let parts: Vec<&str> = route.parts.iter().map(String::as_str).collect();
    match parts.as_slice() {
        [] => Ok(json!({ "ok": true })),
        ["health"] => Ok(json!({ "ok": true, "runtime": "tauri", "dataDir": state.data_dir.to_string_lossy() })),
        ["backup", rest @ ..] => backup_call(state, method, rest, body),
        ["updates", "check"] if method == "GET" => marinara_updates::check_updates(),
        ["updates", "apply"] if method == "POST" => Ok(json!({ "applied": false, "message": "Updates are checked by the local Tauri capability layer; apply is not configured yet." })),
        ["llm", "complete"] if method == "POST" => llm_complete(state, body).await,
        ["llm", "models"] if method == "GET" => llm_models(state, route.query.get("connectionId").map(String::as_str)),
        ["scene", "plan"] if method == "POST" => scene_plan(state, body).await,
        ["scene", "create"] if method == "POST" => scene_create(state, body),
        ["scene", "conclude"] if method == "POST" => scene_conclude(state, body).await,
        ["scene", "abandon"] if method == "POST" => scene_abandon(state, body),
        ["scene", "fork"] if method == "POST" => scene_fork(state, body),
        ["scene", "analyze"] if method == "POST" => scene_analyze(state, body).await,
        ["fonts"] if method == "GET" => Ok(json!([])),
        ["fonts", "open-folder"] if method == "POST" => Ok(json!({ "opened": true })),
        ["sidecar", rest @ ..] => sidecar_call(method, rest, body),
        ["tts", rest @ ..] => tts_call(state, method, rest, body).await,
        ["translate"] if method == "POST" => Ok(json!({
            "translatedText": body.get("text").and_then(Value::as_str).unwrap_or("")
        })),
        ["backgrounds", rest @ ..] => backgrounds_call(state, method, rest, body),
        ["avatars", "npc", chat_id] if method == "POST" => Ok(json!({
            "avatarPath": body.get("avatar").and_then(Value::as_str).unwrap_or("").to_string(),
            "chatId": chat_id
        })),
        ["generate", "abort"] if method == "POST" => Ok(json!({ "aborted": false, "reason": "No active local generation stream is running." })),
        ["gifs", "search"] if method == "GET" => gifs_search(&route).await,
        ["knowledge-sources", rest @ ..] => knowledge_sources_call(state, method, rest, body),
        ["bot-browser", rest @ ..] => bot_browser_call(method, rest, &route, body).await,
        ["import", rest @ ..] => import_call(state, rest, body),
        ["admin", "clear-all"] | ["admin", "expunge"] if method == "POST" => {
            state.storage.clear_all()?;
            Ok(json!({ "success": true }))
        }
        ["app-settings", key] => handle_singleton(state, method, "app-settings", key, body),
        ["characters", "avatar-generation", "preview"] if method == "POST" => avatar_generation_preview(state, body),
        ["characters", "avatar-generation"] if method == "POST" => avatar_generation(state, body).await,
        ["characters", "personas", "list"] if method == "GET" => list_collection(state, "personas", None),
        ["characters", "personas"] => collection_root(state, method, "personas", body),
        ["characters", "personas", id] => collection_item_or_action(state, method, "personas", id, None, body),
        ["characters", "personas", id, "duplicate"] if method == "POST" => duplicate_record(state, "personas", id),
        ["characters", "personas", id, "activate"] if method == "PUT" => {
            state.storage.patch("personas", id, json!({ "active": true }))
        }
        ["characters", "personas", id, "avatar"] if method == "POST" => {
            state.storage.patch("personas", id, json!({ "avatar": body.get("avatar").cloned().unwrap_or(Value::Null) }))
        }
        ["characters", "groups", "list"] if method == "GET" => list_collection(state, "character-groups", None),
        ["characters", "groups"] => collection_root(state, method, "character-groups", body),
        ["characters", "groups", id] => collection_item_or_action(state, method, "character-groups", id, None, body),
        ["characters", "persona-groups", "list"] if method == "GET" => list_collection(state, "persona-groups", None),
        ["characters", "persona-groups"] => collection_root(state, method, "persona-groups", body),
        ["characters", "persona-groups", id] => collection_item_or_action(state, method, "persona-groups", id, None, body),
        ["characters", id, "duplicate"] if method == "POST" => duplicate_record(state, "characters", id),
        ["characters", id, "avatar"] if method == "POST" => {
            state.storage.patch("characters", id, json!({ "avatar": body.get("avatar").cloned().unwrap_or(Value::Null) }))
        }
        ["characters", id, "versions"] if method == "GET" => list_collection(state, "character-versions", Some(("characterId", *id))),
        ["characters", id, "versions", version_id] if method == "DELETE" => {
            let deleted = state.storage.delete("character-versions", version_id)?;
            Ok(json!({ "deleted": deleted, "characterId": id }))
        }
        ["characters", id, "versions", _version_id, "restore"] if method == "POST" => get_required(state, "characters", id),
        ["characters", id, "gallery"] if method == "GET" => list_collection(state, "character-gallery", Some(("characterId", *id))),
        ["characters", id, "gallery", "upload"] if method == "POST" => {
            upload_gallery_image(state, "character-gallery", "characterId", id, body)
        }
        ["characters", id, "gallery", image_id] if method == "DELETE" => {
            let deleted = state.storage.delete("character-gallery", image_id)?;
            Ok(json!({ "deleted": deleted, "characterId": id }))
        }
        ["characters"] => collection_root(state, method, "characters", body),
        ["characters", id] => collection_item_or_action(state, method, "characters", id, None, body),
        ["chats"] => collection_root(state, method, "chats", with_chat_defaults(body)),
        ["chats", "group", group_id] if method == "GET" => list_collection(state, "chats", Some(("groupId", *group_id))),
        ["chats", "group", group_id] if method == "DELETE" => delete_chat_group(state, group_id),
        ["chats", chat_id, "messages"] => chat_messages(state, method, chat_id, body, &route.query),
        ["chats", chat_id, "gallery"] if method == "GET" => list_collection(state, "gallery", Some(("chatId", *chat_id))),
        ["chats", chat_id, "gallery", "upload"] if method == "POST" => {
            upload_gallery_image(state, "gallery", "chatId", chat_id, body)
        }
        ["chats", chat_id, "gallery", image_id] if method == "DELETE" => {
            let deleted = state.storage.delete("gallery", image_id)?;
            Ok(json!({ "deleted": deleted, "chatId": chat_id }))
        }
        ["chats", chat_id, "message-count"] if method == "GET" => {
            let messages = messages_for_chat(state, chat_id)?;
            Ok(json!({ "count": messages.len() }))
        }
        ["chats", chat_id, "messages", "bulk-delete"] if method == "POST" => bulk_delete_messages(state, chat_id, body),
        ["chats", chat_id, "messages", "bulk-hidden"] if method == "PATCH" => bulk_hide_messages(state, chat_id, body),
        ["chats", chat_id, "messages", message_id] => chat_message_item(state, method, chat_id, message_id, body),
        ["chats", chat_id, "messages", message_id, "extra"] if method == "PATCH" => patch_message_extra(state, chat_id, message_id, body),
        ["chats", chat_id, "messages", message_id, "swipes"] => message_swipes(state, method, chat_id, message_id, body),
        ["chats", chat_id, "messages", message_id, "swipes", index] if method == "DELETE" => {
            delete_swipe(state, chat_id, message_id, index)
        }
        ["chats", chat_id, "messages", message_id, "active-swipe"] if method == "PUT" => {
            set_active_swipe(state, chat_id, message_id, body)
        }
        ["chats", chat_id, "metadata"] if method == "PATCH" => patch_chat_object_field(state, chat_id, "metadata", body),
        ["chats", chat_id, "game-state"] if method == "GET" => Ok(get_required(state, "chats", chat_id)?.get("gameState").cloned().unwrap_or_else(|| json!({}))),
        ["chats", chat_id, "game-state"] if method == "PATCH" => patch_chat_object_field(state, chat_id, "gameState", body),
        ["chats", chat_id, "summaries"] if method == "PATCH" => patch_chat_object_field(state, chat_id, "metadata", body),
        ["chats", chat_id, "generate-summary"] | ["chats", chat_id, "backfill-summaries"] if method == "POST" => Ok(json!({
            "chatId": chat_id,
            "summary": "",
            "updated": false
        })),
        ["chats", chat_id, "autonomous-unread"] if method == "POST" => {
            patch_chat_object_field(state, chat_id, "metadata", json!({ "autonomousUnreadCount": body.get("count").cloned().unwrap_or_else(|| json!(1)) }))
        }
        ["chats", chat_id, "autonomous-unread"] if method == "DELETE" => {
            patch_chat_object_field(state, chat_id, "metadata", json!({ "autonomousUnreadCount": 0, "autonomousUnreadCharacterIds": [], "autonomousUnreadAt": Value::Null }))
        }
        ["chats", chat_id, "memories"] if method == "GET" => chat_array_field(state, chat_id, "memories"),
        ["chats", chat_id, "memories"] if method == "DELETE" => set_chat_array_field(state, chat_id, "memories", Vec::new()),
        ["chats", chat_id, "memories", "refresh"] if method == "POST" => Ok(json!({ "chatId": chat_id, "refreshed": false, "chunks": [] })),
        ["chats", chat_id, "memories", _memory_id] if method == "DELETE" => chat_array_field(state, chat_id, "memories"),
        ["chats", chat_id, "notes"] if method == "GET" => chat_array_field(state, chat_id, "notes"),
        ["chats", chat_id, "notes"] if method == "DELETE" => set_chat_array_field(state, chat_id, "notes", Vec::new()),
        ["chats", chat_id, "notes", _note_id] if method == "DELETE" => chat_array_field(state, chat_id, "notes"),
        ["chats", chat_id, "branch"] if method == "POST" => branch_chat(state, chat_id, body),
        ["chats", chat_id, "connect"] if method == "POST" => {
            let target = body.get("targetChatId").and_then(Value::as_str).ok_or_else(|| AppError::invalid_input("targetChatId is required"))?;
            state.storage.patch("chats", chat_id, json!({ "connectedChatId": target }))?;
            state.storage.patch("chats", target, json!({ "connectedChatId": chat_id }))?;
            Ok(json!({ "connected": true }))
        }
        ["chats", chat_id, "disconnect"] if method == "POST" => {
            state.storage.patch("chats", chat_id, json!({ "connectedChatId": Value::Null }))?;
            Ok(json!({ "disconnected": true }))
        }
        ["chats", chat_id, "peek-prompt"] if method == "POST" => peek_prompt(state, chat_id),
        ["chats", chat_id] => collection_item_or_action(state, method, "chats", chat_id, None, body),
        ["chat-folders", "reorder"] | ["chat-folders", "reorder-chats"] | ["chat-folders", "move-chat"] if method == "POST" => Ok(json!({ "ok": true })),
        ["chat-folders"] => collection_root(state, method, "chat-folders", body),
        ["chat-folders", id] => collection_item_or_action(state, method, "chat-folders", id, None, body),
        ["connection-folders", "reorder"] | ["connection-folders", "move-connection"] if method == "POST" => Ok(json!({ "ok": true })),
        ["connection-folders"] => collection_root(state, method, "connection-folders", body),
        ["connection-folders", id] => collection_item_or_action(state, method, "connection-folders", id, None, body),
        ["connections", id, "duplicate"] if method == "POST" => duplicate_record(state, "connections", id),
        ["connections", id, "default-parameters"] if method == "PUT" => {
            state.storage.patch("connections", id, json!({ "defaultParameters": body }))
        }
        ["connections", id, "models"] if method == "GET" => Ok(json!({ "models": [] })),
        ["connections", id, "test"] if method == "POST" => test_connection(state, id).await,
        ["connections", id, "test-message"] if method == "POST" => test_message(state, id).await,
        ["connections", id, "diagnose-claude-subscription"] if method == "POST" => Ok(json!({
            "success": false,
            "requestedModel": null,
            "modelsBilled": [],
            "modelUsageDetail": [],
            "message": "Claude subscription diagnostics are external-service scope in the Tauri build."
        })),
        ["connections", id, "test-image"] if method == "POST" => test_image_generation(state, id).await,
        ["connections"] => collection_root(state, method, "connections", body),
        ["connections", id] => collection_item_or_action(state, method, "connections", id, None, body),
        ["prompts", "default"] if method == "GET" => Ok(Value::Null),
        ["prompts", preset_id, "full"] if method == "GET" => preset_full(state, preset_id),
        ["prompts", preset_id, nested] if matches!(*nested, "groups" | "sections" | "variables") => {
            prompt_nested_root(state, method, preset_id, nested, body)
        }
        ["prompts", preset_id, nested, nested_id] if matches!(*nested, "groups" | "sections" | "variables") => {
            prompt_nested_item(state, method, preset_id, nested, nested_id, body)
        }
        ["prompts", preset_id, nested, "reorder"] if matches!(*nested, "groups" | "sections" | "variables") && method == "PUT" => {
            Ok(json!({ "presetId": preset_id, "ok": true }))
        }
        ["prompts", id, "duplicate"] if method == "POST" => duplicate_record(state, "prompts", id),
        ["prompts", id, "set-default"] if method == "POST" => state.storage.patch("prompts", id, json!({ "isDefault": true })),
        ["prompts"] => collection_root(state, method, "prompts", body),
        ["prompts", id] => collection_item_or_action(state, method, "prompts", id, None, body),
        ["lorebooks", lorebook_id, "entries"] if method == "GET" => list_collection(state, "lorebook-entries", Some(("lorebookId", *lorebook_id))),
        ["lorebooks", lorebook_id, "entries"] if method == "POST" => create_nested(state, "lorebook-entries", "lorebookId", lorebook_id, body),
        ["lorebooks", lorebook_id, "entries", "bulk"] if method == "POST" => create_lorebook_entries_bulk(state, lorebook_id, body),
        ["lorebooks", lorebook_id, "entries", entry_id] => nested_item(state, method, "lorebook-entries", "lorebookId", lorebook_id, entry_id, body),
        ["lorebooks", lorebook_id, "folders"] if method == "GET" => list_collection(state, "lorebook-folders", Some(("lorebookId", *lorebook_id))),
        ["lorebooks", lorebook_id, "folders"] if method == "POST" => create_nested(state, "lorebook-folders", "lorebookId", lorebook_id, body),
        ["lorebooks", lorebook_id, "folders", folder_id] => nested_item(state, method, "lorebook-folders", "lorebookId", lorebook_id, folder_id, body),
        ["lorebooks"] => collection_root(state, method, "lorebooks", body),
        ["lorebooks", id] => collection_item_or_action(state, method, "lorebooks", id, None, body),
        ["game-assets"] if method == "GET" => Ok(json!({
            "items": state.game_assets.list(None)?,
            "root": state.game_assets.root().to_string_lossy()
        })),
        ["game-assets", "manifest"] if method == "GET" => game_assets_manifest(state),
        ["game-assets", "tree"] if method == "GET" => game_assets_tree(state),
        ["game-assets", "upload"] if method == "POST" => game_assets_upload(state, body),
        ["game-assets", "folders"] if method == "POST" => {
            let path = body.get("path").and_then(Value::as_str).unwrap_or("");
            state.game_assets.create_folder(path)?;
            Ok(json!({ "path": path }))
        }
        ["game-assets", "folders", "description"] if method == "PATCH" => Ok(json!({
            "path": body.get("path").cloned().unwrap_or(Value::Null),
            "description": body.get("description").cloned().unwrap_or(Value::Null)
        })),
        ["game-assets", "folders", encoded] if method == "DELETE" => {
            let recursive = route.query.get("recursive").map(String::as_str) == Some("true");
            state.game_assets.remove(&decode_path(encoded), recursive)?;
            Ok(json!({ "deleted": true }))
        }
        ["game-assets", "file", encoded] if method == "DELETE" => {
            state.game_assets.remove(&decode_path(encoded), false)?;
            Ok(json!({ "deleted": true }))
        }
        ["game-assets", "file-path", encoded] if method == "GET" => {
            Ok(json!({ "path": state.game_assets.absolute_path_string(&decode_path(encoded))? }))
        }
        ["game-assets", "file-content", encoded] if method == "GET" => Ok(json!({ "content": state.game_assets.read_text(&decode_path(encoded))? })),
        ["game-assets", "file-content", encoded] if method == "PUT" => {
            let content = body.get("content").and_then(Value::as_str).unwrap_or("");
            state.game_assets.write_text(&decode_path(encoded), content)?;
            Ok(json!({ "saved": true }))
        }
        ["game-assets", "rename"] if method == "POST" => {
            let path = body.get("path").and_then(Value::as_str).ok_or_else(|| AppError::invalid_input("path is required"))?;
            let new_name = body.get("newName").and_then(Value::as_str).ok_or_else(|| AppError::invalid_input("newName is required"))?;
            state.game_assets.rename(path, new_name)
        }
        ["game-assets", "move"] if method == "POST" => {
            let path = body.get("path").and_then(Value::as_str).ok_or_else(|| AppError::invalid_input("path is required"))?;
            let target = body.get("targetFolder").and_then(Value::as_str).unwrap_or("");
            state.game_assets.move_to_folder(path, target)
        }
        ["game-assets", "copy"] if method == "POST" => {
            let path = body.get("path").and_then(Value::as_str).ok_or_else(|| AppError::invalid_input("path is required"))?;
            let target = body.get("targetFolder").and_then(Value::as_str).unwrap_or("");
            state.game_assets.copy_to_folder(path, target)
        }
        ["game-assets", "move-bulk"] if method == "POST" => {
            let paths = string_array_from_value(body.get("paths"));
            let target = body.get("targetFolder").and_then(Value::as_str).unwrap_or("");
            Ok(state.game_assets.move_many(&paths, target))
        }
        ["game-assets", "copy-bulk"] if method == "POST" => {
            let paths = string_array_from_value(body.get("paths"));
            let target = body.get("targetFolder").and_then(Value::as_str).unwrap_or("");
            Ok(state.game_assets.copy_many(&paths, target))
        }
        ["game-assets", "delete-bulk"] if method == "POST" => {
            let paths = string_array_from_value(body.get("paths"));
            Ok(state.game_assets.delete_many(&paths))
        }
        ["game-assets", "file-info", encoded] if method == "GET" => state.game_assets.file_info(&decode_path(encoded)),
        ["game-assets", "rescan"] | ["game-assets", "open-folder"] if method == "POST" => Ok(json!({ "ok": true })),
        ["sprites", "capabilities"] if method == "GET" => Ok(json!({
            "imageProcessingAvailable": false,
            "spriteGenerationAvailable": false,
            "backgroundRemovalAvailable": false,
            "reason": "Sprite image processing is handled by the assets capability and is not configured yet."
        })),
        ["sprites", "generate-sheet", "preview"] if method == "POST" => Ok(json!({ "cells": [], "items": [] })),
        ["sprites", "generate-sheet"] if method == "POST" => Ok(json!({ "sprites": [], "cells": [] })),
        ["sprites", "cleanup"] if method == "POST" => Ok(json!({ "cells": [] })),
        ["sprites", character_id, "cleanup-saved"] if method == "POST" => Ok(json!({ "characterId": character_id, "backups": [], "updated": [] })),
        ["sprites", character_id, "cleanup-restore"] if method == "POST" => Ok(json!({ "characterId": character_id, "restored": true })),
        ["sprites", character_id] if method == "GET" => list_collection(state, "sprites", Some(("characterId", *character_id))),
        ["sprites", character_id] if method == "POST" => create_nested(state, "sprites", "characterId", character_id, body),
        ["sprites", character_id, expression] if method == "DELETE" => delete_sprite(state, character_id, expression),
        ["agents", "toggle", agent_type] if method == "PUT" => toggle_agent_type(state, agent_type),
        ["agents", "type", agent_type] if method == "PATCH" => patch_agent_type(state, agent_type, body),
        ["agents", "cadence", "director", chat_id] if method == "GET" => Ok(json!({ "chatId": chat_id, "injections": [], "updatedAt": Value::Null })),
        ["agents", "runs", chat_id, "custom"] if method == "GET" => list_collection(state, "agent-runs", Some(("chatId", *chat_id))),
        ["agents", "runs", id] if method == "PATCH" => state.storage.patch("agent-runs", id, body),
        ["agents", "runs", id] if method == "DELETE" => {
            let deleted = state.storage.delete("agent-runs", id)?;
            Ok(json!({ "deleted": deleted }))
        }
        ["agents", "memory", _agent_id, _chat_id] if method == "DELETE" => Ok(json!({ "deleted": true })),
        ["agents", "echo-messages", _chat_id] if method == "DELETE" => Ok(json!({ "deleted": true })),
        ["agents", "retry"] if method == "POST" => Ok(json!({ "results": [] })),
        ["agents"] => collection_root(state, method, "agents", body),
        ["agents", id] => collection_item_or_action(state, method, "agents", id, None, body),
        ["conversation", "status", chat_id] if method == "GET" => Ok(json!({ "chatId": chat_id, "active": false })),
        ["conversation", "autonomous", "check"] if method == "POST" => Ok(json!({ "shouldGenerate": false, "reason": "autonomous_disabled" })),
        ["conversation", "busy-delay"] if method == "POST" => Ok(json!({ "delayMs": 0, "reason": "local_tauri" })),
        ["conversation", "activity", _kind] if method == "POST" => Ok(json!({ "ok": true })),
        ["conversation", "schedule", "generate"] if method == "POST" => Ok(json!({ "schedule": null })),
        ["custom-tools", "capabilities"] if method == "GET" => Ok(json!({ "available": false, "engines": [] })),
        ["regex-scripts", "reorder"] if method == "PUT" => Ok(body.get("scriptIds").cloned().unwrap_or_else(|| json!([]))),
        ["themes", "active"] if method == "PUT" => handle_singleton(state, "PUT", "app-settings", "active-theme", body),
        ["themes", "active"] if method == "GET" => handle_singleton(state, "GET", "app-settings", "active-theme", Value::Null),
        ["chat-presets", rest @ ..] => chat_presets_call(state, method, rest, body),
        ["encounter", rest @ ..] => encounter_call(rest, body),
        ["game", rest @ ..] => game_call(state, method, rest, body),
        ["lorebook-maker", "generate"] if method == "POST" => Ok(json!({ "entries": [], "raw": "" })),
        ["haptic", rest @ ..] => haptic_call(rest, body),
        ["spotify", rest @ ..] => spotify_call(state, method, rest, &route, body).await,
        [collection] => collection_root(state, method, collection, body),
        [collection, id] => collection_item_or_action(state, method, collection, id, None, body),
        _ => Err(AppError::new("route_not_found", format!("{method} {path} is not implemented"))),
    }
}
