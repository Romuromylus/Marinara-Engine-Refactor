import type { LorebookEntry } from "@marinara-engine/shared";

export interface LorebookEmbeddingOptions {
  chatEmbedding?: number[] | null;
  topK?: number;
  threshold?: number;
  localEmbedder?: unknown;
  embeddingSource?: unknown;
  [key: string]: unknown;
}

export interface SemanticLorebookMatch {
  entry: LorebookEntry;
  score: number;
}

export function semanticShortlistLorebookEntries(
  entries: LorebookEntry[],
  _query?: string,
  _options: LorebookEmbeddingOptions = {},
): SemanticLorebookMatch[] {
  return entries.map((entry) => ({ entry, score: 0 }));
}
