import type {
  Chat,
  Combatant,
  CombatPlayerAction,
  GameActiveState,
  GameMap,
  GameNpc,
  GameSetupConfig,
  HudWidget,
  RPGAttributes,
  SessionSummary,
} from "@marinara-engine/shared";
import { api } from "../../../shared/api/api-client";
import { llmApi } from "../../../shared/api/llm-api";
import {
  addCombatEntry,
  addEventEntry,
  addInventoryEntry,
  addLocationEntry,
  addNoteEntry,
  buildDeterministicSummary,
  buildStructuredRecap,
  createInitialTime,
  createJournal,
  formatGameTime,
  generateCombatLoot,
  generateLootTable,
  generateWeather,
  getGoverningAttribute,
  inferBiome,
  mapSheetAttributesToRPG,
  processReputationActions,
  resolveCombatRound,
  resolveSkillCheck,
  rollDice as rollGameDice,
  rollEncounter as rollGameEncounter,
  rollEnemyCount,
  validateTransition,
  advanceTime as advanceGameTime,
  withActiveGameMapMeta,
  type GameTime,
  type Journal,
  type JournalEntry,
  type LootDrop,
  type WeatherState,
} from "../../../engine/modes/game";

export interface CreateGameResponse {
  sessionChat: Chat;
  gameId: string;
}

export interface SetupResponse {
  setup: Record<string, unknown>;
  worldOverview: string | null;
}

export interface StartGameResponse {
  status: string;
  alreadyStarted?: boolean;
}

export interface StartSessionResponse {
  sessionChat: Chat;
  sessionNumber: number;
  recap: string;
}

export interface SessionSummaryResponse {
  summary: SessionSummary;
}

export interface RegenerateSessionLorebookResponse {
  sessionNumber: number;
  lorebookId: string;
  entryCount: number;
}

export interface UpdateCampaignProgressionResponse {
  sessionChat: Chat;
  gameId: string;
  campaignProgression: {
    storyArc: string | null;
    plotTwists: string[];
    partyArcs: unknown[];
  };
}

export interface PartyCardResponse {
  sessionChat: Chat;
  added?: boolean;
  removed?: boolean;
  characterName: string;
  cardCreated?: boolean;
  gameCard?: unknown;
}

export interface MapResponse {
  map: GameMap;
  maps?: GameMap[];
  activeGameMapId?: string | null;
}

export interface GameJournalResponse {
  journal: Journal;
  recap: string;
  playerNotes?: string;
}

export interface GameImagePromptReviewItem {
  id: string;
  kind: "background" | "illustration" | "portrait";
  title: string;
  prompt: string;
  width: number;
  height: number;
}

export interface GameAssetGenerationResult {
  generatedBackground: string | null;
  fallbackBackground: string | null;
  generatedIllustration: { tag: string; segment?: number | null } | null;
  generatedNpcAvatars: Array<{ name: string; avatarUrl: string }>;
}

export type GameAssetGenerationPayload = {
  chatId: string;
  backgroundTag?: string;
  npcsNeedingAvatars?: Array<{ name: string; description: string }>;
  forceNpcAvatarNames?: string[];
  illustration?: Record<string, unknown> | null;
  imageConnectionId?: string | null;
  artStylePrompt?: string | null;
  imageSizes?: Record<string, { width?: number; height?: number }>;
  promptOverrides?: PromptOverride[];
  [key: string]: unknown;
};

type ChatMessage = {
  id?: string;
  role?: string;
  content?: string;
  [key: string]: unknown;
};

type PromptOverride = {
  id?: string;
  prompt?: string;
};

const EMPTY_JOURNAL: Journal = createJournal();

function newId(prefix = ""): string {
  const id =
    typeof crypto !== "undefined" && "randomUUID" in crypto
      ? crypto.randomUUID()
      : `${Date.now().toString(36)}-${Math.random().toString(36).slice(2)}`;
  return prefix ? `${prefix}-${id}` : id;
}

function nowIso(): string {
  return new Date().toISOString();
}

function asRecord(value: unknown): Record<string, unknown> {
  if (typeof value === "string") {
    try {
      const parsed = JSON.parse(value);
      return parsed && typeof parsed === "object" && !Array.isArray(parsed) ? (parsed as Record<string, unknown>) : {};
    } catch {
      return {};
    }
  }
  return value && typeof value === "object" && !Array.isArray(value) ? (value as Record<string, unknown>) : {};
}

function chatMeta(chat: Chat | null | undefined): Record<string, unknown> {
  return asRecord(chat?.metadata);
}

async function getChat(chatId: string): Promise<Chat> {
  return api.get<Chat>(`/chats/${encodeURIComponent(chatId)}`);
}

async function patchChatMetadata(chatId: string, patch: Record<string, unknown>): Promise<Chat> {
  return api.patch<Chat>(`/chats/${encodeURIComponent(chatId)}/metadata`, patch);
}

async function listMessages(chatId: string, limit?: number): Promise<ChatMessage[]> {
  const query = limit ? `?limit=${encodeURIComponent(String(limit))}` : "";
  return api.get<ChatMessage[]>(`/chats/${encodeURIComponent(chatId)}/messages${query}`);
}

