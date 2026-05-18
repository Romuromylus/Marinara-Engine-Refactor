use super::shared::ParsedPath;
use super::*;

mod haptic;
mod spotify;
mod spotify_callback;
mod tts;

pub(crate) async fn tts_call(
    state: &AppState,
    method: &str,
    rest: &[&str],
    body: Value,
) -> AppResult<Value> {
    tts::tts_call(state, method, rest, body).await
}

pub(crate) async fn spotify_call(
    state: &AppState,
    method: &str,
    rest: &[&str],
    route: &ParsedPath,
    body: Value,
) -> AppResult<Value> {
    spotify::spotify_call(state, method, rest, route, body).await
}

pub(crate) async fn haptic_call(rest: &[&str], body: Value) -> AppResult<Value> {
    haptic::haptic_call(rest, body).await
}
