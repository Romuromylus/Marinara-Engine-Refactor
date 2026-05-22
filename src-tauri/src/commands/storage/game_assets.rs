// All game-asset command bodies now live in `marinara_handlers::game_assets`
// so the Axum server shares the implementation. This file stays as an empty
// module so the `mod game_assets;` declaration in `src-tauri/src/commands/storage.rs`
// keeps working — the storage module's submodule discovery is path-based.
