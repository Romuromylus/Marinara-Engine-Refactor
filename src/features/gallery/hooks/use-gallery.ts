import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { galleryApi } from "../../../shared/api/image-generation-api";
import { storageApi } from "../../../shared/api/storage-api";
import type { ChatImage } from "../../../shared/types/gallery";

export function useGalleryImages(chatId: string | null) {
  return useQuery({
    queryKey: ["gallery", "images", chatId],
    queryFn: () => storageApi.list<ChatImage>("gallery", { filters: { chatId } }),
    enabled: !!chatId,
    retry: false,
  });
}

export function useUploadGalleryImage(chatId: string | null) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (files: File[]) => {
      if (!chatId) return [];
      const uploaded: ChatImage[] = [];
      for (const file of files) {
        uploaded.push(await galleryApi.uploadChat<ChatImage>(chatId, file));
      }
      return uploaded;
    },
    onSuccess: () => {
      if (chatId) {
        queryClient.invalidateQueries({ queryKey: ["gallery", "images", chatId] });
      }
    },
    meta: { chatId },
  });
}

export function useDeleteGalleryImage(chatId: string | null) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (imageId: string) => storageApi.delete("gallery", imageId),
    onSuccess: () => {
      if (chatId) {
        queryClient.invalidateQueries({ queryKey: ["gallery", "images", chatId] });
      }
    },
    meta: { chatId },
  });
}
