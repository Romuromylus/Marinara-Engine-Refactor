use marinara_core::{AppError, AppResult};
use marinara_security::is_allowed_outbound_url;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LlmMessage {
    pub role: String,
    pub content: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub images: Vec<String>,
    #[serde(default)]
    pub tool_call_id: Option<String>,
    #[serde(default)]
    pub tool_calls: Option<Value>,
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
    #[serde(default)]
    pub tools: Vec<Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LlmCompletion {
    pub content: String,
    #[serde(rename = "toolCalls")]
    pub tool_calls: Vec<Value>,
}

pub async fn complete(request: LlmRequest) -> AppResult<String> {
    Ok(complete_rich(request).await?.content)
}

pub async fn complete_rich(request: LlmRequest) -> AppResult<LlmCompletion> {
    match request.connection.provider.as_str() {
        "anthropic" => complete_anthropic(request)
            .await
            .map(|content| LlmCompletion { content, tool_calls: Vec::new() }),
        "google" | "google_vertex" => complete_google(request)
            .await
            .map(|content| LlmCompletion { content, tool_calls: Vec::new() }),
        _ => complete_openai_compatible_rich(request).await,
    }
}

pub async fn stream_events(
    request: LlmRequest,
    mut emit: impl FnMut(Value) -> AppResult<()> + Send,
) -> AppResult<()> {
    emit(json!({ "type": "start" }))?;
    if request.connection.provider != "anthropic"
        && request.connection.provider != "google"
        && request.connection.provider != "google_vertex"
        && request.tools.is_empty()
    {
        stream_openai_compatible(request, &mut emit).await?;
    } else {
        let result = complete_rich(request).await?;
        if !result.content.is_empty() {
            emit(json!({ "type": "token", "text": result.content, "data": result.content }))?;
        }
        for tool_call in result.tool_calls {
            emit(json!({ "type": "tool_call", "data": tool_call }))?;
        }
    }
    emit(json!({ "type": "done" }))?;
    Ok(())
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
        "mistral" => "https://api.mistral.ai/v1".to_string(),
        "cohere" => "https://api.cohere.ai/compatibility/v1".to_string(),
        "openrouter" => "https://openrouter.ai/api/v1".to_string(),
        "nanogpt" => "https://nano-gpt.com/api/v1".to_string(),
        "xai" => "https://api.x.ai/v1".to_string(),
        _ => "https://api.openai.com/v1".to_string(),
    }
}

fn temperature(parameters: &Value) -> Option<f64> {
    parameters.get("temperature").and_then(Value::as_f64)
}

fn param_f64(parameters: &Value, keys: &[&str]) -> Option<f64> {
    keys.iter()
        .find_map(|key| parameters.get(*key).and_then(Value::as_f64))
}

fn param_i64(parameters: &Value, keys: &[&str]) -> Option<i64> {
    keys.iter()
        .find_map(|key| parameters.get(*key).and_then(Value::as_i64))
}

fn param_string(parameters: &Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        parameters
            .get(*key)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
    })
}

