pub mod admin;
pub mod agents;
pub mod backgrounds;
pub mod chats;
pub mod entities;
pub mod game_assets;
pub mod shared;
pub mod tracker;

/// Marinara has no protected records yet; the function exists so the lift can
/// preserve the original guard-call shape that lives in `src-tauri/src/builtins.rs`.
pub fn is_protected_record(_collection: &str, _id: &str) -> bool {
    false
}
