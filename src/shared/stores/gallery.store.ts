import { create } from "zustand";
import type { ChatImage } from "../types/gallery";

interface GalleryStore {
  pinnedImages: ChatImage[];
  pinImage: (image: ChatImage) => void;
  unpinImage: (imageId: string) => void;
}

export const useGalleryStore = create<GalleryStore>((set) => ({
  pinnedImages: [],
  pinImage: (image) =>
    set((state) => ({
      pinnedImages: state.pinnedImages.some((item) => item.id === image.id)
        ? state.pinnedImages
        : [...state.pinnedImages, image],
    })),
  unpinImage: (imageId) =>
    set((state) => ({
      pinnedImages: state.pinnedImages.filter((item) => item.id !== imageId),
    })),
}));
