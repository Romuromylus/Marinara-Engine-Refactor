use super::*;
use super::shared::*;

pub(crate) fn messages_for_chat(state: &AppState, chat_id: &str) -> AppResult<Vec<Value>> {
    let mut filters = Map::new();
    filters.insert("chatId".to_string(), Value::String(chat_id.to_string()));
    let mut rows = state.storage.list_where("messages", &filters)?;
    rows.sort_by(|a, b| {
        let a_time = a.get("createdAt").and_then(Value::as_str).unwrap_or("");
        let b_time = b.get("createdAt").and_then(Value::as_str).unwrap_or("");
        a_time.cmp(b_time)
    });
    Ok(rows)
}

pub(crate) fn chat_messages(
    state: &AppState,
    method: &str,
    chat_id: &str,
    body: Value,
    query: &HashMap<String, String>,
) -> AppResult<Value> {
    match method {
        "GET" => {
            let mut rows = messages_for_chat(state, chat_id)?;
            if let Some(limit) = query.get("limit").and_then(|value| value.parse::<usize>().ok()).filter(|limit| *limit > 0) {
                if rows.len() > limit {
                    rows = rows.split_off(rows.len() - limit);
                }
            }
            Ok(Value::Array(rows))
        }
        "POST" => {
            let mut object = ensure_object(body)?;
            object.insert("chatId".to_string(), Value::String(chat_id.to_string()));
            object.entry("role".to_string()).or_insert_with(|| Value::String("user".to_string()));
            object.entry("content".to_string()).or_insert_with(|| Value::String(String::new()));
            object.entry("extra".to_string()).or_insert_with(|| json!({}));
            object.entry("activeSwipeIndex".to_string()).or_insert_with(|| json!(0));
            let content = object.get("content").cloned().unwrap_or_else(|| Value::String(String::new()));
            object.entry("swipes".to_string()).or_insert_with(|| json!([{ "content": content }]));
            let record = state.storage.create("messages", Value::Object(object))?;
            touch_chat(state, chat_id)?;
            Ok(record)
        }
        _ => Err(AppError::new("method_not_allowed", "Unsupported messages method")),
    }
}

pub(crate) fn chat_message_item(state: &AppState, method: &str, chat_id: &str, message_id: &str, body: Value) -> AppResult<Value> {
    match method {
        "GET" => get_required(state, "messages", message_id),
        "PATCH" => {
            let updated = state.storage.patch("messages", message_id, body)?;
            touch_chat(state, chat_id)?;
            Ok(updated)
        }
        "DELETE" => {
            let deleted = state.storage.delete("messages", message_id)?;
            touch_chat(state, chat_id)?;
            Ok(json!({ "deleted": deleted }))
        }
        _ => Err(AppError::new("method_not_allowed", "Unsupported message method")),
    }
}

pub(crate) fn patch_message_extra(state: &AppState, chat_id: &str, message_id: &str, body: Value) -> AppResult<Value> {
    let mut message = get_required(state, "messages", message_id)?;
    let patch = ensure_object(body)?;
    {
        let object = message
            .as_object_mut()
            .ok_or_else(|| AppError::invalid_input("Message is not an object"))?;
        let extra = object
            .entry("extra".to_string())
            .or_insert_with(|| json!({}))
            .as_object_mut()
            .ok_or_else(|| AppError::invalid_input("Message extra is not an object"))?;
        for (key, value) in patch {
            extra.insert(key, value);
        }
    }
    let updated = state.storage.patch("messages", message_id, message)?;
    touch_chat(state, chat_id)?;
    Ok(updated)
}

pub(crate) fn message_swipes(state: &AppState, _method: &str, _chat_id: &str, message_id: &str, body: Value) -> AppResult<Value> {
    let mut message = get_required(state, "messages", message_id)?;
    if body.is_null() {
        return Ok(message.get("swipes").cloned().unwrap_or_else(|| json!([])));
    }
    let content = body.get("content").cloned().unwrap_or_else(|| Value::String(String::new()));
    let object = message
        .as_object_mut()
        .ok_or_else(|| AppError::invalid_input("Message is not an object"))?;
    let swipes = object
        .entry("swipes".to_string())
        .or_insert_with(|| json!([]))
        .as_array_mut()
        .ok_or_else(|| AppError::invalid_input("Message swipes is not an array"))?;
    swipes.push(json!({ "content": content, "createdAt": now_iso() }));
    let active_index = swipes.len().saturating_sub(1);
    object.insert("activeSwipeIndex".to_string(), json!(active_index));
    let updated = state.storage.patch("messages", message_id, message)?;
    Ok(updated)
}

pub(crate) fn set_active_swipe(state: &AppState, _chat_id: &str, message_id: &str, body: Value) -> AppResult<Value> {
    let index = body.get("index").and_then(Value::as_i64).unwrap_or(0);
    state.storage.patch("messages", message_id, json!({ "activeSwipeIndex": index }))
}

pub(crate) fn delete_swipe(state: &AppState, _chat_id: &str, message_id: &str, index: &str) -> AppResult<Value> {
    let index = index.parse::<usize>().map_err(|_| AppError::invalid_input("Invalid swipe index"))?;
    let mut message = get_required(state, "messages", message_id)?;
    let object = message
        .as_object_mut()
        .ok_or_else(|| AppError::invalid_input("Message is not an object"))?;
    if let Some(swipes) = object.get_mut("swipes").and_then(Value::as_array_mut) {
        if index < swipes.len() {
            swipes.remove(index);
        }
    }
    object.insert("activeSwipeIndex".to_string(), json!(0));
    state.storage.patch("messages", message_id, message)
}

