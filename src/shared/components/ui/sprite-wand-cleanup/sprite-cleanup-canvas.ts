import { loadUrlBlob } from "../../../lib/url-blob";
import type { CanvasPoint } from "./sprite-cleanup-types";

function clamp(value: number, min: number, max: number): number {
  return Math.max(min, Math.min(max, Number.isFinite(value) ? value : min));
}

export async function loadImageToCanvas(imageUrl: string, canvas: HTMLCanvasElement): Promise<ImageData> {
  const blob = await loadUrlBlob(imageUrl, {
    init: { cache: "no-store" },
    errorMessage: "Sprite image could not be loaded",
  });
  const objectUrl = URL.createObjectURL(blob);

  try {
    const image = await new Promise<HTMLImageElement>((resolve, reject) => {
      const img = new Image();
      img.onload = () => resolve(img);
      img.onerror = () => reject(new Error("Sprite image could not be decoded"));
      img.src = objectUrl;
    });

    canvas.width = image.naturalWidth;
    canvas.height = image.naturalHeight;
    if (canvas.width <= 0 || canvas.height <= 0) throw new Error("Sprite image has no usable size");

    const ctx = canvas.getContext("2d", { willReadFrequently: true });
    if (!ctx) throw new Error("Canvas is unavailable");

    ctx.clearRect(0, 0, canvas.width, canvas.height);
    ctx.drawImage(image, 0, 0);
    return ctx.getImageData(0, 0, canvas.width, canvas.height);
  } finally {
    URL.revokeObjectURL(objectUrl);
  }
}

export function canvasPointFromClient(
  canvas: HTMLCanvasElement | null,
  clientX: number,
  clientY: number,
): CanvasPoint | null {
  if (!canvas || canvas.width <= 0 || canvas.height <= 0) return null;

  const rect = canvas.getBoundingClientRect();
  if (rect.width <= 0 || rect.height <= 0) return null;

  const normalizedX = (clientX - rect.left) / rect.width;
  const normalizedY = (clientY - rect.top) / rect.height;
  if (normalizedX < 0 || normalizedY < 0 || normalizedX > 1 || normalizedY > 1) return null;

  return {
    x: clamp(Math.floor(normalizedX * canvas.width), 0, canvas.width - 1),
    y: clamp(Math.floor(normalizedY * canvas.height), 0, canvas.height - 1),
  };
}
