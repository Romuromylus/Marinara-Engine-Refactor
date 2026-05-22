// Thin Tauri adapter over the lifted `marinara_handlers::imports` module.
// Pre-lift, the import logic lived here inline. Phase 3e moved the bodies into
// the handlers crate so the Axum server target can share them; this shim only
// keeps the call-shape the Tauri command wrappers expect, plus the streaming
// adapter that bridges `tauri::ipc::Channel` into the closure-based progress
// emitter exposed by the lifted bulk-import routine.

use crate::state::AppState;
use marinara_core::{AppError, AppResult};
use marinara_handlers::imports as handlers;
use serde_json::Value;

pub(crate) fn import_call(state: &AppState, rest: &[&str], body: Value) -> AppResult<Value> {
    handlers::import_call(&state.storage, &state.data_dir, rest, body)
}

pub(crate) fn import_stream_channel(
    state: &AppState,
    rest: &[&str],
    body: Value,
    on_event: tauri::ipc::Channel<Value>,
) -> AppResult<()> {
    match rest {
        ["st-bulk", "run"] | ["st-bulk", "run-stream"] => {
            handlers::bulk_imports::run_st_bulk_import_channel(
                &state.storage,
                &state.data_dir,
                body,
                |event| {
                    on_event.send(event).map_err(|error| {
                        AppError::new("import_stream_channel_error", error.to_string())
                    })
                },
            )
        }
        _ => Err(AppError::new(
            "stream_not_supported",
            format!("Streaming is not supported for /import/{}", rest.join("/")),
        )),
    }
}
