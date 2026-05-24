export const galleryKeys = {
  all: ["gallery"] as const,
  images: (chatId: string | null) => [...galleryKeys.all, "images", chatId] as const,
};
