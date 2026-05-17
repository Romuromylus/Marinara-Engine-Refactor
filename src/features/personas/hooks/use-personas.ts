import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "../api/personas-api";

export const personaKeys = {
  list: ["personas"] as const,
  groups: ["persona-groups"] as const,
};

export function usePersonas() {
  return useQuery({
    queryKey: personaKeys.list,
    queryFn: () => api.get<unknown[]>("/characters/personas/list"),
    staleTime: 5 * 60_000,
  });
}

export function useUpdatePersona() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({
      id,
      ...data
    }: {
      id: string;
      name?: string;
      comment?: string;
      description?: string;
      personality?: string;
      scenario?: string;
      backstory?: string;
      appearance?: string;
      tags?: string;
    }) => api.patch(`/characters/personas/${id}`, data),
    onSuccess: () => qc.invalidateQueries({ queryKey: personaKeys.list }),
  });
}

export function useDeletePersona() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => api.delete(`/characters/personas/${id}`),
    onSuccess: () => qc.invalidateQueries({ queryKey: personaKeys.list }),
  });
}

export function useDuplicatePersona() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => api.post(`/characters/personas/${id}/duplicate`, {}),
    onSuccess: () => qc.invalidateQueries({ queryKey: personaKeys.list }),
  });
}

export function useActivatePersona() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => api.put(`/characters/personas/${id}/activate`, {}),
    onSuccess: () => qc.invalidateQueries({ queryKey: personaKeys.list }),
  });
}

export function useUploadPersonaAvatar() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, avatar, filename }: { id: string; avatar: string; filename?: string }) =>
      api.post(`/characters/personas/${id}/avatar`, { avatar, filename }),
    onSuccess: () => qc.invalidateQueries({ queryKey: personaKeys.list }),
  });
}

export function usePersonaGroups() {
  return useQuery({
    queryKey: personaKeys.groups,
    queryFn: () => api.get<unknown[]>("/characters/persona-groups/list"),
  });
}

export function useCreatePersonaGroup() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: { name: string; description?: string; personaIds?: string[] }) =>
      api.post("/characters/persona-groups", data),
    onSuccess: () => qc.invalidateQueries({ queryKey: personaKeys.groups }),
  });
}

export function useUpdatePersonaGroup() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, ...data }: { id: string; name?: string; description?: string; personaIds?: string[] }) =>
      api.patch(`/characters/persona-groups/${id}`, data),
    onSuccess: () => qc.invalidateQueries({ queryKey: personaKeys.groups }),
  });
}

export function useDeletePersonaGroup() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => api.delete(`/characters/persona-groups/${id}`),
    onSuccess: () => qc.invalidateQueries({ queryKey: personaKeys.groups }),
  });
}

