# Layered Module Architecture

This document refines the TypeScript/Rust split into explicit dependency layers. The goal is to make the codebase readable by folder structure alone: the three top-level product modes, chat, roleplay, and game, sit at the top; shared mechanics and capabilities sit below; sibling top-level modes do not reach into each other.

Sidecar and sync-client remain deferred external-service work.

## Core Principle

Marinara should be organized as a layered application, not as one large feature folder graph.

```text
UI app shell
  -> UI feature surfaces
    -> TypeScript mode engines
      -> TypeScript orchestration/building blocks
        -> TypeScript domain primitives/contracts
          -> Rust capability adapters
```

The strongest rule:

```text
chat, roleplay, and game are top-level sibling mode engines.
They may share lower-layer modules, but they must not import each other.
```

## Engine Layers

### Layer 0: Contracts And Primitives

Path:

```text
src/engine/contracts/
src/engine/core/
```

Owns:

- Domain types copied from `packages/shared/src/types`.
- Zod schemas copied from `packages/shared/src/schemas`.
- Constants copied from `packages/shared/src/constants`.
- Small primitives such as result helpers, clock abstraction, IDs, and stable enum guards.

May import:

- Nothing from higher engine layers.
- No React.
- No Tauri.
- No capability adapters.

May be imported by:

- Every TypeScript engine module.
- UI modules.
- `shared/api` adapters.

### Layer 1: Capability Ports

Path:

```text
src/engine/capabilities/
```

Owns TypeScript interfaces only:

- `storage.ts`
- `llm.ts`
- `assets.ts`
- `imports.ts`
- `integrations.ts`
- `events.ts`
- `permissions.ts`

Responsibilities:

- Define what the TypeScript engine needs from Rust.
- Keep Rust capability shapes narrow and stable.
- Avoid importing concrete Tauri adapters.

May import:

- `engine/contracts`
- `engine/core`

Must not import:

- `shared/api`
- React
- mode engines
- feature hooks/stores

### Layer 2: Pure Shared Domain Utilities

Path:

```text
src/engine/shared/
```

Suggested modules:

```text
src/engine/shared/
  text/
    xml-wrapper.ts
    markdown-sanitize.ts
    transcript.ts
  macros/
    macro-engine.ts
    chat-macros.ts
  parsing/
    jsonish.ts
    dialogue-quotes.ts
    party-dialogue-parser.ts
  regex/
    regex-safety.ts
    regex-replacement.ts
  scoring/
    music-score.ts
    agent-cost.ts
  media/
    asset-fuzzy-match.ts
    game-audio-settings.ts
```

Owns:

- Pure deterministic helpers.
- Shared parsers.
- Formatting/scoring helpers that are used by more than one domain.

May import:

- Layer 0 only.

Must not import:

- capability ports
- storage
- generation
- agents
- mode engines
- React

### Layer 3: Entity Domains

Path:

```text
src/engine/entities/
```

Suggested modules:

```text
src/engine/entities/
  agents/
    config.ts
    cadence.ts
    failures.ts
  characters/
    display.ts
    groups.ts
    sprites.ts
  personas/
    active-persona.ts
  chats/
    messages.ts
    swipes.ts
    branches.ts
    folders.ts
    metadata.ts
    summaries.ts
  connections/
    connection-filters.ts
    provider-metadata.ts
  lorebooks/
    models.ts
    folders.ts
  presets/
    choices.ts
  game-state/
    state-text.ts
    tracker.ts
```

Owns:

- Entity-specific pure operations.
- Normalization and selectors.
- Metadata interpretation.
- No orchestration.

May import:

- Layers 0-2.

Must not import:

- generation
- agents runtime
- mode engines
- `shared/api`
- React

### Layer 4: Capability-Backed Repositories

Path:

```text
src/engine/repositories/
```

Suggested modules:

```text
src/engine/repositories/
  chats.repository.ts
  characters.repository.ts
  personas.repository.ts
  lorebooks.repository.ts
  presets.repository.ts
  agents.repository.ts
  connections.repository.ts
  settings.repository.ts
  assets.repository.ts
```

Owns:

- Thin TypeScript repository wrappers over `StorageGateway`.
- Conversion between storage DTOs and engine DTOs when needed.
- Query composition that belongs to the engine, not UI.

May import:

- Layers 0-3.
- capability ports.

Must not import:

- React Query.
- Zustand.
- Tauri invoke directly.
- mode engines.

