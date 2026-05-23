import { previewBackgroundOptions, type PreviewBackground } from "./sprite-cleanup-model";

interface PreviewBackgroundPickerProps {
  previewBackground: PreviewBackground;
  onPreviewBackgroundChange: (value: PreviewBackground) => void;
}

export function PreviewBackgroundPicker({
  previewBackground,
  onPreviewBackgroundChange,
}: PreviewBackgroundPickerProps) {
  return (
    <div className="flex flex-wrap items-center gap-1 rounded-lg bg-[var(--secondary)] p-1">
      {previewBackgroundOptions.map((option) => (
        <button
          key={option.key}
          type="button"
          onClick={() => onPreviewBackgroundChange(option.key)}
          className={[
            "rounded-md px-2 py-1 text-[0.6875rem] font-medium transition-colors",
            previewBackground === option.key
              ? "bg-[var(--primary)] text-[var(--primary-foreground)]"
              : "text-[var(--muted-foreground)] hover:bg-[var(--accent)] hover:text-[var(--foreground)]",
          ].join(" ")}
          aria-pressed={previewBackground === option.key}
        >
          {option.label}
        </button>
      ))}
    </div>
  );
}
