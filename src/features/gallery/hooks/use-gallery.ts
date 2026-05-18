import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "../../../shared/api/api-client";

export interface ChatImage {
  id: string;
  chatId: string;
  url: string;
  prompt?: string | null;
  model?: string | null;
  provider?: string | null;
  width?: number | null;
  height?: number | null;
  createdAt?: string;
}

export function useGalleryImages(chatId: string | null) {
  return useQuery({
    queryKey: ["gallery", "images", chatId],
    queryFn: () => api.get<ChatImage[]>(`/chats/${chatId}/gallery`),
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
        const formData = new FormData();
        formData.append("file", file);
        uploaded.push(await api.upload<ChatImage>(`/chats/${chatId}/gallery/upload`, formData));
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
    mutationFn: (imageId: string) => api.delete(`/chats/${chatId}/gallery/${imageId}`),
    onSuccess: () => {
      if (chatId) {
        queryClient.invalidateQueries({ queryKey: ["gallery", "images", chatId] });
      }
    },
    meta: { chatId },
  });
}
