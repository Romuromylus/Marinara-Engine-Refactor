import { useMutation } from "@tanstack/react-query";
import type { Theme } from "../types";
import { useUIStore, type CustomTheme } from "../../../shared/stores/ui.store";

function toTheme(theme: CustomTheme, activeThemeId: string | null): Theme {
  return {
    id: theme.id,
    name: theme.name,
    css: theme.css,
    installedAt: theme.installedAt,
    isActive: theme.id === activeThemeId,
  };
}

export function useThemes() {
  const themes = useUIStore((s) => s.customThemes);
  const activeThemeId = useUIStore((s) => s.activeCustomTheme);

  return {
    data: themes.map((theme) => toTheme(theme, activeThemeId)),
    isLoading: false,
  };
}

export function useCreateTheme() {
  const addCustomTheme = useUIStore((s) => s.addCustomTheme);
  return useMutation({
    mutationFn: async (theme: Omit<Theme, "id">) => {
      const created: CustomTheme = {
        id: crypto.randomUUID(),
        name: theme.name,
        css: theme.css,
        installedAt: theme.installedAt ?? new Date().toISOString(),
      };
      addCustomTheme(created);
      return toTheme(created, null);
    },
  });
}

export function useUpdateTheme() {
  const updateCustomTheme = useUIStore((s) => s.updateCustomTheme);
  const activeThemeId = useUIStore((s) => s.activeCustomTheme);
  return useMutation({
    mutationFn: async (theme: { id: string; name: string; css: string }) => {
      updateCustomTheme(theme.id, { name: theme.name, css: theme.css });
      return { ...theme, isActive: theme.id === activeThemeId };
    },
  });
}

export function useDeleteTheme() {
  const removeCustomTheme = useUIStore((s) => s.removeCustomTheme);
  return useMutation({
    mutationFn: async (id: string) => {
      removeCustomTheme(id);
    },
  });
}

export function useSetActiveTheme() {
  const setActiveCustomTheme = useUIStore((s) => s.setActiveCustomTheme);
  return useMutation({
    mutationFn: async (id: string | null) => {
      setActiveCustomTheme(id);
    },
  });
}

export function findDuplicateTheme(themes: Theme[], name: string, css: string) {
  const normalizedName = name.trim().toLowerCase();
  const normalizedCss = css.trim();
  return (
    themes.find((theme) => theme.name.trim().toLowerCase() === normalizedName && theme.css.trim() === normalizedCss) ??
    null
  );
}
