# Promansis

## Current Work

- Reloading immediately after tracker edits can lose pending world state.
  - Status: In review
  - Next step: Ready for review on the focused bug-fix branch after TypeScript and docs checks.
  - Blockers: None.

## Owned Bugs

### Reloading immediately after tracker edits can lose pending world state

- Status: In review
- Owner: Promansis
- Impact area: UI | shared state
- Reported: 2026-05-19
- Last updated: 2026-05-19

#### Notes

The local-only bug backlog lists this as bug 3. The fix belongs to the world-state patch queue because tracker edits are debounced before being written through the world state API.

The patch queue now mirrors pending tracker edits into browser storage synchronously and clears them only after the Tauri-backed world-state patch succeeds, so an immediate reload can replay unsaved edits on the next mount.

## Status Notes

No status notes currently listed.