Note: this layer is optional for very small slices. If a service only needs one storage call, it may accept `StorageGateway` directly. Add a repository only when it removes duplication.

### Layer 5: Generation Building Blocks

Path:

```text
src/engine/generation-core/
```

Suggested modules:

```text
src/engine/generation-core/
  llm/
    messages.ts
    context-fit.ts
    usage.ts
    model-routing.ts
  prompt/
    assembler.ts
    macro-context.ts
    format-engine.ts
    marker-expander.ts
    merger.ts
    prompt-overrides.ts
  lorebooks/
    keyword-scanner.ts
    prompt-injector.ts
    game-scope.ts
  regex/
    regex-application.ts
  attachments/
    readable-attachments.ts
    image-attachments.ts
```

Owns:

- Prompt construction.
- Context fitting.
- Lorebook scanning and injection.
- Regex application.
- Attachment shaping.
- Provider-neutral LLM request DTOs.

May import:

- Layers 0-4.
- capability ports for optional helpers only.

Must not import:

- agents runtime.
- mode engines.
- React.

Important: `generation-core` is lower than `agents` and lower than top-level modes. It provides building blocks; it does not decide when a chat, roleplay, or game turn runs.

### Layer 6: Agents Runtime

Path:

```text
src/engine/agents-runtime/
```

Suggested modules:

```text
src/engine/agents-runtime/
  executor/
    agent-executor.ts
    agent-batch.ts
    result-parser.ts
  pipeline/
    agent-pipeline.ts
    phase-runner.ts
    context-builder.ts
  knowledge/
    knowledge-router.ts
    knowledge-retrieval.ts
    memory-recall.ts
  tools/
    definitions.ts
    pure-tools.ts
    capability-tools.ts
    tool-router.ts
    permissions.ts
  debug/
    debug-events.ts
    redaction.ts
```

Owns:

- Agent execution.
- Agent batching.
- Agent phase orchestration.
- Agent context shaping.
- Agent result parsing.
- Tool routing.
- Knowledge/memory orchestration.

May import:

- Layers 0-5.
- capability ports.

Must not import:

- chat mode.
- game mode.
- roleplay mode.
- React.
- `shared/api`.

Tool split:

- Pure tools stay here: dice draft, expression draft, chat summary read, state update draft.
- Capability tools call ports: webhook safe fetch, Spotify, file access, haptic, secrets.
- Script tools should be isolated behind an explicit permission gate.

### Layer 7: Generation Orchestration

Path:

```text
src/engine/generation/
```

Suggested modules:

```text
src/engine/generation/
  start-generation.ts
  retry-generation.ts
  dry-run.ts
  prompt-preview.ts
  stream-controller.ts
  persistence.ts
  generation-events.ts
  generation-input.ts
  generation-result.ts
```

Owns:

- Normal chat generation orchestration.
- Prompt preview.
- Dry run.
- Retry.
- Streaming run lifecycle.
- Calls to agents before/during/after generation.
- Persistence of generated messages through capability-backed repositories.

May import:

- Layers 0-6.
- capability ports.

Must not import:

- chat mode.
- game mode.
- roleplay mode.
- React.

Rationale: generation is a platform service for higher modes. Chat, roleplay, and game can call it, but generation does not know their UI or full mode orchestration.

### Layer 8: Top-Level Mode Engines

Path:

```text
src/engine/modes/
```

Top-level modes:

```text
src/engine/modes/
  chat/
  roleplay/
  game/
```

These are application modes. They are allowed to orchestrate lower layers.

#### `modes/chat`

Suggested modules:

```text
src/engine/modes/chat/
  core/
    send-message.ts
    regenerate-message.ts
    branch-chat.ts
    setup-chat.ts
    chat-settings.ts
    files.ts
    summaries.ts
  autonomous/
    autonomous-check.ts
    background-autonomous.ts
    activity.ts
  awareness/
    awareness.ts
    cross-chat-context.ts
  schedules/
    schedule.ts
    schedule-runner.ts
  commands/
    character-commands.ts
    direct-message-commands.ts
    impersonate-prompt.ts
```

May import:

- Layers 0-7.

Must not import:

- roleplay
- game

Conversation/autonomous behavior is a chat subsystem, not a fourth product mode. It lives under `modes/chat` because it reads and writes chat messages, schedules, summaries, awareness, and character commands as extensions of normal chat behavior.

