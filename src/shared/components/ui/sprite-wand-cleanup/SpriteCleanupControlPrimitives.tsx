import type { ReactNode } from "react";

interface RangeControlProps {
  label: string;
  value: number;
  min: number;
  max: number;
  disabled: boolean;
  onChange: (value: number) => void;
  step?: number;
  title?: string;
  className?: string;
  inputClassName?: string;
  before?: ReactNode;
  after?: ReactNode;
}

interface ToggleControlProps {
  label: string;
  checked: boolean;
  disabled: boolean;
  onChange: (checked: boolean) => void;
  title?: string;
}

export function RangeControl({
  label,
  value,
  min,
  max,
  disabled,
  onChange,
  step = 1,
  title,
  className = "min-w-[12rem] flex-[1_1_12rem]",
  inputClassName = "min-w-0",
  before,
  after,
}: RangeControlProps) {
  return (
    <label
      className={`flex min-w-0 items-center gap-2 rounded-lg bg-[var(--secondary)] px-3 py-2 text-xs ${className}`}
      title={title}
    >
      <span className="shrink-0 whitespace-nowrap font-medium text-[var(--foreground)]">{label}</span>
      {before}
      <input
        type="range"
        min={min}
        max={max}
        step={step}
        value={value}
        onChange={(event) => onChange(Number(event.target.value))}
        disabled={disabled}
        className={`${inputClassName} min-w-0 flex-1 accent-[var(--primary)] disabled:opacity-50`}
      />
      {after}
      <span className="w-8 shrink-0 text-right tabular-nums text-[var(--muted-foreground)]">{value}</span>
    </label>
  );
}

export function ToggleControl({ label, checked, disabled, onChange, title }: ToggleControlProps) {
  return (
    <label
      className="flex min-w-fit items-center gap-2 whitespace-nowrap rounded-lg bg-(--secondary) px-3 py-2 text-xs font-medium text-(--foreground)"
      title={title}
    >
      <input
        type="checkbox"
        checked={checked}
        onChange={(event) => onChange(event.target.checked)}
        disabled={disabled}
        className="h-4 w-4 accent-(--primary) disabled:opacity-50"
      />
      {label}
    </label>
  );
}
