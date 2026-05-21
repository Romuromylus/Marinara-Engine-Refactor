import { clampUnit, colorDistanceSquared, type Rgb } from "./sprite-cleanup-color";
import type { BrushStrokeOptions, CanvasPoint, Rgba } from "./sprite-cleanup-types";

interface BrushStampSourceOptions {
  blurSource?: Uint8ClampedArray | null;
  cleanSource?: Uint8ClampedArray | null;
}

function brushFalloff(dx: number, dy: number, radius: number, hardnessAmount: number): number {
  const normalizedDistance = Math.sqrt(dx * dx + dy * dy) / Math.max(1, radius);
  const hardCore = hardnessAmount >= 0.99 ? 1 : Math.pow(hardnessAmount, 1.35);
  const featherCurve = 1 + (1 - hardnessAmount) * 1.8;
  return normalizedDistance <= hardCore
    ? 1
    : Math.pow(clampUnit((1 - normalizedDistance) / Math.max(0.001, 1 - hardCore)), featherCurve);
}

function writeRgbaIfChanged(data: Uint8ClampedArray, offset: number, next: Rgba): boolean {
  if (
    data[offset] === next[0] &&
    data[offset + 1] === next[1] &&
    data[offset + 2] === next[2] &&
    data[offset + 3] === next[3]
  ) {
    return false;
  }

  data[offset] = next[0];
  data[offset + 1] = next[1];
  data[offset + 2] = next[2];
  data[offset + 3] = next[3];
  return true;
}

function compositeSourceOver(data: Uint8ClampedArray, offset: number, source: Rgba, sourceAlpha: number): Rgba {
  const alpha = clampUnit(sourceAlpha);
  const currentAlpha = clampUnit((data[offset + 3] ?? 0) / 255);
  const retainedAlpha = currentAlpha * (1 - alpha);
  const nextAlphaAmount = alpha + retainedAlpha;

  if (nextAlphaAmount <= 0) return [0, 0, 0, 0];

  return [
    Math.round((source[0] * alpha + (data[offset] ?? 0) * retainedAlpha) / nextAlphaAmount),
    Math.round((source[1] * alpha + (data[offset + 1] ?? 0) * retainedAlpha) / nextAlphaAmount),
    Math.round((source[2] * alpha + (data[offset + 2] ?? 0) * retainedAlpha) / nextAlphaAmount),
    Math.round(nextAlphaAmount * 255),
  ];
}

function blendPixelToward(data: Uint8ClampedArray, offset: number, target: Rgba, amount: number): Rgba {
  const mix = clampUnit(amount);
  const currentAlpha = clampUnit((data[offset + 3] ?? 0) / 255);
  const targetAlpha = clampUnit(target[3] / 255);
  const currentAlphaWeight = currentAlpha * (1 - mix);
  const targetAlphaWeight = targetAlpha * mix;
  const nextAlphaAmount = currentAlphaWeight + targetAlphaWeight;

  if (nextAlphaAmount <= 0) return [0, 0, 0, 0];

  return [
    Math.round(((data[offset] ?? 0) * currentAlphaWeight + target[0] * targetAlphaWeight) / nextAlphaAmount),
    Math.round(((data[offset + 1] ?? 0) * currentAlphaWeight + target[1] * targetAlphaWeight) / nextAlphaAmount),
    Math.round(((data[offset + 2] ?? 0) * currentAlphaWeight + target[2] * targetAlphaWeight) / nextAlphaAmount),
    Math.round(nextAlphaAmount * 255),
  ];
}

