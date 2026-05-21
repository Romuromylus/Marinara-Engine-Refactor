import { clampUnit, colorDistanceSquared, type Rgb } from "./sprite-cleanup-color";
import { clearSelection, expandSelection, visitNeighbors } from "./sprite-cleanup-selection";
import type { Rgba, WandCleanupOptions, WandResult } from "./sprite-cleanup-types";
import { addSelectedSoftHalo, softenKeptCutEdge } from "./sprite-cleanup-wand-edge";

function readPixel(imageData: ImageData, index: number): Rgba {
  const offset = index * 4;
  return [
    imageData.data[offset] ?? 0,
    imageData.data[offset + 1] ?? 0,
    imageData.data[offset + 2] ?? 0,
    imageData.data[offset + 3] ?? 0,
  ];
}

function getEmptyWandResult(imageData: ImageData, startX: number, startY: number): WandResult {
  const target = readPixel(imageData, startY * imageData.width + startX);
  return { removed: 0, target };
}

export function removeWandSelection(
  imageData: ImageData,
  startX: number,
  startY: number,
  tolerance: number,
  options: WandCleanupOptions,
): WandResult {
  const { data, width, height } = imageData;
  const startIndex = startY * width + startX;
  const [red, green, blue, targetAlpha] = readPixel(imageData, startIndex);
  if (targetAlpha <= 8) return getEmptyWandResult(imageData, startX, startY);

  const target: Rgb = [red, green, blue];
  const totalPixels = width * height;
  const sourceData = new Uint8ClampedArray(data);
  const edgeGuardAmount = clampUnit(options.edgeGuard / 100);
  const neighborMode = options.neighborMode ?? "cardinal";

  const canSelect = (index: number, toleranceBoost: number): boolean => {
    const offset = index * 4;
    const alpha = sourceData[offset + 3] ?? 0;
    if (alpha <= 8) return false;

    const boostedTolerance = Math.max(1, tolerance * toleranceBoost);
    const targetDistanceSquared = colorDistanceSquared(sourceData, offset, target);
    if (targetDistanceSquared > boostedTolerance * boostedTolerance) return false;
    if (edgeGuardAmount <= 0) return true;

    const targetDistance = Math.sqrt(targetDistanceSquared);
    if (targetDistance <= tolerance * (0.18 + (1 - edgeGuardAmount) * 0.16)) return true;

    const x = index % width;
    const y = Math.floor(index / width);
    let foregroundNeighbors = 0;
    let neighborCount = 0;
    let closestForegroundDistance = Number.POSITIVE_INFINITY;

    for (let yOffset = -1; yOffset <= 1; yOffset += 1) {
      const sampleY = y + yOffset;
      if (sampleY < 0 || sampleY >= height) continue;

      for (let xOffset = -1; xOffset <= 1; xOffset += 1) {
        if (xOffset === 0 && yOffset === 0) continue;

        const sampleX = x + xOffset;
        if (sampleX < 0 || sampleX >= width) continue;

        const sampleIndex = sampleY * width + sampleX;
        const sampleOffset = sampleIndex * 4;
        const sampleAlpha = sourceData[sampleOffset + 3] ?? 0;
        if (sampleAlpha <= 32) continue;

        neighborCount += 1;
        const neighborTargetDistance = Math.sqrt(colorDistanceSquared(sourceData, sampleOffset, target));
        if (neighborTargetDistance <= tolerance * (1.05 - edgeGuardAmount * 0.18)) continue;

        foregroundNeighbors += 1;
        const redDistance = (sourceData[offset] ?? 0) - (sourceData[sampleOffset] ?? 0);
        const greenDistance = (sourceData[offset + 1] ?? 0) - (sourceData[sampleOffset + 1] ?? 0);
        const blueDistance = (sourceData[offset + 2] ?? 0) - (sourceData[sampleOffset + 2] ?? 0);
        closestForegroundDistance = Math.min(
          closestForegroundDistance,
          Math.hypot(redDistance, greenDistance, blueDistance),
        );
      }
    }

    if (foregroundNeighbors === 0 || neighborCount === 0) return true;

    const edgePressure = foregroundNeighbors / neighborCount;
    const weakTargetMatch = targetDistance > tolerance * (0.36 + (1 - edgeGuardAmount) * 0.32);
    const pulledTowardForeground =
      closestForegroundDistance < targetDistance * (1.1 + edgeGuardAmount * 1.15) + edgeGuardAmount * 10;
    const crowdedByForeground = edgePressure > 0.18 + (1 - edgeGuardAmount) * 0.42;

    return !(weakTargetMatch && pulledTowardForeground && crowdedByForeground);
  };

  const selected = new Uint8Array(totalPixels);
  const visited = new Uint8Array(totalPixels);
  const stack = new Int32Array(totalPixels);
  let stackLength = 0;

  const pushPixel = (index: number) => {
    if (visited[index]) return;
    visited[index] = 1;
    if (!canSelect(index, 1)) return;
    selected[index] = 1;
    stack[stackLength++] = index;
  };

  pushPixel(startIndex);

  while (stackLength > 0) {
    visitNeighbors(stack[--stackLength], width, totalPixels, neighborMode, pushPixel);
  }

  let selectedCount = 0;
  for (let index = 0; index < totalPixels; index += 1) {
    if (selected[index]) selectedCount += 1;
  }

  if (selectedCount === 0) {
    return { removed: 0, target: [target[0], target[1], target[2], targetAlpha] };
  }

  const expandedSelection = expandSelection(selected, width, totalPixels, options.expand, neighborMode, canSelect);
  const removed = clearSelection(imageData, expandedSelection);
  softenKeptCutEdge(imageData, expandedSelection, sourceData, target, tolerance, options.softness, options.feather);
  addSelectedSoftHalo(imageData, expandedSelection, sourceData, target, tolerance, options.feather, options.softness);

  return {
    removed,
    target: [target[0], target[1], target[2], targetAlpha],
  };
}
