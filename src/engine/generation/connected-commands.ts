import type { StorageGateway } from "../capabilities/storage";
import type { IntegrationGateway } from "../capabilities/integrations";
import {
  parseCharacterCommands,
  type CharacterCommand,
  type CreateCharacterCommand,
  type CreateLorebookCommand,
  type CreatePersonaCommand,
  type UpdateCharacterCommand,
  type UpdateLorebookCommand,
  type UpdatePersonaCommand,
} from "../modes/chat/commands/character-commands";
import { newId, nowIso, parseArray, readString, type JsonRecord } from "./runtime-records";

export interface ConnectedCommandResult {
  displayContent: string;
  createdNotes: JsonRecord[];
  executedCommands: string[];
}

function parseData(row: JsonRecord | null | undefined): JsonRecord {
  const raw = row?.data;
  if (typeof raw === "string") {
    try {
      const parsed = JSON.parse(raw);
      return parsed && typeof parsed === "object" && !Array.isArray(parsed) ? (parsed as JsonRecord) : {};
    } catch {
      return {};
    }
  }
  return raw && typeof raw === "object" && !Array.isArray(raw) ? (raw as JsonRecord) : {};
}

function nameOf(row: JsonRecord): string {
  return readString(row.name) || readString(parseData(row).name);
}

function matchesName(row: JsonRecord, name: string): boolean {
  return nameOf(row).trim().toLowerCase() === name.trim().toLowerCase();
}

async function findByName(storage: StorageGateway, entity: string, name: string): Promise<JsonRecord | null> {
  const rows = await storage.list<JsonRecord>(entity);
  return rows.find((row) => matchesName(row, name)) ?? null;
}

function characterDataFromCreate(command: CreateCharacterCommand): JsonRecord {
  return {
    name: command.name,
    description: command.description ?? "",
    personality: command.personality ?? "",
    first_mes: command.firstMessage ?? "",
    scenario: command.scenario ?? "",
    backstory: command.backstory ?? "",
    appearance: command.appearance ?? "",
    mes_example: command.mesExample ?? "",
    creator_notes: command.creatorNotes ?? "",
    system_prompt: command.systemPrompt ?? "",
    post_history_instructions: command.postHistoryInstructions ?? "",
    creator: command.creator ?? "",
    character_version: command.characterVersion ?? "1.0",
    tags: command.tags ?? [],
    alternate_greetings: command.alternateGreetings ?? [],
    extensions: {
      altDescriptions: [],
      ...(command.talkativeness != null ? { talkativeness: command.talkativeness } : {}),
      ...(command.depthPrompt ? { depth_prompt: command.depthPrompt } : {}),
      ...(command.depthPromptDepth != null ? { depth_prompt_depth: command.depthPromptDepth } : {}),
      ...(command.depthPromptRole ? { depth_prompt_role: command.depthPromptRole } : {}),
    },
    character_book: null,
  };
}

function characterDataPatch(data: JsonRecord, command: UpdateCharacterCommand): JsonRecord {
  const next: JsonRecord = { ...data, name: command.name || readString(data.name) };
  const fieldMap: Array<[keyof UpdateCharacterCommand, string]> = [
    ["description", "description"],
    ["personality", "personality"],
    ["firstMessage", "first_mes"],
    ["scenario", "scenario"],
    ["backstory", "backstory"],
    ["appearance", "appearance"],
    ["mesExample", "mes_example"],
    ["creatorNotes", "creator_notes"],
    ["systemPrompt", "system_prompt"],
    ["postHistoryInstructions", "post_history_instructions"],
    ["creator", "creator"],
    ["characterVersion", "character_version"],
    ["world", "world"],
  ];
  for (const [from, to] of fieldMap) {
    if (command[from] !== undefined) next[to] = command[from];
  }
  if (command.tags !== undefined) next.tags = command.tags;
  if (command.alternateGreetings !== undefined) next.alternate_greetings = command.alternateGreetings;
  if (command.fav !== undefined) next.fav = command.fav;
  const extensions = {
    ...(data.extensions && typeof data.extensions === "object" && !Array.isArray(data.extensions)
      ? (data.extensions as JsonRecord)
      : {}),
  };
  if (command.talkativeness !== undefined) extensions.talkativeness = command.talkativeness;
  if (command.depthPrompt !== undefined) extensions.depth_prompt = command.depthPrompt;
  if (command.depthPromptDepth !== undefined) extensions.depth_prompt_depth = command.depthPromptDepth;
  if (command.depthPromptRole !== undefined) extensions.depth_prompt_role = command.depthPromptRole;
  next.extensions = extensions;
  return next;
}

function personaPatch(command: CreatePersonaCommand | UpdatePersonaCommand): JsonRecord {
  return {
    name: command.name,
    ...(command.description !== undefined ? { description: command.description } : {}),
    ...(command.personality !== undefined ? { personality: command.personality } : {}),
    ...(command.appearance !== undefined ? { appearance: command.appearance } : {}),
    ...("scenario" in command && command.scenario !== undefined ? { scenario: command.scenario } : {}),
    ...("backstory" in command && command.backstory !== undefined ? { backstory: command.backstory } : {}),
  };
}