#### `modes/roleplay`

Suggested modules:

```text
src/engine/modes/roleplay/
  scene/
    scene-service.ts
    scene-analysis.ts
    scene-postprocess.ts
  sprites/
    sprite-service.ts
    sprite-placement.ts
    sprite-generation.ts
  encounter/
    encounter-service.ts
    encounter-generation.ts
  visual-novel/
    choices.ts
    state.ts
```

May import:

- Layers 0-7.
- asset/import/integration capability ports.

Must not import:

- game

Shared scene/sprite utilities that game also needs should live below both modes, for example:

```text
src/engine/entities/characters/sprites.ts
src/engine/shared/media/
src/engine/generation-core/attachments/
```

#### `modes/game`

Suggested modules:

```text
src/engine/modes/game/
  turn/
    turn-orchestrator.ts
    turn-input.ts
    turn-output.ts
    party-turn.ts
  prompts/
    gm-prompts.ts
    party-prompts.ts
    templates.ts
  mechanics/
    dice.ts
    skill-check.ts
    combat.ts
    loot.ts
    morale.ts
    reputation.ts
    perception.ts
    element-reactions.ts
  state/
    state-machine.ts
    snapshots.ts
    checkpoints.ts
    repair.ts
    session.ts
    session-summary-normalization.ts
  world/
    map.ts
    map-position.ts
    weather.ts
    time.ts
    journal.ts
    travel.ts
  assets/
    asset-manifest.ts
    game-asset-generation.ts
    music-selection.ts
    npc-avatar-utils.ts
```

May import:

- Layers 0-7.
- capability ports.

Must not import:

- roleplay
- UI chat or roleplay components.

Shared visual-novel/sprite/scene concepts should be pulled down into lower modules rather than imported from `modes/roleplay`.

## Engine Import Matrix

| From | May Import | Must Not Import |
| --- | --- | --- |
| `engine/contracts` | nothing higher | capabilities, repositories, generation, agents, modes, UI |
| `engine/core` | contracts | capabilities, repositories, generation, agents, modes, UI |
| `engine/capabilities` | contracts, core | shared/api, generation, agents, modes, UI |
| `engine/shared` | contracts, core | capabilities, repositories, generation, agents, modes, UI |
| `engine/entities` | contracts, core, shared | repositories, generation, agents, modes, UI |
| `engine/repositories` | contracts, core, shared, entities, capabilities | generation, agents, modes, UI |
| `engine/generation-core` | contracts, core, shared, entities, repositories, capabilities | agents-runtime, generation, modes, UI |
| `engine/agents-runtime` | layers 0-5, capabilities | generation orchestration, chat, game, roleplay, UI |
| `engine/generation` | layers 0-6, capabilities | chat, game, roleplay, UI |
| `engine/modes/chat` | layers 0-7 | roleplay, game, UI |
| `engine/modes/roleplay` | layers 0-7 | chat, game, UI |
| `engine/modes/game` | layers 0-7 | chat, roleplay, UI |

## UI Layering

The UI should also be split by layer. The current copied UI has a lot of cross-feature imports because the original app grew around shared component folders. The refactor should make the UI boundaries visible.

```text
src/app/
  App.tsx
  main.tsx
  shell/
  providers/
  startup/

src/shared/
  components/
  hooks/
  lib/
  stores/
  api/

src/features/
  chat/
  roleplay/
  game/
  agents/
  characters/
  personas/
  lorebooks/
  presets/
  connections/
  settings/
  assets/
  imports/
  integrations/
  tracker/
```

### UI Layer 0: Shared UI Kit

Path:

```text
src/shared/components/
```

Owns:

- Generic modals, context menus, color picker, tooltips, textareas, number inputs.
- No feature-specific business rules.
- No engine orchestration.

Must not import:

- `features/*`
- `engine/modes/*`

### UI Layer 1: Feature Primitives

Each feature can have:

```text
src/features/<feature>/
  components/
  hooks/
  stores/
  lib/
  public.ts
```

Rules:

- Feature internals import their own feature files freely.
- Other features may import only from `src/features/<feature>/public.ts`.
- Avoid deep imports such as `features/chats/components/SpriteOverlay`.
- If two features need the same component, either expose it intentionally through `public.ts` or move it to `shared/components` or a lower feature primitive package.

### UI Layer 2: Top-Level Mode Surfaces

Top-level UI surfaces should mirror top-level engine modes:

