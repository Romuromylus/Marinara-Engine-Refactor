export interface DownloadPayload {
  blob: Blob;
  filename: string;
}

function jsonBlob(value: unknown, type = "application/json") {
  return new Blob([JSON.stringify(value, null, 2)], { type });
}

function base64ToBlob(base64: string, contentType: string) {
  const binary = atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i += 1) bytes[i] = binary.charCodeAt(i);
  return new Blob([bytes], { type: contentType });
}

export function downloadPayloadFromApiValue(
  value: unknown,
  fallbackFilename: string,
  fallbackType = "application/json",
): DownloadPayload {
  if (value && typeof value === "object") {
    const record = value as {
      base64?: unknown;
      data?: unknown;
      body?: unknown;
      contentType?: unknown;
      mimeType?: unknown;
      filename?: unknown;
    };
    const base64 =
      typeof record.base64 === "string"
        ? record.base64
        : typeof record.data === "string"
          ? record.data
          : typeof record.body === "string"
            ? record.body
            : null;
    if (base64) {
      return {
        blob: base64ToBlob(
          base64,
          typeof record.contentType === "string"
            ? record.contentType
            : typeof record.mimeType === "string"
              ? record.mimeType
              : fallbackType,
        ),
        filename: typeof record.filename === "string" && record.filename.trim() ? record.filename : fallbackFilename,
      };
    }
  }
  return { blob: jsonBlob(value, fallbackType), filename: fallbackFilename };
}

export function triggerDownload({ blob, filename }: DownloadPayload) {
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = filename;
  anchor.click();
  URL.revokeObjectURL(url);
}
