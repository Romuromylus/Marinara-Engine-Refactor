use super::*;
use super::shared::*;

pub(crate) fn delete_sprite(state: &AppState, character_id: &str, expression: &str) -> AppResult<Value> {
    let sprites = match list_collection(state, "sprites", Some(("characterId", character_id)))? {
        Value::Array(rows) => rows,
        _ => Vec::new(),
    };
    for sprite in sprites {
        if sprite.get("expression").and_then(Value::as_str) == Some(expression) {
            if let Some(id) = sprite.get("id").and_then(Value::as_str) {
                state.storage.delete("sprites", id)?;
            }
        }
    }
    Ok(json!({ "deleted": true }))
}

