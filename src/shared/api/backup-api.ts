import { api } from "./api-client";

export type BackupExportFormat = "native" | "compatible" | "compatible-png";

export interface DownloadPayload {
  blob: Blob;
  filename: string;
}

function jsonBlob(value: unknown, type = "application/json") {
  return new Blob([JSON.stringify(value, null, 2)], { type });
}

function timestampedBackupName() {
  const timestamp = new Date().toISOString().replace(/[:.]/g, "-").replace("T", "_").slice(0, 19);
  return `marinara-backup-${timestamp}.zip`;
}

export async function exportProfile(format: BackupExportFormat): Promise<DownloadPayload> {
  const value = await api.get(`/backup/export-profile?format=${format}`);
  return {
    blob: jsonBlob(value),
    filename: format === "compatible" ? "marinara-compatible-export.zip" : "marinara-profile.json",
  };
}

export async function createBackupArchive(): Promise<DownloadPayload> {
  const value = await api.post("/backup/download", {});
  return {
    blob: jsonBlob(value, "application/zip"),
    filename: timestampedBackupName(),
  };
}

export async function importProfile<T>(envelope: unknown): Promise<T> {
  return api.post<T>("/backup/import-profile", envelope);
}

export const backupApi = {
  exportProfile,
  createBackupArchive,
  importProfile,
};
