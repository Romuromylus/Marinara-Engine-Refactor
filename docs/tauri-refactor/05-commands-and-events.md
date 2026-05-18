# Commands And Events

Tauri commands replace HTTP routes. The command layer must stay thin and typed.

## Command Rules

1. Commands do not contain business logic.
2. Commands do not read or write files directly.
3. Commands do not construct prompts.
4. Commands do not call providers directly.
5. Commands return DTOs owned by the relevant Rust domain crate.
6. Long-running work emits events and supports cancellation.

## Command Naming

Use feature prefix plus action:

```text
chat_list
chat_get
chat_create
chat_update
chat_delete
generation_start
generation_cancel
game_turn_start
sidecar_install_runtime
spotify_start_oauth
haptic_list_devices
```

Avoid generic names like `create`, `save`, or `run`.

## Command Groups

### Chat

```text
chat_list
chat_get
chat_create
chat_update
chat_delete
chat_list_messages
chat_create_message
chat_update_message
chat_delete_message
chat_create_swipe
chat_select_swipe
chat_list_folders
chat_create_folder
chat_update_folder
chat_delete_folder
```

### Characters And Personas

```text
character_list
character_get
character_create
character_update
character_delete
character_duplicate
character_restore_version
character_upload_avatar
persona_list
persona_get
persona_create
persona_update
persona_delete
persona_set_active
```

### Lorebooks, Prompts, Presets

```text
lorebook_list
lorebook_get
lorebook_create
lorebook_update
lorebook_delete
lorebook_entry_create
lorebook_entry_update
lorebook_entry_delete
prompt_list
prompt_create
prompt_update
prompt_delete
preset_list
preset_create
preset_update
preset_delete
prompt_peek
prompt_review
```

### Connections And Providers

```text
connection_list
connection_get
connection_create
connection_update
connection_delete
connection_duplicate
connection_test
connection_test_message
connection_test_image
connection_list_models
connection_diagnose_claude_subscription
```

### Generation

```text
generation_start
generation_retry
generation_dry_run
generation_cancel
generation_apply_regex
generation_get_prompt_preview
```

Generation returns a `GenerationRunId`. Tokens arrive through events.

Do not emulate the old HTTP/SSE route. `generation_start` creates a run, returns the run ID, and the frontend subscribes to Tauri events for streaming updates.

### Agents

```text
agent_list
agent_get
agent_create
agent_update
agent_delete
agent_run_list
agent_memory_get
agent_memory_update
custom_tool_list
custom_tool_create
custom_tool_update
custom_tool_delete
regex_script_list
regex_script_create
regex_script_update
regex_script_delete
```

### Conversation

```text
conversation_get_schedule
conversation_update_schedule
conversation_run_autonomous_check
conversation_get_awareness
conversation_update_summary
```

### Roleplay

```text
scene_get
scene_update
scene_analyze
sprite_list
sprite_upload
sprite_update
sprite_delete
sprite_generate
encounter_start
encounter_update
encounter_finish
```

### Game

```text
game_state_get
game_state_update
game_turn_start
game_turn_cancel
game_checkpoint_list
game_checkpoint_create
game_checkpoint_restore
game_session_get
game_session_update
game_asset_list
game_asset_generate
game_roll_dice
game_apply_skill_check
game_update_combat
game_update_journal
game_update_inventory
game_repair_json
```

### Assets, Gallery, Fonts

```text
asset_background_list
asset_background_upload
asset_avatar_upload
gallery_list
gallery_add
gallery_delete
gallery_recover
font_list
font_load
```

### Imports And Current Profile Packages

```text
import_pick_folder
import_list_directory
import_scan_sillytavern
import_run_sillytavern_bulk
import_character_card
import_lorebook
profile_export_current
profile_import_current
```

Old backup/archive compatibility is not part of the runtime application. If old install data needs conversion later, that belongs in a separate migration script.

### Sidecar

Sidecar commands are intentionally absent from the active Tauri app until the sidecar scope is reintroduced.

### Integrations

```text
spotify_status
spotify_start_oauth
spotify_exchange_code
spotify_disconnect
spotify_search
spotify_play
spotify_pause

haptic_list_devices
haptic_connect
haptic_disconnect
haptic_test_pattern
haptic_set_enabled

tts_speak
tts_list_voices
tts_clear_cache

translation_translate

bot_browser_search
bot_browser_get_character
bot_browser_download_character
bot_browser_set_auth
bot_browser_validate_auth
bot_browser_logout

gif_search
discord_webhook_test
home_assistant_webhook_handle
```

### Updates, Themes, Extensions

```text
update_check
update_get_release_notes
update_apply
theme_list
theme_create
theme_update
theme_delete
extension_list
extension_create
extension_update
extension_delete
extension_set_enabled
```

### Sync

```text
sync_get_status
sync_get_settings
sync_update_settings
sync_pair_device
sync_disconnect
sync_list_devices
sync_remove_device
sync_push_now
sync_pull_now
sync_resolve_conflict
sync_list_conflicts
sync_open_server_docs
```

## Event Naming

Use stable names:

```text
generation://started
generation://token
generation://agent-update
generation://game-state
generation://done
generation://error

sidecar://status
sidecar://download-progress
sidecar://runtime-progress
sidecar://log

import://progress
import://done
import://error

game://turn-started
game://turn-event
game://turn-done
game://turn-error

haptic://device-status
spotify://status
update://available
sync://status
sync://heads
sync://push-progress
sync://pull-progress
sync://blob-progress
sync://conflict
sync://error
```

Generation event flow:

```text
generation_start(input) -> GenerationRunId
generation://started
generation://token
generation://agent-update
generation://game-state
generation://done
generation://error
generation_cancel(run_id)
```

## Cancellation

Every long-running action should accept or create a run ID:

```text
GenerationRunId
GameTurnRunId
ImportRunId
DownloadRunId
UpdateRunId
```

Cancellation commands:

```text
generation_cancel(run_id)
game_turn_cancel(run_id)
import_cancel(run_id)
sidecar_cancel_download(run_id)
update_cancel(run_id)
sync_cancel(run_id)
```

## TypeScript Bindings

Generated bindings should live at:

```text
src/shared/api/bindings.ts
```

Frontend code should import DTOs from generated bindings, not duplicate shapes by hand.

Rust domain crates own their own frontend-facing DTOs. Do not create a central contracts crate. Shared primitives such as IDs and pagination should come from `core`; sync protocol shapes are deferred with sync.