function parseJsonObject(text: string): Record<string, unknown> | null {
  try {
    const parsed = JSON.parse(text);
    return parsed && typeof parsed === "object" && !Array.isArray(parsed) ? (parsed as Record<string, unknown>) : null;
  } catch {
    const fenced = text.match(/```(?:json)?\s*([\s\S]*?)```/i)?.[1];
    if (fenced) return parseJsonObject(fenced);
    const start = text.indexOf("{");
    const end = text.lastIndexOf("}");
    if (start >= 0 && end > start) {
      try {
        const parsed = JSON.parse(text.slice(start, end + 1));
        return parsed && typeof parsed === "object" && !Array.isArray(parsed) ? (parsed as Record<string, unknown>) : null;
      } catch {
        return null;
      }
    }
    return null;
  }
}

function fallbackGameBlueprint(preferences: string): Record<string, unknown> {
  const overview = preferences.trim()
    ? `A local campaign shaped around: ${preferences.trim()}`
    : "A flexible local campaign ready for play.";
  return {
    worldOverview: overview,
    hudWidgets: [
      { id: "party", type: "party", title: "Party", enabled: true },
      { id: "journal", type: "journal", title: "Journal", enabled: true },
      { id: "inventory", type: "inventory", title: "Inventory", enabled: true },
    ],
    introSequence: ["Frame the opening situation clearly.", "Invite the player to choose the first action."],
    visualTheme: { palette: "default", uiStyle: "classic", moodDefault: "neutral" },
    campaignPlan: {
      questSeeds: [],
      encounterPrinciples: ["Keep conflicts actionable.", "Let player choices alter the world state."],
    },
  };
}

function defaultGameMap(name = "Starting Area", description = "The party's current area."): GameMap {
  return {
    id: newId("map"),
    type: "grid",
    name,
    description,
    width: 3,
    height: 3,
    cells: [
      {
        x: 1,
        y: 1,
        emoji: "Start",
        label: "Start",
        discovered: true,
        terrain: "safe",
        description: "The party's starting point.",
      },
    ],
    partyPosition: { x: 1, y: 1 },
  } as GameMap;
}

function gameTimeFromMeta(meta: Record<string, unknown>): GameTime {
  const raw = asRecord(meta.gameTime);
  const day = Number(raw.day ?? 1);
  const hour = Number(raw.hour ?? 8);
  const minute = Number(raw.minute ?? 0);
  return {
    day: Number.isFinite(day) && day >= 1 ? day : 1,
    hour: Number.isFinite(hour) ? Math.max(0, Math.min(23, Math.floor(hour))) : 8,
    minute: Number.isFinite(minute) ? Math.max(0, Math.min(59, Math.floor(minute))) : 0,
  };
}

function journalFromMeta(meta: Record<string, unknown>): Journal {
  const raw = asRecord(meta.gameJournal);
  return {
    entries: Array.isArray(raw.entries) ? (raw.entries as Journal["entries"]) : [],
    quests: Array.isArray(raw.quests) ? (raw.quests as Journal["quests"]) : [],
    locations: Array.isArray(raw.locations) ? (raw.locations as string[]) : [],
    npcLog: Array.isArray(raw.npcLog) ? (raw.npcLog as Journal["npcLog"]) : [],
    inventoryLog: Array.isArray(raw.inventoryLog) ? (raw.inventoryLog as Journal["inventoryLog"]) : [],
  };
}

function sessionSummary(sessionNumber: number, meta: Record<string, unknown>): SessionSummary {
  const journal = journalFromMeta(meta);
  const npcs = Array.isArray(meta.gameNpcs) ? (meta.gameNpcs as GameNpc[]) : [];
  const map = (meta.gameMap as GameMap | null) ?? null;
  return {
    ...buildDeterministicSummary(journal, sessionNumber, npcs, map),
    nextSessionRequest: null,
    timestamp: nowIso(),
  } as SessionSummary;
}

function normalizeJournalEntry(type: string, data: Record<string, unknown>): Pick<JournalEntry, "type" | "title" | "content"> {
  const title =
    typeof data.title === "string"
      ? data.title
      : typeof data.name === "string"
        ? data.name
        : type === "location"
          ? "Location"
          : type === "npc"
            ? "NPC"
            : type === "combat"
              ? "Combat"
              : type === "item"
                ? "Item"
                : type === "quest"
                  ? "Quest"
                  : type === "note"
                    ? "Note"
                    : "Event";
  const content =
    typeof data.content === "string" ? data.content : typeof data.description === "string" ? data.description : "";
  return { type: type as JournalEntry["type"], title, content };
}

function applyJournalEntry(journal: Journal, type: string, data: Record<string, unknown>): Journal {
  if (type === "location") {
    const { title, content } = normalizeJournalEntry(type, data);
    return addLocationEntry(journal, title, content);
  }
  if (type === "combat") {
    const { content } = normalizeJournalEntry(type, data);
    const outcome =
      data.outcome === "defeat" || data.outcome === "fled" || data.result === "defeat" || data.result === "fled"
        ? (data.outcome ?? data.result)
        : "victory";
    return addCombatEntry(journal, content, outcome as "victory" | "defeat" | "fled");
  }
  if (type === "item") {
    const item = typeof data.name === "string" ? data.name : typeof data.title === "string" ? data.title : "Item";
    const action =
      data.action === "used" || data.action === "lost" || data.action === "removed" ? data.action : "acquired";
    const quantity = Number(data.quantity ?? 1);
    return addInventoryEntry(journal, item, action, Number.isFinite(quantity) ? quantity : 1);
  }
  if (type === "note") {
    const { title, content } = normalizeJournalEntry(type, data);
    const readableType = data.readableType === "book" ? "book" : "note";
    return addNoteEntry(journal, title, content, {
      readableType,
      sourceMessageId: typeof data.sourceMessageId === "string" ? data.sourceMessageId : undefined,
      sourceSegmentIndex: Number.isInteger(data.sourceSegmentIndex) ? (data.sourceSegmentIndex as number) : undefined,
    });
  }
  const { title, content } = normalizeJournalEntry(type, data);
  return addEventEntry(journal, title, content);
}