export function applyBrushStamp(
  imageData: ImageData,
  originalImage: ImageData | null,
  centerX: number,
  centerY: number,
  options: BrushStrokeOptions,
  sources: BrushStampSourceOptions = {},
): number {
  const { data, width, height } = imageData;
  const { mode, radius } = options;
  const restoreSource = originalImage?.data ?? null;
  const blurSource = mode === "blur" ? (sources.blurSource ?? new Uint8ClampedArray(data)) : null;
  const cleanSource = mode === "clean" ? (sources.cleanSource ?? new Uint8ClampedArray(data)) : null;
  const cleanOptions = options.mode === "clean" ? options.clean : null;
  const paintOptions = options.mode === "paint" ? options.paint : null;
  const cleanTarget = cleanOptions?.target ?? null;
  const cleanTargetRgb: Rgb | null = cleanTarget ? [cleanTarget[0], cleanTarget[1], cleanTarget[2]] : null;
  const cleanTolerance = Math.max(1, cleanOptions?.tolerance ?? 1);
  const cleanEdgeGuardAmount = clampUnit((cleanOptions?.edgeGuard ?? 0) / 100);
  const cleanFeatherAmount = clampUnit((cleanOptions?.feather ?? 0) / 100);
  const paintColor = paintOptions?.color ?? null;
  const paintColorAlpha = clampUnit((paintColor?.[3] ?? 0) / 255);
  const softBrushOptions =
    options.mode === "paint" || options.mode === "erase" || options.mode === "restore" ? options : null;
  const brushHardnessAmount = softBrushOptions ? clampUnit(softBrushOptions.hardness / 100) : 0;
  const brushOpacityAmount = softBrushOptions ? clampUnit(softBrushOptions.opacity / 100) : 0;
  const blurAmount = options.mode === "blur" ? clampUnit(options.blurStrength / 100) : 0;
  const minX = Math.max(0, Math.floor(centerX - radius));
  const maxX = Math.min(width - 1, Math.ceil(centerX + radius));
  const minY = Math.max(0, Math.floor(centerY - radius));
  const maxY = Math.min(height - 1, Math.ceil(centerY + radius));
  const radiusSquared = radius * radius;
  let changed = 0;

  for (let y = minY; y <= maxY; y += 1) {
    for (let x = minX; x <= maxX; x += 1) {
      const dx = x - centerX;
      const dy = y - centerY;
      if (dx * dx + dy * dy > radiusSquared) continue;

      const offset = (y * width + x) * 4;
      if (mode === "clean") {
        if (!cleanSource || !cleanTarget || !cleanTargetRgb || cleanTarget[3] <= 8) continue;

        const originalAlpha = data[offset + 3] ?? 0;
        if (originalAlpha <= 0) continue;

        const targetDistance = Math.sqrt(colorDistanceSquared(cleanSource, offset, cleanTargetRgb));
        if (targetDistance > cleanTolerance) continue;

        if (cleanEdgeGuardAmount > 0) {
          let neighborCount = 0;
          let foregroundNeighbors = 0;

          for (let sampleY = Math.max(0, y - 1); sampleY <= Math.min(height - 1, y + 1); sampleY += 1) {
            for (let sampleX = Math.max(0, x - 1); sampleX <= Math.min(width - 1, x + 1); sampleX += 1) {
              if (sampleX === x && sampleY === y) continue;

              const sampleOffset = (sampleY * width + sampleX) * 4;
              const sampleAlpha = cleanSource[sampleOffset + 3] ?? 0;
              if (sampleAlpha <= 32) continue;

              neighborCount += 1;
              const neighborTargetDistance = Math.sqrt(colorDistanceSquared(cleanSource, sampleOffset, cleanTargetRgb));
              if (neighborTargetDistance > cleanTolerance * (1.04 - cleanEdgeGuardAmount * 0.18)) {
                foregroundNeighbors += 1;
              }
            }
          }

          const weakTargetMatch = targetDistance > cleanTolerance * (0.22 + (1 - cleanEdgeGuardAmount) * 0.44);
          const crowdedByForeground =
            neighborCount > 0 && foregroundNeighbors / neighborCount > 0.22 + (1 - cleanEdgeGuardAmount) * 0.45;
          if (weakTargetMatch && crowdedByForeground) continue;
        }

        const normalizedDistance = Math.sqrt(dx * dx + dy * dy) / Math.max(1, radius);
        const hardCore = cleanFeatherAmount <= 0.01 ? 1 : 1 - cleanFeatherAmount * 0.84;
        const eraseAmount =
          normalizedDistance <= hardCore
            ? 1
            : Math.pow(clampUnit((1 - normalizedDistance) / Math.max(0.001, 1 - hardCore)), 1.65);
        const nextAlpha = Math.round(originalAlpha * (1 - eraseAmount));
        if (nextAlpha === originalAlpha) continue;

        data[offset + 3] = nextAlpha;
        changed += 1;
        continue;
      }

      if (mode === "blur") {
        if (!blurSource || blurAmount <= 0) continue;

        const originalAlpha = blurSource[offset + 3] ?? 0;
        if (originalAlpha <= 8) continue;

        let minAlpha = originalAlpha;
        let maxAlpha = originalAlpha;
        let alphaTotal = 0;
        let weightTotal = 0;

        for (let sampleY = Math.max(0, y - 1); sampleY <= Math.min(height - 1, y + 1); sampleY += 1) {
          for (let sampleX = Math.max(0, x - 1); sampleX <= Math.min(width - 1, x + 1); sampleX += 1) {
            const sampleOffset = (sampleY * width + sampleX) * 4;
            const sampleAlpha = blurSource[sampleOffset + 3] ?? 0;
            const distance = Math.max(1, Math.hypot(sampleX - x, sampleY - y));
            const weight = sampleX === x && sampleY === y ? 1.75 : 1 / distance;

            minAlpha = Math.min(minAlpha, sampleAlpha);
            maxAlpha = Math.max(maxAlpha, sampleAlpha);
            alphaTotal += sampleAlpha * weight;
            weightTotal += weight;
          }
        }

        if (maxAlpha - minAlpha < 24 || weightTotal <= 0) continue;

        const averagedAlpha = Math.round(alphaTotal / weightTotal);
        const nextAlpha = Math.min(
          originalAlpha,
          Math.round(originalAlpha * (1 - blurAmount) + averagedAlpha * blurAmount),
        );
        if (nextAlpha === originalAlpha) continue;

        data[offset + 3] = nextAlpha;
        changed += 1;
        continue;
      }

      if (mode === "paint") {
        if (!paintColor || brushOpacityAmount <= 0 || paintColorAlpha <= 0) continue;

        const paintAmount = brushFalloff(dx, dy, radius, brushHardnessAmount);
        const sourceAlpha = paintAmount * brushOpacityAmount * paintColorAlpha;
        if (sourceAlpha <= 0.001) continue;

        if (writeRgbaIfChanged(data, offset, compositeSourceOver(data, offset, paintColor, sourceAlpha))) {
          changed += 1;
        }
        continue;
      }

      if (mode === "erase") {
        const originalAlpha = data[offset + 3] ?? 0;
        if (originalAlpha <= 0) continue;

        const eraseAmount = brushFalloff(dx, dy, radius, brushHardnessAmount);
        const nextAlpha = Math.round(originalAlpha * (1 - eraseAmount * brushOpacityAmount));
        if (nextAlpha === originalAlpha) continue;

        data[offset + 3] = nextAlpha;
        changed += 1;
        continue;
      }

      if (mode === "restore") {
        if (!restoreSource) continue;

        const restoreAmount = brushFalloff(dx, dy, radius, brushHardnessAmount) * brushOpacityAmount;
        if (restoreAmount <= 0) continue;

        const restoredColor: Rgba = [
          restoreSource[offset] ?? 0,
          restoreSource[offset + 1] ?? 0,
          restoreSource[offset + 2] ?? 0,
          restoreSource[offset + 3] ?? 0,
        ];

        if (restoreAmount < 0.999) {
          if (writeRgbaIfChanged(data, offset, blendPixelToward(data, offset, restoredColor, restoreAmount))) {
            changed += 1;
          }
          continue;
        }

        if (writeRgbaIfChanged(data, offset, restoredColor)) {
          changed += 1;
        }
      }
    }
  }

  return changed;
}

export function applyBrushLine(
  imageData: ImageData,
  originalImage: ImageData | null,
  from: CanvasPoint,
  to: CanvasPoint,
  options: BrushStrokeOptions,
): number {
  if (from.x === to.x && from.y === to.y) return 0;

  const distance = Math.hypot(to.x - from.x, to.y - from.y);
  const steps = Math.max(1, Math.ceil(distance / Math.max(1, options.radius * 0.45)));
  const sources: BrushStampSourceOptions = {
    blurSource: options.mode === "blur" ? new Uint8ClampedArray(imageData.data) : null,
    cleanSource: options.mode === "clean" ? new Uint8ClampedArray(imageData.data) : null,
  };
  let changed = 0;

  for (let step = 1; step <= steps; step += 1) {
    const amount = step / steps;
    changed += applyBrushStamp(
      imageData,
      originalImage,
      from.x + (to.x - from.x) * amount,
      from.y + (to.y - from.y) * amount,
      options,
      sources,
    );
  }

  return changed;
}
