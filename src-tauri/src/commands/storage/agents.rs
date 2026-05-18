use super::*;
use super::shared::*;

pub(crate) fn toggle_agent_type(state: &AppState, agent_type: &str) -> AppResult<Value> {
    if let Some(agent) = find_by_field(state, "agents", "type", agent_type)? {
        let id = agent.get("id").and_then(Value::as_str).unwrap_or(agent_type);
        let enabled = !agent.get("enabled").and_then(Value::as_bool).unwrap_or(true);
        state.storage.patch("agents", id, json!({ "enabled": enabled }))
    } else {
        state.storage.create("agents", json!({ "type": agent_type, "enabled": true }))
    }
}

pub(crate) fn patch_agent_type(state: &AppState, agent_type: &str, body: Value) -> AppResult<Value> {
    if let Some(agent) = find_by_field(state, "agents", "type", agent_type)? {
        let id = agent.get("id").and_then(Value::as_str).unwrap_or(agent_type);
        state.storage.patch("agents", id, body)
    } else {
        let mut object = ensure_object(body)?;
        object.insert("type".to_string(), Value::String(agent_type.to_string()));
        state.storage.create("agents", Value::Object(object))
    }
}

