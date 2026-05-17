import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "../api/characters-api";

export const characterKeys = {
  all: ["characters"] as const,
  list: () => [...characterKeys.all, "list"] as const,
  detail: (id: string) => [...characterKeys.all, "detail", id] as const,
  groups: ["character-groups"] as const,
  groupDetail: (id: string) => ["character-groups", "detail", id] as const,
};

export function useCharacters() {
  return useQuery({
    queryKey: characterKeys.list(),
    queryFn: () => api.get<unknown[]>("/characters"),
    staleTime: 5 * 60_000,
  });
}

export function useUpdateCharacter() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, ...data }: { id: string; data?: Record<string, unknown>; avatarPath?: string; comment?: string }) =>
      api.patch(`/characters/${id}`, data),
    onSuccess: (_data, variables) => {
      qc.invalidateQueries({ queryKey: characterKeys.list() });
      qc.invalidateQueries({ queryKey: characterKeys.detail(variables.id) });
    },
  });
}

export function useDeleteCharacter() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => api.delete(`/characters/${id}`),
    onSuccess: () => qc.invalidateQueries({ queryKey: characterKeys.list() }),
  });
}

export function useDuplicateCharacter() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => api.post(`/characters/${id}/duplicate`, {}),
    onSuccess: () => qc.invalidateQueries({ queryKey: characterKeys.list() }),
  });
}

export function useCharacterGroups() {
  return useQuery({
    queryKey: characterKeys.groups,
    queryFn: () => api.get<unknown[]>("/characters/groups/list"),
  });
}

export function useCreateGroup() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: { name: string; description?: string; characterIds?: string[] }) =>
      api.post("/characters/groups", data),
    onSuccess: () => qc.invalidateQueries({ queryKey: characterKeys.groups }),
  });
}

export function useUpdateGroup() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, ...data }: { id: string; name?: string; description?: string; characterIds?: string[] }) =>
      api.patch(`/characters/groups/${id}`, data),
    onSuccess: () => qc.invalidateQueries({ queryKey: characterKeys.groups }),
  });
}

export function useDeleteGroup() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => api.delete(`/characters/groups/${id}`),
    onSuccess: () => qc.invalidateQueries({ queryKey: characterKeys.groups }),
  });
}

