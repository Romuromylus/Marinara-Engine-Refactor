import type { ChatMode } from "../../../engine/contracts/types/chat";

export const chatPresetKeys = {
  all: ["chat-presets"] as const,
  list: (mode?: ChatMode | null) => [...chatPresetKeys.all, "list", mode ?? "all"] as const,
  detail: (id: string) => [...chatPresetKeys.all, "detail", id] as const,
  active: (mode: ChatMode) => [...chatPresetKeys.all, "active", mode] as const,
};
