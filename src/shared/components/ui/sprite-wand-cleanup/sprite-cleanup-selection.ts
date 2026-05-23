import type { NeighborMode } from "./sprite-cleanup-types";

export function visitNeighbors(
  index: number,
  width: number,
  totalPixels: number,
  mode: NeighborMode,
  visit: (neighbor: number) => void,
) {
  const x = index % width;
  const hasLeft = x > 0;
  const hasRight = x < width - 1;
  const hasTop = index >= width;
  const hasBottom = index < totalPixels - width;

  if (hasLeft) visit(index - 1);
  if (hasRight) visit(index + 1);
  if (hasTop) {
    visit(index - width);
    if (mode === "all") {
      if (hasLeft) visit(index - width - 1);
      if (hasRight) visit(index - width + 1);
    }
  }
  if (hasBottom) {
    visit(index + width);
    if (mode === "all") {
      if (hasLeft) visit(index + width - 1);
      if (hasRight) visit(index + width + 1);
    }
  }
}

export function clearSelection(imageData: ImageData, selected: Uint8Array): number {
  const { data } = imageData;
  let removed = 0;

  for (let index = 0; index < selected.length; index += 1) {
    if (!selected[index]) continue;

    const offset = index * 4;
    const originalAlpha = data[offset + 3] ?? 0;
    data[offset + 3] = 0;
    if (originalAlpha !== 0) removed += 1;
  }

  return removed;
}

export function expandSelection(
  selected: Uint8Array,
  width: number,
  totalPixels: number,
  steps: number,
  mode: NeighborMode,
  canSelect: (index: number, toleranceBoost: number) => boolean,
): Uint8Array {
  const expandSteps = Math.min(4, Math.max(0, Math.trunc(steps)));
  if (expandSteps === 0) return selected;

  let current = new Uint8Array(selected);

  for (let step = 0; step < expandSteps; step += 1) {
    const next = new Uint8Array(current);
    const toleranceBoost = 1.08 + step * 0.08;

    for (let index = 0; index < totalPixels; index += 1) {
      if (!current[index]) continue;

      visitNeighbors(index, width, totalPixels, mode, (neighbor) => {
        if (current[neighbor] || !canSelect(neighbor, toleranceBoost)) return;
        next[neighbor] = 1;
      });
    }

    current = next;
  }

  return current;
}

export function buildEdgeBand(
  selected: Uint8Array,
  width: number,
  totalPixels: number,
  radius: number,
  mode: NeighborMode,
): Uint8Array {
  const edgeDistance = new Uint8Array(totalPixels);
  const edgeQueue = new Int32Array(totalPixels);
  let queueLength = 0;

  const pushEdgePixel = (index: number, nextDistance: number) => {
    if (selected[index] || edgeDistance[index] !== 0 || nextDistance > radius) return;

    edgeDistance[index] = nextDistance;
    edgeQueue[queueLength++] = index;
  };

  for (let index = 0; index < totalPixels; index += 1) {
    if (!selected[index]) continue;

    visitNeighbors(index, width, totalPixels, mode, (neighbor) => {
      pushEdgePixel(neighbor, 1);
    });
  }

  for (let queueIndex = 0; queueIndex < queueLength; queueIndex += 1) {
    const index = edgeQueue[queueIndex];
    const nextDistance = edgeDistance[index] + 1;
    if (nextDistance > radius) continue;

    visitNeighbors(index, width, totalPixels, mode, (neighbor) => pushEdgePixel(neighbor, nextDistance));
  }

  return edgeDistance;
}

export function buildSelectedEdgeDistance(
  selected: Uint8Array,
  sourceData: Uint8ClampedArray,
  width: number,
  totalPixels: number,
  radius: number,
  mode: NeighborMode,
): Uint8Array {
  const edgeDistance = new Uint8Array(totalPixels);
  const edgeQueue = new Int32Array(totalPixels);
  let queueLength = 0;

  const pushSelectedPixel = (index: number, nextDistance: number) => {
    if (!selected[index] || edgeDistance[index] !== 0 || nextDistance > radius) return;

    edgeDistance[index] = nextDistance;
    edgeQueue[queueLength++] = index;
  };

  for (let index = 0; index < totalPixels; index += 1) {
    if (!selected[index]) continue;

    let touchesKeptOpaquePixel = false;
    visitNeighbors(index, width, totalPixels, mode, (neighbor) => {
      if (selected[neighbor]) return;

      const neighborAlpha = sourceData[neighbor * 4 + 3] ?? 0;
      if (neighborAlpha > 8) touchesKeptOpaquePixel = true;
    });

    if (touchesKeptOpaquePixel) pushSelectedPixel(index, 1);
  }

  for (let queueIndex = 0; queueIndex < queueLength; queueIndex += 1) {
    const index = edgeQueue[queueIndex];
    const nextDistance = edgeDistance[index] + 1;
    if (nextDistance > radius) continue;

    visitNeighbors(index, width, totalPixels, mode, (neighbor) => pushSelectedPixel(neighbor, nextDistance));
  }

  return edgeDistance;
}
