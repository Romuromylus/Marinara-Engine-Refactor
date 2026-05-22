use crate::shared::string_array_from_value;
use marinara_core::{AppError, AppResult};
use marinara_storage::FileStorage;
use serde_json::{json, Value};

/// Returns the list of collections to clean up in the caller's media layer
/// (filesystem on Tauri, server-storage on Axum) when scopes include "media"
/// or the destructive `clear_all` is invoked.
#[derive(Debug, Default)]
pub struct AdminOutcome {
    pub cleared_collections: Vec<String>,
    pub clear_runtime_media: bool,
}

pub fn clear_all_storage(storage: &FileStorage) -> AppResult<AdminOutcome> {
    storage.clear_all()?;
    Ok(AdminOutcome {
        cleared_collections: Vec::new(),
        clear_runtime_media: true,
    })
}

pub fn expunge(storage: &FileStorage, body: Value) -> AppResult<AdminOutcome> {
    if body.get("confirm").and_then(Value::as_bool) != Some(true) {
        return Err(AppError::invalid_input("confirm must be true"));
    }
    let scopes = string_array_from_value(body.get("scopes"));
    if scopes.is_empty() {
        return Err(AppError::invalid_input(
            "At least one expunge scope is required",
        ));
    }
    let mut cleared_collections = Vec::new();
    let mut clear_runtime_media = false;
    for scope in scopes {
        match scope.as_str() {
            "chats" => clear_collections(
                storage,
                &[
                    "chats",
                    "chat-folders",
                    "messages",
                    "gallery",
                    "agent-runs",
                    "knowledge-sources",
                ],
                &mut cleared_collections,
            )?,
            "characters" => clear_collections(
                storage,
                &[
                    "characters",
                    "character-groups",
                    "character-versions",
                    "character-gallery",
                    "sprites",
                ],
                &mut cleared_collections,
            )?,
            "personas" => clear_collections(
                storage,
                &["personas", "persona-groups"],
                &mut cleared_collections,
            )?,
            "lorebooks" => clear_collections(
                storage,
                &["lorebooks", "lorebook-entries", "lorebook-folders"],
                &mut cleared_collections,
            )?,
            "presets" => clear_collections(
                storage,
                &[
                    "prompts",
                    "prompt-groups",
                    "prompt-sections",
                    "prompt-variables",
                    "chat-presets",
                ],
                &mut cleared_collections,
            )?,
            "connections" => clear_collections(
                storage,
                &["connections", "connection-folders"],
                &mut cleared_collections,
            )?,
            "automation" => clear_collections(
                storage,
                &[
                    "agents",
                    "custom-tools",
                    "regex-scripts",
                    "themes",
                    "extensions",
                ],
                &mut cleared_collections,
            )?,
            "media" => {
                clear_collections(
                    storage,
                    &[
                        "gallery",
                        "character-gallery",
                        "background-metadata",
                        "sprites",
                        "knowledge-sources",
                    ],
                    &mut cleared_collections,
                )?;
                clear_runtime_media = true;
            }
            other => {
                return Err(AppError::invalid_input(format!(
                    "Unknown expunge scope: {other}"
                )))
            }
        }
    }
    cleared_collections.sort();
    cleared_collections.dedup();
    Ok(AdminOutcome {
        cleared_collections,
        clear_runtime_media,
    })
}

/// Returns the JSON response shape that matches the legacy `admin_expunge` Tauri
/// command response. Callers that need the raw outcome (so they can also do
/// runtime-media cleanup) should use `expunge` directly.
pub fn expunge_response(outcome: &AdminOutcome) -> Value {
    json!({
        "success": true,
        "clearedCollections": outcome.cleared_collections,
    })
}

pub fn clear_all_response() -> Value {
    json!({ "success": true, "cleared": "all" })
}

fn clear_collections(
    storage: &FileStorage,
    collections: &[&str],
    cleared: &mut Vec<String>,
) -> AppResult<()> {
    for collection in collections {
        storage.replace_all(collection, Vec::new())?;
        cleared.push((*collection).to_string());
    }
    Ok(())
}
