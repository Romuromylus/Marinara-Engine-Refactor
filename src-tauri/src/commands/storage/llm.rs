// Thin Tauri-side wrapper over the lifted LLM handlers. Non-streaming entry
// points delegate to `marinara_handlers::llm`, which is shared with the Axum
// server target. The streaming command keeps a local body — it depends on the
// per-process cancellation registry on `AppState` plus Tauri's
// `tauri::ipc::Channel`. Phase 4b will lift the streaming surface onto an SSE
// route and the local helpers below can disappear.
use super::*;
use marinara_core::AppError;

// Re-exported so existing call sites in this crate keep their import path.
pub(crate) use marinara_handlers::llm::{
    llm_connection_from_value, llm_request_from_body, resolve_llm_connection_for_request,
};

pub(crate) async fn llm_complete(state: &AppState, body: Value) -> AppResult<Value> {
    marinara_handlers::llm::llm_complete(&state.storage, body).await
}

pub(crate) async fn llm_models(state: &AppState, connection_id: Option<&str>) -> AppResult<Value> {
    marinara_handlers::llm::llm_models(&state.storage, connection_id).await
}

pub(crate) async fn connection_models(state: &AppState, id: &str) -> AppResult<Value> {
    marinara_handlers::llm::connection_models(&state.storage, id).await
}

pub(crate) async fn llm_stream_channel(
    state: &AppState,
    stream_id: String,
    body: Value,
    on_event: tauri::ipc::Channel<Value>,
) -> AppResult<()> {
    let request = llm_request_from_body(&state.storage, body)?;
    let mut cancellation = state.register_llm_stream(&stream_id)?;
    if *cancellation.borrow() {
        state.unregister_llm_stream(&stream_id);
        return Ok(());
    }
    let result = tokio::select! {
        result = marinara_llm::stream_events(request, |event| {
            on_event
                .send(event)
                .map_err(|error| AppError::new("stream_channel_error", error.to_string()))
        }) => result,
        _ = cancellation.changed() => Ok(()),
    };
    state.unregister_llm_stream(&stream_id);
    result
}

pub(crate) fn llm_stream_cancel(state: &AppState, stream_id: &str) -> AppResult<Value> {
    Ok(json!({ "cancelled": state.cancel_llm_stream(stream_id)? }))
}
