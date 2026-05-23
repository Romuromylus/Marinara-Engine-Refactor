export interface WandResult {
  removed: number;
  target: Rgba;
}

export interface CanvasPoint {
  x: number;
  y: number;
}

export type BrushMode = "erase" | "restore" | "blur" | "clean" | "paint";
export type Rgba = [number, number, number, number];
export type NeighborMode = "cardinal" | "all";

export interface WandCleanupOptions {
  neighborMode?: NeighborMode;
  edgeGuard: number;
  expand: number;
  softness: number;
  feather: number;
}

interface TargetCleanBrushOptions {
  target: Rgba;
  tolerance: number;
  edgeGuard: number;
  feather: number;
}

interface PaintBrushOptions {
  color: Rgba;
}

interface BrushStrokeBaseOptions {
  radius: number;
}

interface SoftBrushStrokeOptions extends BrushStrokeBaseOptions {
  hardness: number;
  opacity: number;
}

export type BrushStrokeOptions =
  | (BrushStrokeBaseOptions & {
      mode: "clean";
      clean: TargetCleanBrushOptions;
    })
  | (SoftBrushStrokeOptions & {
      mode: "paint";
      paint: PaintBrushOptions;
    })
  | (SoftBrushStrokeOptions & {
      mode: "erase" | "restore";
    })
  | (BrushStrokeBaseOptions & {
      mode: "blur";
      blurStrength: number;
    });
