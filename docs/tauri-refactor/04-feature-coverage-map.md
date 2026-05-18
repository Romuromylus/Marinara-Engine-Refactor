# Feature Coverage Map

This map ensures every current backend and frontend feature has a target home in the refactor.

## Original Route Coverage Checklist

Every route from `E:/Personal Projects/Marinara-Engine/packages/server/src/routes` must be mapped before implementation begins. This table is the route-level guard against missing backend behavior during the Rust rewrite.

| Original route | Rust target | Frontend target / consumer |
| --- | --- | --- |
| `admin.routes.ts` | `security/admin_gate.rs`, command permissions | settings/admin prompts |
| `agents.routes.ts` | `agents` | `features/agents` |
| `app-settings.routes.ts` | `storage/repositories/app_settings.rs` | `features/settings` |
| `avatars.routes.ts` | `assets/avatars.rs` | `features/characters`, `features/personas` |
| `backgrounds.routes.ts` | `assets/backgrounds.rs` | `features/settings`, `features/assets` |
| `backup.routes.ts` | Removed from active runtime; current profile package import/export lives in `import/profile` | `features/imports`, settings profile package UI |
| `bot-browser.routes.ts` | `integrations/bot_browser` | `features/bot-browser` |
| `bot-browser-chartavern.routes.ts` | `integrations/bot_browser/chartavern.rs` | `features/bot-browser` |
| `bot-browser-datacat.routes.ts` | `integrations/bot_browser/datacat.rs` | `features/bot-browser` |
| `bot-browser-janny.routes.ts` | `integrations/bot_browser/janny.rs` | `features/bot-browser` |
| `bot-browser-pygmalion.routes.ts` | `integrations/bot_browser/pygmalion.rs` | `features/bot-browser` |
| `bot-browser-wyvern.routes.ts` | `integrations/bot_browser/wyvern.rs` | `features/bot-browser` |
| `character-maker.routes.ts` | `agents` or `generation` character maker workflow | character maker modal |
| `characters.routes.ts` | `storage/repositories/characters.rs` | `features/characters` |
| `chat-folders.routes.ts` | `chat/folders.rs` | `features/chat` |
| `chat-presets.routes.ts` | `storage/repositories/chat_presets.rs` | `features/presets` |
| `chats.routes.ts` | `chat`, `storage/repositories/chats.rs` | `features/chat` |
| `connections.routes.ts` | `storage/repositories/connections.rs`, `llm/model_discovery.rs` | `features/connections` |
| `conversation.routes.ts` | `conversation` | `features/conversation` |
| `custom-tools.routes.ts` | `agents/tools/custom.rs` | `features/agents` |
| `encounter.routes.ts` | `roleplay/encounter` | `features/roleplay` |
| `extensions.routes.ts` | `storage/repositories/extensions.rs`, `security/permissions.rs` | `features/extensions` |
| `fonts.routes.ts` | `assets/fonts.rs` | `features/settings` |
| `gallery.routes.ts` | `assets/gallery.rs` | `features/gallery`, chat gallery |
| `game-assets.routes.ts` | `game/assets`, `assets/default_assets.rs` | `features/game`, `features/assets` |
| `game.routes.ts` | `game` | `features/game` |
| `generate.routes.ts` | `generation` | `features/chat`, `features/roleplay`, `features/game` |
| `gifs.routes.ts` | `integrations/gifs` | GIF picker UI |
| `haptic.routes.ts` | `integrations/haptic` | `features/haptic`, settings |
| `import.routes.ts` | `import` | `features/imports` |
| `knowledge-sources.routes.ts` | `agents/knowledge_retrieval.rs`, `agents/knowledge_router.rs` | `features/knowledge`, `features/agents` |
| `lorebook-maker.routes.ts` | `generation` or `agents` lorebook maker workflow | lorebook maker modal |
| `lorebooks.routes.ts` | `storage/repositories/lorebooks.rs`, `generation/pipeline/lorebook.rs` | `features/lorebooks` |
| `persona-maker.routes.ts` | `generation` persona maker workflow | persona maker modal |
| `prompt-overrides.routes.ts` | `generation/prompt/overrides.rs` | `features/prompts` |
| `prompt-reviewer.routes.ts` | `generation/prompt` review workflow | prompt reviewer UI |
| `prompts.routes.ts` | `storage/repositories/prompts.rs` | `features/prompts` |
| `regex-scripts.routes.ts` | `generation/pipeline/regex.rs`, `agents` | `features/agents` |
| `scene.routes.ts` | `roleplay/scene` | `features/roleplay` |
| `sidecar.routes.ts` | Deferred out of active app scope | none |
| `spotify-auth.routes.ts` | `integrations/spotify/oauth.rs`, `integrations/spotify/tokens.rs` | `features/spotify`, agent tools |
| `sprites.routes.ts` | `roleplay/sprites`, `assets` | `features/roleplay`, `features/assets` |
| `themes.routes.ts` | `storage/repositories/themes.rs` | `features/themes`, settings |
| `translate.routes.ts` | `integrations/translation` | `features/translation` |
| `tts.routes.ts` | `integrations/tts` | `features/tts`, settings |
| `updates.routes.ts` | `updates` | `features/updates` |
| `index.ts` | command registration in `src-tauri/src/commands/mod.rs` | none |

