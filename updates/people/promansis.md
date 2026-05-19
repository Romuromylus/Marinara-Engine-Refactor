# Promansis

## Current Work

No current work listed.




## Owned Bugs

## Suggested Fix Order

1. **Filesystem safety and data loss risks**: symlink escapes, profile import stale assets, tracker edits lost on reload.
2. **Storage contract and transcript correctness**: character `data` shape. Chat pagination and regenerate swipe contract are fixed in the current codebase.
3. **Mode/session continuity**: game session carryover, concluded roleplay scene send guard, checkpoint visibility.
4. **Native integration correctness**: haptic agent execution, haptic action contract, stop-generation cancellation, translation errors.
5. **UI polish and accessibility**: onboarding focus trap, mobile drawer tab order, default/random-pool badge state.

## Completion Order

1. Game asset operations can follow symlinks outside the managed asset root.
2. Profile import leaves stale asset files behind.
3. Reloading immediately after tracker edits can lose pending world state.
4. Saving a character can persist card data in the wrong shape.
5. Stop generation does not cancel the provider stream command.
6. Starting a new game session drops carried inventory and player state.
7. Stored generation replay metadata is not applied on replay/regenerate.
8. Love Toys Control agent results never reach the haptic integration.
9. Haptic inflate actions are advertised but execute as vibrate or fail.
10. Concluded roleplay scenes remain editable when reopened.
11. Game checkpoint manager is not reachable from the game surface.
12. Marinara ZIP imports ignore file timestamps on restore.
13. Replacing avatars and lorebook images leaves old managed files behind.
14. New-chat setup flags can open the wizard on the wrong chat after a quick switch.
15. Mobile shell panels leave hidden content in the tab order.
16. First-launch tutorial does not trap keyboard focus.
17. Connection random-pool toggle can invert stored boolean state.
18. Prompt preset default badge does not recognize boolean defaults.
19. Message translation failures leave no visible error.
20. Spotify setup tells users to set a redirect URI env var that native auth ignores.

## App Shell And Accessibility

### First-launch tutorial does not trap keyboard focus

- Status: Resolved
- Impact area: UI
- Risk: Medium, accessibility regression on first launch.
- Likely owner: `src/app/shell/AppShell.tsx`, `src/features/onboarding/components/OnboardingTutorial.tsx`
- Summary: The onboarding tutorial appears as an overlay, but pressing `Tab` can move focus to shell/window controls behind it. The tutorial needs modal-style focus containment until skipped or completed.

### Mobile shell panels leave hidden content in the tab order

- Status: Resolved
- Impact area: UI
- Risk: Medium, mobile accessibility and keyboard navigation.
- Likely owner: `src/app/shell/AppShell.tsx`
- Summary: At mobile width, closed sidebars and underlying page controls remain tabbable while another drawer is open. The active drawer needs focus containment, closed panels need inert handling, and `Escape` should dismiss the open mobile panel.

## Library Objects And Configuration

### Saving a character can persist card data in the wrong shape

- Status: Resolved
- Impact area: UI | shared/api | Rust capability | engine
- Risk: Medium-high, persisted character records can become unreadable by strict consumers.
- Likely owner: `src/features/characters/hooks/use-characters.ts` or the storage update boundary for character records.
- Summary: `CharacterEditor` sends `data` as an object, while several consumers still expect persisted `characters[].data` to be a JSON string. The fix should normalize at the lowest correct owner and review roleplay scene memory updates that also write character data.

### Prompt preset default badge does not recognize boolean defaults

- Status: Resolved
- Impact area: UI
- Risk: Low-medium, configuration appears wrong after save/reload.
- Likely owner: `src/features/presets/components/PresetsPanel.tsx`
- Summary: Preset updates write `isDefault: true`, but the list only treats `isDefault === "true"` as default. Rendering should accept both legacy string values and current boolean values through a shared boolish reader.

