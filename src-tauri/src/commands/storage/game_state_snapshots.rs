use crate::state::AppState;
use marinara_core::{ensure_object, AppError, AppResult};
use serde_json::{json, Map, Value};

const SNAPSHOT_COLLECTION: &str = "game-state-snapshots";
const TRACKER_KIND: &str = "tracker";
const TEXT_FIELDS: [&str; 5] = ["date", "time", "location", "weather", "temperature"];

pub(crate) fn latest_tracker_snapshot(state: &AppState, chat_id: &str) -> AppResult<Option<Value>> {
    let mut rows = tracker_snapshots_for_chat(state, chat_id)?;
    sort_newest_first(&mut rows);
    Ok(rows.into_iter().next())
}

pub(crate) fn tracker_snapshot_for_target(
    state: &AppState,
    chat_id: &str,
    message_id: &str,
    swipe_index: i64,
) -> AppResult<Option<Value>> {
    let mut rows = tracker_snapshots_for_target(state, chat_id, message_id, swipe_index)?;
    sort_newest_first(&mut rows);
    Ok(rows.into_iter().next())
}

pub(crate) fn save_tracker_snapshot(
    state: &AppState,
    chat_id: &str,
    body: Value,
) -> AppResult<Value> {
    let mut snapshot = normalize_tracker_snapshot(chat_id, body)?;
    let message_id =
        required_message_id(string_value(&snapshot, "messageId").as_deref())?.to_string();
    let swipe_index = swipe_index_value(snapshot.get("swipeIndex")).unwrap_or(0);
    let mut existing = tracker_snapshots_for_target(state, chat_id, &message_id, swipe_index)?;
    sort_newest_first(&mut existing);

    for duplicate in existing.iter().skip(1) {
        if let Some(id) = non_empty_string(duplicate, "id") {
            state.storage.delete(SNAPSHOT_COLLECTION, id)?;
        }
    }

    let Some(primary) = existing.into_iter().next() else {
        return state
            .storage
            .create(SNAPSHOT_COLLECTION, Value::Object(snapshot));
    };
    let Some(id) = non_empty_string(&primary, "id") else {
        return state
            .storage
            .create(SNAPSHOT_COLLECTION, Value::Object(snapshot));
    };
    if !snapshot.contains_key("createdAt") {
        if let Some(created_at) = primary.get("createdAt").and_then(Value::as_str) {
            snapshot.insert(
                "createdAt".to_string(),
                Value::String(created_at.to_string()),
            );
        }
    }
    state
        .storage
        .upsert_with_id(SNAPSHOT_COLLECTION, id, Value::Object(snapshot))
}

fn tracker_snapshots_for_chat(state: &AppState, chat_id: &str) -> AppResult<Vec<Value>> {
    let chat_id = required_chat_id(chat_id)?;
    let mut filters = Map::new();
    filters.insert("chatId".to_string(), Value::String(chat_id.to_string()));
    Ok(state
        .storage
        .list_where(SNAPSHOT_COLLECTION, &filters)?
        .into_iter()
        .filter(is_tracker_snapshot)
        .collect())
}

fn tracker_snapshots_for_target(
    state: &AppState,
    chat_id: &str,
    message_id: &str,
    swipe_index: i64,
) -> AppResult<Vec<Value>> {
    let chat_id = required_chat_id(chat_id)?;
    let message_id = required_message_id(Some(message_id))?;
    let swipe_index = swipe_index.max(0);
    let mut filters = Map::new();
    filters.insert("chatId".to_string(), Value::String(chat_id.to_string()));
    filters.insert(
        "messageId".to_string(),
        Value::String(message_id.to_string()),
    );
    Ok(state
        .storage
        .list_where(SNAPSHOT_COLLECTION, &filters)?
        .into_iter()
        .filter(|row| is_tracker_snapshot(row))
        .filter(|row| swipe_index_value(row.get("swipeIndex")) == Some(swipe_index))
        .collect())
}