async function createLorebookEntries(storage: StorageGateway, lorebookId: string, command: CreateLorebookCommand | UpdateLorebookCommand) {
  if (!command.entries?.length) return;
  for (const entry of command.entries) {
    await storage.create("lorebook-entries", {
      lorebookId,
      name: entry.name,
      content: entry.content ?? entry.description ?? "",
      keys: entry.keys ?? [],
      secondaryKeys: entry.secondaryKeys ?? [],
      enabled: true,
      constant: entry.constant ?? false,
      selective: entry.selective ?? false,
      tag: entry.tag ?? "",
      order: 0,
    });
  }
}

async function executeCommand(
  storage: StorageGateway,
  integrations: IntegrationGateway | undefined,
  chat: JsonRecord,
  command: CharacterCommand,
  createdNotes: JsonRecord[],
): Promise<string | null> {
  const chatId = readString(chat.id);
  switch (command.type) {
    case "note":
    case "influence":
      createdNotes.push({
        id: newId(command.type),
        type: command.type,
        content: command.content,
        sourceChatId: chatId,
        targetChatId: null,
        createdAt: nowIso(),
      });
      return command.type;
    case "memory":
      createdNotes.push({
        id: newId("memory"),
        type: "memory",
        content: `${command.target}: ${command.summary}`,
        sourceChatId: chatId,
        targetChatId: null,
        createdAt: nowIso(),
      });
      return "memory";
    case "haptic":
      if (integrations) {
        await integrations.haptic.command({
          action: command.action,
          intensity: command.intensity,
          duration: command.duration,
        });
        return "haptic";
      }
      return null;
    case "spotify":
      if (integrations) {
        const search = await integrations.spotify.searchTracks<{ tracks?: Array<{ uri?: string }> }>({
          query: `${command.title} ${command.artist}`,
          limit: 1,
        });
        const track = search.tracks?.find((item) => item.uri);
        if (track) await integrations.spotify.playTrack({ track });
        return "spotify";
      }
      return null;
    case "create_persona":
      await storage.create("personas", personaPatch(command));
      return "create_persona";
    case "update_persona": {
      const row = await findByName(storage, "personas", command.name);
      if (!row?.id) return null;
      await storage.update("personas", readString(row.id), personaPatch(command));
      return "update_persona";
    }
    case "create_character":
      await storage.create("characters", { name: command.name, data: characterDataFromCreate(command) });
      return "create_character";
    case "update_character": {
      const row = await findByName(storage, "characters", command.name);
      if (!row?.id) return null;
      await storage.update("characters", readString(row.id), {
        name: command.name,
        data: characterDataPatch(parseData(row), command),
      });
      return "update_character";
    }
    case "create_lorebook": {
      const lorebook = await storage.create<JsonRecord>("lorebooks", {
        name: command.name,
        description: command.description ?? "",
        category: command.category ?? "uncategorized",
        tags: command.tags ?? [],
      });
      await createLorebookEntries(storage, readString(lorebook.id), command);
      return "create_lorebook";
    }
    case "update_lorebook": {
      const row = await findByName(storage, "lorebooks", command.name);
      if (!row?.id) return null;
      const lorebookId = readString(row.id);
      await storage.update("lorebooks", lorebookId, {
        ...(command.newName ? { name: command.newName } : {}),
        ...(command.description !== undefined ? { description: command.description } : {}),
        ...(command.category !== undefined ? { category: command.category } : {}),
        ...(command.tags !== undefined ? { tags: command.tags } : {}),
      });
      await createLorebookEntries(storage, lorebookId, command);
      return "update_lorebook";
    }
    case "create_chat": {
      const character = await findByName(storage, "characters", command.character);
      await storage.create("chats", {
        name: command.character,
        mode: command.mode ?? "conversation",
        characterIds: character?.id ? [readString(character.id)] : [],
        metadata: {},
      });
      return "create_chat";
    }
    case "schedule_update":
    case "cross_post":
    case "selfie":
    case "scene":
    case "navigate":
    case "fetch":
    case "dm":
      createdNotes.push({
        id: newId(command.type),
        type: command.type,
        content: JSON.stringify(command),
        sourceChatId: chatId,
        targetChatId: null,
        createdAt: nowIso(),
      });
      return command.type;
  }
}

export async function persistConnectedCommandTags(
  storage: StorageGateway,
  chat: JsonRecord,
  content: string,
  integrations?: IntegrationGateway,
): Promise<ConnectedCommandResult> {
  const chatId = readString(chat.id);
  const existingNotes = parseArray(chat.notes).filter((entry): entry is JsonRecord => !!entry && typeof entry === "object");
  const createdNotes: JsonRecord[] = [];
  const parsed = parseCharacterCommands(content);
  const executedCommands: string[] = [];

  for (const command of parsed.commands) {
    const executed = await executeCommand(storage, integrations, chat, command, createdNotes).catch(() => null);
    if (executed) executedCommands.push(executed);
  }

  if (createdNotes.length > 0 && chatId) {
    await storage.update("chats", chatId, { notes: [...existingNotes, ...createdNotes] });
  }

  return {
    displayContent: parsed.cleanContent,
    createdNotes,
    executedCommands,
  };
}
