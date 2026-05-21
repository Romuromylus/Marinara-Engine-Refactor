import type { CSSProperties } from "react";
import {
  DEFAULT_TRACKER_CARD_ACCENT,
  getTrackerCardCssVariableStyle,
  getTrackerCardStyleVars,
  type TrackerCardCssVariableStyle,
} from "../../../shared/lib/tracker-card-colors";
import type { TrackerProfilePalette } from "./tracker-profile-colors";

type TrackerProfileStyle = CSSProperties & TrackerCardCssVariableStyle;

export function withTrackerProfileStyle(palette: TrackerProfilePalette, background?: string): CSSProperties {
  const vars = getTrackerCardStyleVars({ palette, background });
  const style = getTrackerCardCssVariableStyle({ vars, prefix: "--tracker-profile" }) as TrackerProfileStyle;
  style["--tracker-profile-dialogue"] = vars.accent;
  style["--tracker-inline-foreground"] = "var(--tracker-profile-text)";
  style["--tracker-inline-muted"] = "var(--tracker-profile-muted-text)";
  style["--tracker-inline-number"] = "var(--tracker-profile-number-text)";
  style["--tracker-inline-rule"] = "var(--tracker-profile-row-rule)";

  if (palette.accent !== DEFAULT_TRACKER_CARD_ACCENT) {
    style["--primary"] = vars.accent;
  }

  return style;
}
