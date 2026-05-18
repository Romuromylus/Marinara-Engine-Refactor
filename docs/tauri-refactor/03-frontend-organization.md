# Frontend Organization

The frontend migration should preserve the current UI and visual structure. This is a code organization refactor first, not a redesign.

Do not rewrite the frontend wholesale. Move the existing React UI into clearer modules, make minor organization improvements, split obvious large files when ownership is clear, and preserve existing user workflows. Larger frontend rewrites or redesigns require explicit human approval for that specific slice.

## UI-First Migration Rule

The first copied UI pass does not need to be functional. Navigation and render paths can be kept only where they naturally survive the move, but do not add temporary behavior to make backend-dependent buttons, forms, saves, imports, generation, provider tests, or integrations appear to work.

Do not create fake Tauri commands, mock stores, compatibility adapters, or temporary local persistence unless they are part of the final architecture. If code needs the old server to work, isolate it for later replacement or leave that action non-functional until the matching Rust backend exists.

## Source Inventory Rule

Every frontend migration slice must update the source inventory status. The inventory must account for original UI components, hooks, stores, frontend lib files, styles, and public assets from `E:/Personal Projects/Marinara-Engine/packages/client`.

A source item may be marked as moved, mapped to a new module, deferred to a later reviewed slice, or intentionally removed only with explicit human approval. Do not silently drop source UI files during migration.

## Top-Level Folders

```text
src/
  app/
  shared/
  features/
```

## `src/app`

```text
app/
  main.tsx
  App.tsx
  providers/
    QueryProvider.tsx
    ThemeProvider.tsx
    TauriEventProvider.tsx
    ToastProvider.tsx
  shell/
    AppShell.tsx
    TopBar.tsx
    ChatSidebar.tsx
    RightPanel.tsx
    ModalRenderer.tsx
  startup/
    version-check.ts
    pwa-registration.ts
    font-loader.ts
    range-slider-sync.ts
```

Responsibilities:

- app bootstrap
- global providers
- shell layout
- global side effects
- version/update notifications

## `src/shared`

```text
shared/
  api/
    client.ts
    commands.ts
    events.ts
    bindings.ts
    errors.ts
  components/
    ui/
    overlays/
    menus/
    forms/
    media/
  hooks/
    useTauriEvent.ts
    useDebouncedValue.ts
    usePageActivity.ts
    useIdleDetection.ts
  lib/
    markdown.tsx
    utils.ts
    browser-runtime.ts
    csrf-compat.ts
    file-download.ts
    image-preview.ts
  stores/
    ui/
      layout.store.ts
      preferences.store.ts
      modals.store.ts
      panels.store.ts
    dialog.store.ts
  styles/
    globals.css
    themes/
  types/
    generated.ts
```

Responsibilities:

- Tauri command wrapper
- generated Rust DTO bindings
- shared UI primitives
- mode-agnostic hooks
- layout and preference stores

During the UI-only phase, `shared/api` should contain only final-shape APIs that are backed by real Rust commands or generated bindings. It should not become a temporary clone of the old HTTP server API.

## Feature Folders

Each feature should use this internal shape when useful:

```text
features/<feature>/
  api/
  components/
  hooks/
  lib/
  stores/
  types.ts
  index.ts
```

Do not force every folder to contain every subfolder. Add files only when the feature needs them.

## Feature Map

### `features/chat`

Current source: `components/chat`, `hooks/use-chats.ts`, `stores/chat.store.ts`.

```text
chat/
  api/chat.api.ts
  components/
    ChatArea.tsx
    ChatMessage.tsx
    ChatInput.tsx
    ChatGalleryDrawer.tsx
    ChatFilesDrawer.tsx
    ChatBranchSelector.tsx
    MessageActions.tsx
    SummaryPopover.tsx
  hooks/
    useChats.ts
    useChatFolders.ts
    useMessageBranches.ts
  stores/
    chat.store.ts
  lib/
    chat-display.ts
    chat-macros.ts
    dialogue-quotes.ts
```

### `features/conversation`

```text
conversation/
  components/
    ConversationView.tsx
    ConversationMessage.tsx
    ConversationInput.tsx
    ConversationSurface.tsx
    ConversationSettingsPanel.tsx
  hooks/
    useAutonomousMessaging.ts
    useBackgroundAutonomous.ts
    useConversationSchedules.ts
  lib/
    conversation-transcript.ts
```

### `features/roleplay`

```text
roleplay/
  components/
    RoleplaySurface.tsx
    RoleplayHUD.tsx
    RoleplayHUDActionsMenu.tsx
    RoleplayHUDPanels.tsx
    SpriteOverlay.tsx
    SpriteSidebar.tsx
    SceneBanner.tsx
    CyoaChoices.tsx
    ExpressionPanel.tsx
    EncounterModal.tsx
    WeatherEffects.tsx
  hooks/
    useScene.ts
    useSceneAnalysis.ts
    useEncounter.ts
  stores/
    encounter.store.ts
  lib/
    game-tag-parser.ts
    game-character-name-match.ts
```

### `features/game`

Current large files must be split.

```text
game/
  components/
    GameSurface.tsx
    GameInput.tsx
    GameNarration.tsx
    GameCombatUI.tsx
    GameChoiceCards.tsx
    GameCheckpoints.tsx
    GameCharacterSheet.tsx
    GameSessionHistory.tsx
    panels/
      InventoryPanel.tsx
      JournalPanel.tsx
      MapPanel.tsx
      WeatherPanel.tsx
      PartyPanel.tsx
      LogPanel.tsx
  hooks/
    useGame.ts
    useGameTurn.ts
    useGameAssets.ts
    useGameCombat.ts
    useGameSession.ts
    usePartyTurn.ts
  stores/
    game-mode.store.ts
    game-state.store.ts
    game-asset.store.ts
  lib/
    game-audio.ts
    game-segment-edits.ts
    game-full-body-pose.ts
    party-dialogue-parser.ts
```

