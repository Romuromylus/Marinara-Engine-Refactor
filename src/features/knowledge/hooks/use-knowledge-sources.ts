// ──────────────────────────────────────────────
// React Query: Knowledge Source file hooks
// ──────────────────────────────────────────────
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "../../../shared/lib/api-client";
import { knowledgeSourcesApi } from "../../../shared/api/integration-utility-api";

export interface KnowledgeSource {
  id: string;
  originalName: string;
  filename: string;
  size: number;
  uploadedAt: string;
}

const ksKeys = {
  all: ["knowledge-sources"] as const,
  list: () => [...ksKeys.all, "list"] as const,
};

export function useKnowledgeSources() {
  return useQuery({
    queryKey: ksKeys.list(),
    queryFn: () => api.get<KnowledgeSource[]>("/knowledge-sources"),
    staleTime: 60_000,
  });
}

export function useUploadKnowledgeSource() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (file: File) => knowledgeSourcesApi.upload(file) as Promise<KnowledgeSource>,
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ksKeys.all });
    },
  });
}

export function useDeleteKnowledgeSource() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => api.delete(`/knowledge-sources/${id}`),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ksKeys.all });
    },
  });
}
