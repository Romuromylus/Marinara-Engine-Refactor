use super::{bot_browser, shared};
use crate::state::AppState;
use marinara_core::AppError;
use serde_json::Value;
use tauri::State;

fn bot_browser_route(path: &str) -> shared::ParsedPath {
    let trimmed = path.trim_start_matches('/');
    let local = trimmed.strip_prefix("bot-browser/").unwrap_or(trimmed);
    shared::ParsedPath::new(local)
}

#[tauri::command]
pub async fn bot_browser_get(state: State<'_, AppState>, path: String) -> Result<Value, AppError> {
    let route = bot_browser_route(&path);
    let rest = route.parts.iter().map(String::as_str).collect::<Vec<_>>();
    bot_browser::bot_browser_call(&state, "GET", &rest, &route, Value::Null).await
}

#[tauri::command]
pub async fn bot_browser_post(
    state: State<'_, AppState>,
    path: String,
    body: Option<Value>,
) -> Result<Value, AppError> {
    let route = bot_browser_route(&path);
    let rest = route.parts.iter().map(String::as_str).collect::<Vec<_>>();
    bot_browser::bot_browser_call(&state, "POST", &rest, &route, body.unwrap_or(Value::Null)).await
}
