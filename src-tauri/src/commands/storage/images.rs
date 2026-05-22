// Thin Tauri-side wrapper. All image-generation logic (provider HTTP
// dispatch + prompt building + connection helpers) lives in
// `marinara_handlers::images` so the Axum server can call the same code
// path. The local `images/providers.rs` file dropped out of the tree as
// part of Phase 4c — the canonical module is `crates/handlers/src/images/`.
use super::*;

// Re-exports the other Tauri-side modules (notably `sprites.rs`) still
// reach for under their old paths.
pub(crate) use marinara_handlers::images::{
    generate_image_with_options, image_generation_options, is_openai_gpt_image_model,
    prompt_override,
};

// Used by several sibling modules (bot_browser, fonts, translation, spotify,
// tts) under their old import path; kept here as a stable re-export rather
// than churning every call site.
pub(crate) use marinara_handlers::shared::percent_encode_component;

pub(crate) fn avatar_generation_preview(_state: &AppState, body: Value) -> AppResult<Value> {
    marinara_handlers::images::avatar_generation_preview(body)
}

pub(crate) async fn avatar_generation(state: &AppState, body: Value) -> AppResult<Value> {
    marinara_handlers::images::avatar_generation(&state.storage, body).await
}

pub(crate) async fn generate_image(state: &AppState, body: Value) -> AppResult<Value> {
    marinara_handlers::images::generate_image(&state.storage, body).await
}

pub(crate) async fn test_image_generation(state: &AppState, id: &str) -> AppResult<Value> {
    marinara_handlers::images::test_image_generation(&state.storage, id).await
}
