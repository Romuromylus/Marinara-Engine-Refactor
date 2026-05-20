import { useCallback, useMemo } from "react";
import { useChat } from "../../chats/hooks/use-chats";
import { useCharacters, usePersonas, useUpdateCharacter, useUpdatePersona } from "../../characters/hooks/use-characters";
import { parseCharacterDisplayData } from "../../../shared/lib/character-display";
import {
  parseTrackerCardColorConfig,
  serializeTrackerCardColorConfig,
} from "../../../shared/lib/tracker-card-colors";
import { useChatStore } from "../../../shared/stores/chat.store";
import type { TrackerCardColorConfig } from "../../../engine/contracts/types/persona";

interface CharacterRow {
  id: string;
  data: unknown;
}

interface PersonaRow {
  id: string;
  name: string;
  isActive?: boolean | string;
  nameColor?: string | null;
  dialogueColor?: string | null;
  boxColor?: string | null;
  trackerCardColors?: TrackerCardColorConfig | string | null;
}

export type TrackerCardColorChatColors = {
  nameColor?: string | null;
  dialogueColor?: string | null;
  boxColor?: string | null;
};

export type TrackerCardColorTarget = {
  key: string;
  kind: "persona" | "character";
  id: string;
  name: string;
  chatColors: TrackerCardColorChatColors;
  trackerCardColors: TrackerCardColorConfig;
};

type TrackerCardColorTargetInternal =
  | (TrackerCardColorTarget & { kind: "persona" })
  | (TrackerCardColorTarget & { kind: "character"; row: CharacterRow });

function parseCharacterDataRecord(value: unknown): Record<string, unknown> | null {
  if (!value) return null;
  if (typeof value === "string") {
    try {
      const parsed = JSON.parse(value);
      return parsed && typeof parsed === "object" && !Array.isArray(parsed) ? (parsed as Record<string, unknown>) : null;
    } catch {
      return null;
    }
  }
  return typeof value === "object" && !Array.isArray(value) ? (value as Record<string, unknown>) : null;
}

function normalizeStringArray(value: unknown): string[] {
  if (Array.isArray(value)) return value.filter((entry): entry is string => typeof entry === "string" && !!entry);
  if (typeof value !== "string" || !value.trim()) return [];
  try {
    const parsed = JSON.parse(value);
    return Array.isArray(parsed) ? normalizeStringArray(parsed) : [];
  } catch {
    return value
      .split(",")
      .map((entry) => entry.trim())
      .filter(Boolean);
  }
}

function getCharacterData(rawData: unknown) {
  const data = parseCharacterDataRecord(rawData);
  if (!data) return null;
  const extensions =
    data.extensions && typeof data.extensions === "object" && !Array.isArray(data.extensions)
      ? (data.extensions as Record<string, unknown>)
      : {};
  return { data, extensions };
}

function getString(value: unknown) {
  return typeof value === "string" ? value : "";
}

function getCharacterColors(row: CharacterRow): TrackerCardColorChatColors & {
  trackerCardColors: TrackerCardColorConfig;
} {
  const characterData = getCharacterData(row.data);
  const extensions = characterData?.extensions ?? {};
  return {
    nameColor: getString(extensions.nameColor),
    dialogueColor: getString(extensions.dialogueColor),
    boxColor: getString(extensions.boxColor),
    trackerCardColors: parseTrackerCardColorConfig(extensions.trackerCardColors),
  };
}

function buildCharacterDataWithTrackerColors(row: CharacterRow, trackerCardColors: TrackerCardColorConfig) {
  const characterData = getCharacterData(row.data);
  if (!characterData) {
    throw new Error("Character data could not be parsed.");
  }
  const { data, extensions } = characterData;
  return {
    ...data,
    extensions: {
      ...extensions,
      trackerCardColors: serializeTrackerCardColorConfig(trackerCardColors),
    },
  };
}

function publicTarget(target: TrackerCardColorTargetInternal): TrackerCardColorTarget {
  return {
    key: target.key,
    kind: target.kind,
    id: target.id,
    name: target.name,
    chatColors: target.chatColors,
    trackerCardColors: target.trackerCardColors,
  };
}

export function useTrackerCardColorSettingsTargets() {
  const activeChatId = useChatStore((s) => s.activeChatId);
  const { data: chat, isLoading: chatLoading } = useChat(activeChatId);
  const { data: charactersData, isLoading: charactersLoading } = useCharacters(!!activeChatId);
  const { data: personasData, isLoading: personasLoading } = usePersonas(!!activeChatId);
  const updateCharacter = useUpdateCharacter();
  const updatePersona = useUpdatePersona();

  const internalTargets = useMemo<TrackerCardColorTargetInternal[]>(() => {
    if (!activeChatId) return [];
    const rawChat = chat as { characterIds?: unknown; personaId?: unknown } | undefined;
    const chatCharacterIds = normalizeStringArray(rawChat?.characterIds);
    const characterIdSet = new Set(chatCharacterIds);
    const personas = Array.isArray(personasData) ? (personasData as PersonaRow[]) : [];
    const chatPersonaId = typeof rawChat?.personaId === "string" ? rawChat.personaId : "";
    const activePersona =
      personas.find((persona) => persona.id === chatPersonaId) ??
      personas.find((persona) => persona.isActive === true || persona.isActive === "true") ??
      null;
    const rows = Array.isArray(charactersData) ? (charactersData as CharacterRow[]) : [];

    const nextTargets: TrackerCardColorTargetInternal[] = [];
    if (activePersona) {
      nextTargets.push({
        key: `persona:${activePersona.id}`,
        kind: "persona",
        id: activePersona.id,
        name: activePersona.name || "Persona",
        chatColors: {
          nameColor: activePersona.nameColor,
          dialogueColor: activePersona.dialogueColor,
          boxColor: activePersona.boxColor,
        },
        trackerCardColors: parseTrackerCardColorConfig(activePersona.trackerCardColors),
      });
    }

    for (const id of chatCharacterIds) {
      const row = rows.find((candidate) => candidate.id === id);
      if (!row || !characterIdSet.has(row.id)) continue;
      const display = parseCharacterDisplayData(row);
      const colors = getCharacterColors(row);
      nextTargets.push({
        key: `character:${row.id}`,
        kind: "character",
        id: row.id,
        name: display.name,
        chatColors: colors,
        trackerCardColors: colors.trackerCardColors,
        row,
      });
    }

    return nextTargets;
  }, [activeChatId, charactersData, chat, personasData]);

  const targets = useMemo(() => internalTargets.map(publicTarget), [internalTargets]);

  const saveTrackerCardColorTarget = useCallback(
    async (targetKey: string, trackerCardColors: TrackerCardColorConfig) => {
      const target = internalTargets.find((candidate) => candidate.key === targetKey);
      if (!target) {
        throw new Error("Tracker card color target was not found.");
      }

      if (target.kind === "persona") {
        await updatePersona.mutateAsync({
          id: target.id,
          trackerCardColors: serializeTrackerCardColorConfig(trackerCardColors),
        });
        return;
      }

      await updateCharacter.mutateAsync({
        id: target.id,
        data: buildCharacterDataWithTrackerColors(target.row, trackerCardColors),
      });
    },
    [internalTargets, updateCharacter, updatePersona],
  );

  return {
    activeChatId,
    targets,
    loading: chatLoading || charactersLoading || personasLoading,
    saving: updateCharacter.isPending || updatePersona.isPending,
    saveTrackerCardColorTarget,
  };
}
