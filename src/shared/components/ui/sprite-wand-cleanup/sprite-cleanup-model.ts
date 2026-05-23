import type { CSSProperties } from "react";
import type { BrushMode, BrushStrokeOptions, CanvasPoint, Rgba } from "./sprite-cleanup-types";

export interface HoverPoint extends CanvasPoint {
  color: Rgba;
}

export type CleanupTool = "wand" | "clean" | "erase" | "brush" | "blur" | "pan";
export type BrushToolMode = "paint" | "restore";
export type PreviewBackground = "checker" | "dark" | "light" | "pink";

export interface BrushGesture {
  pointerId: number;
  before: ImageData;
  lastPoint: CanvasPoint;
  changedPixels: number;
  options: BrushStrokeOptions;
  interrupted: boolean;
}

export interface PanGesture {
  pointerId: number;
  startClientX: number;
  startClientY: number;
  startScrollLeft: number;
  startScrollTop: number;
}

interface BrushStrokeBuildInput {
  mode: BrushMode;
  radius: number;
  brushHardness: number;
  brushOpacity: number;
  blurStrength: number;
  cleanTarget: Rgba;
  cleanTolerance: number;
  cleanEdgeGuard: number;
  cleanFeather: number;
  brushColor: string;
}

export const DEFAULT_TOLERANCE = 36;
export const DEFAULT_BRUSH_SIZE = 18;
export const DEFAULT_BRUSH_HARDNESS = 100;
export const DEFAULT_BRUSH_OPACITY = 100;
export const DEFAULT_BRUSH_COLOR = "#ffffff";
export const DEFAULT_BLUR_STRENGTH = 65;
export const DEFAULT_CLEAN_TOLERANCE = 36;
export const DEFAULT_CLEAN_EDGE_GUARD = 45;
export const DEFAULT_CLEAN_FEATHER = 8;
export const DEFAULT_WAND_STRONG = false;
export const DEFAULT_WAND_SOFTNESS = 55;
export const DEFAULT_WAND_FEATHER = 12;
export const WAND_EDGE_GUARD = 55;
export const STRONG_WAND_EDGE_GUARD = 28;
export const WAND_EXPAND = 1;
export const STRONG_WAND_EXPAND = 2;
export const MAX_HISTORY = 12;
const MIN_ZOOM = 0.125;
const MAX_ZOOM = 8;

const checkerboardStyle: CSSProperties = {
  backgroundColor: "var(--secondary)",
  backgroundImage:
    "linear-gradient(45deg, var(--border) 25%, transparent 25%), linear-gradient(-45deg, var(--border) 25%, transparent 25%), linear-gradient(45deg, transparent 75%, var(--border) 75%), linear-gradient(-45deg, transparent 75%, var(--border) 75%)",
  backgroundPosition: "0 0, 0 10px, 10px -10px, -10px 0",
  backgroundSize: "20px 20px",
};

export const previewBackgroundStyles: Record<PreviewBackground, CSSProperties> = {
  checker: checkerboardStyle,
  dark: { backgroundColor: "#161321" },
  light: { backgroundColor: "#f3eef8" },
  pink: { backgroundColor: "#ff4fa3" },
};

export const previewBackgroundOptions: Array<{ key: PreviewBackground; label: string }> = [
  { key: "dark", label: "Dark" },
  { key: "checker", label: "Grid" },
  { key: "light", label: "Light" },
  { key: "pink", label: "Pink" },
];

export const brushActionLabels: Record<BrushMode, string> = {
  clean: "target-cleaned",
  erase: "erased",
  paint: "painted",
  restore: "restored",
  blur: "edge-blurred",
};

function clamp(value: number, min: number, max: number): number {
  return Math.max(min, Math.min(max, Number.isFinite(value) ? value : min));
}

export function cleanupToolToBrushMode(tool: CleanupTool, brushToolMode: BrushToolMode): BrushMode | null {
  switch (tool) {
    case "clean":
    case "erase":
    case "blur":
      return tool;
    case "brush":
      return brushToolMode;
    default:
      return null;
  }
}

export function usesOpacityHardnessControls(tool: CleanupTool): boolean {
  return tool === "erase" || tool === "brush";
}

export function brushOpacityTitle(mode: BrushMode | null): string {
  switch (mode) {
    case "paint":
      return "How much color each brush stroke applies";
    case "restore":
      return "How strongly each stroke restores the original sprite";
    default:
      return "How much alpha each eraser stroke removes";
  }
}

export function brushHardnessTitle(mode: BrushMode | null): string {
  switch (mode) {
    case "paint":
      return "How crisp the brush edge should be";
    case "restore":
      return "How crisp the restore brush edge should be";
    default:
      return "How crisp the eraser edge should be";
  }
}

function colorComponentToHex(value: number): string {
  return clamp(Math.round(value), 0, 255).toString(16).padStart(2, "0");
}

export function rgbaToHex(color: Rgba): string {
  return `#${colorComponentToHex(color[0])}${colorComponentToHex(color[1])}${colorComponentToHex(color[2])}`;
}

export function imageDataEquals(left: ImageData | null, right: ImageData | null): boolean {
  if (!left || !right || left.width !== right.width || left.height !== right.height) return false;
  if (left.data.length !== right.data.length) return false;

  for (let index = 0; index < left.data.length; index += 1) {
    if (left.data[index] !== right.data[index]) return false;
  }

  return true;
}

function hexToRgba(hex: string): Rgba {
  const normalized = /^#[0-9a-f]{6}$/i.test(hex) ? hex : DEFAULT_BRUSH_COLOR;
  return [
    Number.parseInt(normalized.slice(1, 3), 16),
    Number.parseInt(normalized.slice(3, 5), 16),
    Number.parseInt(normalized.slice(5, 7), 16),
    255,
  ];
}

export function createBrushStrokeOptions({
  mode,
  radius,
  brushHardness,
  brushOpacity,
  blurStrength,
  cleanTarget,
  cleanTolerance,
  cleanEdgeGuard,
  cleanFeather,
  brushColor,
}: BrushStrokeBuildInput): BrushStrokeOptions {
  switch (mode) {
    case "clean":
      return {
        mode,
        radius,
        clean: {
          target: cleanTarget,
          tolerance: cleanTolerance,
          edgeGuard: cleanEdgeGuard,
          feather: cleanFeather,
        },
      };
    case "paint":
      return {
        mode,
        radius,
        hardness: brushHardness,
        opacity: brushOpacity,
        paint: { color: hexToRgba(brushColor) },
      };
    case "blur":
      return { mode, radius, blurStrength };
    case "erase":
    case "restore":
      return { mode, radius, hardness: brushHardness, opacity: brushOpacity };
  }
}

export function clampZoom(value: number): number {
  return clamp(value, MIN_ZOOM, MAX_ZOOM);
}
