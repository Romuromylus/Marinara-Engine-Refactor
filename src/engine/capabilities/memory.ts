import type {
  MemoryEvent,
  MemoryEventDraft,
  MemoryLayoutInfo,
  MemoryListOptions,
  MemoryManifest,
  MemoryNote,
  MemoryNoteDraft,
  MemoryNotePatch,
  MemoryRebuildRequest,
  MemoryRebuildResult,
  MemoryValidationReport,
} from "../contracts/types/memory";

export interface MemoryGateway {
  ensureLayout(): Promise<MemoryLayoutInfo>;
  listNotes(options?: MemoryListOptions): Promise<MemoryNote[]>;
  getNote(id: string): Promise<MemoryNote | null>;
  createNote(value: MemoryNoteDraft): Promise<MemoryNote>;
  updateNote(id: string, patch: MemoryNotePatch): Promise<MemoryNote>;
  archiveNote(id: string): Promise<MemoryNote>;
  appendEvent(value: MemoryEventDraft): Promise<MemoryEvent>;
  getManifest(): Promise<MemoryManifest>;
  validateVault(): Promise<MemoryValidationReport>;
  rebuildIndexes(request?: MemoryRebuildRequest): Promise<MemoryRebuildResult>;
}
