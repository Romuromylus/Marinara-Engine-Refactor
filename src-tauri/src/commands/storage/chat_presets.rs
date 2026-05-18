use super::*;
use super::shared::*;

pub(crate) fn chat_presets_call(state: &AppState, method: &str, rest: &[&str], body: Value) -> AppResult<Value> {
    match (method, rest) {
        ("GET", []) => list_collection(state, "chat-presets", None),
        ("POST", []) => state.storage.create("chat-presets", body),
        ("GET", ["active", mode]) => Ok(find_by_field(state, "chat-presets", "mode", mode)?.unwrap_or(Value::Null)),
        ("POST", ["import"]) => state.storage.create("chat-presets", body),
        ("POST", [id, "duplicate"]) => duplicate_record(state, "chat-presets", id),
        ("POST", [id, "set-active"]) => state.storage.patch("chat-presets", id, json!({ "active": true })),
        ("PUT", [id, "settings"]) => state.storage.patch("chat-presets", id, json!({ "settings": body })),
        ("POST", [id, "apply", chat_id]) => {
            let preset = get_required(state, "chat-presets", id)?;
            let chat = state.storage.patch("chats", chat_id, json!({ "chatPresetId": id }))?;
            Ok(json!({ "preset": preset, "chat": chat }))
        }
        ("GET", [id, "export"]) => get_required(state, "chat-presets", id),
        ("GET", [id]) => get_required(state, "chat-presets", id),
        ("PATCH", [id]) => state.storage.patch("chat-presets", id, body),
        ("DELETE", [id]) => {
            let deleted = state.storage.delete("chat-presets", id)?;
            Ok(json!({ "deleted": deleted }))
        }
        _ => Ok(json!({ "ok": true })),
    }
}
