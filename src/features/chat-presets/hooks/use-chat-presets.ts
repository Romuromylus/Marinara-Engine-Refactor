// ──────────────────────────────────────────────
// React Query: Chat Preset hooks
// ──────────────────────────────────────────────
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { invokeTauri } from "../../../shared/api/tauri-client";
import { storageApi } from "../../../shared/api/storage-api";
import { chatKeys } from "../../chats/hooks/use-chats";
import type { Chat, ChatMode } from "../../../engine/contracts/types/chat";
import type { ChatPreset, ChatPresetSettings } from "../../../engine/contracts/types/chat-preset";

export const chatPresetKeys = {
  all: ["chat-presets"] as const,
  list: (mode?: ChatMode | null) => [...chatPresetKeys.all, "list", mode ?? "all"] as const,
  detail: (id: string) => [...chatPresetKeys.all, "detail", id] as const,
  active: (mode: ChatMode) => [...chatPresetKeys.all, "active", mode] as const,
};

async function setOnlyActivePreset(id: string): Promise<ChatPreset> {
  const selected = await storageApi.get<ChatPreset>("chat-presets", id);
  if (!selected) throw new Error(`Chat preset ${id} was not found`);
  const presets = await storageApi.list<ChatPreset>("chat-presets");
  await Promise.all(
    presets
      .filter((preset) => preset.mode === selected.mode)
      .map((preset) =>
        storageApi.update<ChatPreset>("chat-presets", preset.id, {
          isActive: preset.id === id,
          active: preset.id === id,
        }),
      ),
  );
  return { ...selected, isActive: true, active: true } as ChatPreset;
}

export function useChatPresets(mode?: ChatMode | null) {
  return useQuery({
    queryKey: chatPresetKeys.list(mode ?? null),
    queryFn: async () => {
      const presets = await storageApi.list<ChatPreset>("chat-presets");
      return mode ? presets.filter((preset) => preset.mode === mode) : presets;
    },
    staleTime: 60_000,
  });
}

export function useActiveChatPreset(mode: ChatMode | null) {
  return useQuery({
    queryKey: mode ? chatPresetKeys.active(mode) : chatPresetKeys.all,
    queryFn: async () => {
      const presets = await storageApi.list<ChatPreset>("chat-presets");
      return (
        presets.find(
          (preset) =>
            preset.mode === mode &&
            ((preset as ChatPreset & { isActive?: boolean; active?: boolean }).isActive ||
              (preset as ChatPreset & { isActive?: boolean; active?: boolean }).active),
        ) ??
        presets.find((preset) => preset.mode === mode) ??
        null
      );
    },
    enabled: !!mode,
    staleTime: 60_000,
  });
}

export function useCreateChatPreset() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: { name: string; mode: ChatMode; settings?: ChatPresetSettings }) =>
      storageApi.create<ChatPreset>("chat-presets", data as Record<string, unknown>),
    onSuccess: () => qc.invalidateQueries({ queryKey: chatPresetKeys.all }),
  });
}

export function useUpdateChatPreset() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, ...data }: { id: string; name?: string; settings?: ChatPresetSettings }) =>
      storageApi.update<ChatPreset>("chat-presets", id, data as Record<string, unknown>),
    onSuccess: () => qc.invalidateQueries({ queryKey: chatPresetKeys.all }),
  });
}

export function useSaveChatPresetSettings() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, settings }: { id: string; settings: ChatPresetSettings }) =>
      storageApi.update<ChatPreset>("chat-presets", id, { settings: settings as unknown as Record<string, unknown> }),
    onSuccess: () => qc.invalidateQueries({ queryKey: chatPresetKeys.all }),
  });
}

export function useDuplicateChatPreset() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async ({ id, name }: { id: string; name?: string }) => {
      const duplicated = await invokeTauri<ChatPreset>("storage_duplicate", { entity: "chat-presets", id });
      return name?.trim()
        ? storageApi.update<ChatPreset>("chat-presets", duplicated.id, { name: name.trim() })
        : duplicated;
    },
    onSuccess: () => qc.invalidateQueries({ queryKey: chatPresetKeys.all }),
  });
}

export function useSetActiveChatPreset() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => setOnlyActivePreset(id),
    onSuccess: () => qc.invalidateQueries({ queryKey: chatPresetKeys.all }),
  });
}

export function useDeleteChatPreset() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => storageApi.delete("chat-presets", id),
    onSuccess: () => qc.invalidateQueries({ queryKey: chatPresetKeys.all }),
  });
}

export function useImportChatPreset() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (envelope: unknown) => storageApi.create<ChatPreset>("chat-presets", envelope as Record<string, unknown>),
    onSuccess: () => qc.invalidateQueries({ queryKey: chatPresetKeys.all }),
  });
}

/** Apply a preset's settings to an existing chat. Refetches the chat afterward. */
export function useApplyChatPreset() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ presetId, chatId }: { presetId: string; chatId: string }) =>
      storageApi.update<Chat>("chats", chatId, { chatPresetId: presetId }),
    onSuccess: (_data, variables) => {
      qc.invalidateQueries({ queryKey: chatKeys.detail(variables.chatId) });
      qc.invalidateQueries({ queryKey: chatKeys.list() });
    },
  });
}
