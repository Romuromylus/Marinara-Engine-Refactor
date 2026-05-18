use marinara_core::{AppError, AppResult};
use marinara_security::is_allowed_outbound_url;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LlmMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LlmConnection {
    pub provider: String,
    pub model: String,
    #[serde(rename = "apiKey", default)]
    pub api_key: String,
    #[serde(rename = "baseUrl", default)]
    pub base_url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LlmRequest {
    pub connection: LlmConnection,
    pub messages: Vec<LlmMessage>,
    #[serde(default)]
    pub parameters: Value,
}

pub async fn complete(request: LlmRequest) -> AppResult<String> {
    match request.connection.provider.as_str() {
        "anthropic" => complete_anthropic(request).await,
        "google" | "google_vertex" => complete_google(request).await,
        _ => complete_openai_compatible(request).await,
    }
}

pub fn unavailable_payload(message: impl Into<String>) -> Value {
    json!({ "type": "error", "error": message.into() })
}

fn base_url(provider: &str, configured: &str) -> String {
    let configured = configured.trim().trim_end_matches('/');
    if !configured.is_empty() {
        return configured.to_string();
    }
    match provider {
        "anthropic" => "https://api.anthropic.com".to_string(),
        "google" | "google_vertex" => "https://generativelanguage.googleapis.com".to_string(),
        "openrouter" => "https://openrouter.ai/api/v1".to_string(),
        "xai" => "https://api.x.ai/v1".to_string(),
        _ => "https://api.openai.com/v1".to_string(),
    }
}

fn temperature(parameters: &Value) -> Option<f64> {
    parameters.get("temperature").and_then(Value::as_f64)
}

fn max_tokens(parameters: &Value, fallback: u64) -> u64 {
    parameters
        .get("maxTokens")
        .or_else(|| parameters.get("max_tokens"))
        .and_then(Value::as_u64)
        .unwrap_or(fallback)
}

fn ensure_url_allowed(url: &str) -> AppResult<()> {
    if is_allowed_outbound_url(url, true) {
        Ok(())
    } else {
        Err(AppError::invalid_input(format!("Outbound URL is not allowed: {url}")))
    }
}

async fn complete_openai_compatible(request: LlmRequest) -> AppResult<String> {
    let base = base_url(&request.connection.provider, &request.connection.base_url);
    let url = format!("{base}/chat/completions");
    ensure_url_allowed(&url)?;
    let messages: Vec<Value> = request
        .messages
        .iter()
        .map(|message| json!({ "role": message.role, "content": message.content }))
        .collect();
    let mut body = json!({
        "model": request.connection.model,
        "messages": messages,
        "stream": false,
        "max_tokens": max_tokens(&request.parameters, 1024),
    });
    if let Some(temp) = temperature(&request.parameters) {
        body["temperature"] = json!(temp);
    }
    let client = reqwest::Client::new();
    let mut req = client.post(url).json(&body);
    if !request.connection.api_key.trim().is_empty() {
        req = req.bearer_auth(request.connection.api_key.trim());
    }
    if request.connection.provider == "openrouter" {
        req = req.header("HTTP-Referer", "https://marinara.local").header("X-Title", "Marinara Engine");
    }
    let response = req
        .send()
        .await
        .map_err(|error| AppError::new("llm_network_error", error.to_string()))?;
    parse_json_response(response, |json| {
        json.get("choices")
            .and_then(Value::as_array)
            .and_then(|choices| choices.first())
            .and_then(|choice| choice.get("message"))
            .and_then(|message| message.get("content"))
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
    })
    .await
}

async fn complete_anthropic(request: LlmRequest) -> AppResult<String> {
    let base = base_url(&request.connection.provider, &request.connection.base_url);
    let url = format!("{base}/v1/messages");
    ensure_url_allowed(&url)?;
    let mut system = Vec::new();
    let mut messages = Vec::new();
    for message in request.messages {
        if message.role == "system" {
            system.push(message.content);
        } else {
            let role = if message.role == "assistant" { "assistant" } else { "user" };
            messages.push(json!({ "role": role, "content": message.content }));
        }
    }
    let mut body = json!({
        "model": request.connection.model,
        "messages": messages,
        "max_tokens": max_tokens(&request.parameters, 1024),
    });
    if !system.is_empty() {
        body["system"] = json!(system.join("\n\n"));
    }
    if let Some(temp) = temperature(&request.parameters) {
        body["temperature"] = json!(temp);
    }
    let response = reqwest::Client::new()
        .post(url)
        .header("x-api-key", request.connection.api_key.trim())
        .header("anthropic-version", "2023-06-01")
        .json(&body)
        .send()
        .await
        .map_err(|error| AppError::new("llm_network_error", error.to_string()))?;
    parse_json_response(response, |json| {
        json.get("content")
            .and_then(Value::as_array)
            .and_then(|items| items.iter().find_map(|item| item.get("text").and_then(Value::as_str)))
            .map(ToOwned::to_owned)
    })
    .await
}

async fn complete_google(request: LlmRequest) -> AppResult<String> {
    let base = base_url(&request.connection.provider, &request.connection.base_url);
    let url = format!(
        "{base}/v1beta/models/{}:generateContent?key={}",
        request.connection.model,
        request.connection.api_key.trim()
    );
    ensure_url_allowed(&url)?;
    let contents: Vec<Value> = request
        .messages
        .into_iter()
        .filter(|message| message.role != "system")
        .map(|message| {
            let role = if message.role == "assistant" { "model" } else { "user" };
            json!({ "role": role, "parts": [{ "text": message.content }] })
        })
        .collect();
    let body = json!({
        "contents": contents,
        "generationConfig": {
            "temperature": temperature(&request.parameters).unwrap_or(0.7),
            "maxOutputTokens": max_tokens(&request.parameters, 1024),
        }
    });
    let response = reqwest::Client::new()
        .post(url)
        .json(&body)
        .send()
        .await
        .map_err(|error| AppError::new("llm_network_error", error.to_string()))?;
    parse_json_response(response, |json| {
        json.get("candidates")
            .and_then(Value::as_array)
            .and_then(|items| items.first())
            .and_then(|candidate| candidate.get("content"))
            .and_then(|content| content.get("parts"))
            .and_then(Value::as_array)
            .and_then(|parts| parts.iter().find_map(|part| part.get("text").and_then(Value::as_str)))
            .map(ToOwned::to_owned)
    })
    .await
}

async fn parse_json_response<F>(response: reqwest::Response, extract: F) -> AppResult<String>
where
    F: Fn(&Value) -> Option<String>,
{
    let status = response.status();
    let json: Value = response
        .json()
        .await
        .map_err(|error| AppError::new("llm_response_error", error.to_string()))?;
    if !status.is_success() {
        return Err(AppError::with_details(
            "llm_provider_error",
            format!("Provider returned HTTP {status}"),
            json,
        ));
    }
    extract(&json).ok_or_else(|| {
        AppError::with_details(
            "llm_response_error",
            "Provider response did not contain assistant text",
            json,
        )
    })
}
