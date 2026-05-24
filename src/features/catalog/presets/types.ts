export type WrapFormat = "xml" | "markdown" | "plain";

export interface Preset {
  id: string;
  name: string;
  description: string;
  wrapFormat?: WrapFormat;
  isDefault?: boolean | string;
  author?: string;
  sectionOrder?: string[] | string;
  parameters?: Record<string, unknown> | string | null;
  createdAt?: string | null;
  updatedAt?: string | null;
}

export interface PresetSection {
  id: string;
  presetId: string;
  name: string;
  role: "system" | "user" | "assistant";
  content: string;
  enabled: boolean | string;
  isMarker?: boolean | string;
  markerConfig?: Record<string, unknown> | string | null;
  groupId?: string | null;
}

export interface PresetGroup {
  id: string;
  presetId: string;
  name: string;
}

export interface PresetFull {
  preset: Preset;
  sections: PresetSection[];
  groups: PresetGroup[];
  choiceBlocks?: unknown[];
}
