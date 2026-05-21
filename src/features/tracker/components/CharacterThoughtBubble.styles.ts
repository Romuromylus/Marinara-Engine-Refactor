import { cn } from "../../../shared/lib/utils";

export const TRACKER_PROFILE_THOUGHT_BUBBLE_SURFACE_CLASS = cn(
  "border-[color-mix(in_srgb,var(--tracker-profile-dialogue-border)_48%,transparent)]",
  "bg-[linear-gradient(145deg,color-mix(in_srgb,var(--background)_88%,var(--tracker-profile-surface-solid)_12%)_0%,color-mix(in_srgb,var(--background)_80%,var(--tracker-profile-surface-solid)_20%)_52%,color-mix(in_srgb,var(--background)_76%,var(--tracker-profile-display-solid)_24%)_100%)]",
  "text-[color:var(--tracker-profile-text)] shadow-[0_8px_18px_color-mix(in_srgb,var(--background)_36%,transparent),0_0_16px_color-mix(in_srgb,var(--tracker-profile-accent-solid)_12%,transparent),inset_0_1px_0_color-mix(in_srgb,var(--foreground)_10%,transparent),inset_0_0_0_1px_color-mix(in_srgb,var(--foreground)_5%,transparent)] backdrop-blur-xl",
);

export const TRACKER_PROFILE_THOUGHT_BUBBLE_OVERLAY_CLASS =
  "pointer-events-none absolute inset-0 rounded-[inherit] bg-[radial-gradient(circle_at_26%_12%,color-mix(in_srgb,var(--foreground)_9%,transparent),transparent_34%),radial-gradient(circle_at_88%_92%,color-mix(in_srgb,var(--tracker-profile-accent-solid)_13%,transparent),transparent_48%),linear-gradient(180deg,color-mix(in_srgb,var(--foreground)_3%,transparent)_0%,transparent_48%,color-mix(in_srgb,var(--background)_24%,transparent)_100%)]";

export const TRACKER_PROFILE_THOUGHT_BUBBLE_EDIT_CLASS =
  "[--foreground:color-mix(in_srgb,var(--tracker-profile-text)_96%,var(--foreground)_4%)] [--muted-foreground:color-mix(in_srgb,var(--tracker-profile-muted-text)_76%,var(--tracker-profile-text)_24%)] hover:bg-[color-mix(in_srgb,var(--tracker-profile-accent-solid)_10%,transparent)]";

export const TRACKER_PROFILE_THOUGHT_BUBBLE_TEXT_CLASS =
  "text-[color-mix(in_srgb,var(--tracker-profile-text)_96%,var(--foreground)_4%)]";
