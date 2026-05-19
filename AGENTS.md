# Agent Instructions

This repo is a Tauri application with a TypeScript product engine and Rust local capability layer. Maintain it as a clean, layered application. Every bug fix or feature should leave the code easier to reason about, not more tangled.

## Repo Skills

Load the relevant repo-local skill before editing code in that area:

- `skills/marinara-architecture-guard/SKILL.md`: architecture, imports, file layout, shared modules, Tauri adapters, Rust capabilities, repositories, or cross-feature boundaries.
- `skills/marinara-mode-separation/SKILL.md`: chat/conversation, roleplay, game, generation guides, prompt assembly, scene logic, autonomous flows, game turns, or mode UI.
- `skills/marinara-bugfix-discipline/SKILL.md`: bugs, regressions, broken UI actions, failing checks, storage/provider/import/generation issues, or any fix with dependent callers.
- `skills/marinara-getting-started/SKILL.md`: onboarding, "how do I get started?", repo tours, running docs, running the app, first testing paths, and guided bug-fixing flow.

Keep these skills updated when architectural decisions change.

## User Docs

User-facing developer docs live in `docs/developer/`.

- Main entry: `docs/developer/index.html`
- Getting started guide: `docs/developer/getting-started.html`
- Run/build guide: `docs/developer/run-build.html`
- Architecture guide: `docs/developer/architecture.html`
- Module guide: `docs/developer/modules.html`
- Impact guide: `docs/developer/impact-areas.html`

When the user asks to view or "run" the docs, start the static docs server from the repo root:

```text
pnpm docs:dev
```

Then give the user this URL:

```text
http://127.0.0.1:4174/
```

If port `4174` is busy, run Vite directly with another port:

```text
pnpm exec vite docs/developer --host 127.0.0.1 --port <free-port>
```

For docs-only edits, run `pnpm check:docs`. If the run/build commands, scripts, or Tauri config change, update `docs/developer/run-build.html` in the same change.

Do not tell users to run `pnpm docs`; that collides with pnpm/npm's package documentation command in this environment.

## Bug And Work Updates

Use `updates/` to track bug reports, active bug ownership, current work, and status updates.

- Report new bugs in `updates/unowned-bugs.md` when they do not have an owner.
- Move a bug from `updates/unowned-bugs.md` into the correct `updates/people/*.md` file when someone starts fixing it.
- Use the user's GitHub name to determine who is working on a bug.
- If the user asks "who am I?" or asks how to track their bugs, check local identity first with `git config user.name`, then `git config user.email`, and use `gh auth status` when GitHub CLI is logged in.
- Compare the identity against the existing files in `updates/people/` and choose the closest matching owner file instead of creating a new mapping for every Git username.
- Map GitHub user `Coda` to Chai; this is the known confusing name.
- Keep owner files updated with the bug status, next step, blockers, and resolution.

## Getting Started Requests

When a user asks "how do I get started?", "onboard me", "teach me this repo", or anything similar:

1. Load `skills/marinara-getting-started/SKILL.md`.
2. Start the developer docs with `pnpm docs:dev` unless the user explicitly wants text only.
3. Give the docs URL: `http://127.0.0.1:4174/`.
4. Point them first to `docs/developer/getting-started.html`, then `run-build.html`, `architecture.html`, `modules.html`, and `impact-areas.html`.
5. Explain the repo shape: React UI in `src/features`, product behavior in `src/engine`, Tauri adapters in `src/shared/api`, Rust capabilities in `src-tauri`.
6. Instruct them to run the app with `pnpm install` and `pnpm tauri dev`.
7. Guide manual testing through chat, roleplay, game, settings/providers, imports, exports, and assets.
8. When they find a bug, ask for workflow, steps, expected result, actual result, mode/feature, data used, and any error output. Then switch to `skills/marinara-bugfix-discipline/SKILL.md` and fix the root cause.

Do not begin code edits from a vague getting-started prompt. Onboard, run, test, then edit only after a concrete bug or feature request exists.

## Core Standard

Build code with bricks, not sticks.

- Prefer small, explicit owner modules over compact files that touch many things.
- Keep modules separated and clear about what they import and use.
- Use shared modules for real reusable primitives, not copied code.
- Fix root causes. Never stack patches, fake-success branches, compatibility shims, or UI-only guards over broken contracts.
- Each change should have a clear impact area, clear dependent callers, and a coherent commit shape.

## Architecture

The main architecture references are:

- `docs/developer/architecture.html`
- `docs/developer/modules.html`
- `docs/developer/impact-areas.html`

Treat the layered structure in these docs as the maintenance architecture unless a newer decision replaces it.

Core flow:

```text
React app/features
  -> shared/api Tauri adapters
  -> TypeScript engine use cases
  -> engine capability ports
  -> Rust Tauri commands
  -> Rust capability crates
```

TypeScript owns product behavior:

- chat, autonomous conversation, roleplay, and game rules
- agents, prompt rules, generation orchestration, and mode state transitions
- deterministic parsing, formatting, scoring, prompt-building, and UI-facing application flow

Rust owns privileged local capabilities:

- Tauri commands, events, and channels
- storage, atomic writes, path safety, managed files, and assets
- provider transport, secrets, OAuth, safe fetch, and native filesystem access
- integrations such as Spotify, TTS, translation, haptics, imports, exports, and local file operations