### Connection random-pool toggle can invert stored boolean state

- Status: Resolved
- Impact area: UI | engine
- Risk: Medium, UI and random connection resolution can disagree.
- Likely owner: `src/features/connections`, `src/features/chats/components/QuickConnectionSwitcher.tsx`, `src/features/chats/components/QuickSwitcherMobile.tsx`, random connection resolvers.
- Summary: Some UI paths only treat `useForRandom === "true"` as in-pool, while schema/update paths use booleans and engine resolvers require boolean `true`. Read boundaries should normalize the flag and new writes should avoid mixed string/boolean semantics.

## Chat Transcript And Regeneration

### Chat history Load More repeats the first page instead of older messages

- Status: Fixed in codebase, kept for audit
- Impact area: UI | shared/api | Rust capability
- Risk: Medium-high, transcript pagination and `/goto` cannot reach older messages.
- Likely owner: Rust `storage_list` pagination for `messages`, or a dedicated chat-message list command with a real cursor contract.
- Summary: `useChatMessages` sends a `before` cursor, but Rust `storage_list` ignores it and returns the same limited page repeatedly. The fix needs a real cursor contract for message lists without disturbing unpaginated prompt/export paths. Current Rust `storage_list` applies message pagination with a parsed `before` cursor before limiting results.

### Regenerate creates hidden swipes that cannot be selected or read as active content

- Status: Fixed in codebase, kept for audit
- Impact area: UI | shared/api | Rust capability | engine
- Risk: High, stored regeneration output can be invisible and excluded from active transcript context.
- Likely owner: Chat message storage contract for swipes, with UI renderers and prompt assembly reviewed as dependent callers.
- Summary: `chat_message_add_swipe` appends to `swipes` and updates `activeSwipeIndex`, but does not update `content` or `swipeCount`. Renderers gate controls on `swipeCount` and display `content`, so regenerated swipes are usually hidden. The product contract needs to decide whether active swipe content is denormalized onto the row or derived by readers. Current swipe writes update `activeSwipeIndex`, `swipeCount`, and denormalized active `content`; message reads also materialize swipe fields.

### Regenerate streaming state is never attached to the target message

- Status: Fixed in codebase, kept for audit
- Impact area: UI | engine
- Risk: Medium, regenerate UX shows the old message while new tokens stream elsewhere.
- Likely owner: `src/features/modes/components/ModeSurface.tsx` or `src/features/generation/hooks/use-generate.ts`
- Summary: Regenerate passes `regenerateMessageId` to the generation request but never stores it in `useChatStore`. The renderer therefore cannot attach streaming output to the target message and falls back to generic typing state. The target should be set before generation and cleared in `finally`. Current generation UI stores `regenerateMessageId` before streaming and clears it in `finally`.

## Roleplay Lifecycle

### Concluded roleplay scenes remain editable when reopened

- Status: Resolved
- Impact area: UI | engine
- Risk: Medium, concluded scene archives can diverge from their final summaries and memory writes.
- Likely owner: `src/features/roleplay/components/ChatRoleplaySurface.tsx` plus an authoritative generation/send guard.
- Summary: Concluding a scene marks `sceneStatus: "concluded"`, but reopened scene chats still render normal input and the generation path accepts sends. Concluded scenes should be read-only unless explicitly converted or reopened.

## Game Mode

### Starting a new game session drops carried inventory and player state

- Status: Resolved
- Impact area: UI | engine | storage
- Risk: High, continuity-critical campaign state can disappear between sessions.
- Likely owner: `src/features/game/api/game-api.ts`, with dependent readers in `src/features/game/components/GameSurface.tsx`
- Summary: `gameApi.startSession` carries over only selected metadata and omits inventory, widget state, time, weather, morale, notes, and `chat.gameState`. The next session therefore hydrates with missing inventory/player stats. The fix should define the authoritative carryover fields and avoid copying stale combat-only state.