## Core App

| Current Area | Rust Target | Frontend Target | Notes |
| --- | --- | --- | --- |
| Fastify app bootstrap | `src-tauri/src/app.rs` | `src/app` | Replace HTTP server with Tauri state and commands. |
| Middleware security | `security` | `shared/api` for compatibility headers only | CSRF mostly disappears for local IPC, but path/network/auth protections stay. |
| Runtime config and `.env` | `core/config.rs` | settings UI | Keep support for advanced env/config overrides where useful. |
| Static file serving | Tauri asset system | Vite build | No Fastify static server required. |

## Storage And Data

| Feature | Rust Target | Frontend Target |
| --- | --- | --- |
| Chats | `chat`, `storage/repositories/chats.rs` | `features/chat` |
| Messages | `chat/messages.rs` | `features/chat` |
| Swipes and branches | `chat/swipes.rs`, `branch.rs` | `features/chat` |
| Chat folders | `chat/folders.rs` | `features/chat` |
| App settings | `storage/repositories/app_settings.rs` | `features/settings` |
| Current profile packages | `import/profile` | `features/imports`, `features/settings` |
| File-storage import | `import/file_storage.rs` | file-based import only; SQLite import is intentionally removed |

## Characters, Personas, Lorebooks, Prompts

| Feature | Rust Target | Frontend Target |
| --- | --- | --- |
| Characters | `storage/repositories/characters.rs` | `features/characters` |
| Character gallery | `assets/gallery.rs` | `features/gallery`, `features/characters` |
| Personas | `storage/repositories/characters.rs` or dedicated `personas.rs` | `features/personas` |
| Character groups | `storage/repositories/characters.rs` | `features/characters` |
| Persona groups | `storage/repositories/characters.rs` | `features/personas` |
| Lorebooks | `storage/repositories/lorebooks.rs`, `generation/pipeline/lorebook.rs` | `features/lorebooks` |
| Prompt presets | `storage/repositories/prompts.rs`, `generation/prompt/presets.rs` | `features/prompts`, `features/presets` |
| Chat presets | `storage/repositories/chat_presets.rs` | `features/presets` |
| Prompt overrides | `generation/prompt/overrides.rs` | `features/prompts` |
| Regex scripts | `generation/pipeline/regex.rs`, `agents` | `features/agents` |

## Connections And Providers

| Feature | Rust Target | Frontend Target |
| --- | --- | --- |
| API connections | `storage/repositories/connections.rs` | `features/connections` |
| API key encryption | `security/secrets.rs` | same connection UI flow; raw saved secrets never returned |
| Model discovery | `llm/model_discovery.rs` | `features/connections` |
| Provider test message | `llm/providers/*` | `features/connections` |
| Image provider test | `llm/images/*` | `features/connections` |
| Local provider URL policy | `security/outbound_url.rs` | settings hints only |

## Generation

| Feature | Rust Target | Frontend Target |
| --- | --- | --- |
| Normal generation | `generation/service.rs` | `features/chat/hooks/useMessageGeneration.ts` |
| Streaming tokens | `generation/stream.rs`, `src-tauri/src/events/generation.rs` | `shared/api/events.ts` |
| Prompt preview | `generation/prompt/peek.rs` | `features/prompts`, `features/chat` |
| Dry run | `generation/dry_run.rs` | `features/prompts` |
| Retry generation | `generation/retry.rs` | `features/chat` |
| Attachments | `generation/context/attachments.rs` | upload UI in chat/game |
| Regex application | `generation/pipeline/regex.rs` | `features/agents` |
| Lorebook injection | `generation/pipeline/lorebook.rs` | `features/lorebooks` |

## Agents And Tools

| Feature | Rust Target | Frontend Target |
| --- | --- | --- |
| Agent configs | `agents/service.rs` | `features/agents` |
| Agent execution | `agents/executor.rs` | `features/agents` |
| Agent pipeline | `agents/pipeline.rs` | `features/chat`, `features/game` |
| Knowledge routing | `agents/knowledge_router.rs` | `features/knowledge`, `features/agents` |
| Agent memory | `agents/memory.rs` | `features/agents` |
| Custom tools | `agents/tools/custom.rs`, `agents/tools/permissions.rs` | `features/agents` |
| Spotify tools | `agents/tools/spotify.rs`, `integrations/spotify` | `features/spotify`, `features/agents` |

## Conversation Mode

| Feature | Rust Target | Frontend Target |
| --- | --- | --- |
| Conversation view | storage only | `features/conversation` |
| Autonomous messages | `conversation/autonomous.rs` | `features/conversation/hooks` |
| Background autonomous | `conversation/background_autonomous.rs` | `features/conversation/hooks` |
| Schedules | `conversation/schedules.rs` | `features/conversation` |
| Awareness | `conversation/awareness.rs` | `features/conversation` |
| Conversation summaries | `conversation/summaries.rs` | `features/conversation`, `features/chat` |

## Roleplay And Visual Novel