```text
src/features/chat/
  screens/ChatScreen.tsx
  components/messages/
  components/input/
  components/branches/
  components/autonomous/
  components/schedules/
  components/awareness/
  components/summaries/
  hooks/
  public.ts

src/features/roleplay/
  screens/RoleplayScreen.tsx
  components/scene/
  components/sprites/
  components/encounter/
  components/visual-novel/
  hooks/
  public.ts

src/features/game/
  screens/GameScreen.tsx
  components/turn/
  components/mechanics/
  components/world/
  components/party/
  components/session/
  components/assets/
  hooks/
  stores/
  public.ts
```

Mode UI rule:

```text
chat UI, roleplay UI, and game UI do not deep-import each other.
```

If game needs message rendering, it imports a stable chat primitive from:

```text
src/features/chat/public.ts
```

If roleplay and game both need sprite display, that component should move to:

```text
src/features/visuals/
  sprites/
  weather/
  public.ts
```

or to `src/shared/components` if it is truly generic.

### UI Layer 3: App Shell

Path:

```text
src/app/shell/
```

Owns:

- Top bar.
- Main shell layout.
- Right panel routing.
- Modal root.
- Sidebar placement.

May import:

- Feature public entrypoints.
- Shared UI.
- Shared stores.

Must not import:

- Engine internals directly except boot-level providers.
- Deep feature components.

## Proposed UI Reorganization From Current Files

Current `src/features/chats/components` contains normal chat, chat autonomous/conversation behavior, roleplay, shared sprite/weather, and several game support pieces. Split it as follows:

| Current area | New home | Notes |
| --- | --- | --- |
| `ChatArea`, `ChatConversationSurface`, `ConversationView`, `ConversationMessage`, `ConversationInput`, `ChatMessage`, `ChatInput`, `ChatBranchSelector`, `SwipeJumpControl` | `features/chat/components/*` | Normal chat primitives and normal chat screen. |
| `ConversationAutonomousEffects`, autonomous notifications, summary popovers/editors when chat-autonomous-specific | `features/chat/components/autonomous/*` or `features/chat/components/summaries/*` | Autonomous conversation behavior is part of chat, not a separate product mode. |
| `ChatRoleplaySurface`, `RoleplayHUD`, `RoleplayHUDPanels`, `RoleplayHUDActionsMenu`, `SceneBanner`, `CyoaChoices`, `EncounterModal`, `EchoChamberPanel` | `features/roleplay/components/*` | Roleplay-specific UI. |
| `SpriteOverlay`, `SpriteSidebar`, `ExpressionPanel`, `WeatherEffects`, `PinnedImageOverlay` | `features/visuals/components/*` or `features/roleplay/public.ts` | Use a shared visuals feature if game and roleplay both need them. |
| `ChatGallery`, `ChatGalleryDrawer`, `ChatFilesDrawer` | `features/gallery` or `features/chat/components/files` | Gallery/file UI should call assets capability. |
| `ActiveWorldInfoButton`, `ImagePromptPanel`, `GenerationReplayDetailsModal`, `PeekPromptModal` | place by owner: chat/generation/game | Avoid a dumping-ground chat folder. |

Current `src/features/game/components` should be split internally:

```text
src/features/game/components/
  screen/
    GameScreen.tsx
    GameSurface.tsx
  turn/
    GameInput.tsx
    GameNarration.tsx
    GameReadableDisplay.tsx
    GameTransitionManager.tsx
  mechanics/
    GameCombatUI.tsx
    GameDiceResult.tsx
    GameSkillCheckResult.tsx
    GameElementReaction.tsx
    GameQteOverlay.tsx
  world/
    GameMap.tsx
    GameGridMap.tsx
    GameNodeMap.tsx
    GameTravelView.tsx
    GameJournal.tsx
    GameInventory.tsx
  party/
    GamePartyBar.tsx
    GamePartySidebar.tsx
    GameCharacterSheet.tsx
    GameNpcTracker.tsx
    GameDialogueOverlay.tsx
  session/
    GameSetupWizard.tsx
    GameSessionBanner.tsx
    GameSessionHistory.tsx
    GameCheckpoints.tsx
    GameJsonRepairModal.tsx
  assets/
    GameImagePromptReviewModal.tsx
    game-asset-generation-payload.ts
  layout/
    DraggablePanel.tsx
    DirectionEngine.tsx
    AnimatedText.tsx
```

