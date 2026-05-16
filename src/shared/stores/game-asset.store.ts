import { create } from "zustand";

interface GameAssetStore {
  rescanAssets: () => Promise<void>;
}

export const useGameAssetStore = create<GameAssetStore>(() => ({
  rescanAssets: async () => {},
}));
