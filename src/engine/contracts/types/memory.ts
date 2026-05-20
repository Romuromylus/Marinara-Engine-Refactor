export type MemoryVisibility = "shared" | "private" | "model_only";

export type MemoryMode = "chat" | "roleplay" | "game";

export type MemoryNoteType =
  | "fact"
  | "preference"
  | "summary"
  | "relationship"
  | "world"
  | "quest"
  | "scene"
  | "custom";

export type MemoryNoteStatus = "active" | "archived";

export type MemoryEventType =
  | "created"
  | "updated"
  | "archived"
  | "section_added"
  | "section_updated"
  | "linked"
  | "unlinked"
  | "validated"
  | "reindexed";

export interface MemoryScope {
  universeId: string;
  conversationId: string | null;
  roleplayId: string | null;
  gameId: string | null;
  visibility: MemoryVisibility;
}

export interface MemorySection {
  text: string;
  confidence: number;
  salience: number;
  visibility?: MemoryVisibility;
  gates?: string[];
  evidence?: string[];
  updatedAt?: string;
}

export interface MemoryLink {
  noteId: string;
  relationship: string;
}

export interface MemoryNote {
  id: string;
  type: MemoryNoteType;
  status: MemoryNoteStatus;
  modes: MemoryMode[];
  scope: MemoryScope;
  tags: string[];
  links: MemoryLink[];
  sections: Record<string, MemorySection>;
  createdAt: string;
  updatedAt: string;
  version: number;
  previousHash: string | null;
}

export interface MemoryNoteDraft {
  type: MemoryNoteType;
  modes?: MemoryMode[];
  scope: MemoryScope;
  tags?: string[];
  links?: MemoryLink[];
  sections: Record<string, MemorySection>;
}

export interface MemoryNotePatch {
  type?: MemoryNoteType;
  status?: MemoryNoteStatus;
  modes?: MemoryMode[];
  scope?: MemoryScope;
  tags?: string[];
  links?: MemoryLink[];
  sections?: Record<string, MemorySection>;
}

export interface MemoryListOptions {
  status?: MemoryNoteStatus;
  types?: MemoryNoteType[];
  modes?: MemoryMode[];
  scope?: Partial<MemoryScope>;
  tags?: string[];
  includeArchived?: boolean;
  limit?: number;
}

export interface MemoryEventTarget {
  kind: "note" | "section" | "manifest" | "vault";
  id?: string;
  section?: string;
}

export interface MemoryEvent {
  ts: string;
  type: MemoryEventType;
  target: MemoryEventTarget;
  field?: string;
  old?: unknown;
  new?: unknown;
  cause?: string;
  turn?: number;
  mode?: MemoryMode;
  scope?: MemoryScope;
}

export type MemoryEventDraft = Omit<MemoryEvent, "ts"> & {
  ts?: string;
};

export interface MemoryManifestFile {
  path: string;
  hash: string;
  size: number;
  updatedAt: string;
}

export interface MemoryManifest {
  version: string;
  embeddingModel: string | null;
  generatedAt: string;
  vaultHash: string;
  files: MemoryManifestFile[];
}

export interface MemoryValidationIssue {
  severity: "error" | "warning";
  path: string;
  message: string;
}

export interface MemoryValidationReport {
  ok: boolean;
  issues: MemoryValidationIssue[];
  staleIndexes: string[];
  counts: {
    notes: number;
    events: number;
    files: number;
  };
}

export interface MemoryRebuildRequest {
  force?: boolean;
  embeddingModel?: string | null;
  noteIds?: string[];
  scope?: MemoryScope | null;
}

export interface MemoryRebuildResult {
  noteCount: number;
  eventCount: number;
  reindexedNoteIds: string[];
  removedNoteIds: string[];
  manifest: MemoryManifest;
  warnings: string[];
}

export interface MemoryLayoutInfo {
  root: string;
  vaultDir: string;
  eventsPath: string;
  indexesDir: string;
  usageDir: string;
  configDir: string;
}
