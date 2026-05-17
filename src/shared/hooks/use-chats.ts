import { useMutation } from "@tanstack/react-query";

export type ExpungeScope =
  | "chats"
  | "characters"
  | "personas"
  | "lorebooks"
  | "presets"
  | "connections"
  | "automation"
  | "media";

export const chatKeys = {
  list: () => ["chats"] as const,
  messages: (chatId: string) => ["chats", chatId, "messages"] as const,
};

const deferred = async (): Promise<never> => {
  throw new Error("Chat data actions move in a later Tauri backend slice.");
};

export function useUpdateChatMetadata() {
  return useMutation({
    mutationFn: (_metadata: unknown) => deferred(),
  });
}

export function useUpdateChat() {
  return useMutation({
    mutationFn: (_chat: unknown) => deferred(),
  });
}

export function useCreateMessage(_chatId: string | null) {
  return useMutation({
    mutationFn: (_message: unknown): Promise<{ id: string }> => deferred(),
  });
}

export function useClearAllData() {
  return useMutation({
    mutationFn: () => deferred(),
  });
}

export function useExpungeData() {
  return useMutation({
    mutationFn: (_scopes: ExpungeScope[]) => deferred(),
  });
}
