export type Rgb = [number, number, number];

function clamp(value: number, min: number, max: number): number {
  return Math.max(min, Math.min(max, Number.isFinite(value) ? value : min));
}

export function clampUnit(value: number): number {
  return clamp(value, 0, 1);
}

export function colorDistanceSquared(data: Uint8ClampedArray, offset: number, target: Rgb): number {
  const red = data[offset] - target[0];
  const green = data[offset + 1] - target[1];
  const blue = data[offset + 2] - target[2];
  return red * red + green * green + blue * blue;
}