## Dependency Rules

- A behavior has one owner: React UI, TypeScript engine, or Rust capability.
- Import from owner files or explicit public feature APIs. Do not reach into another feature's private internals.
- Avoid convenience barrels, one-line re-export shims, `legacy-*` aliases, and dumping-ground `utils` files.
- Engine code must not import React, Zustand stores, `@tauri-apps/api`, or concrete `src/shared/api` adapters.
- React components must not duplicate engine rules. They call feature hooks, feature APIs, or engine use cases through adapters.
- Rust commands stay thin: validate inputs, call capability services, return DTOs/events.
- Shared code is allowed only for genuinely mode-neutral primitives, generic UI atoms, deterministic helpers, capability ports, repositories, transport, and asset IO.

When an import feels convenient but crosses ownership, make a lower-layer contract or move the reusable primitive down.

## Mode Separation

Chat/conversation, roleplay, and game are separate product paths.

- `src/engine/modes/chat` owns normal chat, autonomous behavior, schedules, summaries, awareness, and chat commands.
- `src/engine/modes/roleplay` owns scenes, roleplay encounters, roleplay scene memory, roleplay sprites, and visual-novel choices.
- `src/engine/modes/game` owns game turns, GM/party prompts, game state, maps, combat, loot, checks, weather, time, game assets, sessions, and game scene analysis.
- Top-level mode engines must not import each other.
- Shared scene, sprite, transcript, macro, parser, game-state text, and attachment utilities belong in lower layers when more than one mode needs them.
- Do not hide mode differences behind a generic guide string, a mode flag, or a shared catch-all orchestrator when a mode needs its own entry point.

Editing one mode must not silently alter another mode. If a shared layer change affects multiple modes, say so before editing and verify the affected modes.

## API And Capability Rules

- Frontend code calls typed Tauri wrappers in `src/shared/api` or local TypeScript feature/engine APIs.
- Engine code accepts capability interfaces from `src/engine/capabilities`; it does not invoke Tauri directly.
- Rust Tauri commands are grouped by capability and backed by focused crates/modules.
- Do not add generic string routers, fake local API bridges, server-shaped fallback paths, or browser fetches for local app behavior.
- Provider URL paths are allowed only inside the appropriate Rust transport or integration capability code.

## Bug Fix Workflow

Before editing a bug:

1. State the failing behavior.
2. Identify the owning layer and module.
3. Identify the expected impact area and dependent callers.
4. Trace the data contract across UI, engine, adapter, command, and capability boundaries as needed.
5. Decide which focused checks will prove the fix.

While fixing:

- Fix the lowest correct owner, not the most convenient caller.
- Remove obsolete fallbacks or placeholders exposed by the fix.
- Keep the change scoped to the behavior under repair.
- If the scope grows, update the impact area before continuing.

Do not hide failures behind silent catches, fake success, broad defaults, old-shape compatibility, or UI-only conditionals. If the state contract is wrong, fix the contract. If the behavior owner is wrong, move it.

## New Feature Workflow

Before adding a feature:

1. Identify the product owner: chat, roleplay, game, another feature, shared engine, shared UI, or Rust capability.
2. Define the public entry point and the private implementation files.
3. Decide which existing contracts, repositories, capabilities, and UI primitives can be reused.
4. Add new shared code only when at least two owners truly need the same primitive.
5. Keep mode-specific prompts, orchestration, memory semantics, and state transitions in the owning mode.
6. Add verification that covers the feature's real path through UI, engine, adapters, and Rust when applicable.

New features should be vertical enough to work end to end, but split into focused modules. Avoid one large feature file that renders UI, loads data, mutates storage, calls providers, and owns orchestration.

## File Size And Separation

Large files are a warning sign when they combine multiple concerns.

Split when a file mixes several of these:

- UI rendering
- data loading
- state transitions
- prompt assembly
- mode orchestration
- provider transport
- storage persistence
- import/export parsing
- Tauri command registration
- filesystem/path safety

Prefer focused files named after one responsibility. Do not duplicate code to avoid splitting. Move mode-neutral reuse into `engine/shared`, `engine/entities`, `engine/generation-core`, `src/shared`, or Rust capability helpers as appropriate.

## Informative Agent Workflow

Agents must be clear about impact.

Before edits, state:

- owner
- expected impact area
- likely files/folders
- affected modes or capabilities
- checks planned

Final responses for code changes must include:

- behavior changed
- primary files/modules touched
- impact area and dependent areas reviewed
- verification run, or why a check could not be run
- remaining risk or external QA needed

Each commit should make sense as a coherent unit. Do not mix unrelated formatting, cleanup, bug fixes, and architecture moves in one commit.

## Verification

Run checks that match the changed area:

- TypeScript/UI/engine: `pnpm typecheck`
- Build/import graph/bundling: `pnpm build`
- Rust commands/capabilities/provider transport: `cargo check --manifest-path src-tauri/Cargo.toml`
- Docs/skills/agent guidance: `pnpm check:docs`

Prefer the full set when touching shared contracts, generation, storage, provider transport, mode orchestration, or architecture boundaries. If a check cannot be run, say exactly why.