### Game checkpoint manager is not reachable from the game surface

- Status: Resolved
- Impact area: UI | engine
- Risk: Medium, implemented checkpoint/repair capability has no visible entry point.
- Likely owner: `src/features/game/components/GameSurface.tsx` or the game toolbar/panel that owns utility overlays.
- Summary: `GameCheckpoints` and checkpoint hooks/API exist, but no visible game surface imports or renders the manager. Exposing it should also verify load restores chat detail, messages, metadata-backed game store, and `useGameStateStore`.

## Generation, Prompts, Agents, And Provider Boundary

### Stored generation replay metadata is not applied on replay/regenerate

- Status: Resolved
- Impact area: Generation | prompts | agents | provider boundary
- Risk: Medium-high, replay can silently use different guidance, agents, connection, or preset from the stored metadata.
- Likely owner: `src/engine/generation/start-generation.ts`, with caller context from `src/features/modes/components/ModeSurface.tsx`
- Summary: User-message regenerate is enabled when `extra.generationReplay` exists, and `applyGenerationReplayToRegenerateInput` exists, but no generation path calls it. Regenerate sends only basic request fields unless new typed guidance is present.

### Stop generation does not cancel the provider stream command

- Status: Resolved
- Impact area: Generation | prompts | agents | provider boundary
- Risk: Medium-high, provider work and billing can continue after the UI says generation stopped.
- Likely owner: `src/shared/api/llm-api.ts`, `src-tauri/src/commands/storage/llm.rs`, and the `marinara_llm` streaming capability.
- Summary: The frontend abort signal makes `llmApi.stream` throw locally, but `llm_stream_channel` has no request id or cancellation token. Rust provider streaming continues until completion or channel send failure. The stream command needs native cancellation semantics.

## Storage, Repositories, Imports, And Exports

### Marinara ZIP imports ignore file timestamps on restore

- Status: Resolved
- Impact area: UI | shared/api | Rust capability
- Risk: Medium, imported library ordering and timestamps are wrong.
- Likely owner: `src/shared/api/import-api.ts`, character/persona import callers, and settings import callers.
- Summary: Character and persona import modals build `FormData` containing `timestampOverrides`, but call `importApi.marinaraFile(file)` with only the bare file. Rust never receives overrides and falls back to archive or current timestamps.

### Profile import leaves stale asset files behind

- Status: Resolved
- Impact area: UI | Rust capability
- Risk: Medium-high, profile restore can show files that are no longer in the restored profile.
- Likely owner: `src-tauri/src/commands/storage/profile.rs`
- Summary: `import_profile_collections` replaces storage collections, but `restore_profile_assets` only writes exported files and does not clear existing asset directories first. Asset browsers enumerate disk directly, so stale backgrounds, fonts, sprites, and knowledge-source files remain visible.

### Reloading immediately after tracker edits can lose pending world state

- Status: Resolved
- Impact area: UI | shared/api | storage
- Risk: Medium-high, accepted tracker edits can be lost on quick reload/close.
- Likely owner: `src/features/world-state/hooks/use-world-state-patcher.ts`, with support from `src/features/world-state/api/world-state-api.ts` or lower storage capability.
- Summary: Tracker writes are debounced. On `beforeunload`, pending patches are removed from memory before an unawaited async Tauri storage patch completes, and the `keepalive` option has no effect on `invokeTauri`. The queue should not be discarded until durable write success is guaranteed.

## Managed Assets And Filesystem Safety

### Game asset operations can follow symlinks outside the managed asset root

- Status: Resolved
- Impact area: shared/api | Rust capability
- Risk: High, managed asset commands can escape the intended root through symlinks.
- Likely owner: `src-tauri/crates/assets/src/lib.rs`, `src-tauri/crates/security/src/lib.rs`
- Summary: `AssetService::absolute_path` rejects absolute paths and `..`, then joins with the asset root, but it does not canonicalize and re-check symlink-resolved paths. Reads, writes, moves, deletes, copies, file info, and returned absolute paths can target files outside `game-assets` if a symlink is present.

