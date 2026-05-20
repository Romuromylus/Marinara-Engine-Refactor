import { cn } from "../../../../shared/lib/utils";
import type { TrackerPanelSizeProfile } from "../../../../shared/stores/ui.store";

export function TrackerSizeTierIcon({
  profile,
  className,
}: {
  profile: TrackerPanelSizeProfile;
  className?: string;
}) {
  const activeBars = profile === "compact" ? 1 : profile === "standard" ? 2 : 3;

  return (
    <span
      aria-hidden="true"
      className={cn("grid h-4 w-6 grid-cols-3 items-end gap-0.5 text-current", className)}
    >
      {[1, 2, 3].map((bar) => (
        <span
          key={bar}
          className={cn(
            "block rounded-[1px] bg-current transition-opacity",
            bar === 1 ? "h-2" : bar === 2 ? "h-3" : "h-4",
            bar <= activeBars ? "opacity-95" : "opacity-22",
          )}
        />
      ))}
    </span>
  );
}