### `features/characters` and `features/personas`

```text
characters/
  components/
    CharacterEditor.tsx
    CharacterLibraryView.tsx
    CharacterCardUpdateModal.tsx
  hooks/
    useCharacters.ts
  lib/
    character-display.ts
    character-import.ts
    png-parser.ts

personas/
  components/
    PersonaEditor.tsx
    PersonaMakerModal.tsx
  hooks/
    usePersonas.ts
```

### `features/lorebooks`, `features/prompts`, `features/presets`

```text
lorebooks/
  components/
    LorebookEditor.tsx
    LorebookEntryRow.tsx
    LorebookFormFields.tsx
  hooks/
    useLorebooks.ts
  lib/
    token-estimate.ts

prompts/
  components/
    PromptEditor.tsx
    PeekPromptModal.tsx
    PromptOverrideEditor.tsx
  hooks/
    usePrompts.ts
    usePromptOverrides.ts

presets/
  components/
    PresetEditor.tsx
    ChoiceSelectionModal.tsx
  hooks/
    usePresets.ts
    useChatPresets.ts
```

### `features/connections`

```text
connections/
  components/
    ConnectionEditor.tsx
    CreateConnectionModal.tsx
    ConnectionTestPanel.tsx
  hooks/
    useConnections.ts
  lib/
    provider-display.ts
```

### `features/agents`

```text
agents/
  components/
    AgentEditor.tsx
    AgentDebugPanel.tsx
    AgentThoughtBubbles.tsx
    ContextInjectionPanel.tsx
    RegexScriptEditor.tsx
    SecretPlotPanel.tsx
    ToolEditor.tsx
  hooks/
    useAgents.ts
    useCustomTools.ts
    useRegexScripts.ts
  stores/
    agent.store.ts
  lib/
    agent-cadence.ts
```

### `features/assets` and `features/gallery`

```text
assets/
  components/
    BackgroundManager.tsx
    AvatarUploader.tsx
    SpriteGenerationModal.tsx
    SpriteFrameEditor.tsx
    SpriteWandCleanupEditor.tsx
  hooks/
    useGameAssets.ts
  lib/
    asset-fuzzy-match.ts
    avatar-color-extraction.ts

gallery/
  components/
    ChatGallery.tsx
    GalleryDrawer.tsx
    PinnedImageOverlay.tsx
  hooks/
    useGallery.ts
  stores/
    gallery.store.ts
```

### `features/sidecar`

```text
Sidecar is deferred by scope and has no active frontend feature surface in the Tauri app.
```

### `features/sync`

```text
Sync is deferred by scope and has no active frontend feature surface in the Tauri app.
```

Do not keep sync settings, pairing, queue, or conflict review UI in the active app until sync is intentionally reopened.

### `features/spotify`

```text
spotify/
  components/
    SpotifyMiniPlayer.tsx
    SpotifyConnectionPanel.tsx
  hooks/
    useSpotify.ts
  lib/
    spotify-sdk-loader.ts
```

Frontend Spotify code should only control UI and playback SDK display. OAuth token exchange and refresh stay in Rust.

### `features/haptic`

```text
haptic/
  components/
    HapticSettingsPanel.tsx
    HapticDeviceStatus.tsx
  hooks/
    useHaptic.ts
```

Frontend haptic code should not directly talk to devices. It should call Rust commands and subscribe to device status events.

### `features/tts` and `features/translation`

```text
tts/
  hooks/
    useTts.ts
  lib/
    tts-audio-cache.ts
    tts-dialogue.ts
    tts-service.ts

translation/
  hooks/
    useTranslate.ts
  stores/
    translation.store.ts
  lib/
    draft-translation.ts
```

Provider calls stay in Rust. Audio playback can remain frontend-side.

### `features/bot-browser`

```text
bot-browser/
  components/
    BotBrowserView.tsx
    BotBrowserModal.tsx
  hooks/
    useBotBrowser.ts
```

Frontend displays search/import UI. Rust owns authenticated tokens, proxying, and provider-specific downloads.

### `features/imports`

```text
imports/
  components/
    ImportCharacterModal.tsx
    ImportLorebookModal.tsx
    ImportPersonaModal.tsx
    ImportPresetModal.tsx
    STBulkImportModal.tsx
  hooks/
    useImports.ts
```

Folder scanning and local path access stay in Rust.

### `features/settings`

```text
settings/
  components/
    SettingsPanel.tsx
    ChatSettingsDrawer.tsx
    GenerationParametersEditor.tsx
    ExportFormatDialog.tsx
```

Split settings by section as the migration progresses. Do not keep all settings in one huge component.

### `features/themes`, `features/extensions`, `features/updates`

```text
themes/
  components/
    ThemeEditor.tsx
    CustomThemeInjector.tsx
  hooks/
    useThemes.ts

extensions/
  components/
    ExtensionManager.tsx
  hooks/
    useExtensions.ts

updates/
  components/
    UpdateNotice.tsx
  hooks/
    useUpdates.ts
```

Extension execution must be reviewed carefully. User-provided JS is a security boundary.

## Import Rules

1. `features/*` can import from `shared/*`.
2. `features/*` should not import from sibling features unless using a public `index.ts`.
3. Shared UI should not import feature code.
4. App shell can compose features, but should not own feature logic.
5. Use generated TypeScript bindings from Rust domain DTOs instead of hand-written duplicate DTOs.
6. React hooks must not make provider/authenticated network calls directly. Calls that use secrets, cookies, OAuth, provider auth, or local URL policy go through Rust commands.