fn normalize_tracker_snapshot(chat_id: &str, body: Value) -> AppResult<Map<String, Value>> {
    let chat_id = required_chat_id(chat_id)?;
    let incoming = ensure_object(body)?;
    if let Some(body_chat_id) = incoming.get("chatId").and_then(Value::as_str) {
        let body_chat_id = body_chat_id.trim();
        if !body_chat_id.is_empty() && body_chat_id != chat_id {
            return Err(AppError::invalid_input(
                "Tracker snapshot chatId does not match the target chat",
            ));
        }
    }

    let message_id =
        required_message_id(incoming.get("messageId").and_then(Value::as_str))?.to_string();
    let swipe_index = swipe_index_value(incoming.get("swipeIndex")).unwrap_or(0);

    let mut snapshot = Map::new();
    snapshot.insert("kind".to_string(), Value::String(TRACKER_KIND.to_string()));
    snapshot.insert("chatId".to_string(), Value::String(chat_id.to_string()));
    snapshot.insert("messageId".to_string(), Value::String(message_id));
    snapshot.insert("swipeIndex".to_string(), json!(swipe_index));
    for field in TEXT_FIELDS {
        snapshot.insert(field.to_string(), coerce_text_value(incoming.get(field)));
    }
    snapshot.insert(
        "presentCharacters".to_string(),
        array_value(incoming.get("presentCharacters")),
    );
    snapshot.insert(
        "recentEvents".to_string(),
        array_value(incoming.get("recentEvents")),
    );
    snapshot.insert(
        "playerStats".to_string(),
        object_or_null(incoming.get("playerStats")),
    );
    snapshot.insert(
        "personaStats".to_string(),
        nullable_array(incoming.get("personaStats")),
    );
    snapshot.insert(
        "manualOverrides".to_string(),
        object_or_null(incoming.get("manualOverrides")),
    );
    snapshot.insert(
        "committed".to_string(),
        Value::Bool(bool_value(incoming.get("committed"))),
    );
    if let Some(created_at) = incoming.get("createdAt").and_then(Value::as_str) {
        if !created_at.trim().is_empty() {
            snapshot.insert(
                "createdAt".to_string(),
                Value::String(created_at.trim().to_string()),
            );
        }
    }
    Ok(snapshot)
}

fn is_tracker_snapshot(row: &Value) -> bool {
    row.get("kind").and_then(Value::as_str) == Some(TRACKER_KIND)
}

fn required_chat_id(chat_id: &str) -> AppResult<&str> {
    let chat_id = chat_id.trim();
    if chat_id.is_empty() {
        return Err(AppError::invalid_input("chatId is required"));
    }
    Ok(chat_id)
}

fn required_message_id(message_id: Option<&str>) -> AppResult<&str> {
    let message_id = message_id.unwrap_or("").trim();
    if message_id.is_empty() {
        return Err(AppError::invalid_input("messageId is required"));
    }
    Ok(message_id)
}

fn non_empty_string<'a>(row: &'a Value, key: &str) -> Option<&'a str> {
    row.get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn string_value(row: &Map<String, Value>, key: &str) -> Option<String> {
    row.get(key).and_then(Value::as_str).map(ToOwned::to_owned)
}

fn swipe_index_value(value: Option<&Value>) -> Option<i64> {
    match value {
        Some(Value::Number(number)) => number
            .as_i64()
            .or_else(|| number.as_u64().map(|value| value as i64))
            .map(|value| value.max(0)),
        Some(Value::String(raw)) => raw.trim().parse::<i64>().ok().map(|value| value.max(0)),
        _ => None,
    }
}

fn bool_value(value: Option<&Value>) -> bool {
    match value {
        Some(Value::Bool(value)) => *value,
        Some(Value::Number(number)) => number.as_i64().unwrap_or(0) != 0,
        Some(Value::String(raw)) => {
            matches!(raw.trim().to_ascii_lowercase().as_str(), "true" | "1")
        }
        _ => false,
    }
}

fn parse_json_string(value: &Value) -> Option<Value> {
    let raw = value.as_str()?.trim();
    if raw.is_empty() {
        return None;
    }
    serde_json::from_str::<Value>(raw).ok()
}

fn array_value(value: Option<&Value>) -> Value {
    match value {
        Some(Value::Array(items)) => Value::Array(items.clone()),
        Some(value) => parse_json_string(value)
            .and_then(|parsed| parsed.as_array().cloned())
            .map(Value::Array)
            .unwrap_or_else(|| json!([])),
        None => json!([]),
    }
}

fn nullable_array(value: Option<&Value>) -> Value {
    match value {
        Some(Value::Null) | None => Value::Null,
        Some(Value::Array(items)) => Value::Array(items.clone()),
        Some(value) => parse_json_string(value)
            .and_then(|parsed| parsed.as_array().cloned())
            .map(Value::Array)
            .unwrap_or(Value::Null),
    }
}

