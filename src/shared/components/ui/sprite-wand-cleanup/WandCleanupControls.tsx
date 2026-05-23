import { RotateCcw } from "lucide-react";
import { RangeControl, ToggleControl } from "./SpriteCleanupControlPrimitives";

interface WandCleanupControlsProps {
  loading: boolean;
  applying: boolean;
  tolerance: number;
  wandStrong: boolean;
  wandSoftness: number;
  wandFeather: number;
  onResetWandDefaults: () => void;
  onToleranceChange: (value: number) => void;
  onWandStrongChange: (value: boolean) => void;
  onWandSoftnessChange: (value: number) => void;
  onWandFeatherChange: (value: number) => void;
}

export function WandCleanupControls({
  loading,
  applying,
  tolerance,
  wandStrong,
  wandSoftness,
  wandFeather,
  onResetWandDefaults,
  onToleranceChange,
  onWandStrongChange,
  onWandSoftnessChange,
  onWandFeatherChange,
}: WandCleanupControlsProps) {
  const disabled = loading || applying;

  return (
    <>
      <button
        type="button"
        onClick={onResetWandDefaults}
        disabled={disabled}
        className="inline-flex min-w-fit items-center gap-1.5 whitespace-nowrap rounded-lg bg-[var(--secondary)] px-3 py-2 text-xs font-medium text-[var(--muted-foreground)] ring-1 ring-[var(--border)] transition-colors hover:text-[var(--foreground)] disabled:opacity-45"
        title="Reset wand controls to their defaults"
      >
        <RotateCcw size="0.875rem" />
        Defaults
      </button>
      <RangeControl
        label="Tolerance"
        min={4}
        max={128}
        value={tolerance}
        onChange={onToleranceChange}
        disabled={disabled}
        className="min-w-[12rem] flex-[1_0_12rem]"
      />
      <ToggleControl
        label="Strong"
        checked={wandStrong}
        onChange={onWandStrongChange}
        disabled={disabled}
        title="Reach farther into matching debris"
      />
      <RangeControl
        label="Softness"
        min={0}
        max={100}
        value={wandSoftness}
        onChange={onWandSoftnessChange}
        disabled={disabled}
        title="0 is a hard cut; higher values leave a softer low-alpha edge"
        className="min-w-[14rem] flex-[1_0_14rem]"
      />
      <RangeControl
        label="Feather"
        min={0}
        max={100}
        value={wandFeather}
        onChange={onWandFeatherChange}
        disabled={disabled}
        title="How much soft border the wand leaves behind, and how gradually it fades"
        className="min-w-[14rem] flex-[1_0_14rem]"
      />
    </>
  );
}
