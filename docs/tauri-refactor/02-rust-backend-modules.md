# Rust Backend Modules

This file lists the Rust crates and the expected files inside each crate. File names are suggestions, but the responsibilities should stay stable.

## Rust Simplicity Rule

The team is mainly JavaScript developers, so Rust should be used deliberately and kept approachable. Rust owns the backend, filesystem, secrets, provider calls, sidecars, integrations, and other security-sensitive responsibilities, but the implementation should stay minimal and explicit.

Prefer:

- plain structs and functions
- small services with clear inputs and outputs
- explicit repository methods
- straightforward error enums
- simple module trees

Avoid:

- framework-heavy designs
- clever trait hierarchies without immediate need
- excessive generic abstractions
- splitting crates or modules before there is real ownership pressure
- recreating a generic SQL/query layer

## Source Inventory Rule

Every Rust/backend rewrite slice must update source inventory status for the original app. Account for source routes, services, storage modules, schema/source metadata, shared types/schemas/constants/utilities, server assets, scripts, platform integrations, and behavioral tests from `E:/Personal Projects/Marinara-Engine`.

Each source item should be marked as ported, mapped to a Rust module, deferred to a later reviewed slice, or intentionally removed only with explicit human approval. Do not silently drop backend behavior during the Rust rewrite.

## `src-tauri/src`

### `main.rs`

Binary entrypoint. Calls `app_lib::run()`.

### `lib.rs`

Tauri runtime entrypoint. Builds the app, registers plugins, registers commands, registers state, and installs event listeners. No domain logic.

### `app.rs`

Creates the application graph:

- config loader
- storage manager
- secret manager
- provider registry
- sidecar manager
- event bus
- service registry

### `state.rs`

Defines `AppState` and typed handles passed into commands. State should use `Arc` and async locks only where necessary.

### `commands/mod.rs`

Registers every command group.

### `commands/*.rs`

Thin Tauri command modules. Each command validates input, calls a service, maps errors, and returns DTOs.

Planned files:

```text
commands/
  agents.rs
  app_settings.rs
  assets.rs
  bot_browser.rs
  characters.rs
  chat.rs
  connections.rs
  conversation.rs
  custom_tools.rs
  extensions.rs
  fonts.rs
  gallery.rs
  game.rs
  generation.rs
  haptic.rs
  imports.rs
  knowledge.rs
  lorebooks.rs
  personas.rs
  presets.rs
  prompts.rs
  regex_scripts.rs
  roleplay.rs
  spotify.rs
  themes.rs
  translation.rs
  tts.rs
  updates.rs
```

### `events/`

Typed event helpers. Events should be centrally named so frontend subscriptions are stable.

```text
events/
  mod.rs
  generation.rs
  imports.rs
  game.rs
  haptic.rs
  updates.rs
```

### `windows/`

Window setup, tray, deep links, external auth callback windows, and app menu behavior.

## Domain DTOs And Generated TypeScript

Do not use a giant central contracts crate. Each domain crate owns its frontend-facing DTOs next to the behavior that produces or consumes them.

Use a consistent local shape inside domain crates:

```text
src/
  dto.rs
  commands.rs
  service.rs
```

For larger domains, split DTOs by workflow:

```text
src/
  dto/
    mod.rs
    request.rs
    response.rs
    events.rs
```

Rust DTOs are the source of truth. Generate TypeScript bindings from all exported domain DTOs into `src/shared/api/bindings.ts` or a generated folder under `src/shared/api/bindings/`.

Rules:

- Do not hand-maintain duplicate TypeScript DTOs long term.
- Do not expose repository records directly when a narrower command DTO is safer.
- Do not put service methods or persistence behavior in DTO modules.
- Shared primitives such as IDs, pagination, timestamps, and app errors belong in `core`.
- Sync protocol DTOs are deferred with sync and do not belong in the active app graph.

## `core`

Shared backend foundations.

```text
src/
  lib.rs
  error.rs
  result.rs
  ids.rs
  clock.rs
  task.rs
  cancellation.rs
  event_bus.rs
  config.rs
  paths.rs
  logging.rs
```

Responsibilities:

- app error type
- ID generation
- cancellation tokens
- app data directory resolution
- event bus trait
- structured logging helpers

## `security`

Security-sensitive helpers.