fn object_or_null(value: Option<&Value>) -> Value {
    match value {
        Some(Value::Null) | None => Value::Null,
        Some(Value::Object(object)) => Value::Object(object.clone()),
        Some(value) => parse_json_string(value)
            .and_then(|parsed| parsed.as_object().cloned())
            .map(Value::Object)
            .unwrap_or(Value::Null),
    }
}

fn coerce_text_value(value: Option<&Value>) -> Value {
    match coerce_text_value_inner(value, 0) {
        Some(text) if !text.trim().is_empty() => Value::String(text),
        _ => Value::Null,
    }
}

fn coerce_text_value_inner(value: Option<&Value>, depth: usize) -> Option<String> {
    if depth > 6 {
        return None;
    }
    match value {
        Some(Value::String(text)) => {
            let text = text.trim();
            if text.is_empty() {
                None
            } else {
                Some(text.to_string())
            }
        }
        Some(Value::Number(number)) => Some(number.to_string()),
        Some(Value::Array(items)) => {
            let parts: Vec<String> = items
                .iter()
                .filter_map(|item| coerce_text_value_inner(Some(item), depth + 1))
                .collect();
            if parts.is_empty() {
                None
            } else {
                Some(parts.join(", "))
            }
        }
        Some(Value::Object(object)) => {
            for key in [
                "name",
                "label",
                "title",
                "value",
                "text",
                "description",
                "summary",
                "current",
                "location",
                "weather",
                "temperature",
                "date",
                "time",
                "timeOfDay",
                "condition",
                "type",
            ] {
                if let Some(text) = coerce_text_value_inner(object.get(key), depth + 1) {
                    return Some(text);
                }
            }
            let scalar_parts: Vec<String> = object
                .iter()
                .filter_map(|(key, entry)| match entry {
                    Value::String(_) | Value::Number(_) => {
                        coerce_text_value_inner(Some(entry), depth + 1)
                            .map(|text| format!("{key}: {text}"))
                    }
                    _ => None,
                })
                .collect();
            match scalar_parts.len() {
                1 => scalar_parts.into_iter().next(),
                2..=3 => Some(scalar_parts.join(", ")),
                _ => None,
            }
        }
        _ => None,
    }
}

fn sort_newest_first(rows: &mut [Value]) {
    rows.sort_by(|left, right| {
        timestamp(right)
            .cmp(timestamp(left))
            .then_with(|| non_empty_string(right, "id").cmp(&non_empty_string(left, "id")))
    });
}

fn timestamp(row: &Value) -> &str {
    row.get("createdAt")
        .or_else(|| row.get("updatedAt"))
        .and_then(Value::as_str)
        .unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_tracker_snapshot_uses_repo_json_shape() {
        let snapshot = normalize_tracker_snapshot(
            "chat-1",
            json!({
                "chatId": "chat-1",
                "messageId": "message-1",
                "swipeIndex": "2",
                "location": { "name": "Harbor" },
                "presentCharacters": "[{\"name\":\"Mari\"}]",
                "playerStats": "{\"status\":\"ready\"}",
                "personaStats": "[{\"name\":\"Energy\",\"value\":5,\"max\":10}]",
                "manualOverrides": "{\"location\":\"Harbor\"}",
                "committed": 1
            }),
        )
        .expect("snapshot should normalize");

        assert_eq!(snapshot["kind"], "tracker");
        assert_eq!(snapshot["chatId"], "chat-1");
        assert_eq!(snapshot["messageId"], "message-1");
        assert_eq!(snapshot["swipeIndex"], json!(2));
        assert_eq!(snapshot["location"], "Harbor");
        assert!(snapshot["presentCharacters"].is_array());
        assert!(snapshot["playerStats"].is_object());
        assert!(snapshot["personaStats"].is_array());
        assert!(snapshot["manualOverrides"].is_object());
        assert_eq!(snapshot["committed"], true);
    }

    #[test]
    fn normalize_tracker_snapshot_rejects_chat_mismatch() {
        let error = normalize_tracker_snapshot(
            "chat-1",
            json!({
                "chatId": "chat-2",
                "messageId": "message-1"
            }),
        )
        .expect_err("mismatched chat should fail");

        assert_eq!(error.code, "invalid_input");
    }

    #[test]
    fn normalize_tracker_snapshot_requires_message_id() {
        let error = normalize_tracker_snapshot(
            "chat-1",
            json!({
                "chatId": "chat-1",
                "messageId": "  "
            }),
        )
        .expect_err("message id should be required");

        assert_eq!(error.code, "invalid_input");
    }
}
