export async function fetchUrlBlob(
  url: string,
  options: { init?: RequestInit; errorMessage?: string } = {},
): Promise<Blob> {
  const response = await fetch(url, options.init);
  if (!response.ok) {
    throw new Error(options.errorMessage ?? `Failed to load ${url}`);
  }
  return response.blob();
}

export async function fetchUrlArrayBuffer(
  url: string,
  options: { init?: RequestInit; errorMessage?: string } = {},
): Promise<ArrayBuffer> {
  const response = await fetch(url, options.init);
  if (!response.ok) {
    throw new Error(options.errorMessage ?? `Failed to load ${url}`);
  }
  return response.arrayBuffer();
}

export async function blobToDataUrl(blob: Blob, errorMessage = "Failed to convert file."): Promise<string> {
  return await new Promise<string>((resolve, reject) => {
    const reader = new FileReader();
    reader.onloadend = () => {
      if (typeof reader.result === "string") resolve(reader.result);
      else reject(new Error(errorMessage));
    };
    reader.onerror = () => reject(reader.error ?? new Error(errorMessage));
    reader.readAsDataURL(blob);
  });
}

export async function urlToDataUrl(url: string, errorMessage = "Failed to read file."): Promise<string> {
  if (url.startsWith("data:")) return url;
  return blobToDataUrl(await fetchUrlBlob(url, { errorMessage }), errorMessage);
}

export function downloadBlob(blob: Blob, filename: string): void {
  const objectUrl = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = objectUrl;
  anchor.download = filename;
  document.body.appendChild(anchor);
  anchor.click();
  anchor.remove();
  URL.revokeObjectURL(objectUrl);
}