```text
src/
  lib.rs
  secrets.rs
  encryption.rs
  path_safety.rs
  filenames.rs
  outbound_url.rs
  safe_fetch.rs
  content_types.rs
  permissions.rs
  admin_gate.rs
  csrf_compat.rs
```

Responsibilities:

- API key and OAuth token storage
- encryption key handling
- path traversal protection
- safe filename generation
- image MIME verification
- SSRF protection
- local URL policy
- admin secret compatibility where still needed

Secret handling should improve safety without changing the normal user workflow. Users should still add and manage provider keys and integration credentials through the same copied UI patterns; Rust changes where secrets are stored and what is returned to the frontend.

Rules:

- Store API keys, OAuth refresh tokens, auth cookies, and provider secrets outside normal file snapshots.
- Keep non-secret connection metadata in file storage.
- After save, return redacted secret status to the frontend, not raw saved values.
- Preserve existing add/edit/test connection UX wherever practical.

## `storage`

Durable storage for the fresh Tauri app.

The Tauri desktop app uses raw file storage only. Do not introduce SQLite, SQLx, libsql, or another database for local app data. Do not implement legacy SQLite import.

Use the current file-native model: `storage/manifest.json` plus one JSON snapshot per logical table under `storage/tables/`. Runtime compatibility for old profile backups, old archives, or old install layouts is intentionally excluded; write a separate migration script if old data conversion is needed later.

```text
src/
  lib.rs
  manager.rs
  manifest.rs
  migrations.rs
  file_store/
    mod.rs
    atomic_write.rs
    table_snapshot.rs
    ndjson.rs
  repositories/
    agents.rs
    app_settings.rs
    characters.rs
    character_gallery.rs
    chat_folders.rs
    chat_presets.rs
    chats.rs
    custom_tools.rs
    extensions.rs
    gallery.rs
    game_state.rs
    lorebooks.rs
    prompts.rs
    prompt_overrides.rs
    regex_scripts.rs
    themes.rs
    connections.rs
```

Responsibilities:

- load and save `DATA_DIR/storage`
- store one JSON snapshot per logical table plus a manifest
- read current file-native JSON/table snapshots
- repository APIs for each domain
- atomic writes
- storage version migrations for the current Tauri data model

Do not let services read or write files directly when repository methods are appropriate.
Do not recreate a generic SQL query layer in Rust. Prefer explicit repository methods and small domain-specific indexes where needed.
Do not rename or reshape existing table snapshot files without a documented migration.

## `llm`

Provider abstraction and model calling.

```text
src/
  lib.rs
  provider.rs
  registry.rs
  message.rs
  stream.rs
  usage.rs
  model_catalog.rs
  model_discovery.rs
  request_options.rs
  providers/
    mod.rs
    openai_compatible.rs
    anthropic.rs
    google.rs
    cohere.rs
    image_generation.rs
    claude_subscription.rs
  images/
    mod.rs
    stability.rs
    novelai.rs
    comfyui.rs
    automatic1111.rs
    nanogpt.rs
  embeddings/
    mod.rs
    provider.rs
```

Responsibilities:

- normalize chat requests
- stream model tokens
- list remote models
- test provider connections
- call image providers
- call embedding providers
- keep provider API keys out of frontend
- own every provider/authenticated network call that needs secrets, auth headers, cookies, or local URL policy checks

## `generation`

The replacement for `generate.routes.ts`.

```text
src/
  lib.rs
  service.rs
  request.rs
  response.rs
  stream.rs
  orchestrator.rs
  cancellation.rs
  prompt/
    mod.rs
    builder.rs
    sections.rs
    presets.rs
    overrides.rs
    peek.rs
  context/
    mod.rs
    window.rs
    attachments.rs
    summaries.rs
    memory.rs
  pipeline/
    mod.rs
    preflight.rs
    lorebook.rs
    regex.rs
    agents.rs
    postprocess.rs
    persistence.rs
  retry.rs
  dry_run.rs
  commands.rs
```

Responsibilities:

- normal chat/roleplay generation
- streaming event production
- prompt preview
- retry and dry run flows
- attachment handling
- lorebook injection
- regex script application
- agent pipeline integration
- saving generated messages and swipes

Streaming uses Tauri events, not HTTP/SSE compatibility. The service creates a run ID, emits typed events, supports cancellation, and lets the frontend preserve the existing streaming visual behavior through event subscriptions.

