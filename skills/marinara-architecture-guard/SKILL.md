---
name: marinara-architecture-guard
description: "Protect Marinara's layered Tauri architecture, module ownership, dependency direction, explicit imports, Rust capability boundaries, shared-code placement, and file-splitting discipline. Use when changing folders, imports, shared modules, TypeScript engine layers, Tauri command wrappers, Rust capability crates, repositories, adapters, feature APIs, or any code structure that could widen impact area."
---

# Marinara Architecture Guard

## Overview

Use this skill to keep Marinara readable and modular while changing code. The goal is to build with stable bricks: small owner modules, explicit contracts, narrow adapters, and visible dependency direction.

## Load First

Read these references only when needed:

- `references/repo-layout.md` for the current architecture map and owner paths.
- `references/dependency-boundaries.md` for import direction and placement decisions.

Also keep the root `AGENTS.md` in force.

## Workflow

1. Name the owner before editing: UI feature, TypeScript engine layer, shared API adapter, or Rust capability.
2. List imports the changed module may use. If an import crosses a boundary, redesign before patching.
3. Keep behavior in its owner. Move reusable logic down to a lower layer instead of sideways into another mode or feature.
4. Prefer direct owner imports over barrels or compatibility shims.
5. Split large mixed files when adding behavior would make the file broader.
6. Update docs or skill references when a durable architecture decision changes.
7. Report the impact area and dependent areas reviewed.

## Placement Rules

- Product rules live in `src/engine`, not Rust and not React components.
- React feature code lives in layered packages under `src/features/<layer>/<package>` and calls hooks, feature APIs, or shared API adapters through public entrypoints.
- Generic UI and browser-only utilities live in `src/shared`.
- Tauri invoke wrappers live in `src/shared/api`.
- Privileged local IO, storage, secrets, provider transport, and native integrations live in Rust.
- Mode-neutral deterministic helpers live below modes in `engine/shared`, `engine/entities`, or `engine/generation-core`.

## Stop Conditions

Pause and re-evaluate if the change requires a generic router, a broad catch-all helper, cross-mode imports, direct Tauri calls from engine code, React imports from engine code, or a new fallback branch for old runtime shapes. Those are architecture smells in this repo.