| Feature | Rust Target | Frontend Target |
| --- | --- | --- |
| Roleplay surface | storage/generation only | `features/roleplay` |
| Scene analysis | `roleplay/scene/analyzer.rs` | `features/roleplay/hooks/useSceneAnalysis.ts` |
| Scene postprocess | `roleplay/scene/postprocess.rs` | `features/roleplay` |
| Sprites | `roleplay/sprites` | `features/roleplay` |
| Sprite generation | `roleplay/sprites/generation.rs`, `llm/images` | `features/assets` |
| Encounters | `roleplay/encounter` | `features/roleplay` |
| CYOA choices | `roleplay/visual_novel/choices.rs` | `features/roleplay` |
| Weather effects display | game/roleplay state from Rust | `features/roleplay/components/WeatherEffects.tsx` |

## Game Mode

| Feature | Rust Target | Frontend Target |
| --- | --- | --- |
| Game turn generation | `game/turn/orchestrator.rs` | `features/game/hooks/useGameTurn.ts` |
| GM prompts | `game/prompts/gm.rs` | no direct frontend ownership |
| Party prompts | `game/prompts/party.rs` | no direct frontend ownership |
| Dice | `game/mechanics/dice.rs` | display in `features/game` |
| Skill checks | `game/mechanics/skill_check.rs` | display in `features/game` |
| Combat | `game/mechanics/combat.rs` | `features/game/components/GameCombatUI.tsx` |
| Loot | `game/mechanics/loot.rs` | inventory panel |
| Morale | `game/mechanics/morale.rs` | party/combat panels |
| Reputation | `game/mechanics/reputation.rs` | character/session panels |
| Element reactions | `game/mechanics/elements.rs` | combat/narration display |
| Perception | `game/mechanics/perception.rs` | narration display |
| Map | `game/world/map.rs` | map panel |
| Map position | `game/world/map_position.rs` | map panel |
| Weather | `game/world/weather.rs` | weather panel/effects |
| Time | `game/world/time.rs` | HUD panels |
| Journal | `game/world/journal.rs` | journal panel |
| Checkpoints | `game/state/checkpoints.rs` | `GameCheckpoints.tsx` |
| Session history | `game/session/history.rs` | `GameSessionHistory.tsx` |
| Game assets | `game/assets`, `assets` | `features/game`, `features/assets` |

## Assets And Media

| Feature | Rust Target | Frontend Target |
| --- | --- | --- |
| Avatars | `assets/avatars.rs` | `features/characters`, `features/personas` |
| Backgrounds | `assets/backgrounds.rs` | `features/settings`, `features/assets` |
| Gallery | `assets/gallery.rs` | `features/gallery` |
| Fonts | `assets/fonts.rs` | `features/settings` |
| Default game assets | `assets/default_assets.rs`, `game/assets` | `features/game` |
| TTS audio cache | `integrations/tts/cache.rs` | playback in `features/tts` |

## Sidecar

Sidecar is excluded from the active Tauri migration. Do not keep sidecar UI, commands, or placeholder status surfaces in the app until that scope is intentionally reopened.

## Integrations

| Feature | Rust Target | Frontend Target |
| --- | --- | --- |
| Spotify OAuth | `integrations/spotify/oauth.rs` | `features/spotify` |
| Spotify playback/tools | `integrations/spotify/playback.rs`, `tools.rs` | `features/spotify`, agent tool UI |
| Haptic devices | `integrations/haptic` | `features/haptic` |
| TTS | `integrations/tts` | `features/tts` |
| Translation | `integrations/translation` | `features/translation` |
| GIF search | `integrations/gifs` | gif picker UI |
| Bot browser Chub | `integrations/bot_browser/chub.rs` | `features/bot-browser` |
| Bot browser Janny | `integrations/bot_browser/janny.rs` | `features/bot-browser` |
| Bot browser Chartavern | `integrations/bot_browser/chartavern.rs` | `features/bot-browser` |
| Bot browser Pygmalion | `integrations/bot_browser/pygmalion.rs` | `features/bot-browser` |
| Bot browser Wyvern | `integrations/bot_browser/wyvern.rs` | `features/bot-browser` |
| Bot browser Datacat | `integrations/bot_browser/datacat.rs` | `features/bot-browser` |
| Discord webhook | `integrations/discord/webhook.rs` | settings/agent UI |
| Home Assistant | `integrations/home_assistant/webhook.rs` | optional settings UI |

## Admin, Updates, Extensions, Themes

| Feature | Rust Target | Frontend Target |
| --- | --- | --- |
| Admin privileged operations | `security/admin_gate.rs` | settings prompt only |
| Updates | `updates` | `features/updates` |
| Themes | `storage/repositories/themes.rs` | `features/themes` |
| Extensions | `storage/repositories/extensions.rs`, `security/permissions.rs` | `features/extensions` |
| Custom CSS preview | no Rust needed except persistence | `features/themes` |

## Sync And Docker Deployment

Sync is excluded from the active Tauri migration. Do not keep sync settings, pairing, queue, conflict review, or Docker sync UI in the app until that scope is intentionally reopened.
