import { Brush, Minus, Pipette, Plus, Undo2 } from "lucide-react";
import { RangeControl } from "./SpriteCleanupControlPrimitives";
import {
  brushHardnessTitle,
  brushOpacityTitle,
  usesOpacityHardnessControls,
  type BrushToolMode,
  type CleanupTool,
} from "./sprite-cleanup-model";
import type { BrushMode } from "./sprite-cleanup-types";

interface BrushCleanupControlsProps {
  tool: CleanupTool;
  activeBrushMode: BrushMode;
  brushToolMode: BrushToolMode;
  pickingBrushColor: boolean;
  loading: boolean;
  applying: boolean;
  brushSize: number;
  cleanTolerance: number;
  cleanEdgeGuard: number;
  cleanFeather: number;
  brushColor: string;
  brushOpacity: number;
  brushHardness: number;
  blurStrength: number;
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
}

export function BrushCleanupControls({
  tool,
  activeBrushMode,
  brushToolMode,
  pickingBrushColor,
  loading,
  applying,
  brushSize,
  cleanTolerance,
  cleanEdgeGuard,
  cleanFeather,
  brushColor,
  brushOpacity,
  brushHardness,
  blurStrength,
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
}: BrushCleanupControlsProps) {
  const disabled = loading || applying;

  return (
    <>
      <RangeControl
        label="Brush"
        min={2}
        max={96}
        value={brushSize}
        onChange={onBrushSizeChange}
        disabled={disabled}
        inputClassName="min-w-20"
        before={<Minus size="0.75rem" className="text-[var(--muted-foreground)]" />}
        after={<Plus size="0.75rem" className="text-[var(--muted-foreground)]" />}
      />

      {tool === "brush" && (
        <div className="flex min-w-fit items-center gap-1 rounded-lg bg-[var(--secondary)] p-1 text-xs">
          <button
            type="button"
            onClick={() => onSelectBrushToolMode("paint")}
            disabled={disabled}
            className={[
              "inline-flex items-center gap-1.5 rounded-md px-2.5 py-1.5 font-medium transition-colors disabled:opacity-45",
              brushToolMode === "paint"
                ? "bg-[var(--primary)] text-[var(--primary-foreground)]"
                : "text-[var(--muted-foreground)] hover:bg-[var(--accent)] hover:text-[var(--foreground)]",
            ].join(" ")}
            aria-pressed={brushToolMode === "paint"}
            title="Paint with the selected color"
          >
            <Brush size="0.75rem" />
            Color
          </button>
          <button
            type="button"
            onClick={() => onSelectBrushToolMode("restore")}
            disabled={disabled}
            className={[
              "inline-flex items-center gap-1.5 rounded-md px-2.5 py-1.5 font-medium transition-colors disabled:opacity-45",
              brushToolMode === "restore"
                ? "bg-[var(--primary)] text-[var(--primary-foreground)]"
                : "text-[var(--muted-foreground)] hover:bg-[var(--accent)] hover:text-[var(--foreground)]",
            ].join(" ")}
            aria-pressed={brushToolMode === "restore"}
            title="Paint original pixels back in"
          >
            <Undo2 size="0.75rem" />
            Restore
          </button>
        </div>
      )}

      {tool === "clean" && (
        <>
          <RangeControl
            label="Tolerance"
            min={4}
            max={128}
            value={cleanTolerance}
            onChange={onCleanToleranceChange}
            disabled={disabled}
            title="How closely pixels must match the sampled cleanup color"
            className="min-w-[12rem] flex-[1_0_12rem]"
          />
          <RangeControl
            label="Edge Guard"
            min={0}
            max={100}
            value={cleanEdgeGuard}
            onChange={onCleanEdgeGuardChange}
            disabled={disabled}
            title="How strongly the brush avoids character-like edge pixels"
            className="min-w-[16rem] flex-[1_0_16rem]"
          />
          <RangeControl
            label="Feather"
            min={0}
            max={100}
            value={cleanFeather}
            onChange={onCleanFeatherChange}
            disabled={disabled}
            title="Soften the edge of the cleaned brush stroke"
            className="min-w-[14rem] flex-[1_0_14rem]"
          />
        </>
      )}

      {tool === "brush" && brushToolMode === "paint" && (
        <div
          className="flex min-w-fit items-center gap-2 rounded-lg bg-[var(--secondary)] px-3 py-2 text-xs"
          title="Brush color"
        >
          <span className="shrink-0 whitespace-nowrap font-medium text-[var(--foreground)]">Color</span>
          <input
            type="color"
            value={brushColor}
            onChange={(event) => onBrushColorChange(event.target.value)}
            disabled={disabled}
            className="h-7 w-9 cursor-pointer rounded-md border border-[var(--border)] bg-transparent p-0.5 disabled:opacity-45"
            aria-label="Brush color"
          />
          <span className="font-mono text-[0.6875rem] uppercase text-[var(--muted-foreground)]">{brushColor}</span>
          <button
            type="button"
            onClick={onToggleBrushColorPicker}
            disabled={disabled}
            className={[
              "inline-flex items-center gap-1.5 rounded-md px-2 py-1 font-medium ring-1 transition-colors disabled:opacity-45",
              pickingBrushColor
                ? "bg-[var(--primary)] text-[var(--primary-foreground)] ring-transparent"
                : "text-[var(--muted-foreground)] ring-[var(--border)] hover:text-[var(--foreground)]",
            ].join(" ")}
            aria-pressed={pickingBrushColor}
            title="Pick brush color from the sprite"
          >
            <Pipette size="0.75rem" />
            Pick
          </button>
        </div>
      )}

      {usesOpacityHardnessControls(tool) && (
        <div className="grid min-w-0 flex-[1_1_100%] grid-cols-1 gap-2 sm:min-w-[24rem] sm:flex-[2_1_24rem] sm:grid-cols-2">
          <RangeControl
            label="Opacity"
            min={0}
            max={100}
            value={brushOpacity}
            onChange={onBrushOpacityChange}
            disabled={disabled}
            title={brushOpacityTitle(activeBrushMode)}
            className="w-full min-w-0"
          />
          <RangeControl
            label="Hardness"
            min={0}
            max={100}
            value={brushHardness}
            onChange={onBrushHardnessChange}
            disabled={disabled}
            title={brushHardnessTitle(activeBrushMode)}
            className="w-full min-w-0"
          />
        </div>
      )}

      {tool === "blur" && (
        <RangeControl
          label="Strength"
          min={0}
          max={100}
          value={blurStrength}
          onChange={onBlurStrengthChange}
          disabled={disabled}
          title="How strongly the blur brush smooths alpha edges"
          className="min-w-48 flex-1"
        />
      )}
    </>
  );
}
