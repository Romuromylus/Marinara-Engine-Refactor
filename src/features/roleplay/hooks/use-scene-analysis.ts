import { useMutation } from "@tanstack/react-query";
import type { SceneAnalysis } from "@marinara-engine/shared";
import { api } from "../../../shared/lib/api-client";

type SceneAnalysisRequest = {
  chatId?: string;
  connectionId?: string;
  narration: string;
  context?: Record<string, unknown>;
};

export function useSceneAnalysis() {
  return useMutation({
    mutationFn: (request: SceneAnalysisRequest) => api.post<SceneAnalysis>("/scene/analyze", request),
  });
}
