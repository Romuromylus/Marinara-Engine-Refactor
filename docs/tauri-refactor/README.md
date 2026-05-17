# Marinara Tauri Refactor

This documentation describes the plan for moving Marinara Engine from the current Node/React app into a Rust + Tauri desktop app while preserving the existing React UI.

## Agent Entry Path

Read these in order before implementing:

1. [Refactor Rules](../../RULES.md): hard rules, workflow, and review handoff.
2. [Migration Plan](./06-migration-plan.md): executable sequence.
3. [Target Structure](./01-target-structure.md): workspace layout and ownership boundaries.
4. [Source Inventory](./00-source-inventory.md): original source areas that must be accounted for.
5. [Quality Rules](./08-quality-rules.md): testing, file size, and human review gates.

Use the remaining docs as references for specific implementation areas.

## Non-Negotiables

- Move/lightly reorganize the frontend; do not rewrite it wholesale.
- Rebuild backend responsibilities in Rust with minimal, explicit code.
- Do not add temporary functionality or fake behavior.
- Do not copy the old server into Tauri.
- Use raw file storage only for local desktop data.
- Keep existing file-native storage readable where practical.
- Keep secrets and authenticated provider calls in Rust.
- Use Tauri events for generation streaming.
- Implement sync last.
- Stop after every slice for human review.

## Documentation Index

- [Source Inventory](./00-source-inventory.md): source areas every slice must account for.
- [Target Structure](./01-target-structure.md): final workspace layout and ownership rules.
- [Rust Backend Modules](./02-rust-backend-modules.md): Rust module responsibilities.
- [Frontend Organization](./03-frontend-organization.md): React folder structure and migration rules.
- [Feature Coverage Map](./04-feature-coverage-map.md): current features and target homes.
- [Commands And Events](./05-commands-and-events.md): Tauri IPC, event, and cancellation conventions.
- [Migration Plan](./06-migration-plan.md): step-by-step implementation order.
- [Rust Crates And Tauri Plugins](./07-crates-and-plugins.md): dependency candidates.
- [Quality Rules](./08-quality-rules.md): file size, testing, and review gates.
- [Sync Server](./09-sync-server.md): optional sync module, implemented last.
- [Research Notes](./10-research-notes.md): ecosystem notes and links.
- [Mermaid Diagrams](./11-mermaid-diagrams.md): architecture diagrams.
- [TypeScript/Rust Organization Plan](./13-typescript-rust-organization-plan.md): proposed TS engine + Rust capability split with file-by-file inventory.
- [Layered Module Architecture](./14-layered-module-architecture.md): dependency layers, mode boundaries, and UI reorganization rules.

## Visual Guides

- [Visual Overview](./index.html)
- [Architecture Diagram](./architecture.html)
- [Sync Server Diagram](./sync-server.html)