function buildGameCard(characterName: string): Record<string, unknown> {
  return {
    name: characterName,
    shortDescription: "",
    class: "Adventurer",
    abilities: ["Attack", "Assist"],
    strengths: [],
    weaknesses: [],
    extra: {},
    rpgStats: {
      attributes: [
        { name: "STR", value: 10 },
        { name: "DEX", value: 10 },
        { name: "CON", value: 10 },
        { name: "INT", value: 10 },
        { name: "WIS", value: 10 },
        { name: "CHA", value: 10 },
      ],
      hp: { value: 20, max: 20 },
    },
  };
}

function playerAttributes(meta: Record<string, unknown>): Partial<RPGAttributes> {
  const cards = Array.isArray(meta.gameCharacterCards) ? meta.gameCharacterCards : [];
  const first = asRecord(cards[0]);
  const rpgStats = asRecord(first.rpgStats);
  return mapSheetAttributesToRPG(Array.isArray(rpgStats.attributes) ? (rpgStats.attributes as any[]) : undefined);
}

function generatedAssetSlug(value: string): string {
  const slug = value
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/(^-|-$)/g, "")
    .slice(0, 80);
  return slug || `generated-${Date.now()}`;
}

function imageReviewId(kind: string, key: string): string {
  return `${kind}:${generatedAssetSlug(key)}`;
}

function promptOverride(payload: Record<string, unknown>, id: string): string | null {
  const overrides = Array.isArray(payload.promptOverrides) ? (payload.promptOverrides as PromptOverride[]) : [];
  const override = overrides.find((item) => item.id === id && typeof item.prompt === "string" && item.prompt.trim());
  return override?.prompt?.trim() ?? null;
}

function imageSize(payload: Record<string, unknown>, bucket: string, axis: "width" | "height", fallback: number): number {
  const bucketSize = asRecord(asRecord(payload.imageSizes)[bucket]);
  const value = Number(bucketSize[axis]);
  return Number.isFinite(value) && value >= 128 && value <= 2048 ? value : fallback;
}

function sceneAssetPrompt(kind: string, label: string, detail: string, artStyle: string): string {
  const style = artStyle.trim() || "polished fantasy visual novel art, cinematic lighting, high detail";
  if (kind === "background") {
    return `Wide establishing background of ${label}. ${detail}. ${style}. No characters, no text, immersive environment art.`;
  }
  if (kind === "illustration") {
    return `Cinematic scene illustration: ${label}. ${detail}. ${style}. Dynamic composition, no text, high detail.`;
  }
  return `Portrait of ${label}. ${detail}. ${style}. Centered bust portrait, expressive face, clean readable silhouette, no text.`;
}

function assetTagFromPath(path: string): string {
  return path.replace(/\.[^.]+$/, "").replace(/[\\/]/g, ":");
}

function imageExt(mimeType: string): string {
  if (mimeType.includes("png")) return "png";
  if (mimeType.includes("webp")) return "webp";
  return "jpg";
}

function base64File(base64: string, name: string, type: string): File {
  const binary = atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let index = 0; index < binary.length; index += 1) bytes[index] = binary.charCodeAt(index);
  return new File([bytes], name, { type });
}

async function uploadGeneratedAsset(
  category: string,
  subcategory: string,
  slug: string,
  base64: string,
  mimeType: string,
): Promise<string> {
  const form = new FormData();
  form.set("category", category);
  form.set("subcategory", subcategory);
  form.set("file", base64File(base64, `${slug}.${imageExt(mimeType)}`, mimeType));
  const uploaded = await api.upload<{ item?: { path?: string } }>("/game-assets/upload", form);
  const path = uploaded.item?.path;
  if (!path) throw new Error("Generated asset path missing.");
  return assetTagFromPath(path);
}

function spotifyQuery(payload: Record<string, unknown>): string {
  const text = [payload.narration, payload.playerAction].filter((value): value is string => typeof value === "string").join(" ");
  const words = text.split(/[^a-zA-Z0-9]+/).filter((word) => word.length > 3).slice(0, 8);
  return words.length ? words.join(" ") : "cinematic adventure soundtrack";
}

function recentSpotifyTracks(payload: Record<string, unknown>): string[] {
  const context = asRecord(payload.context);
  return Array.isArray(context.recentSpotifyTracks)
    ? context.recentSpotifyTracks.filter((uri): uri is string => typeof uri === "string" && uri.startsWith("spotify:track:"))
    : [];
}

async function llmJson(input: {
  connectionId?: string | null;
  system: string;
  user: string;
  fallback: Record<string, unknown>;
  parameters?: Record<string, unknown>;
}): Promise<Record<string, unknown>> {
  if (!input.connectionId) return input.fallback;
  try {
    const raw = await llmApi.complete({
      connectionId: input.connectionId,
      messages: [
        { role: "system", content: input.system },
        { role: "user", content: input.user },
      ],
      parameters: input.parameters,
    });
    return parseJsonObject(raw) ?? input.fallback;
  } catch {
    return input.fallback;
  }
}

async function sessionTranscript(chatId: string, limit = 80): Promise<string> {
  const messages = await listMessages(chatId, limit);
  return messages
    .map((message) => `${message.role ?? "message"}: ${message.content ?? ""}`)
    .filter((line) => line.trim())
    .join("\n");
}

