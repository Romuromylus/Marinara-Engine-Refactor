import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

export interface Extension {
  id: string;
  name: string;
  description?: string;
  enabled: boolean;
  installedAt?: string;
  css?: string;
  js?: string;
  code?: string;
}

const extensionsKey = ["settings", "extensions"] as const;

export function useExtensions() {
  return useQuery<Extension[]>({
    queryKey: extensionsKey,
    queryFn: async () => [],
    staleTime: Infinity,
  });
}

export function useCreateExtension() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (extension: Omit<Extension, "id">) => {
      const created: Extension = { id: crypto.randomUUID(), ...extension };
      queryClient.setQueryData<Extension[]>(extensionsKey, (extensions = []) => [...extensions, created]);
      return created;
    },
  });
}

export function useUpdateExtension() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (patch: Partial<Extension> & { id: string }) => {
      queryClient.setQueryData<Extension[]>(extensionsKey, (extensions = []) =>
        extensions.map((extension) => (extension.id === patch.id ? { ...extension, ...patch } : extension)),
      );
      return patch;
    },
  });
}

export function useDeleteExtension() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (id: string) => {
      queryClient.setQueryData<Extension[]>(extensionsKey, (extensions = []) =>
        extensions.filter((extension) => extension.id !== id),
      );
    },
  });
}
