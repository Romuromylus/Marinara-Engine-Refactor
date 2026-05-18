# Mermaid Diagrams

These diagrams mirror the visual HTML pages. They are kept in Markdown so they can render in GitHub and be copied into issues, PRs, or external docs.

## Platform Map

```mermaid
flowchart LR
  subgraph Desktop["Tauri desktop app"]
    UI["React UI<br/>features/*"]
    API["shared/api<br/>typed invoke + events"]
    Host["Tauri host<br/>commands, events, state"]
    Rust["Rust domain crates<br/>services + repositories"]
    LocalData[("Local app data<br/>storage + blobs")]
    Sidecar["Local AI sidecar<br/>llama.cpp / MLX"]
    Native["Native devices<br/>filesystem, haptics"]
  end

  subgraph Providers["External services"]
    LLM["LLM APIs"]
    Image["Image APIs"]
    Spotify["Spotify"]
    TTS["TTS / translation"]
    Bot["Bot-browser sources"]
  end

  subgraph Docker["Optional Docker sync"]
    SyncAPI["sync-api<br/>Rust + Axum"]
    SyncDb[("Raw metadata files or Postgres")]
    Blob[("Local FS / S3 / MinIO")]
    Redis[("Redis optional")]
  end

  UI --> API --> Host --> Rust
  Rust --> LocalData
  Rust --> Sidecar
  Rust --> Native
  Rust --> LLM
  Rust --> Image
  Rust --> Spotify
  Rust --> TTS
  Rust --> Bot
  Rust -. device auth, changes, blobs .-> SyncAPI
  SyncAPI --> SyncDb
  SyncAPI --> Blob
  SyncAPI -. optional fanout .-> Redis
  SyncAPI -. heads + conflicts .-> Rust
```

## Rust Module Dependency Graph

```mermaid
flowchart TB
  Commands["src-tauri/src/commands<br/>thin IPC layer"]
  Events["src-tauri/src/events<br/>typed event emitters"]
  Core["core<br/>errors, config, IDs, event bus"]
  Security["security<br/>secrets, paths, safe fetch"]
  Storage["storage<br/>repositories, current Tauri data migrations"]

  LLM["llm<br/>providers, images, embeddings"]
  Generation["generation<br/>prompt + streaming pipeline"]
  Agents["agents<br/>tools, memory, knowledge"]
  Chat["chat<br/>messages, swipes, folders"]
  Conversation["conversation<br/>schedules, awareness"]
  Roleplay["roleplay<br/>scene, sprites, encounters"]
  Game["game<br/>turns, mechanics, world"]
  Assets["assets<br/>avatars, gallery, fonts"]
  Imports["import<br/>ST import, current profile packages"]
  Integrations["integrations<br/>spotify, haptic, tts, translation, bot browser"]
  Updates["updates<br/>check + apply"]

  Commands --> Generation
  Commands --> Chat
  Commands --> Game
  Commands --> Roleplay
  Commands --> Integrations
  Events --> Core

  Generation --> LLM
  Generation --> Agents
  Generation --> Chat
  Generation --> Storage
  Generation --> Security

  Agents --> LLM
  Agents --> Storage
  Agents --> Integrations

  Conversation --> Chat
  Roleplay --> Generation
  Roleplay --> Assets
  Game --> Generation
  Game --> Assets
  Game --> Sidecar

  LLM --> Security
  Sidecar --> Security
  Imports --> Security
  Imports --> Storage
  Assets --> Storage
  Updates --> Security
  SyncClient --> SyncProtocol
  SyncClient --> Storage
  SyncClient --> Security

  Security --> Core
  Storage --> Core
```

## Generation Runtime Sequence

```mermaid
sequenceDiagram
  participant UI as React feature hook
  participant API as shared/api client
  participant CMD as Tauri command
  participant GEN as generation
  participant AG as agents
  participant LLM as llm
  participant STORE as storage
  participant EVT as Tauri events

  UI->>API: generation_start(input)
  API->>CMD: invoke("generation_start")
  CMD->>GEN: start_generation(input, app_state)
  GEN->>STORE: load chat, settings, lorebooks, presets
  GEN->>AG: prepare enabled agents
  GEN->>LLM: stream provider request
  loop provider tokens
    LLM-->>GEN: token or provider event
    GEN-->>EVT: generation://token
    EVT-->>UI: append token
  end
  GEN->>AG: run post-generation agents
  GEN->>STORE: persist message, swipes, metadata
  GEN-->>EVT: generation://done
  EVT-->>UI: invalidate queries and finalize
```

## Self-Hosted Sync Topology

```mermaid
flowchart LR
  subgraph DeviceA["Desktop A"]
    AUI["React sync UI"]
    AClient["sync-client"]
    AStore[("Local storage")]
    AQueue[("Sync queue")]
  end

  subgraph DeviceB["Desktop B"]
    BUI["React sync UI"]
    BClient["sync-client"]
    BStore[("Local storage")]
    BQueue[("Sync queue")]
  end

  subgraph Server["Docker sync server"]
    API["Axum API<br/>REST + WebSocket"]
    Auth["Auth + devices"]
    Sync["Sync service<br/>heads + operations"]
    Conflicts["Conflict service"]
    Db[("Raw metadata files or Postgres")]
    Blobs[("Local FS / S3 / MinIO")]
    Redis[("Redis optional")]
  end

  AUI --> AClient
  AClient --> AStore
  AClient --> AQueue
  BUI --> BClient
  BClient --> BStore
  BClient --> BQueue

  AClient <-->|pair, push, pull, blobs, heads| API
  BClient <-->|pair, push, pull, blobs, heads| API
  API --> Auth
  API --> Sync
  Sync --> Conflicts
  Sync --> Db
  Auth --> Db
  Conflicts --> Db
  API --> Blobs
  API -. websocket fanout .-> Redis
```

## Sync Lifecycle

```mermaid
sequenceDiagram
  participant UI as React UI
  participant LOCAL as Local Rust repositories
  participant QUEUE as Sync queue
  participant CLIENT as Sync client
  participant SERVER as Docker sync API
  participant DB as Sync metadata DB
  participant BLOB as Blob store

  UI->>LOCAL: save chat/message/file/settings
  LOCAL->>QUEUE: append operation + blob refs
  CLIENT->>SERVER: push operations since cursor
  SERVER->>DB: validate and store changes
  SERVER-->>CLIENT: ack heads + missing blob refs
  CLIENT->>BLOB: upload missing blobs
  SERVER-->>CLIENT: websocket heads update
  CLIENT->>SERVER: pull missing operations
  SERVER->>DB: read changes by collection
  SERVER-->>CLIENT: changes + conflict markers
  CLIENT->>LOCAL: merge safe changes
  CLIENT-->>UI: sync://status and sync://conflict
```

## Data Classification

```mermaid
flowchart TB
  Data["Marinara local data"] --> Metadata["Metadata operations"]
  Data --> Settings["Settings"]
  Data --> Files["Blob files"]
  Data --> Secrets["Secrets"]
  Data --> LocalOnly["Local-only runtime data"]

  Metadata --> CRDT["CRDT or append-only log<br/>chats, messages, characters, lorebooks, game notes"]
  Settings --> LWW["LWW + revision history<br/>UI prefs, haptic, TTS, translation, generation params"]
  Files --> CAS["Content-addressed blobs<br/>avatars, gallery, sprites, backgrounds, current profile packages"]
  Secrets --> NoSync["Do not sync by default<br/>API keys, OAuth refresh tokens"]
  LocalOnly --> Ignore["Never sync<br/>sidecar binaries, runtime installs, temp logs"]
```
