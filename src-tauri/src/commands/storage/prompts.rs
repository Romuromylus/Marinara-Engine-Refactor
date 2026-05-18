use super::*;
use super::shared::*;

pub(crate) fn preset_full(state: &AppState, preset_id: &str) -> AppResult<Value> {
    Ok(json!({
        "preset": get_required(state, "prompts", preset_id)?,
        "sections": list_collection(state, "prompt-sections", Some(("presetId", preset_id)))?,
        "groups": list_collection(state, "prompt-groups", Some(("presetId", preset_id)))?,
        "choiceBlocks": list_collection(state, "prompt-variables", Some(("presetId", preset_id)))?,
    }))
}

pub(crate) fn prompt_nested_collection(nested: &str) -> &'static str {
    match nested {
        "groups" => "prompt-groups",
        "sections" => "prompt-sections",
        "variables" => "prompt-variables",
        _ => "prompt-items",
    }
}

pub(crate) fn prompt_nested_root(state: &AppState, method: &str, preset_id: &str, nested: &str, body: Value) -> AppResult<Value> {
    let collection = prompt_nested_collection(nested);
    match method {
        "GET" => list_collection(state, collection, Some(("presetId", preset_id))),
        "POST" => create_nested(state, collection, "presetId", preset_id, body),
        _ => Err(AppError::new("method_not_allowed", "Unsupported prompt nested method")),
    }
}

pub(crate) fn prompt_nested_item(
    state: &AppState,
    method: &str,
    preset_id: &str,
    nested: &str,
    nested_id: &str,
    body: Value,
) -> AppResult<Value> {
    nested_item(state, method, prompt_nested_collection(nested), "presetId", preset_id, nested_id, body)
}

pub(crate) fn create_nested(state: &AppState, collection: &str, parent_field: &str, parent_id: &str, body: Value) -> AppResult<Value> {
    let mut object = ensure_object(body)?;
    object.insert(parent_field.to_string(), Value::String(parent_id.to_string()));
    state.storage.create(collection, Value::Object(object))
}

pub(crate) fn nested_item(
    state: &AppState,
    method: &str,
    collection: &str,
    _parent_field: &str,
    _parent_id: &str,
    id: &str,
    body: Value,
) -> AppResult<Value> {
    collection_item_or_action(state, method, collection, id, None, body)
}

pub(crate) fn create_lorebook_entries_bulk(state: &AppState, lorebook_id: &str, body: Value) -> AppResult<Value> {
    let entries = body.get("entries").and_then(Value::as_array).cloned().unwrap_or_default();
    let mut created = Vec::new();
    for entry in entries {
        created.push(create_nested(state, "lorebook-entries", "lorebookId", lorebook_id, entry)?);
    }
    Ok(Value::Array(created))
}