pub(crate) fn bulk_delete_messages(state: &AppState, chat_id: &str, body: Value) -> AppResult<Value> {
    let ids = body
        .get("messageIds")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut deleted = 0;
    for id in ids.iter().filter_map(Value::as_str) {
        if state.storage.delete("messages", id)? {
            deleted += 1;
        }
    }
    touch_chat(state, chat_id)?;
    Ok(json!({ "deleted": deleted }))
}

pub(crate) fn bulk_hide_messages(state: &AppState, chat_id: &str, body: Value) -> AppResult<Value> {
    let ids = body
        .get("messageIds")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let hidden = body.get("hidden").and_then(Value::as_bool).unwrap_or(true);
    for id in ids.iter().filter_map(Value::as_str) {
        patch_message_extra(state, chat_id, id, json!({ "hiddenFromAi": hidden }))?;
    }
    Ok(json!({ "updated": true }))
}

pub(crate) fn patch_chat_object_field(state: &AppState, chat_id: &str, field: &str, body: Value) -> AppResult<Value> {
    let mut chat = get_required(state, "chats", chat_id)?;
    let patch = ensure_object(body)?;
    {
        let object = chat
            .as_object_mut()
            .ok_or_else(|| AppError::invalid_input("Chat is not an object"))?;
        let target = object
            .entry(field.to_string())
            .or_insert_with(|| json!({}))
            .as_object_mut()
            .ok_or_else(|| AppError::invalid_input(format!("Chat {field} is not an object")))?;
        for (key, value) in patch {
            target.insert(key, value);
        }
    }
    state.storage.patch("chats", chat_id, chat)
}

pub(crate) fn chat_array_field(state: &AppState, chat_id: &str, field: &str) -> AppResult<Value> {
    let chat = get_required(state, "chats", chat_id)?;
    Ok(chat.get(field).cloned().unwrap_or_else(|| json!([])))
}

pub(crate) fn set_chat_array_field(state: &AppState, chat_id: &str, field: &str, values: Vec<Value>) -> AppResult<Value> {
    state.storage.patch("chats", chat_id, json!({ field: values }))
}

pub(crate) fn touch_chat(state: &AppState, chat_id: &str) -> AppResult<()> {
    if state.storage.get("chats", chat_id)?.is_some() {
        state.storage.patch("chats", chat_id, json!({ "lastMessageAt": now_iso() }))?;
    }
    Ok(())
}

pub(crate) fn delete_chat_group(state: &AppState, group_id: &str) -> AppResult<Value> {
    let chats = match list_collection(state, "chats", Some(("groupId", group_id)))? {
        Value::Array(rows) => rows,
        _ => Vec::new(),
    };
    let mut deleted = 0;
    for chat in chats {
        if let Some(id) = chat.get("id").and_then(Value::as_str) {
            if state.storage.delete("chats", id)? {
                deleted += 1;
            }
        }
    }
    Ok(json!({ "deleted": deleted }))
}

pub(crate) fn branch_chat(state: &AppState, chat_id: &str, body: Value) -> AppResult<Value> {
    let mut chat = get_required(state, "chats", chat_id)?;
    let new_chat_id = new_id();
    let object = chat
        .as_object_mut()
        .ok_or_else(|| AppError::invalid_input("Chat is not an object"))?;
    let base_name = object
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or("Chat")
        .to_string();
    let group_id = object
        .get("groupId")
        .and_then(Value::as_str)
        .unwrap_or(chat_id)
        .to_string();
    object.insert("id".to_string(), Value::String(new_chat_id.clone()));
    object.insert("name".to_string(), Value::String(format!("{base_name} Branch")));
    object.insert("groupId".to_string(), Value::String(group_id));
    let new_chat = state.storage.create("chats", chat)?;
    let up_to = body.get("upToMessageId").and_then(Value::as_str);
    for mut message in messages_for_chat(state, chat_id)? {
        let stop = up_to.is_some_and(|id| message.get("id").and_then(Value::as_str) == Some(id));
        if let Some(obj) = message.as_object_mut() {
            obj.remove("id");
            obj.insert("chatId".to_string(), Value::String(new_chat_id.clone()));
        }
        state.storage.create("messages", message)?;
        if stop {
            break;
        }
    }
    Ok(new_chat)
}

pub(crate) fn peek_prompt(state: &AppState, chat_id: &str) -> AppResult<Value> {
    let messages: Vec<Value> = messages_for_chat(state, chat_id)?
        .into_iter()
        .map(|message| {
            json!({
                "role": message.get("role").and_then(Value::as_str).unwrap_or("user"),
                "content": message.get("content").and_then(Value::as_str).unwrap_or("")
            })
        })
        .collect();
    Ok(json!({ "messages": messages, "parameters": {}, "generationInfo": Value::Null }))
}

pub(crate) fn delete_chat_with_messages(state: &AppState, chat_id: &str) -> AppResult<()> {
    for message in messages_for_chat(state, chat_id)? {
        if let Some(id) = message.get("id").and_then(Value::as_str) {
            state.storage.delete("messages", id)?;
        }
    }
    state.storage.delete("chats", chat_id)?;
    Ok(())
}

