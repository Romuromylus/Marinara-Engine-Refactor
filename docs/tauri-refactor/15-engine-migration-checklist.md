# Engine Migration Checklist

This file tracks the full Marinara Engine migration into the Tauri refactor. Keep it updated after every migration pass.

Last updated: 2026-05-18.

## Status Legend

- `[ ]` Not started
- `[~]` In progress
- `[x]` Moved/wired
- `[d]` Deferred by scope

## Architecture Anchors

- [x] Confirmed top-level modes are `chat`, `roleplay`, and `game`.
- [x] Confirmed conversation/autonomous is a chat subsystem under `engine/modes/chat`.
- [x] Confirmed sidecar and sync-client are deferred external-service scope.
- [~] Create `src/engine` layers from `docs/tauri-refactor/14-layered-module-architecture.md`.
- [x] Replace server-stub frontend API with Tauri-backed API adapter.
- [~] Build Rust capability commands for storage, assets/imports, LLM transport, integrations, and updates.
- [x] Removed direct browser `fetch('/api/...')` calls for local Fastify routes; local app calls now go through Tauri API adapters or local TypeScript helpers.
- [~] Removed sidecar from active onboarding, agent, connection, and game scene UI while preserving deferred sidecar code for later reintroduction.

## TypeScript Engine Layers

- [x] Layer 0: `src/engine/contracts` from `packages/shared/src`.
- [x] Layer 0: `src/engine/core` primitives.
- [x] Layer 1: `src/engine/capabilities` TypeScript ports.
- [x] Layer 2: `src/engine/shared` pure helpers.
- [ ] Layer 3: `src/engine/entities` pure entity operations.
- [x] Layer 4: `src/engine/repositories` capability-backed repositories.
- [~] Layer 5: `src/engine/generation-core` prompt, lorebook, regex, LLM DTO helpers.
- [~] Layer 6: `src/engine/agents-runtime` agent executor, pipeline, knowledge, and tools.
- [~] Layer 7: `src/engine/generation` generation orchestration and stream event DTOs.
- [~] Layer 8: `src/engine/modes/chat` chat core, autonomous, awareness, schedules, commands.
- [ ] Layer 8: `src/engine/modes/roleplay` scene, sprites, encounter, visual-novel.
- [~] Layer 8: `src/engine/modes/game` turn, prompts, mechanics, state, world, assets.

## Rust Capability Layers

- [ ] `src-tauri/src/app.rs` capability graph.
- [x] `src-tauri/src/state.rs` shared state.
- [x] `src-tauri/src/commands/mod.rs` command registration.
- [x] `src-tauri/src/commands/storage.rs` command facade split into capability modules under `src-tauri/src/commands/storage/`.
- [~] `src-tauri/src/commands/assets.rs` asset commands; currently routed through the storage command shim and `marinara-assets`.
- [~] Import commands: character JSON/PNG/CharX, native `.marinara` packages, lorebooks, presets, personas, JSONL chat imports, and basic ST bulk scan/run are routed through native storage; full media/archive/profile parity is still pending.
- [ ] `src-tauri/src/commands/llm.rs` provider transport commands.
- [ ] `src-tauri/src/commands/integrations.rs` integration commands.
- [ ] `src-tauri/src/commands/updates.rs` update commands.
- [ ] `src-tauri/src/events/mod.rs` event names/helpers.
- [x] `src-tauri/crates/core` paths, IDs, errors, timestamps.
- [x] `src-tauri/crates/security` path and outbound policy helpers.
- [x] `src-tauri/crates/storage` raw file storage and atomic writes.
- [x] `src-tauri/crates/assets` file/blob/media handling for managed game assets and backgrounds.
- [~] `src-tauri/crates/import` import file helpers.
- [x] `src-tauri/crates/llm` provider transport.
- [~] `src-tauri/crates/integrations` Spotify/TTS/translation/haptic/webhook placeholders or transport.
- [x] `src-tauri/crates/updates` update check/apply planning.
- [d] `src-tauri/crates/sidecar` deferred external-service scope.
- [d] `src-tauri/crates/sync-client` deferred external-service scope.
- [d] `src-tauri/crates/sync-protocol` deferred until sync returns.

## Original Source Coverage

