import { BrushCleanupControls } from "./BrushCleanupControls";
import { PreviewBackgroundPicker } from "./PreviewBackgroundPicker";
import { SpriteCleanupToolbar } from "./SpriteCleanupToolbar";
import { WandCleanupControls } from "./WandCleanupControls";
import type { BrushToolMode, CleanupTool, PreviewBackground } from "./sprite-cleanup-model";
import type { BrushMode } from "./sprite-cleanup-types";

interface SpriteCleanupControlsProps {
  tool: CleanupTool;
  activeBrushMode: BrushMode | null;
  brushToolMode: BrushToolMode;
  pickingBrushColor: boolean;
  previewBackground: PreviewBackground;
  loading: boolean;
  applying: boolean;
  zoom: number;
  tolerance: number;
  wandStrong: boolean;
  wandSoftness: number;
  wandFeather: number;
  brushSize: number;
  cleanTolerance: number;
  cleanEdgeGuard: number;
  cleanFeather: number;
  brushColor: string;
  brushOpacity: number;
  brushHardness: number;
  blurStrength: number;
  onSelectTool: (tool: CleanupTool) => void;
  onZoomOut: () => void;
  onFitCanvasToStage: () => void;
  onZoomIn: () => void;
  onResetWandDefaults: () => void;
  onToleranceChange: (value: number) => void;
  onWandStrongChange: (value: boolean) => void;
  onWandSoftnessChange: (value: number) => void;
  onWandFeatherChange: (value: number) => void;
  onBrushSizeChange: (value: number) => void;
  onSelectBrushToolMode: (mode: BrushToolMode) => void;
  onCleanToleranceChange: (value: number) => void;
  onCleanEdgeGuardChange: (value: number) => void;
  onCleanFeatherChange: (value: number) => void;
  onBrushColorChange: (value: string) => void;
  onToggleBrushColorPicker: () => void;
  onBrushOpacityChange: (value: number) => void;
  onBrushHardnessChange: (value: number) => void;
  onBlurStrengthChange: (value: number) => void;
  onPreviewBackgroundChange: (value: PreviewBackground) => void;
}

export function SpriteCleanupControls({
  tool,
  activeBrushMode,
  brushToolMode,
  pickingBrushColor,
  previewBackground,
  loading,
  applying,
  zoom,
  tolerance,
  wandStrong,
  wandSoftness,
  wandFeather,
  brushSize,
  cleanTolerance,
  cleanEdgeGuard,
  cleanFeather,
  brushColor,
  brushOpacity,
  brushHardness,
  blurStrength,
  onSelectTool,
  onZoomOut,
  onFitCanvasToStage,
  onZoomIn,
  onResetWandDefaults,
  onToleranceChange,
  onWandStrongChange,
  onWandSoftnessChange,
  onWandFeatherChange,
  onBrushSizeChange,
  onSelectBrushToolMode,
  onCleanToleranceChange,
  onCleanEdgeGuardChange,
  onCleanFeatherChange,
  onBrushColorChange,
  onToggleBrushColorPicker,
  onBrushOpacityChange,
  onBrushHardnessChange,
  onBlurStrengthChange,
  onPreviewBackgroundChange,
}: SpriteCleanupControlsProps) {
  return (
    <>
      <SpriteCleanupToolbar
        tool={tool}
        loading={loading}
        applying={applying}
        zoom={zoom}
        onSelectTool={onSelectTool}
        onZoomOut={onZoomOut}
        onFitCanvasToStage={onFitCanvasToStage}
        onZoomIn={onZoomIn}
      />

      <div className="grid min-w-0 shrink-0 gap-2 lg:grid-cols-[minmax(0,1fr)_auto]">
        <div className="flex min-w-0 flex-wrap items-center gap-2">
          {tool === "wand" && (
            <WandCleanupControls
              loading={loading}
              applying={applying}
              tolerance={tolerance}
              wandStrong={wandStrong}
              wandSoftness={wandSoftness}
              wandFeather={wandFeather}
              onResetWandDefaults={onResetWandDefaults}
              onToleranceChange={onToleranceChange}
              onWandStrongChange={onWandStrongChange}
              onWandSoftnessChange={onWandSoftnessChange}
              onWandFeatherChange={onWandFeatherChange}
            />
          )}

          {activeBrushMode && (
            <BrushCleanupControls
              tool={tool}
              activeBrushMode={activeBrushMode}
              brushToolMode={brushToolMode}
              pickingBrushColor={pickingBrushColor}
              loading={loading}
              applying={applying}
              brushSize={brushSize}
              cleanTolerance={cleanTolerance}
              cleanEdgeGuard={cleanEdgeGuard}
              cleanFeather={cleanFeather}
              brushColor={brushColor}
              brushOpacity={brushOpacity}
              brushHardness={brushHardness}
              blurStrength={blurStrength}
              onBrushSizeChange={onBrushSizeChange}
              onSelectBrushToolMode={onSelectBrushToolMode}
              onCleanToleranceChange={onCleanToleranceChange}
              onCleanEdgeGuardChange={onCleanEdgeGuardChange}
              onCleanFeatherChange={onCleanFeatherChange}
              onBrushColorChange={onBrushColorChange}
              onToggleBrushColorPicker={onToggleBrushColorPicker}
              onBrushOpacityChange={onBrushOpacityChange}
              onBrushHardnessChange={onBrushHardnessChange}
              onBlurStrengthChange={onBlurStrengthChange}
            />
          )}
        </div>

        <PreviewBackgroundPicker
          previewBackground={previewBackground}
          onPreviewBackgroundChange={onPreviewBackgroundChange}
        />
      </div>
    </>
  );
}
