// Thin Tauri adapter over the lifted `marinara_handlers::profile` module.
// Pre-lift the snapshot / import / asset-restoration logic lived here.
// Phase 3e2 moved the bodies into the handlers crate so the Axum server can
// share them; this shim just bridges Tauri's AppState into the
// (`storage`, `data_dir`) pair the lifted functions take.

use super::shared::ParsedPath;
use crate::state::AppState;
use marinara_core::AppResult;
use marinara_handlers::profile as handlers;
use serde_json::Value;

pub(crate) fn profile_snapshot(state: &AppState) -> AppResult<Value> {
    handlers::profile_snapshot(&state.storage, &state.data_dir)
}

pub(crate) fn import_profile_file_path(state: &AppState, value: &str) -> AppResult<Value> {
    handlers::import_profile_file_path(&state.storage, &state.data_dir, value)
}

pub(crate) fn profile_call(
    state: &AppState,
    method: &str,
    rest: &[&str],
    route: &ParsedPath,
    body: Value,
) -> AppResult<Value> {
    handlers::profile_call(
        &state.storage,
        &state.data_dir,
        method,
        rest,
        route.query.get("format").map(String::as_str),
        body,
    )
}
