# Target Structure

## Repository Layout

```text
Marinara-Engine-Refactor/
  docs/
    tauri-refactor/
  public/
  src/
    app/
    shared/
    features/
  src-tauri/
    Cargo.toml
    tauri.conf.json
    capabilities/
    src/
    crates/
```

## Rust Workspace Layout

```text
src-tauri/
  src/
    main.rs
    lib.rs
    app.rs
    state.rs
    commands/
    events/
    windows/
  crates/
    core/
    security/
    storage/
    llm/
    generation/
    agents/
    chat/
    conversation/
    roleplay/
    game/
    assets/
    import/
    integrations/
    updates/
```

Sidecar and sync crates/services are outside the active app structure for this migration.

## Frontend Layout

```text
src/
  app/
    App.tsx
    main.tsx
    providers/
    shell/
    startup/
  shared/
    api/
    components/
    hooks/
    lib/
    stores/
    styles/
    types/
  features/
    agents/
    assets/
    bot-browser/
    characters/
    chat/
    connections/
    conversation/
    extensions/
    gallery/
    game/
    haptic/
    imports/
    knowledge/
    lorebooks/
    personas/
    presets/
    prompts/
    roleplay/
    settings/
    sidecar/
    spotify/
    sync/
    themes/
    translation/
    tts/
    updates/
```

## Ownership Boundaries

### Rust Owns

- Persistent data.
- API keys, OAuth tokens, encryption keys, and provider credentials.
- Provider calls and model discovery.
- Authenticated network calls, provider tests, image generation, TTS, translation, Spotify OAuth/playback calls, and bot-browser authenticated fetches.
- Prompt assembly and generation orchestration.
- Agent execution and tool calls.
- Local model downloads and sidecar process management.
- Imports from local folders.
- Backups and restores.
- Media validation, blob placement, character card PNG parsing/export, and thumbnail generation when needed.
- Haptic device access.
- Spotify OAuth token exchange and refresh.
- Bot-browser authenticated proxy flows.
- Update checks and update application.
- Local sync queue, sync client, conflict metadata, and device pairing.
- Filesystem access outside browser-selected blobs.
- SSRF, path traversal, and content-type protections.

### Frontend Owns

- Rendering and layout.
- React Query hooks and local optimistic UI state.
- Dialog state, drawers, tabs, filters, sort order.
- Form drafts and validation display.
- Canvas previews, image cropping previews, drag/drop UX.
- File picker triggers, import review dialogs, media previews, and progress display.
- Audio playback UI and mute state.
- Markdown display.
- Prompt preview rendering.
- Keyboard shortcuts and responsive layout behavior.
- Sync status display, device list, and conflict review UI.
- Existing provider/settings UI flows and display state.

### Domain DTOs Own

- DTOs crossing the Tauri boundary, colocated with the domain crate that owns the behavior.
- Enums and mode-specific metadata for that domain.
- Validation schemas where both Rust and TypeScript need the same shape.
- Provider/model metadata that does not include secrets.
- Pure helpers that do not touch DOM, filesystem, network, or Tauri.

Do not create a giant central contracts crate. Rust domain crates are the source of truth for their own frontend-facing DTOs, and TypeScript bindings are generated from those Rust types.

## Dependency Direction

```text
commands -> services -> repositories
commands -> domain DTOs
services -> domain DTOs, core, storage, security
frontend -> shared/api -> Tauri commands
frontend -> generated TypeScript bindings
domain DTOs -> no app crate dependencies
```

No domain crate should import `tauri`. Tauri imports domain crates, not the other way around.
