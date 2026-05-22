// Thin Tauri-side wrapper. All Spotify dispatch lives in
// `marinara_handlers::integrations::spotify` so the Axum server can call
// the same code path. The local TCP loopback callback listener stays here
// (`spotify_callback.rs`) — it depends on `localhost:8754` reachability that
// the server target can't provide. After `authorize` returns from the
// shared handler, this shim spins up the listener and patches the
// `callbackListenerStarted` flag in the response.

use super::super::shared::ParsedPath;
use super::super::*;
use super::spotify_callback::start_callback_listener;

pub(crate) async fn spotify_call(
    state: &AppState,
    method: &str,
    rest: &[&str],
    route: &ParsedPath,
    body: Value,
) -> AppResult<Value> {
    // The shared dispatcher takes its own ParsedPath type; rebuild it from
    // the same parts/query the Tauri shim already parsed.
    let mut lifted_route = marinara_handlers::integrations::spotify::ParsedPath::new("");
    lifted_route.parts = route.parts.clone();
    lifted_route.query = route.query.clone();
    let mut result = marinara_handlers::integrations::spotify::spotify_call(
        &state.storage,
        method,
        rest,
        &lifted_route,
        body,
    )
    .await?;

    // Authorize is the only route that needs desktop-specific augmentation:
    // it stashed `callbackListenerStarted: false` in the lifted code, and we
    // patch the real value here after spinning up the loopback listener.
    if matches!((method, rest), ("GET" | "POST", ["authorize"])) {
        if let Some(object) = result.as_object_mut() {
            let started = start_callback_listener(state.clone());
            object.insert("callbackListenerStarted".to_string(), Value::Bool(started));
        }
    }

    Ok(result)
}

// `exchange` keeps a re-export so `spotify_callback.rs` can stay unchanged.
pub(super) async fn exchange(state: &AppState, body: Value) -> AppResult<Value> {
    marinara_handlers::integrations::spotify::exchange(&state.storage, body).await
}
