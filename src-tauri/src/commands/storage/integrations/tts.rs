// Thin Tauri-side wrapper over `marinara_handlers::integrations::tts`. The
// shared module owns all the OpenAI / ElevenLabs / PocketTTS / NanoGPT
// dispatch + config persistence + voice-list normalization; this file just
// hands `&state.storage` over to it.
use super::super::*;

pub(crate) async fn tts_call(
    state: &AppState,
    method: &str,
    rest: &[&str],
    body: Value,
) -> AppResult<Value> {
    marinara_handlers::integrations::tts::tts_call(&state.storage, method, rest, body).await
}
