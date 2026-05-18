use super::*;
use super::shared::*;

pub(crate) fn resolve_llm_connection_for_request(state: &AppState, body: &Value) -> AppResult<Value> {
    if let Some(connection) = body.get("connection").filter(|value| value.is_object()) {
        return Ok(connection.clone());
    }
    if let Some(connection_id) = body
        .get("connectionId")
        .and_then(Value::as_str)
        .filter(|id| !id.is_empty())
    {
        return get_required(state, "connections", connection_id);
    }
    if body.get("provider").is_some() && body.get("model").is_some() {
        return Ok(body.clone());
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

pub(crate) fn llm_request_from_body(state: &AppState, body: Value) -> AppResult<marinara_llm::LlmRequest> {
    let connection = resolve_llm_connection_for_request(state, &body)?;
    let messages = body
        .get("messages")
        .and_then(Value::as_array)
        .ok_or_else(|| AppError::invalid_input("messages is required"))?
        .iter()
        .map(|message| {
            Ok(marinara_llm::LlmMessage {
                role: message
                    .get("role")
                    .and_then(Value::as_str)
                    .unwrap_or("user")
                    .to_string(),
                content: message
                    .get("content")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
            })
        })
        .collect::<AppResult<Vec<_>>>()?;
    Ok(marinara_llm::LlmRequest {
        connection: llm_connection_from_value(&connection)?,
        messages,
        parameters: body.get("parameters").cloned().unwrap_or_else(|| json!({})),
    })
}

pub(crate) async fn llm_complete(state: &AppState, body: Value) -> AppResult<Value> {
    let content = marinara_llm::complete(llm_request_from_body(state, body)?).await?;
    Ok(Value::String(content))
}

pub(crate) async fn llm_stream_events(state: &AppState, body: Value) -> AppResult<Vec<Value>> {
    let content = marinara_llm::complete(llm_request_from_body(state, body)?).await?;
    Ok(vec![
        json!({ "type": "start" }),
        json!({ "type": "token", "text": content }),
        json!({ "type": "done" }),
    ])
}

pub(crate) fn llm_models(_state: &AppState, _connection_id: Option<&str>) -> AppResult<Value> {
    Ok(json!([]))
}
pub(crate) fn llm_connection_from_value(value: &Value) -> AppResult<marinara_llm::LlmConnection> {
    let provider = value
        .get("provider")
        .and_then(Value::as_str)
        .ok_or_else(|| AppError::invalid_input("Connection provider is required"))?
        .to_string();
    let model = value
        .get("model")
        .and_then(Value::as_str)
        .filter(|model| !model.trim().is_empty())
        .ok_or_else(|| AppError::invalid_input("Connection model is required"))?
        .to_string();
    let api_key = value.get("apiKey").and_then(Value::as_str).unwrap_or("").to_string();
    let base_url = value.get("baseUrl").and_then(Value::as_str).unwrap_or("").to_string();
    Ok(marinara_llm::LlmConnection {
        provider,
        model,
        api_key,
        base_url,
    })
}