## `agents`

Agent definitions, execution, memory, and tool routing.

Agents, tools, and permissions are one coherent backend module. Do not port executable tools without the Rust permission model in the same slice.

```text
src/
  lib.rs
  registry.rs
  service.rs
  executor.rs
  pipeline.rs
  knowledge_router.rs
  knowledge_retrieval.rs
  memory.rs
  cost.rs
  debug.rs
  tools/
    mod.rs
    executor.rs
    custom.rs
    permissions.rs
    spotify.rs
    web.rs
    character.rs
```

Responsibilities:

- agent config CRUD service
- agent run execution
- secret plot driver
- expression agent
- knowledge routing
- custom tool execution policy
- Spotify tools
- tool result persistence
- permission checks for filesystem, network, process, secret, and integration access

## `chat`

Core chat data and message behavior.

```text
src/
  lib.rs
  service.rs
  messages.rs
  swipes.rs
  folders.rs
  summaries.rs
  transcript.rs
  branch.rs
  files.rs
  metadata.rs
```

Responsibilities:

- chats and messages
- swipes and branches
- chat folders
- transcript sanitation
- message metadata normalization
- chat file/gallery links

## `conversation`

Conversation-mode behavior.

```text
src/
  lib.rs
  service.rs
  schedules.rs
  autonomous.rs
  background_autonomous.rs
  awareness.rs
  commands.rs
  summaries.rs
```

Responsibilities:

- autonomous check-ins
- character schedules
- cross-chat awareness
- conversation command handling
- day/week summaries

## `roleplay`

Roleplay and visual-novel behavior.

```text
src/
  lib.rs
  service.rs
  scene/
    mod.rs
    service.rs
    analyzer.rs
    postprocess.rs
    prompts.rs
  sprites/
    mod.rs
    service.rs
    placement.rs
    import.rs
    generation.rs
  encounter/
    mod.rs
    service.rs
    generation.rs
    state.rs
  visual_novel/
    mod.rs
    state.rs
    choices.rs
```

Responsibilities:

- scenes
- sprite metadata and placement
- expression inference
- CYOA choices
- roleplay encounters
- visual-novel state

## `game`

Game-mode behavior. Split by ownership because this area is large, but keep the Rust modules explicit and reviewable.

```text
src/
  lib.rs
  service.rs
  turn/
    mod.rs
    orchestrator.rs
    request.rs
    response.rs
    stream.rs
    persistence.rs
  prompts/
    mod.rs
    gm.rs
    party.rs
    templates.rs
    formatting.rs
  state/
    mod.rs
    machine.rs
    snapshots.rs
    checkpoints.rs
    repair.rs
  mechanics/
    mod.rs
    dice.rs
    skill_check.rs
    combat.rs
    loot.rs
    morale.rs
    reputation.rs
    elements.rs
    perception.rs
  world/
    mod.rs
    map.rs
    map_position.rs
    weather.rs
    time.rs
    journal.rs
    travel.rs
  assets/
    mod.rs
    manifest.rs
    generation.rs
    sprites.rs
    music.rs
    sfx.rs
  session/
    mod.rs
    service.rs
    history.rs
    summary.rs
    normalization.rs
```

Responsibilities:

- game turns
- GM prompt assembly
- party prompts
- dice and checks
- combat
- inventory, loot, journal
- maps, travel, weather, time
- game sessions and checkpoints
- generated game assets
- game state repair

## `assets`

Images, backgrounds, avatars, gallery, fonts, media, and asset seeding.

Keep this crate explicit and workflow-oriented. Rust owns validation, storage paths, blob placement, thumbnails, and default asset seeding; React keeps the existing upload, preview, crop, and review UI.

```text
src/
  lib.rs
  service.rs
  image_validation.rs
  avatars.rs
  backgrounds.rs
  gallery.rs
  fonts.rs
  default_assets.rs
  generated_images.rs
  thumbnails.rs
```

Responsibilities:

- avatar upload and serving
- background upload and default seeding
- gallery image storage
- image type validation
- thumbnails
- font discovery and serving

## `sidecar`

Local AI runtime and model management.