fn stop_sequences(parameters: &Value) -> Option<Vec<String>> {
    let value = parameters
        .get("stop")
        .or_else(|| parameters.get("stopSequences"))
        .or_else(|| parameters.get("stop_sequences"))?;
    if let Some(stop) = value.as_str().map(str::trim).filter(|value| !value.is_empty()) {
        return Some(vec![stop.to_string()]);
    }
    let stops = value
        .as_array()?
        .iter()
        .filter_map(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    (!stops.is_empty()).then_some(stops)
}

fn data_url_image(value: &str) -> Option<(&str, &str)> {
    let (meta, data) = value.split_once(',')?;
    let mime = meta.strip_prefix("data:")?.split(';').next()?;
    if !meta.to_ascii_lowercase().contains(";base64") || !mime.starts_with("image/") || data.is_empty() {
        return None;
    }
    Some((mime, data))
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

async fn complete_openai_compatible_rich(request: LlmRequest) -> AppResult<LlmCompletion> {
    let base = base_url(&request.connection.provider, &request.connection.base_url);
    let url = format!("{base}/chat/completions");
    ensure_url_allowed(&url)?;
    let messages: Vec<Value> = request
        .messages
        .iter()
        .map(openai_message)
        .collect();
    let mut body = json!({
        "model": request.connection.model,
        "messages": messages,
        "stream": false,
        "max_tokens": max_tokens(&request.parameters, 1024),
    });
    if !request.tools.is_empty() {
        body["tools"] = Value::Array(
            request
                .tools
                .iter()
                .map(|tool| json!({ "type": "function", "function": tool }))
                .collect(),
        );
        body["tool_choice"] = json!("auto");
    }
    if let Some(temp) = temperature(&request.parameters) {
        body["temperature"] = json!(temp);
    }
    apply_openai_parameters(&mut body, &request.parameters);
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
    parse_json_response_rich(response)
    .await
}

async fn stream_openai_compatible(
    request: LlmRequest,
    emit: &mut (impl FnMut(Value) -> AppResult<()> + Send),
) -> AppResult<()> {
    let base = base_url(&request.connection.provider, &request.connection.base_url);
    let url = format!("{base}/chat/completions");
    ensure_url_allowed(&url)?;
    let messages: Vec<Value> = request.messages.iter().map(openai_message).collect();
    let mut body = json!({
        "model": request.connection.model,
        "messages": messages,
        "stream": true,
        "max_tokens": max_tokens(&request.parameters, 1024),
    });
    if let Some(temp) = temperature(&request.parameters) {
        body["temperature"] = json!(temp);
    }
    apply_openai_parameters(&mut body, &request.parameters);
    let client = reqwest::Client::new();
    let mut req = client.post(url).json(&body);
    if !request.connection.api_key.trim().is_empty() {
        req = req.bearer_auth(request.connection.api_key.trim());
    }
    if request.connection.provider == "openrouter" {
        req = req
            .header("HTTP-Referer", "https://marinara.local")
            .header("X-Title", "Marinara Engine");
    }
    let response = req
        .send()
        .await
        .map_err(|error| AppError::new("llm_network_error", error.to_string()))?;
    let status = response.status();
    if !status.is_success() {
        let error_body = response.json::<Value>().await.unwrap_or_else(|_| json!({}));
        return Err(AppError::with_details(
            "llm_provider_error",
            format!("Provider returned HTTP {status}"),
            error_body,
        ));
    }

    let mut stream = response.bytes_stream();
    let mut buffer = String::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|error| AppError::new("llm_stream_error", error.to_string()))?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));
        while let Some(index) = buffer.find("\n\n") {
            let block = buffer[..index].to_string();
            buffer = buffer[index + 2..].to_string();
            process_openai_sse_block(&block, emit)?;
        }
    }
    if !buffer.trim().is_empty() {
        process_openai_sse_block(&buffer, emit)?;
    }
    Ok(())
}

fn process_openai_sse_block(
    block: &str,
    emit: &mut (impl FnMut(Value) -> AppResult<()> + Send),
) -> AppResult<()> {
    let payload = block
        .lines()
        .filter_map(|line| line.trim_start().strip_prefix("data:"))
        .map(str::trim)
        .collect::<Vec<_>>()
        .join("\n");
    if payload.is_empty() || payload == "[DONE]" {
        return Ok(());
    }
    let value: Value = serde_json::from_str(&payload)
        .map_err(|error| AppError::new("llm_stream_parse_error", error.to_string()))?;
    let Some(choices) = value.get("choices").and_then(Value::as_array) else {
        return Ok(());
    };
    for choice in choices {
        let delta = choice.get("delta").unwrap_or(choice);
        if let Some(content) = delta.get("content").and_then(Value::as_str) {
            if !content.is_empty() {
                emit(json!({ "type": "token", "text": content, "data": content }))?;
            }
        }
    }
    Ok(())
}

fn openai_message(message: &LlmMessage) -> Value {
    let mut object = serde_json::Map::new();
    object.insert("role".to_string(), json!(message.role));
    if message.images.is_empty() {
        object.insert("content".to_string(), json!(message.content));
    } else {
        let mut content = Vec::new();
        if !message.content.is_empty() {
            content.push(json!({ "type": "text", "text": message.content }));
        }
        for image in &message.images {
            content.push(json!({ "type": "image_url", "image_url": { "url": image } }));
        }
        object.insert("content".to_string(), Value::Array(content));
    }
    if let Some(name) = message.name.as_ref().filter(|value| !value.trim().is_empty()) {
        object.insert("name".to_string(), json!(name));
    }
    if let Some(tool_call_id) = message
        .tool_call_id
        .as_ref()
        .filter(|value| !value.trim().is_empty())
    {
        object.insert("tool_call_id".to_string(), json!(tool_call_id));
    }
    if let Some(tool_calls) = message.tool_calls.as_ref() {
        object.insert("tool_calls".to_string(), tool_calls.clone());
    }
    Value::Object(object)
}

