# Source Inventory

Use this file as the guardrail for implementation slices. Every file from the original app must be moved, mapped, deferred, or explicitly approved for removal.

Original source root: `E:/Personal Projects/Marinara-Engine`.

## Frontend Sources

Source: `packages/client`.

Track these groups during frontend slices:

- `src/components/layout`: app shell, top bar, sidebars, modal root, theme injector.
- `src/components/panels`: settings and feature panels.
- `src/components/ui`: reusable UI primitives and shared controls.
- `src/components/modals`: modal bodies owned by their feature modules.
- `src/components/chat`: chat, conversation, roleplay, gallery, scene, and prompt preview UI.
- `src/components/game`: game mode UI.
- `src/components/agents`: agents, tools, regex, context, and debug UI.
- `src/components/bot-browser`, `characters`, `connections`, `lorebooks`, `onboarding`, `personas`, `presets`, `spotify`: feature-owned UI.
- `src/hooks`: migrate with the feature that owns the behavior.
- `src/stores`: migrate with the feature or app/shared state boundary that owns the data.
- `src/lib`: keep frontend-only helpers in React; move filesystem, secret, provider, import, and backend behavior to Rust.
- `src/styles`: global styles and themes.
- `public`: copy assets only when the slice that renders them moves.

## Backend Sources

Source: `packages/server` and `packages/shared`.

Track these groups during Rust backend slices:

- `src/routes`: route behavior maps to Tauri commands and Rust services.
- `src/services`: service behavior maps to Rust domain modules.
- `src/services/storage`: repository behavior maps to raw file-backed Rust repositories.
- `src/db`: use only as source metadata for the current file-native shapes; do not port SQL, Drizzle, SQLite, or migrations.
- `src/middleware`: map relevant protections to Tauri capabilities, command validation, filesystem policy, outbound URL policy, and secret handling.
- `src/utils`, `src/config`, `src/lib`: map useful behavior to `core`, `security`, or owning domain modules.
- `packages/shared/src/types`, `schemas`, `constants`, `utils`: map to Rust domain DTOs, generated TypeScript bindings, or frontend-only helpers.
- `assets`: copy defaults only when the owning feature slice needs them.
- `scripts`, platform folders, installer/launcher support: account for them during sidecar, updates, packaging, and sync phases.
- `tests`: use as behavior references for Rust service/repository tests.

## Required Slice Update

Each slice handoff must list source inventory status:

- moved
- mapped to a new module
- deferred with reason
- removed with explicit approval

Do not leave a touched source area unaccounted for.

## Slice Status

### Phase 0 Baseline Structure

Status: Complete.

- Starter Tauri frontend bootstrap moved from `src/main.tsx`, `src/App.tsx`, and `src/App.css` to `src/app`.
- Starter sample assets in `src/assets` and `public` are deferred until sample Tauri code is removed during hardening.
- Rust backend source remains starter Tauri code in `src-tauri/src`; planned domain crate homes are mapped under `src-tauri/crates`.
- No original Marinara source from `E:/Personal Projects/Marinara-Engine` has been moved yet.

### Phase 1 Frontend Shell And Shared Foundations

Status: Complete.

- Moved original `packages/client/src/App.tsx` and `main.tsx` into `src/app`, then adapted them for the Tauri refactor shell without old HTTP health checks, PWA registration, keep-alive, CSRF fetch shims, or server font preloading.
- Moved original `components/layout/AppShell.tsx`, `TopBar.tsx`, `RightPanel.tsx`, `ModalRenderer.tsx`, and `CustomThemeInjector.tsx` into `src/app/shell` and `src/app/providers`.
- Mapped original shell-owned range slider setup into `src/app/startup/range-slider-sync.ts`.
- Moved original shared UI dialog/modal primitives from `components/ui/AppDialogRenderer.tsx` and `components/ui/Modal.tsx` into `src/shared/components/ui`.
- Moved original frontend-only helpers `lib/utils.ts` and `lib/app-dialogs.ts` into `src/shared/lib`.
- Moved original browser UI state stores `stores/ui.store.ts` and `stores/dialog.store.ts` into `src/shared/stores`.
- Global styles are present under `src/styles/globals.css` and `src/styles/themes/sillytavern.css`; `src/app/main.tsx` imports them directly.
- Deferred feature screens and backend-backed surfaces from `components/chat`, `components/panels`, `components/modals`, `components/spotify`, `components/onboarding`, feature hooks, feature stores, and backend API clients to Phase 2+ slices. Current shell placeholders preserve navigation locations without adding fake backend behavior.