Preserve the current sidecar UX and feature behavior, but do not port the old custom sidecar internals line-for-line. Prefer rewriting this portion around an existing Rust inference package/runtime where practical. Rust owns runtime/model/package management, downloads, process control, logs, health checks, local inference, and sidecar-backed scene analysis. React keeps the current status, settings, model download, logs, and progress UI.

Package/runtime details should be inventoried during the sidecar implementation slice from the original source and scripts, especially `scripts/build-sidecar-runtime.mjs`, `scripts/install-backgroundremover.mjs`, and `packages/server/src/services/sidecar/*`.

Crane (`https://github.com/lucasjinreal/Crane`) is a sidecar runtime candidate to evaluate. It is a Rust/Candle inference engine with LLM, VLM, TTS, OCR, and OpenAI-compatible server support. Do not lock it in until model coverage, platform support, binary size, packaging complexity, and maintenance risk are reviewed.

```text
src/
  lib.rs
  service.rs
  config.rs
  models.rs
  model_files.rs
  downloads.rs
  runtime/
    mod.rs
    install.rs
    llama_cpp.rs
    mlx.rs
    env.rs
  process/
    mod.rs
    manager.rs
    launch_plan.rs
    logs.rs
    health.rs
  inference.rs
  scene_analysis.rs
```

Responsibilities:

- model catalog
- model downloads
- runtime install
- sidecar package/runtime management
- llama.cpp and MLX process control
- logs
- local inference
- sidecar-backed scene analysis

## `import`

Imports, exports, and migration tools.

Keep import code explicit by workflow. Do not build a generic importer framework before it is needed. Rust owns file/folder access, path tokens, parsing, validation, current profile package import/export, and persistence; React keeps picker triggers, drag/drop UI, review dialogs, and progress display.

```text
src/
  lib.rs
  service.rs
  folder_picker.rs
  path_tokens.rs
  sillytavern/
    mod.rs
    scanner.rs
    character_import.rs
    chat_import.rs
    bulk_import.rs
  profile/
    mod.rs
    export.rs
    import.rs
  png_card.rs
  pdf.rs
  file_storage.rs
```

Responsibilities:

- SillyTavern imports
- character card import/export
- profile import/export
- folder browsing with tokens
- PDF/text import helpers
- file-storage compatibility migration

## `integrations`

External integrations that are not core LLM providers.

Keep these integrations together for the first Rust plan. Split one into a separate crate only after implementation proves it has independent complexity, build constraints, or release needs.

```text
src/
  lib.rs
  spotify/
    mod.rs
    oauth.rs
    tokens.rs
    playback.rs
    tools.rs
  haptic/
    mod.rs
    service.rs
    devices.rs
    patterns.rs
    permissions.rs
  tts/
    mod.rs
    service.rs
    providers.rs
    cache.rs
    cues.rs
  translation/
    mod.rs
    service.rs
    deeplx.rs
    provider.rs
  bot_browser/
    mod.rs
    chub.rs
    janny.rs
    chartavern.rs
    pygmalion.rs
    wyvern.rs
    datacat.rs
    auth_store.rs
  gifs/
    mod.rs
    giphy.rs
  discord/
    mod.rs
    webhook.rs
  home_assistant/
    mod.rs
    webhook.rs
```

Responsibilities:

- Spotify OAuth, refresh, playback tools
- haptic device communication
- TTS provider calls and cache keys
- translation provider calls
- bot-browser proxy and authenticated sessions
- GIF search
- Discord webhooks
- Home Assistant bridge support

All authenticated integration calls belong here or in `llm`, not in React hooks. The frontend keeps the existing UI and display state but calls Rust commands for network work.

## `updates`

App update checks and update application.

```text
src/
  lib.rs
  service.rs
  check.rs
  release_notes.rs
  apply.rs
  permissions.rs
```

Responsibilities:

- update metadata checks
- release note rendering
- gated update application
- remote apply safety

## Deferred Sync

Sync protocol, sync client, and Docker sync server work are outside the active Tauri migration. Do not add sync runtime code, settings UI, queue status, pairing, or conflict-review surfaces until sync is intentionally reopened.

```text
sync-server/
  src/
    main.rs
    app.rs
    state.rs
    routes/
    services/
    storage/
    sync/
    security/
```

Responsibilities:

- account and device auth
- sync push/pull API
- WebSocket heads broadcast
- blob storage
- Docker deployment
- OpenAPI docs
