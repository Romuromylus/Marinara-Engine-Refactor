import { create } from "zustand";

interface ActiveChatSnapshot {
  id: string;
  characterIds?: string | string[] | null;
  mode?: string;
}

interface ChatStore {
  activeChatId: string | null;
  activeChat: ActiveChatSnapshot | null;
  setActiveChatId: (id: string | null) => void;
}

export const useChatStore = create<ChatStore>((set) => ({
  activeChatId: null,
  activeChat: null,
  setActiveChatId: (id) => set({ activeChatId: id }),
}));
