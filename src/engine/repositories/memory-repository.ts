import type { MemoryGateway } from "../capabilities/memory";
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

export interface MemoryRepository {
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

export function createMemoryRepository(memory: MemoryGateway): MemoryRepository {
  return {
    ensureLayout: () => memory.ensureLayout(),
    listNotes: (options) => memory.listNotes(options),
    getNote: (id) => memory.getNote(id),
    createNote: (value) => memory.createNote(value),
    updateNote: (id, patch) => memory.updateNote(id, patch),
    archiveNote: (id) => memory.archiveNote(id),
    appendEvent: (value) => memory.appendEvent(value),
    getManifest: () => memory.getManifest(),
    validateVault: () => memory.validateVault(),
    rebuildIndexes: (request) => memory.rebuildIndexes(request),
  };
}