Current agents UI should stay UI-only:

```text
src/features/agents/
  components/
    editor/
    tools/
    debug/
    thought-bubbles/
  hooks/
  public.ts
```

Agent runtime code should not live in `features/agents`; it belongs in:

```text
src/engine/agents-runtime/
```

## Rust Capability Layering

Rust should also be layered, but much smaller than the TypeScript engine.

```text
src-tauri/src/
  lib.rs
  app.rs
  state.rs
  commands/
  events/

src-tauri/crates/
  core/
  security/
  storage/
  llm/
  assets/
  import/
  integrations/
  updates/
```

Allowed Rust crate direction:

```text
commands -> capability crates
storage -> core
security -> core
llm -> security, core
assets -> storage, security, core
import -> storage, security, assets, core
integrations -> security, storage, core
updates -> security, core
```

Rust must not import TypeScript concepts like agent phases, game turns, prompt sections, or roleplay scenes except as opaque DTO payloads needed to perform a capability.

## Original Source Layer Mapping

| Original source area | New layer |
| --- | --- |
| `packages/shared/src/types`, `schemas`, `constants` | `engine/contracts` |
| `packages/shared/src/utils` | `engine/shared` or domain-specific lower module |
| `packages/server/src/services/prompt` | `engine/generation-core/prompt` |
| `packages/server/src/services/lorebook` | `engine/generation-core/lorebooks`, except embeddings/capability pieces |
| `packages/server/src/services/regex` | `engine/generation-core/regex` |
| `packages/server/src/services/agents` | `engine/agents-runtime` |
| `packages/server/src/services/tools` | split between `engine/agents-runtime/tools` and Rust capabilities |
| `packages/server/src/routes/generate/*` | `engine/generation` plus `shared/api` event adapters |
| `packages/server/src/services/conversation` | `engine/modes/chat/autonomous`, `engine/modes/chat/awareness`, `engine/modes/chat/schedules`, and `engine/modes/chat/commands` |
| `packages/server/src/services/game` | `engine/modes/game` with pure lower utilities pulled into `engine/shared` if reused |
| `packages/server/src/services/llm` | split between `engine/generation-core/llm` and Rust `llm` transport |
| `packages/server/src/services/storage`, `db`, `utils/security`, `middleware` | Rust capability crates |
| `packages/client/src/components/chat` | split into `features/chat`, `features/roleplay`, `features/visuals`, `features/gallery`; autonomous conversation UI stays under `features/chat` |
| `packages/client/src/components/game` | `features/game` internal subfolders matching game UI layers |
| `packages/client/src/hooks` | feature hooks that call mode engines and capability adapters |
| `packages/client/src/stores` | shared/feature UI stores only; durable state should not live only in stores |

## Enforcement Plan

Use structure first, then tooling.

1. Add `public.ts` barrels to every feature.
2. Ban deep feature imports across feature boundaries.
3. Add ESLint `no-restricted-imports` rules once folders exist.
4. Add a small dependency check script that rejects forbidden imports:
   - `engine/modes/game` importing `engine/modes/roleplay`
   - `engine/modes/roleplay` importing `engine/modes/chat`
   - `engine/modes/game` importing `engine/modes/chat`
   - any `engine/*` importing `src/features/*`
   - any `engine/*` importing `@tauri-apps/api`
   - any `engine/*` importing React
   - any `features/*` deep-importing another feature without `public.ts`
5. Keep `docs/tauri-refactor/13-typescript-rust-organization-plan.md` as the exhaustive inventory, and use this document as the architectural dependency rulebook.

## Migration Order

1. Create `engine/contracts`, `engine/core`, `engine/capabilities`, and `engine/shared`.
2. Move `legacy-shared` into `engine/contracts`.
3. Move pure shared utilities into `engine/shared`.
4. Create feature `public.ts` barrels without moving UI yet.
5. Split `features/chats/components` into `features/chat`, `features/roleplay`, `features/visuals`, and `features/gallery`; place autonomous conversation UI under `features/chat`.
6. Split `features/game/components` into the internal game UI subfolders.
7. Move prompt/lorebook/regex into `engine/generation-core`.
8. Move agents into `engine/agents-runtime`.
9. Move generation orchestration into `engine/generation`.
10. Move chat/autonomous, roleplay, and game service logic into `engine/modes/*`.
11. Add dependency enforcement.
12. Implement Rust capabilities slice by slice behind the stable ports.
