import type { MemoryGateway } from "../../engine/capabilities/memory";
import {
  memoryEventDraftSchema,
  memoryEventSchema,
  memoryLayoutInfoSchema,
  memoryListOptionsSchema,
  memoryManifestSchema,
  memoryNoteDraftSchema,
  memoryNotePatchSchema,
  memoryNoteSchema,
  memoryRebuildRequestSchema,
  memoryRebuildResultSchema,
  memoryValidationReportSchema,
} from "../../engine/contracts/schemas/memory.schema";
import type {
  MemoryEventDraft,
  MemoryListOptions,
  MemoryNoteDraft,
  MemoryNotePatch,
  MemoryRebuildRequest,
} from "../../engine/contracts/types/memory";
import { invokeTauri } from "./tauri-client";

const memoryNotesSchema = memoryNoteSchema.array();

export const memoryApi: MemoryGateway = {
  ensureLayout: async () => memoryLayoutInfoSchema.parse(await invokeTauri("memory_ensure_layout")),
  listNotes: async (options?: MemoryListOptions) =>
    memoryNotesSchema.parse(
      await invokeTauri("memory_list_notes", {
        options: options ? memoryListOptionsSchema.parse(options) : null,
      }),
    ),
  getNote: (id: string) =>
    invokeTauri("memory_get_note", {
      id,
    }).then((note) => (note === null ? null : memoryNoteSchema.parse(note))),
  createNote: async (value: MemoryNoteDraft) =>
    memoryNoteSchema.parse(
      await invokeTauri("memory_create_note", {
        value: memoryNoteDraftSchema.parse(value),
      }),
    ),
  updateNote: async (id: string, patch: MemoryNotePatch) =>
    memoryNoteSchema.parse(
      await invokeTauri("memory_update_note", {
        id,
        patch: memoryNotePatchSchema.parse(patch),
      }),
    ),
  archiveNote: async (id: string) =>
    memoryNoteSchema.parse(
      await invokeTauri("memory_archive_note", {
        id,
      }),
    ),
  appendEvent: async (value: MemoryEventDraft) =>
    memoryEventSchema.parse(
      await invokeTauri("memory_append_event", {
        value: memoryEventDraftSchema.parse(value),
      }),
    ),
  getManifest: async () => memoryManifestSchema.parse(await invokeTauri("memory_get_manifest")),
  validateVault: async () => memoryValidationReportSchema.parse(await invokeTauri("memory_validate_vault")),
  rebuildIndexes: async (request?: MemoryRebuildRequest) =>
    memoryRebuildResultSchema.parse(
      await invokeTauri("memory_rebuild_indexes", {
        request: request ? memoryRebuildRequestSchema.parse(request) : null,
      }),
    ),
};
