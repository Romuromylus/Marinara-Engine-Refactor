# Promansis

## Current Work

- Stop generation does not cancel the provider stream command.
  - Status: In review
  - Next step: Ready for review on the focused bug-fix branch after TypeScript, Rust, and docs checks.
  - Blockers: None.

## Owned Bugs

### Stop generation does not cancel the provider stream command

- Status: In review
- Owner: Promansis
- Impact area: Generation | prompts | agents | provider boundary
- Reported: 2026-05-19
- Last updated: 2026-05-19

#### Notes

The local-only bug backlog lists this as bug 5. The frontend abort signal stopped the local async stream iterator, but `llm_stream_channel` had no request id or cancellation token, so Rust provider streaming could continue until completion or channel send failure.

The fix should add an explicit stream cancellation contract between `src/shared/api/llm-api.ts` and the Rust LLM command boundary without moving provider transport behavior out of `marinara_llm`.

`llmApi.stream` now assigns each stream a native cancellation id and calls `llm_stream_cancel` when the abort signal fires. The Rust command registers active stream ids in app state and uses `tokio::select!` to drop the provider stream future when cancellation is requested.
The local-only bug backlog lists this as bug 4. Character create and version restore already serialize card `data`, but generic `storage_update` patches could persist object-shaped `data` from the character editor, agent card updates, roleplay scene memories, chat schedules, and connected character commands.

Generic character update patches now normalize card `data` at the Rust storage command boundary before writing to storage, so all `storage_update` callers keep the persisted JSON-string contract.

## Status Notes

No status notes currently listed.