export const gameApi = {
  async createGame(data: {
    name: string;
    setupConfig: GameSetupConfig;
    connectionId?: string;
    characterConnectionId?: string;
    promptPresetId?: string;
    chatId?: string;
    partyCharacterIds?: string[];
  }): Promise<CreateGameResponse> {
    const gameId = newId("game");
    if (data.chatId) {
      const sessionChat = await patchChatMetadata(data.chatId, {
        gameId,
        gameSessionNumber: 1,
        gameSessionStatus: "setup",
        gameSetupConfig: data.setupConfig,
      });
      return { sessionChat, gameId };
    }
    const sessionChat = await api.post<Chat>("/chats", {
      name: data.name || "New Game",
      mode: "game",
      characterIds: data.partyCharacterIds ?? [],
      connectionId: data.connectionId ?? null,
      metadata: {
        gameId,
        gameSessionNumber: 1,
        gameSessionStatus: "setup",
        gameSetupConfig: data.setupConfig,
        gameJournal: createJournal(),
      },
    });
    return { sessionChat, gameId };
  },

  async setupGame(data: { chatId: string; connectionId?: string; preferences: string; setupConfig?: GameSetupConfig }): Promise<SetupResponse> {
    const fallback = fallbackGameBlueprint(data.preferences);
    const setup = await llmJson({
      connectionId: data.connectionId,
      fallback,
      system:
        "Create a game-mode setup blueprint for a roleplay campaign. Return strict JSON only with worldOverview, hudWidgets, introSequence, visualTheme, and campaignPlan.",
      user: `Player preferences:\n${data.preferences}`,
      parameters: { temperature: 0.7, maxTokens: 2200 },
    });
    const worldOverview =
      typeof setup.worldOverview === "string"
        ? setup.worldOverview
        : typeof setup.overview === "string"
          ? setup.overview
          : (fallback.worldOverview as string);
    const map = defaultGameMap();
    await patchChatMetadata(data.chatId, {
      gameSetupConfig: data.setupConfig ?? data.preferences ?? null,
      gameSessionStatus: "ready",
      gameBlueprint: setup,
      gameMap: map,
      gameMaps: [map],
      activeGameMapId: map.id ?? null,
      enableSpriteGeneration: Boolean((data.setupConfig as Record<string, unknown> | undefined)?.enableSpriteGeneration),
      gameImageConnectionId: (data.setupConfig as Record<string, unknown> | undefined)?.imageConnectionId ?? null,
      gameTime: createInitialTime(),
      gameJournal: createJournal(),
    });
    return { setup, worldOverview };
  },

  async startGame(data: { chatId: string }): Promise<StartGameResponse> {
    await patchChatMetadata(data.chatId, { gameSessionStatus: "active", gameActiveState: "exploration" });
    return { status: "active", alreadyStarted: false };
  },

  async startSession(data: { gameId: string; connectionId?: string }): Promise<StartSessionResponse> {
    const chats = await api.get<Chat[]>("/chats");
    const existing = chats.filter((chat) => chatMeta(chat).gameId === data.gameId);
    const sessionNumber = existing.length + 1;
    const previousMeta = chatMeta(existing[existing.length - 1]);
    const sessionChat = await api.post<Chat>("/chats", {
      name: `Game Session ${sessionNumber}`,
      mode: "game",
      characterIds: [],
      connectionId: data.connectionId ?? null,
      metadata: {
        gameId: data.gameId,
        gameSessionNumber: sessionNumber,
        gameSessionStatus: "active",
        gameActiveState: "exploration",
        gamePreviousSessionSummaries: Array.isArray(previousMeta.gamePreviousSessionSummaries)
          ? previousMeta.gamePreviousSessionSummaries
          : [],
        gameJournal: createJournal(),
      },
    });
    return { sessionChat, sessionNumber, recap: "" };
  },

  async concludeSession(data: { chatId: string; connectionId?: string; nextSessionRequest?: string; summary?: SessionSummary }): Promise<SessionSummaryResponse> {
    const chat = await getChat(data.chatId);
    const meta = chatMeta(chat);
    const sessionNumber = Number(meta.gameSessionNumber ?? 1);
    const summary = {
      ...(data.summary ?? sessionSummary(sessionNumber, meta)),
      nextSessionRequest: data.nextSessionRequest ?? (data.summary as { nextSessionRequest?: string } | undefined)?.nextSessionRequest ?? null,
      timestamp: (data.summary as { timestamp?: string } | undefined)?.timestamp ?? nowIso(),
    } as SessionSummary;
    const summaries = Array.isArray(meta.gamePreviousSessionSummaries)
      ? [...(meta.gamePreviousSessionSummaries as SessionSummary[])]
      : [];
    const nextSummaries = summaries.filter((item) => item.sessionNumber !== sessionNumber).concat(summary);
    await patchChatMetadata(data.chatId, {
      gameSessionStatus: "concluded",
      gamePreviousSessionSummaries: nextSummaries,
    });
    return { summary };
  },

  async regenerateSessionLorebook(data: { chatId: string; sessionNumber: number; connectionId?: string }): Promise<RegenerateSessionLorebookResponse> {
    const transcript = await sessionTranscript(data.chatId);
    const fallbackEntries = transcript.trim()
      ? [{ name: `Session ${data.sessionNumber} Recap`, content: transcript.split("\n").slice(0, 12).join("\n"), keys: [`session ${data.sessionNumber}`, "recap", "campaign"] }]
      : [{ name: `Session ${data.sessionNumber} State`, content: "No transcript was available; preserve the current campaign state from the chat metadata.", keys: [`session ${data.sessionNumber}`] }];
    const parsed = await llmJson({
      connectionId: data.connectionId,
      fallback: { entries: fallbackEntries },
      system:
        "Extract durable campaign lore from the session transcript. Return strict JSON with an entries array; each entry has name, content, and keys array.",
      user: transcript,
      parameters: { temperature: 0.3, maxTokens: 2500 },
    });
    const entries = Array.isArray(parsed.entries) && parsed.entries.length ? parsed.entries : fallbackEntries;
    const lorebook = await api.post<{ id: string }>("/lorebooks", {
      name: `Game Session ${data.sessionNumber} Lore`,
      description: "Generated from local game session state.",
      category: "game",
      chatId: data.chatId,
      enabled: true,
      generatedBy: "game-session",
    });
    let entryCount = 0;
    for (const [index, rawEntry] of entries.entries()) {
      const entry = asRecord(rawEntry);
      await api.post("/lorebook-entries", {
        lorebookId: lorebook.id,
        name: typeof entry.name === "string" ? entry.name : "Session Lore",
        content: typeof entry.content === "string" ? entry.content : "",
        keys: Array.isArray(entry.keys) ? entry.keys : [`session ${data.sessionNumber}`],
        secondaryKeys: [],
        enabled: true,
        constant: false,
        selective: false,
        order: index,
        sortOrder: index,
        position: 0,
        role: "system",
        excludeFromVectorization: false,
      });
      entryCount += 1;
    }
    await patchChatMetadata(data.chatId, {
      gameSessionLorebookId: lorebook.id,
      gameSessionLorebookEntryCount: entryCount,
    });
    return { sessionNumber: data.sessionNumber, lorebookId: lorebook.id, entryCount };
  },

  async updateCampaignProgression(data: { chatId: string; sessionNumber: number; connectionId?: string }): Promise<UpdateCampaignProgressionResponse> {
    const chat = await getChat(data.chatId);
    const meta = chatMeta(chat);
    const transcript = await sessionTranscript(data.chatId);
    const fallback = {
      storyArc: transcript.trim() ? `Session ${data.sessionNumber} advanced the campaign.` : null,
      plotTwists: [],
      partyArcs: [],
    };
    const campaignProgression = (await llmJson({
      connectionId: data.connectionId,
      fallback,
      system: "Update campaign progression from this game session. Return strict JSON with storyArc, plotTwists, and partyArcs.",
      user: transcript,
      parameters: { temperature: 0.4, maxTokens: 1800 },
    })) as UpdateCampaignProgressionResponse["campaignProgression"];
    const sessionChat = await patchChatMetadata(data.chatId, {
      gameCampaignProgression: campaignProgression,
      gameCampaignProgressionUpdatedAt: nowIso(),
    });
    return { sessionChat, gameId: String(meta.gameId ?? ""), campaignProgression };
  },

  async upsertPartyCard(data: { chatId: string; characterName: string; characterId?: string; connectionId?: string; added?: boolean }): Promise<PartyCardResponse> {
    const chat = await getChat(data.chatId);
    const meta = chatMeta(chat);
    const cards = Array.isArray(meta.gameCharacterCards) ? [...meta.gameCharacterCards] : [];
    const card = buildGameCard(data.characterName);
    const nextCards = cards.filter((item) => asRecord(item).name !== data.characterName).concat(card);
    const sessionChat = await patchChatMetadata(data.chatId, { gameCharacterCards: nextCards });
    return {
      sessionChat,
      added: data.added,
      characterName: data.characterName,
      cardCreated: true,
      gameCard: card,
    };
  },

  async removePartyMember(data: { chatId: string; characterName: string }): Promise<PartyCardResponse> {
    const chat = await getChat(data.chatId);
    const meta = chatMeta(chat);
    const cards = Array.isArray(meta.gameCharacterCards) ? [...meta.gameCharacterCards] : [];
    const nextCards = cards.filter((item) => asRecord(item).name !== data.characterName);
    const sessionChat = await patchChatMetadata(data.chatId, { gameCharacterCards: nextCards });
    return { sessionChat, removed: nextCards.length !== cards.length, characterName: data.characterName };
  },

  async rollDice(data: { notation: string }) {
    return { result: rollGameDice(data.notation) };
  },

  async skillCheck(data: {
    chatId: string;
    skill: string;
    dc: number;
    advantage?: boolean;
    disadvantage?: boolean;
    preRolledD20?: number;
    skillModifier?: number;
  }) {
    const meta = chatMeta(await getChat(data.chatId));
    const attrs = playerAttributes(meta);
    const attr = getGoverningAttribute(data.skill);
    const attrScore = Number(attrs[attr] ?? 10);
    return {
      result: resolveSkillCheck({
        skill: data.skill,
        dc: data.dc,
        skillModifier: Number(data.skillModifier ?? 0),
        attributeModifier: Math.floor((attrScore - 10) / 2),
        advantage: data.advantage,
        disadvantage: data.disadvantage,
        preRolledD20: data.preRolledD20,
      }),
      updatedContent: undefined as string | undefined,
    };
  },

  async transitionGameState(data: { chatId: string; newState: GameActiveState }) {
    const meta = chatMeta(await getChat(data.chatId));
    const previousState = (meta.gameActiveState as GameActiveState | undefined) ?? "exploration";
    const newState = validateTransition(previousState, data.newState);
    await patchChatMetadata(data.chatId, { gameActiveState: newState });
    return { previousState, newState };
  },

  async generateMap(data: { chatId: string; locationType: string; context: string }): Promise<MapResponse> {
    const map = defaultGameMap(data.locationType || "Area", data.context || "");
    const chat = await getChat(data.chatId);
    const meta = withActiveGameMapMeta(chatMeta(chat), map);
    await patchChatMetadata(data.chatId, meta);
    return { map, maps: [map], activeGameMapId: map.id ?? null };
  },

  async moveOnMap(data: { chatId: string; position: { x: number; y: number } | string; mapId?: string | null }): Promise<MapResponse> {
    const chat = await getChat(data.chatId);
    const meta = chatMeta(chat);
    const maps = Array.isArray(meta.gameMaps) ? (meta.gameMaps as GameMap[]) : [];
    const current = (maps.find((map) => map.id === data.mapId) ?? (meta.gameMap as GameMap | undefined) ?? defaultGameMap()) as GameMap;
    const map = { ...current, partyPosition: data.position } as GameMap;
    const nextMeta = withActiveGameMapMeta(meta, map);
    await patchChatMetadata(data.chatId, nextMeta);
    return {
      map,
      maps: Array.isArray(nextMeta.gameMaps) ? (nextMeta.gameMaps as GameMap[]) : [map],
      activeGameMapId: typeof nextMeta.activeGameMapId === "string" ? nextMeta.activeGameMapId : (map.id ?? null),
    };
  },

  async updateWidgets(data: { chatId: string; widgets: HudWidget[] }) {
    await patchChatMetadata(data.chatId, { gameWidgetState: data.widgets });
    return { ok: true };
  },

  async gameSessions(gameId: string): Promise<Chat[]> {
    const chats = await api.get<Chat[]>("/chats");
    return chats.filter((chat) => chatMeta(chat).gameId === gameId);
  },

  async combatRound(data: {
    combatants: Array<Omit<Combatant, "sprite">>;
    round: number;
    playerAction?: CombatPlayerAction;
    mechanics?: import("@marinara-engine/shared").CombatMechanic[];
  }) {
    const combatants = data.combatants.map((combatant) => ({ ...combatant })) as any[];
    const result = resolveCombatRound(combatants, data.round, "normal", undefined, data.playerAction as any, data.mechanics);
    return { result, combatants: combatants as Combatant[] };
  },

  async combatLoot(data: { enemyCount: number; difficulty?: string }) {
    return { drops: generateCombatLoot(data.enemyCount, data.difficulty ?? "normal") };
  },

  async lootGenerate(data: { count?: number; difficulty?: string }): Promise<{ drops: LootDrop[] }> {
    return { drops: generateLootTable(Math.max(0, Math.min(10, data.count ?? 1)), data.difficulty ?? "normal") };
  },

  async advanceTime(data: { chatId: string; action: string }): Promise<{ time: GameTime; formatted: string }> {
    const meta = chatMeta(await getChat(data.chatId));
    const time = advanceGameTime(gameTimeFromMeta(meta), data.action);
    const formatted = formatGameTime(time);
    await patchChatMetadata(data.chatId, { gameTime: time, gameTimeFormatted: formatted });
    return { time, formatted };
  },

  async updateWeather(data: { chatId: string; action: string; location?: string; season?: string; type?: string }): Promise<{ changed: boolean; weather: WeatherState }> {
    const forced = data.type
      ? ({ type: data.type, temperature: 20, description: "", wind: "calm", visibility: "clear" } as WeatherState)
      : generateWeather(inferBiome(data.location ?? ""), (data.season as any) ?? "summer");
    const changed = Boolean(data.type) || Math.random() < (data.action === "travel" ? 0.35 : data.action === "rest_long" ? 0.6 : data.action === "explore" ? 0.2 : 0.08);
    if (changed) await patchChatMetadata(data.chatId, { gameWeather: forced });
    return { changed, weather: forced };
  },

  async rollEncounter(data: { action: string; location?: string; difficulty?: string; partySize?: number }) {
    const encounter = rollGameEncounter(data.action, data.difficulty ?? "normal", data.location ?? "");
    const enemyCount = encounter.type === "combat" ? rollEnemyCount(data.partySize ?? 1, data.difficulty ?? "normal") : 0;
    return { encounter, enemyCount };
  },

  async updateReputation(data: { chatId: string; actions: Array<{ npcId: string; action: string; modifier?: number }> }) {
    const chat = await getChat(data.chatId);
    const meta = chatMeta(chat);
    const npcs = Array.isArray(meta.gameNpcs) ? (meta.gameNpcs as GameNpc[]) : [];
    const result = processReputationActions(npcs, data.actions);
    await patchChatMetadata(data.chatId, { gameNpcs: result.npcs });
    return { npcs: result.npcs, changes: result.changes };
  },

  async addJournalEntry(data: { chatId: string; type: string; data: Record<string, unknown> }): Promise<{ journal: Journal }> {
    const chat = await getChat(data.chatId);
    const journal = applyJournalEntry(journalFromMeta(chatMeta(chat)), data.type, data.data);
    await patchChatMetadata(data.chatId, { gameJournal: journal });
    return { journal };
  },

  async getJournal(chatId: string): Promise<GameJournalResponse> {
    const meta = chatMeta(await getChat(chatId));
    const journal = journalFromMeta(meta);
    const sessionNumber = Number(meta.gameSessionNumber ?? 1);
    return {
      journal,
      recap: buildStructuredRecap(journal, sessionNumber),
      playerNotes: typeof meta.gamePlayerNotes === "string" ? meta.gamePlayerNotes : "",
    };
  },

  async updateNotes(chatId: string, notes: string) {
    await patchChatMetadata(chatId, { gamePlayerNotes: notes });
    return { ok: true };
  },

  async listCheckpoints(chatId: string) {
    const all = await api.get<import("@marinara-engine/shared").GameCheckpoint[]>("/game-checkpoints");
    return all.filter((checkpoint) => (checkpoint as { chatId?: string }).chatId === chatId);
  },

  async createCheckpoint(data: { chatId: string; label: string; triggerType: string }) {
    const chat = await getChat(data.chatId);
    const snapshot = await api.post<{ id: string }>("/game-state-snapshots", {
      chatId: data.chatId,
      messageId: null,
      gameState: (chat as { gameState?: unknown }).gameState ?? {},
      metadata: chatMeta(chat),
    });
    const record = await api.post<{ id: string }>("/game-checkpoints", {
      chatId: data.chatId,
      snapshotId: snapshot.id,
      messageId: "",
      label: data.label || "Checkpoint",
      triggerType: data.triggerType || "manual",
      location: null,
      gameState: null,
      weather: null,
      timeOfDay: null,
      turnNumber: null,
    });
    return { id: record.id };
  },

  async loadCheckpoint(data: { chatId: string; checkpointId: string }) {
    const checkpoint = await api.get<{ id: string; chatId?: string; label?: string }>(`/game-checkpoints/${encodeURIComponent(data.checkpointId)}`);
    if (checkpoint.chatId !== data.chatId) throw new Error("Checkpoint does not belong to this chat.");
    const message = await api.post<{ id: string }>(`/chats/${encodeURIComponent(data.chatId)}/messages`, {
      role: "system",
      characterId: null,
      content: `[Checkpoint restored: ${checkpoint.label || "Checkpoint"}]`,
    });
    return { ok: true, messageId: message.id };
  },

  async deleteCheckpoint(id: string) {
    const result = await api.delete<{ deleted?: boolean }>(`/game-checkpoints/${encodeURIComponent(id)}`);
    return { ok: Boolean(result.deleted) };
  },

  async partyTurn(input: { chatId: string; narration: string; playerAction?: string; connectionId?: string | null; debugMode?: boolean }) {
    const meta = chatMeta(await getChat(input.chatId));
    const cards = Array.isArray(meta.gameCharacterCards) ? meta.gameCharacterCards : [];
    const names = cards.map((card) => asRecord(card).name).filter((name): name is string => typeof name === "string" && !!name.trim());
    const partyNames = names.length ? names.join(", ") : "The party";
    let raw = `[${partyNames}] [dialogue] [neutral]: We take this in and prepare for what comes next.`;
    if (input.connectionId) {
      try {
        raw = await llmApi.complete({
          connectionId: input.connectionId,
          messages: [
            {
              role: "system",
              content: `You write short party banter for a game. Reply using lines like [Name] [dialogue] [neutral]: text. Party: ${partyNames}.`,
            },
            {
              role: "user",
              content: `GM narration:\n${input.narration}\n\nPlayer action:\n${input.playerAction ?? ""}\n\nWrite the party's immediate reactions.`,
            },
          ],
          parameters: { temperature: 0.9, maxTokens: 1200 },
        });
      } catch {
        raw = `[${partyNames}] [dialogue] [neutral]: We take this in and prepare for what comes next.`;
      }
    }
    const clean = raw.replace(/\[party-turn\]/gi, "").trim();
    await api.post(`/chats/${encodeURIComponent(input.chatId)}/messages`, {
      role: "assistant",
      characterId: null,
      content: `[party-turn]\n${clean}`,
      extra: {},
      swipes: [{ content: `[party-turn]\n${clean}` }],
      activeSwipeIndex: 0,
    });
    return { raw: clean };
  },

  async spotifyCandidates(payload: Record<string, unknown>) {
    try {
      return await api.post("/spotify/search-tracks", {
        query: spotifyQuery(payload),
        limit: Math.max(1, Math.min(50, Number(payload.limit ?? 50))),
        recentTrackUris: recentSpotifyTracks(payload),
      });
    } catch (error) {
      return { enabled: false, tracks: [], error: error instanceof Error ? error.message : "Spotify search failed" };
    }
  },

  async spotifyPlay(payload: { track: unknown; deviceId?: string | null }) {
    return api.post("/spotify/play-track", payload);
  },

  async previewGeneratedAssets(payload: GameAssetGenerationPayload): Promise<{ items: GameImagePromptReviewItem[] }> {
    const record = payload as unknown as Record<string, unknown>;
    const meta = chatMeta(await getChat(String(record.chatId)));
    const setup = asRecord(meta.gameSetupConfig);
    const artStyle =
      (typeof record.artStylePrompt === "string" && record.artStylePrompt) ||
      (typeof setup.artStylePrompt === "string" && setup.artStylePrompt) ||
      "";
    const items: GameImagePromptReviewItem[] = [];
    if (typeof record.backgroundTag === "string" && record.backgroundTag.trim()) {
      const id = imageReviewId("background", record.backgroundTag);
      items.push({
        id,
        kind: "background",
        title: `Background: ${record.backgroundTag}`,
        prompt: promptOverride(record, id) ?? sceneAssetPrompt("background", record.backgroundTag, record.backgroundTag, artStyle),
        width: imageSize(record, "background", "width", 1280),
        height: imageSize(record, "background", "height", 720),
      });
    }
    const illustration = asRecord(record.illustration);
    if (Object.keys(illustration).length > 0) {
      const label =
        (typeof illustration.reason === "string" && illustration.reason) ||
        (typeof illustration.slug === "string" && illustration.slug) ||
        (typeof illustration.prompt === "string" && illustration.prompt) ||
        "Scene illustration";
      const id = imageReviewId("illustration", label);
      items.push({
        id,
        kind: "illustration",
        title: `Illustration: ${label}`,
        prompt: promptOverride(record, id) ?? sceneAssetPrompt("illustration", label, String(illustration.prompt ?? label), artStyle),
        width: imageSize(record, "background", "width", 1280),
        height: imageSize(record, "background", "height", 720),
      });
    }
    const npcs = Array.isArray(record.npcsNeedingAvatars) ? record.npcsNeedingAvatars : [];
    for (const npc of npcs.slice(0, 10)) {
      const npcRecord = asRecord(npc);
      const name = typeof npcRecord.name === "string" && npcRecord.name.trim() ? npcRecord.name : "NPC";
      const detail =
        typeof npcRecord.description === "string" && npcRecord.description.trim()
          ? npcRecord.description
          : "distinctive character portrait";
      const id = imageReviewId("portrait", name);
      items.push({
        id,
        kind: "portrait",
        title: `Portrait: ${name}`,
        prompt: promptOverride(record, id) ?? sceneAssetPrompt("portrait", name, detail, artStyle),
        width: imageSize(record, "portrait", "width", 768),
        height: imageSize(record, "portrait", "height", 1024),
      });
    }
    return { items };
  },

  async generateAssets(payload: GameAssetGenerationPayload, signal?: AbortSignal): Promise<GameAssetGenerationResult> {
    const record = payload as unknown as Record<string, unknown>;
    const chatId = String(record.chatId);
    const chat = await getChat(chatId);
    const meta = chatMeta(chat);
    if (!meta.enableSpriteGeneration) {
      return {
        generatedBackground: null,
        fallbackBackground: null,
        generatedIllustration: null,
        generatedNpcAvatars: [],
      };
    }
    const imageConnectionId =
      (typeof record.imageConnectionId === "string" && record.imageConnectionId) ||
      (typeof meta.gameImageConnectionId === "string" && meta.gameImageConnectionId) ||
      (typeof meta.imageConnectionId === "string" && meta.imageConnectionId) ||
      (typeof asRecord(meta.gameSetupConfig).imageConnectionId === "string" && (asRecord(meta.gameSetupConfig).imageConnectionId as string));
    if (!imageConnectionId) throw new Error("Game image generation requires an image connection.");

    const preview = await gameApi.previewGeneratedAssets(payload);
    let generatedBackground: string | null = null;
    let generatedIllustration: GameAssetGenerationResult["generatedIllustration"] = null;
    const generatedNpcAvatars: GameAssetGenerationResult["generatedNpcAvatars"] = [];

    for (const item of preview.items) {
      if (signal?.aborted) throw new DOMException("The operation was aborted.", "AbortError");
      const image = await api.post<{ base64: string; mimeType: string; image?: string }>(
        "/images/generate",
        { connectionId: imageConnectionId, prompt: item.prompt, width: item.width, height: item.height },
        { signal },
      );
      if (item.kind === "background") {
        const key = typeof record.backgroundTag === "string" ? record.backgroundTag : "generated-background";
        const tag = await uploadGeneratedAsset("backgrounds", "generated", generatedAssetSlug(key), image.base64, image.mimeType);
        generatedBackground = tag;
        await patchChatMetadata(chatId, { gameSceneBackground: tag });
      } else if (item.kind === "illustration") {
        const illustration = asRecord(record.illustration);
        const key =
          (typeof illustration.slug === "string" && illustration.slug) ||
          item.title ||
          "scene-illustration";
        const tag = await uploadGeneratedAsset("backgrounds", "illustrations", generatedAssetSlug(key), image.base64, image.mimeType);
        generatedIllustration = {
          tag,
          segment: Number.isInteger(illustration.segment) ? (illustration.segment as number) : null,
        };
      } else if (item.kind === "portrait") {
        generatedNpcAvatars.push({
          name: item.title.replace(/^Portrait:\s*/, "") || "NPC",
          avatarUrl: image.image ?? `data:${image.mimeType};base64,${image.base64}`,
        });
      }
    }

    if (generatedNpcAvatars.length > 0) {
      const freshMeta = chatMeta(await getChat(chatId));
      const npcs = Array.isArray(freshMeta.gameNpcs) ? [...(freshMeta.gameNpcs as GameNpc[])] : [];
      for (const avatar of generatedNpcAvatars) {
        const existing = npcs.find((npc) => npc.name.toLowerCase() === avatar.name.toLowerCase());
        if (existing) {
          (existing as GameNpc & { avatarUrl?: string }).avatarUrl = avatar.avatarUrl;
        } else {
          npcs.push({
            id: newId("npc"),
            emoji: "👤",
            name: avatar.name,
            description: "",
            location: "",
            reputation: 0,
            met: true,
            notes: [],
            avatarUrl: avatar.avatarUrl,
          } as GameNpc);
        }
      }
      await patchChatMetadata(chatId, { gameNpcs: npcs });
    }

    return { generatedBackground, fallbackBackground: null, generatedIllustration, generatedNpcAvatars };
  },
};

export function getEmptyJournal(): Journal {
  return { ...EMPTY_JOURNAL, entries: [], quests: [], locations: [], npcLog: [], inventoryLog: [] };
}
