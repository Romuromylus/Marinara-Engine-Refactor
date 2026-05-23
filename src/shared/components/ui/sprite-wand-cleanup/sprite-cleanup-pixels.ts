import type { CanvasPoint, Rgba } from "./sprite-cleanup-types";

export function cloneImageData(imageData: ImageData): ImageData {
  return new ImageData(new Uint8ClampedArray(imageData.data), imageData.width, imageData.height);
}

export function rgbaAt(imageData: ImageData, point: CanvasPoint): Rgba {
  const offset = (point.y * imageData.width + point.x) * 4;
  return [
    imageData.data[offset] ?? 0,
    imageData.data[offset + 1] ?? 0,
    imageData.data[offset + 2] ?? 0,
    imageData.data[offset + 3] ?? 0,
  ];
}

export function formatRgba(color: Rgba): string {
  return `rgba(${color[0]}, ${color[1]}, ${color[2]}, ${color[3]})`;
}