### Phase 2 Slice 1 Settings Shell And Settings Sections

Status: Complete.

- Moved original `components/panels/SettingsPanel.tsx` into `src/features/settings/components/SettingsPanel.tsx` with layout and markup preserved.
- Moved original `components/panels/settings/SettingControls.tsx` into `src/features/settings/components/settings/SettingControls.tsx`.
- Moved original shared UI primitives used by settings, `components/ui/HelpTooltip.tsx`, `DraftNumberInput.tsx`, and `ExportFormatDialog.tsx`, into `src/shared/components/ui`.
- Mapped server-backed settings dependencies to Phase 2-safe placeholders: themes/extensions query hooks, admin/server API client, chat metadata/data-clearing hooks, game asset rescanning, character embedded-lorebook inspection, browser refresh, and notification sound helper.
- Wired the existing right panel settings route to render the migrated `SettingsPanel`.
- Deferred real persistence, imports, backups, updates, background/font file operations, extension execution, and destructive data actions until their owning Rust backend slices.

### Phase 2 Slice 2 Theme/Preferences UI

Status: Complete.

- Mapped theme and appearance preferences from original `components/panels/SettingsPanel.tsx` to the migrated settings feature, preserving the Appearance and Themes tab UI.
- Mapped original theme preference state from `stores/ui.store.ts` to `src/shared/stores/ui.store.ts`, including color scheme, visual theme, font sizing, conversation gradient, text appearance, avatar style, and local custom theme fields.
- Wired app-level preference effects in `src/app/App.tsx` and `src/app/providers/CustomThemeInjector.tsx` so color scheme, visual theme, font size, font family, and active custom CSS apply to the document shell.
- Reworked `src/features/settings/hooks/use-themes.ts` to use the existing persisted UI store for local custom theme create, edit, import, activate, export, and delete behavior instead of the temporary query-cache placeholder.
- Deferred Rust-backed theme storage/sync, custom font folder operations, Google Fonts download, background file operations, and chat metadata persistence until their owning backend/file slices.

### Phase 2 Slice 3 Character/Persona Library Read Surfaces

Status: Complete.

- Moved original `components/panels/CharactersPanel.tsx` into `src/features/characters/components/CharactersPanel.tsx` and wired the right-panel `characters` route to render it.
- Moved original `components/panels/PersonasPanel.tsx` into `src/features/personas/components/PersonasPanel.tsx` and wired the right-panel `personas` route to render it.
- Moved original `components/characters/CharacterLibraryView.tsx` into `src/features/characters/components/CharacterLibraryView.tsx` and wired the existing character-library UI state to render it in the center shell.
- Moved original shared `components/ui/ContextMenu.tsx` into `src/shared/components/ui/ContextMenu.tsx`.
- Mapped original `lib/character-display.ts` to `src/features/characters/lib/character-display.ts`.
- Added feature-owned frontend API seams in `src/features/characters/api/characters-api.ts` and `src/features/personas/api/personas-api.ts`; these intentionally fail with explicit Rust-backend-slice errors instead of fake persistence.
- Added Phase 2-safe character/persona query and mutation hooks under `src/features/characters/hooks` and `src/features/personas/hooks`, preserving click paths while deferring real storage.
- Deferred full character editor, persona editor, create/import/maker modals, PNG import/export, avatar upload, group persistence, duplicate/delete, active persona persistence, and start-chat behavior until their owning frontend/modal and Rust backend slices.
