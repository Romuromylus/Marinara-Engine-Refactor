// ──────────────────────────────────────────────
// Hooks: Custom Themes
// ──────────────────────────────────────────────
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { storageApi } from "../../../shared/api/storage-api";
import type { CreateThemeInput, UpdateThemeInput } from "../../../engine/contracts/schemas/theme.schema";
import type { Theme } from "../../../engine/contracts/types/theme";

export const themeKeys = {
  all: ["themes"] as const,
  list: () => [...themeKeys.all, "list"] as const,
};

export function findDuplicateTheme(themes: Theme[], name: string, css: string) {
  return themes.find((theme) => theme.name === name && theme.css === css) ?? null;
}

export function useThemes() {
  return useQuery({
    queryKey: themeKeys.list(),
    queryFn: () => storageApi.list<Theme>("themes"),
    staleTime: 0,
    refetchOnWindowFocus: true,
    refetchOnReconnect: true,
    refetchInterval: () => (document.hidden ? false : 15_000),
  });
}

export function useCreateTheme() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: CreateThemeInput) => storageApi.create<Theme>("themes", data as Record<string, unknown>),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: themeKeys.all });
    },
  });
}

export function useUpdateTheme() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, ...data }: { id: string } & UpdateThemeInput) =>
      storageApi.update<Theme>("themes", id, data as Record<string, unknown>),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: themeKeys.all });
    },
  });
}

export function useDeleteTheme() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => storageApi.delete("themes", id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: themeKeys.all });
    },
  });
}

export function useSetActiveTheme() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (id: string | null) => {
      const themes = await storageApi.list<Theme>("themes");
      let selected: Theme | null = null;
      await Promise.all(
        themes.map(async (theme) => {
          const isActive = !!id && theme.id === id;
          const updated = await storageApi.update<Theme>("themes", theme.id, { isActive, active: isActive });
          if (isActive) selected = updated;
        }),
      );
      return selected;
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: themeKeys.all });
    },
  });
}
