use super::*;
use super::shared::*;

use super::chats::{chat_messages, messages_for_chat};
use super::llm::llm_connection_from_value;

pub(crate) async fn generate_events(state: &AppState, body: Value) -> AppResult<Vec<Value>> {
    let chat_id = body
        .get("chatId")
        .and_then(Value::as_str)
        .ok_or_else(|| AppError::invalid_input("chatId is required"))?;
    let message = body.get("message").and_then(Value::as_str).unwrap_or("").trim();
    if !message.is_empty() {
        chat_messages(state, "POST", chat_id, json!({ "role": "user", "content": message }), &HashMap::new())?;
    }
    let connection = resolve_generation_connection(state, chat_id, &body)?;
    let mut prompt_messages: Vec<marinara_llm::LlmMessage> = messages_for_chat(state, chat_id)?
        .into_iter()
        .filter_map(|message| {
            let role = message.get("role").and_then(Value::as_str)?.to_string();
            let content = message.get("content").and_then(Value::as_str)?.to_string();
            Some(marinara_llm::LlmMessage { role, content })
        })
        .collect();
    if let Some(guide) = body.get("generationGuide").and_then(Value::as_str).filter(|value| !value.trim().is_empty()) {
        prompt_messages.push(marinara_llm::LlmMessage {
            role: "user".to_string(),
            content: guide.to_string(),
        });
    }
    if prompt_messages.is_empty() {
        return Err(AppError::invalid_input("Cannot generate without chat messages"));
    }
    let parameters = connection
        .get("defaultParameters")
        .and_then(Value::as_str)
        .and_then(|raw| serde_json::from_str::<Value>(raw).ok())
        .unwrap_or_else(|| json!({}));
    let request = marinara_llm::LlmRequest {
        connection: llm_connection_from_value(&connection)?,
        messages: prompt_messages,
        parameters,
    };
    let content = marinara_llm::complete(request).await?;
    let assistant = chat_messages(
        state,
        "POST",
        chat_id,
        json!({ "role": "assistant", "content": content.clone() }),
        &HashMap::new(),
    )?;
    Ok(vec![
        json!({ "type": "phase", "data": "Calling model..." }),
        json!({ "type": "token", "data": content }),
        json!({ "type": "assistant_message", "data": assistant }),
        json!({ "type": "done", "data": null }),
    ])
}

pub(crate) async fn test_connection(state: &AppState, id: &str) -> AppResult<Value> {
    let started = std::time::Instant::now();
    let connection = get_required(state, "connections", id)?;
    let request = marinara_llm::LlmRequest {
        connection: llm_connection_from_value(&connection)?,
        messages: vec![marinara_llm::LlmMessage {
            role: "user".to_string(),
            content: "Reply with exactly: ok".to_string(),
        }],
        parameters: json!({ "maxTokens": 16, "temperature": 0 }),
    };
    let response = marinara_llm::complete(request).await?;
    Ok(json!({
        "success": true,
        "message": response,
        "latencyMs": started.elapsed().as_millis(),
        "modelName": connection.get("model").and_then(Value::as_str)
    }))
}

pub(crate) async fn test_message(state: &AppState, id: &str) -> AppResult<Value> {
    let started = std::time::Instant::now();
    let connection = get_required(state, "connections", id)?;
    let request = marinara_llm::LlmRequest {
        connection: llm_connection_from_value(&connection)?,
        messages: vec![marinara_llm::LlmMessage {
            role: "user".to_string(),
            content: "hi".to_string(),
        }],
        parameters: json!({ "maxTokens": 64, "temperature": 0.7 }),
    };
    let response = marinara_llm::complete(request).await?;
    Ok(json!({
        "success": true,
        "response": response,
        "latencyMs": started.elapsed().as_millis()
    }))
}

pub(crate) fn resolve_generation_connection(state: &AppState, chat_id: &str, body: &Value) -> AppResult<Value> {
    if let Some(connection_id) = body.get("connectionId").and_then(Value::as_str).filter(|id| !id.is_empty()) {
        return get_required(state, "connections", connection_id);
    }
    let chat = get_required(state, "chats", chat_id)?;
    if let Some(connection_id) = chat.get("connectionId").and_then(Value::as_str).filter(|id| !id.is_empty()) {
        return get_required(state, "connections", connection_id);
    }
    let connections = state.storage.list("connections")?;
    if let Some(default) = connections
        .iter()
        .find(|connection| connection.get("isDefault").and_then(Value::as_bool).unwrap_or(false))
        .cloned()
    {
        return Ok(default);
    }
    connections
        .into_iter()
        .next()
        .ok_or_else(|| AppError::invalid_input("No LLM connection is configured"))
}