- [x] `packages/shared/src/constants`
- [x] `packages/shared/src/schemas`
- [x] `packages/shared/src/types`
- [x] `packages/shared/src/utils`
- [~] `packages/server/src/services/prompt`
- [~] `packages/server/src/services/lorebook`
- [x] `packages/server/src/services/regex`
- [~] `packages/server/src/services/agents`
- [ ] `packages/server/src/services/tools`
- [~] `packages/server/src/routes/generate`
- [~] `packages/server/src/services/conversation`
- [~] `packages/server/src/services/game`
- [~] `packages/server/src/services/llm`
- [~] `packages/server/src/services/image`
- [ ] `packages/server/src/services/spotify`
- [~] `packages/server/src/services/import`
- [~] `packages/server/src/services/storage`
- [ ] `packages/server/src/db`
- [ ] `packages/server/src/routes`
- [ ] `packages/server/src/utils`
- [ ] `packages/server/src/middleware`
- [d] `packages/server/src/services/sidecar`
- [~] `packages/client/src/components`
- [~] `packages/client/src/hooks`
- [~] `packages/client/src/lib`
- [~] `packages/client/src/stores`
- [~] `packages/client/src/styles`

## 2026-05-17 Migration Pass

- [x] Replaced remaining direct local Fastify browser fetches with Tauri-backed API helpers or local asset URL resolution.
- [x] Added managed local file URL handling for backgrounds and game assets through Tauri asset protocol paths.
- [x] Wired character, persona, preset, lorebook, connection, chat, gallery, backup, import, knowledge-source, bot-browser, GIF, and prompt-review utility frontend paths away from deferred throwing API shims.
- [x] Added Tauri-backed chat gallery and character gallery upload/list/delete storage.
- [x] Added Tauri-backed default Pollinations image test and avatar generation route, plus NPC avatar upload persistence.
- [x] Hid active sidecar model controls from onboarding, agents, connections, and game scene setup/runtime surfaces.
- [~] Added basic local backup/import/profile routes; full archive format, binary media bundling, and original parser parity remain pending.
- [~] Added Chub-focused bot browser routes; Pygmalion/CharTavern/session parity remains pending or intentionally inactive.

## 2026-05-18 Migration Pass

- [x] Split the previous monolithic native storage route shim into focused modules (`router`, `shared`, `chats`, `imports`, `bulk_imports`, `llm`, `scene`, `game`, assets, integrations, and related capability slices).
- [x] Added visible error toasts to create connection, character, persona, and preset modals so failed native calls no longer look like dead sidebar buttons.
- [x] Reworked import routes for the sidebar/settings import flows: ST character inspect/batch now returns the frontend shape, PNG character-card metadata is parsed from `chara`/`ccv3` chunks, CharX reads `card.json` and embedded icons, and native `.marinara` packages read `data.json` plus avatar assets.
- [x] Added native JSONL chat import and branch import routes used by the chat/settings import UI.
- [x] Added local SillyTavern folder browsing response shape plus basic ST bulk scan/run for characters, chats, group chats, presets, lorebooks, backgrounds, and personas.
- [~] Normalized lorebook imports into lorebook rows plus `lorebook-entries`; deeper original importer parity, timestamp fidelity, category/tag heuristics, media bundling, and profile archive restore still need follow-up.

## Current Blockers Before Migration Can Be Called Complete

- [ ] Finish full image-generation provider parity beyond the default Pollinations path: OpenAI-compatible image APIs, NovelAI, Stability, Horde, ComfyUI, Automatic1111, RunPod ComfyUI, Draw Things, and NanoGPT.
- [ ] Finish TTS provider transport and playback parity; `/tts/speak` is still not a real migrated backend.
- [ ] Finish Spotify OAuth/player/search/playback parity and haptic integration parity.
- [ ] Finish sprite sheet generation, cleanup, and restore parity; current routes are compile-safe but do not perform real image processing.
- [ ] Finish bot-browser parity for non-Chub sources and authenticated source sessions.
- [~] Finish import/export parity for full SillyTavern, Marinara archive, profile archive, media bundling, timestamp fidelity, and exact original importer heuristics.
- [ ] Finish game route parity for mechanics that currently return minimal state transitions or generated summaries.
- [ ] Finish prompt reviewer, character maker, persona maker, lorebook maker, and generation-agent workflows so they use migrated orchestration rather than minimal deterministic fallbacks.

## Verification

- [x] `pnpm typecheck` passed on 2026-05-17.
- [x] `cargo check --manifest-path src-tauri/Cargo.toml` passed on 2026-05-17.
- [x] `pnpm check:docs` passed on 2026-05-17.
- [x] `pnpm build` passed on 2026-05-17 with Vite large-chunk warnings only.
- [x] `cargo check --manifest-path src-tauri/Cargo.toml` passed on 2026-05-18 after storage/import split.
- [x] `pnpm typecheck` passed on 2026-05-18.
- [x] `pnpm build` passed on 2026-05-18 with Vite large-chunk warnings only.
- [x] `pnpm check:docs` passed on 2026-05-18.