### Replacing avatars and lorebook images leaves old managed files behind

- Status: Resolved
- Impact area: UI | Rust capability
- Risk: Medium, repeated replacements orphan managed media files.
- Likely owner: `src-tauri/src/commands/storage/media_uploads.rs`, `avatars.rs`, `lorebook_images.rs`, and relevant record delete paths.
- Summary: Avatar and lorebook image replacements always write a new unique file and patch the record, but never remove the previous managed file. Generic record deletion also does not clean up owned media. Cleanup should only delete files inside managed media folders, not external/user-provided URLs.

## Native Integrations

### Love Toys Control agent results never reach the haptic integration

- Status: Resolved
- Impact area: UI | engine | shared/api | Rust capability
- Risk: Medium-high, a built-in agent can report success without controlling connected devices.
- Likely owner: `src/features/generation/hooks/use-generate.ts`, `src/engine/generation/agent-runner.ts`, `src/shared/api/integration-gateway.ts`
- Summary: The haptic agent can produce `haptic_command` results, but `applyAgentResultEffects` does not execute them through `integrationGateway.haptic.command`. Native haptic commands are only reached through connected-command syntax, not the dedicated agent result.

### Haptic inflate actions are advertised but execute as vibrate or fail

- Status: Resolved
- Impact area: engine | shared/api | Rust capability
- Risk: Medium, prompt/schema/parser/native contracts disagree.
- Likely owner: `src/engine/contracts/constants/agent-prompts.ts`, `src/engine/contracts/types/haptic.ts`, `src/engine/modes/chat/commands/character-commands.ts`, `src-tauri/src/commands/storage/integrations/haptic.rs`
- Summary: The prompt and TypeScript type advertise `inflate`, but the connected-command parser does not allow it and Rust normalization rejects it. The product contract should either implement inflation through the correct native output type or remove it from advertised actions.

### Spotify setup tells users to set a redirect URI env var that native auth ignores

- Status: Resolved
- Impact area: UI | Rust capability
- Risk: Medium, setup instructions are misleading for non-loopback/remote auth.
- Likely owner: `src/features/agents/components/AgentEditor.tsx`, `src-tauri/src/commands/storage/integrations/spotify.rs`
- Summary: UI copy references `SPOTIFY_REDIRECT_URI`, but both frontend display and Rust OAuth use the hardcoded loopback callback. Either native auth should honor the configured redirect URI or the setup copy should stop advertising an unsupported override.

### Message translation failures leave no visible error

- Status: Resolved
- Impact area: UI | shared/api | Rust capability
- Risk: Low-medium, provider/auth/network failures can become silent unhandled promise rejections.
- Likely owner: `src/shared/hooks/use-translate.ts` and message action call sites in `src/features/chats/components`
- Summary: `useTranslate.translate` resets state in `finally` but does not catch rejected native calls or show a toast. Message action buttons call it without awaiting or catching. Message translation should surface errors consistently with draft translation while preserving successful hide/show behavior.

## Cross-Domain Async State

### New-chat setup flags can open the wizard on the wrong chat after a quick switch

- Status: Resolved
- Impact area: UI | shared state
- Risk: Medium, setup UI can attach to the wrong active chat after async creation work.
- Likely owner: `src/shared/stores/chat.store.ts`, `src/features/characters/hooks/use-start-chat-from-character.ts`, `src/features/chats/components/NewChatConnectionGate.tsx`, `src/features/modes/components/ModeSurface.tsx`
- Summary: New-chat flows store setup intent in global booleans, then `ModeSurface` consumes them against the current `activeChatId`. If the user switches chats while preset/greeting work is still running, the wizard/settings drawer can open on an unrelated chat. The intent should carry a target chat id and mode.



## Status Notes

No status notes currently listed.
