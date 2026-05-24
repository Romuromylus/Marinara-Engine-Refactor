import type { TrackerCardColorConfig } from "../../../../engine/contracts/types/persona";
import {
  DEFAULT_TRACKER_CARD_ACCENT,
  getTrackerCardFinish,
  getTrackerCardPaintEnabled,
  getTrackerCardPaintOpacity,
  getTrackerCardPortraitStageBackground,
  getTrackerCardStylePalette,
  normalizeTrackerCardColorMode,
  type TrackerCardStylePalette,
} from "../../../../shared/lib/tracker-card-colors";

export interface TrackerProfileColors {
  dialogueColor?: string | null;
  nameColor?: string | null;
  boxColor?: string | null;
  trackerCardColors?: TrackerCardColorConfig | null;
}

export type TrackerProfilePalette = TrackerCardStylePalette;

export function getTrackerProfilePalette(
  profileColors: TrackerProfileColors | null | undefined,
  fallbackAccent = DEFAULT_TRACKER_CARD_ACCENT,
): TrackerProfilePalette {
  const trackerCardColors = profileColors?.trackerCardColors ?? null;
  const mode = normalizeTrackerCardColorMode(trackerCardColors?.mode);
  const finish = getTrackerCardFinish(trackerCardColors, mode);
  const enabled = getTrackerCardPaintEnabled(trackerCardColors);
  const opacity = getTrackerCardPaintOpacity(trackerCardColors);
  const effectiveColors = mode === "default" ? null : mode === "custom" ? trackerCardColors : profileColors;
  const effectiveFallback = mode === "chat" ? fallbackAccent : DEFAULT_TRACKER_CARD_ACCENT;

  return getTrackerCardStylePalette({
    colors: effectiveColors,
    enabled,
    finish,
    opacity,
    portraitStageBackground: getTrackerCardPortraitStageBackground(trackerCardColors),
    fallbackAccent: effectiveFallback,
  });
}
