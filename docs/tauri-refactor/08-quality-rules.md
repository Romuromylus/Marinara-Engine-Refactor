# Quality Rules

These rules keep the refactor maintainable.

## File Size Targets

| File Type | Target Max | Hard Review Point |
| --- | ---: | ---: |
| Tauri command file | 250 lines | 400 lines |
| Rust service file | 400 lines | 700 lines |
| Rust orchestrator file | 500 lines | 900 lines |
| Rust repository file | 450 lines | 800 lines |
| React component | 350 lines | 600 lines |
| React hook | 250 lines | 450 lines |
| Zustand store slice | 250 lines | 450 lines |

If a file crosses the hard review point, split it or document why it should remain together.

## Rust Layering Rules

1. `commands` call services.
2. Services call repositories and provider clients.
3. Repositories own persistence.
4. Security helpers are centralized.
5. Domain crates do not import `tauri`.
6. DTO modules do not import services.
7. Side effects are explicit and testable.

## Frontend Layering Rules

1. Feature code imports shared code.
2. Shared code does not import feature code.
3. App shell composes features but does not own feature behavior.
4. Generated bindings are the source of DTO truth.
5. No feature should call `invoke` directly; use `shared/api`.
6. No provider secret should be stored in frontend state or localStorage.
7. Do not add temporary frontend behavior, fake persistence, mock APIs, or placeholder Tauri commands during the UI-only migration.

## Testing Expectations

### Required Rust Tests

- path traversal checks
- outbound URL policy checks
- secret save/load/delete
- repository round trips
- current profile package export/import
- prompt assembly snapshots
- lorebook injection ordering
- regex pipeline behavior
- dice and combat mechanics
- game state checkpoint/restore
- import folder token validation

### Required Frontend Tests

- command wrapper error handling
- generated type import smoke test
- primary app shell render
- chat send flow with mocked command stream
- settings persistence with mocked commands
- game turn event handling

## Security Gates

Before shipping:

1. Remove sample `greet` command.
2. Set a real CSP in `tauri.conf.json`.
3. Audit Tauri capabilities.
4. Verify no API keys are in localStorage.
5. Verify extension execution cannot access secret command APIs.
6. Verify file access is app-scoped or native-dialog-scoped.
7. Verify provider local URL flags default to safe behavior.
8. Verify update apply requires explicit permission.

## Migration Review Checklist

For each migrated feature:

- Does the UI look and behave the same?
- If the backend is not migrated yet, did we avoid adding temporary functionality to hide that fact?
- Are DTOs generated from Rust domain DTOs?
- Are commands thin?
- Are secrets kept in Rust?
- Are filesystem paths validated?
- Are long-running tasks cancellable?
- Are progress events typed?
- Are tests added at the service/repository level?
- Is there no direct `/api` fetch left for the migrated feature?
- If this is a backend module, is it a complete vertical slice where feasible: storage, DTOs, commands, services, security, events, frontend hook adaptation, inventory updates, and tests?

## Required Human Review Handoff

Every implementation slice must end with a review handoff that includes:

- what changed
- source inventory items moved, mapped, deferred, or intentionally removed with approval
- what is intentionally non-functional
- command to run
- where to click or test
- expected visual or behavioral result
- files or areas to inspect
- tests run
- one explicit question: change this slice or continue?
