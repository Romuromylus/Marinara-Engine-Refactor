import { Check } from "lucide-react";
import {
  getTrackerPanelWidthForProfile,
  useUIStore,
  type TrackerPanelSizeProfile,
  type TrackerTemperatureUnit,
  type TrackerThoughtBubbleDisplay,
} from "../../../../shared/stores/ui.store";
import { HelpTooltip } from "../../../../shared/components/ui/HelpTooltip";
import { TrackerSizeTierIcon } from "./TrackerSizeTierIcon";
import { cn } from "../../../../shared/lib/utils";
import { ToggleSetting } from "./SettingControls";

const TRACKER_PANEL_SIZE_OPTIONS: Array<{ id: TrackerPanelSizeProfile; label: string; desc: string }> = [
  { id: "compact", label: "Compact", desc: "Lean tracker width for tighter screens." },
  { id: "standard", label: "Standard", desc: "Balanced tracker width for regular play." },
  { id: "expanded", label: "Expanded", desc: "Wider tracker width for denser cards." },
];

const TRACKER_THOUGHT_DISPLAY_OPTIONS: Array<{ id: TrackerThoughtBubbleDisplay; label: string; desc: string }> = [
  { id: "inline", label: "Docked", desc: "Show character thoughts inside the tracker card." },
  { id: "floating", label: "Floating", desc: "Open thoughts as a popover beside the portrait." },
];

const TRACKER_TEMPERATURE_UNIT_OPTIONS: Array<{ id: TrackerTemperatureUnit; label: string }> = [
  { id: "celsius", label: "C" },
  { id: "fahrenheit", label: "F" },
];

export function TrackerPanelDisplaySettings() {
  const trackerPanelSizeProfile = useUIStore((s) => s.trackerPanelSizeProfile);
  const setTrackerPanelSizeProfile = useUIStore((s) => s.setTrackerPanelSizeProfile);
  const trackerPanelThoughtBubbleDisplay = useUIStore((s) => s.trackerPanelThoughtBubbleDisplay);
  const setTrackerPanelThoughtBubbleDisplay = useUIStore((s) => s.setTrackerPanelThoughtBubbleDisplay);
  const trackerPanelDockedThoughtsAlwaysVisible = useUIStore((s) => s.trackerPanelDockedThoughtsAlwaysVisible);
  const setTrackerPanelDockedThoughtsAlwaysVisible = useUIStore(
    (s) => s.setTrackerPanelDockedThoughtsAlwaysVisible,
  );
  const trackerTemperatureUnit = useUIStore((s) => s.trackerTemperatureUnit);
  const setTrackerTemperatureUnit = useUIStore((s) => s.setTrackerTemperatureUnit);

  return (
    <div className="mt-1.5 grid gap-2 rounded-lg bg-[var(--background)]/36 p-1.5 ring-1 ring-[var(--border)]">
      <div className="grid gap-1.5">
        <span className="inline-flex min-w-0 items-center gap-1 px-0.5 text-[0.625rem] font-medium text-[var(--foreground)]">
          Desktop size
          <HelpTooltip text="Sets the fixed desktop width for the tracker panel." />
        </span>
        <div className="grid grid-cols-3 gap-1">
          {TRACKER_PANEL_SIZE_OPTIONS.map((option) => {
            const selected = trackerPanelSizeProfile === option.id;
            return (
              <button
                key={option.id}
                type="button"
                aria-pressed={selected}
                title={`${option.desc} ${getTrackerPanelWidthForProfile(option.id)}px.`}
                onClick={() => setTrackerPanelSizeProfile(option.id)}
                className={cn(
                  "flex min-h-14 min-w-0 flex-col items-center justify-center gap-1 rounded-md px-1.5 py-1.5 text-center text-[0.625rem] transition-all",
                  selected
                    ? "bg-[var(--primary)]/12 text-[var(--primary)] ring-1 ring-[var(--primary)]/35"
                    : "bg-[var(--secondary)]/42 text-[var(--muted-foreground)] ring-1 ring-[var(--border)]/60 hover:bg-[var(--secondary)] hover:text-[var(--foreground)]",
                )}
              >
                <TrackerSizeTierIcon
                  profile={option.id}
                  className={selected ? "text-[var(--primary)]" : "text-[var(--muted-foreground)]"}
                />
                <span className="truncate font-semibold">{option.label}</span>
                <span className="font-mono text-[0.5625rem] tabular-nums opacity-70">
                  {getTrackerPanelWidthForProfile(option.id)}px
                </span>
              </button>
            );
          })}
        </div>
      </div>

      <div className="grid gap-1.5">
        <span className="inline-flex min-w-0 items-center gap-1 px-0.5 text-[0.625rem] font-medium text-[var(--foreground)]">
          Thoughts
          <HelpTooltip text="Controls where character thoughts appear when the tracker thought button is used." />
        </span>
        <div className="grid grid-cols-2 gap-1 rounded-md bg-[var(--secondary)]/35 p-0.5">
          {TRACKER_THOUGHT_DISPLAY_OPTIONS.map((option) => {
            const selected = trackerPanelThoughtBubbleDisplay === option.id;
            return (
              <button
                key={option.id}
                type="button"
                aria-pressed={selected}
                title={option.desc}
                onClick={() => setTrackerPanelThoughtBubbleDisplay(option.id)}
                className={cn(
                  "flex min-h-8 min-w-0 items-center justify-center gap-1 rounded-sm px-1.5 text-[0.625rem] font-semibold transition-colors",
                  selected
                    ? "bg-[var(--primary)]/12 text-[var(--primary)] ring-1 ring-[var(--primary)]/24"
                    : "text-[var(--muted-foreground)] hover:bg-[var(--accent)]/45 hover:text-[var(--foreground)]",
                )}
              >
                {selected && <Check size="0.625rem" />}
                <span className="truncate">{option.label}</span>
              </button>
            );
          })}
        </div>
        {trackerPanelThoughtBubbleDisplay === "inline" && (
          <ToggleSetting
            label="Keep docked thoughts visible"
            checked={trackerPanelDockedThoughtsAlwaysVisible}
            onChange={setTrackerPanelDockedThoughtsAlwaysVisible}
            help="When on, docked character thoughts stay visible in featured tracker cards instead of requiring the thought button."
          />
        )}
      </div>

      <div className="grid gap-1.5">
        <span className="inline-flex min-w-0 items-center gap-1 px-0.5 text-[0.625rem] font-medium text-[var(--foreground)]">
          Temperature
          <HelpTooltip text="Controls the unit used by the tracker panel's world temperature display." />
        </span>
        <div className="grid grid-cols-2 gap-1 rounded-md bg-[var(--secondary)]/35 p-0.5">
          {TRACKER_TEMPERATURE_UNIT_OPTIONS.map((option) => {
            const selected = trackerTemperatureUnit === option.id;
            return (
              <button
                key={option.id}
                type="button"
                aria-pressed={selected}
                onClick={() => setTrackerTemperatureUnit(option.id)}
                className={cn(
                  "min-h-8 rounded-sm px-2 text-xs font-semibold transition-colors",
                  selected
                    ? "bg-[var(--primary)]/12 text-[var(--primary)] ring-1 ring-[var(--primary)]/24"
                    : "text-[var(--muted-foreground)] hover:bg-[var(--accent)]/45 hover:text-[var(--foreground)]",
                )}
              >
                °{option.label}
              </button>
            );
          })}
        </div>
      </div>
    </div>
  );
}