fn apply_openai_parameters(body: &mut Value, parameters: &Value) {
    if let Some(top_p) = param_f64(parameters, &["topP", "top_p"]) {
        body["top_p"] = json!(top_p);
    }
    if let Some(frequency_penalty) = param_f64(parameters, &["frequencyPenalty", "frequency_penalty"]) {
        body["frequency_penalty"] = json!(frequency_penalty);
    }
    if let Some(presence_penalty) = param_f64(parameters, &["presencePenalty", "presence_penalty"]) {
        body["presence_penalty"] = json!(presence_penalty);
    }
    if let Some(seed) = param_i64(parameters, &["seed"]) {
        body["seed"] = json!(seed);
    }
    if let Some(stop) = stop_sequences(parameters) {
        body["stop"] = json!(stop);
    }
    if let Some(format) = param_string(parameters, &["responseFormat", "response_format"]) {
        body["response_format"] = json!({ "type": format });
    }
    if let Some(extra) = parameters.get("customParameters").or_else(|| parameters.get("custom_params")) {
        if let Some(entries) = extra.as_object() {
            for (key, value) in entries {
                if !body.get(key).is_some() {
                    body[key] = value.clone();
                }
            }
        }
    }
    if let Some(openrouter) = parameters.get("openrouter").or_else(|| parameters.get("openRouter")) {
        if !openrouter.is_null() {
            body["provider"] = openrouter.clone();
        }
    }
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
            if message.images.is_empty() {
                messages.push(json!({ "role": role, "content": message.content }));
            } else {
                let mut content = Vec::new();
                if !message.content.is_empty() {
                    content.push(json!({ "type": "text", "text": message.content }));
                }
                for image in &message.images {
                    if let Some((media_type, data)) = data_url_image(image) {
                        content.push(json!({
                            "type": "image",
                            "source": {
                                "type": "base64",
                                "media_type": media_type,
                                "data": data
                            }
                        }));
                    }
                }
                messages.push(json!({ "role": role, "content": content }));
            }
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
    if let Some(top_p) = param_f64(&request.parameters, &["topP", "top_p"]) {
        body["top_p"] = json!(top_p);
    }
    if let Some(top_k) = param_i64(&request.parameters, &["topK", "top_k"]) {
        body["top_k"] = json!(top_k);
    }
    if let Some(stop) = stop_sequences(&request.parameters) {
        body["stop_sequences"] = json!(stop);
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
            let mut parts = Vec::new();
            if !message.content.is_empty() {
                parts.push(json!({ "text": message.content }));
            }
            for image in &message.images {
                if let Some((mime_type, data)) = data_url_image(image) {
                    parts.push(json!({ "inlineData": { "mimeType": mime_type, "data": data } }));
                }
            }
            json!({ "role": role, "parts": parts })
        })
        .collect();
    let mut body = json!({
        "contents": contents,
        "generationConfig": {
            "temperature": temperature(&request.parameters).unwrap_or(0.7),
            "maxOutputTokens": max_tokens(&request.parameters, 1024),
        }
    });
    if let Some(top_p) = param_f64(&request.parameters, &["topP", "top_p"]) {
        body["generationConfig"]["topP"] = json!(top_p);
    }
    if let Some(top_k) = param_i64(&request.parameters, &["topK", "top_k"]) {
        body["generationConfig"]["topK"] = json!(top_k);
    }
    if let Some(stop) = stop_sequences(&request.parameters) {
        body["generationConfig"]["stopSequences"] = json!(stop);
    }
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

async fn parse_json_response_rich(response: reqwest::Response) -> AppResult<LlmCompletion> {
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
    let message = json
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|choices| choices.first())
        .and_then(|choice| choice.get("message"))
        .ok_or_else(|| {
            AppError::with_details(
                "llm_response_error",
                "Provider response did not contain an assistant message",
                json.clone(),
            )
        })?;
    let content = message
        .get("content")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let tool_calls = message
        .get("tool_calls")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(normalize_tool_call)
        .collect::<Vec<_>>();
    if content.is_empty() && tool_calls.is_empty() {
        return Err(AppError::with_details(
            "llm_response_error",
            "Provider response did not contain assistant text or tool calls",
            json,
        ));
    }
    Ok(LlmCompletion { content, tool_calls })
}

fn normalize_tool_call(call: Value) -> Value {
    let function = call
        .get("function")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let name = function
        .get("name")
        .or_else(|| call.get("name"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let arguments = function
        .get("arguments")
        .or_else(|| call.get("arguments"))
        .and_then(Value::as_str)
        .unwrap_or("{}")
        .to_string();
    json!({
        "id": call.get("id").and_then(Value::as_str).unwrap_or("").to_string(),
        "name": name,
        "arguments": arguments,
        "function": {
            "name": name,
            "arguments": arguments
        }
    })
}
