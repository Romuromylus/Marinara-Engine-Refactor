// ──────────────────────────────────────────────
// Visual Novel Types
// ──────────────────────────────────────────────

/** A Visual Novel scene definition. */
export interface VNScene {
  /** Background image path */
  backgroundPath: string | null;
  /** Characters on screen with their positions */
  sprites: VNSprite[];
  /** Currently speaking character */
  speakerId: string | null;
  /** Display text in the text box */
  text: string;
  /** Scene transition effect */
  transition: VNTransition | null;
  /** Choices offered to the player (null = no choices) */
  choices: VNChoice[] | null;
}

/** A character sprite positioned on the VN stage. */
export interface VNSprite {
  characterId: string;
  /** Horizontal position: left, center-left, center, center-right, right */
  position: VNSpritePosition;
  /** Which expression sprite to show */
  expression: string;
  /** Opacity 0-1 (for fade effects) */
  opacity: number;
  /** Whether the sprite is flipped horizontally */
  flipped: boolean;
}

/** Allowed sprite positions on the VN stage. */
export type VNSpritePosition = "far-left" | "left" | "center-left" | "center" | "center-right" | "right" | "far-right";

/** Scene transition effects. */
export interface VNTransition {
  type: "fade" | "dissolve" | "slide_left" | "slide_right" | "wipe" | "none";
  durationMs: number;
}

/** A choice in a VN branch point. */
export interface VNChoice {
  id: string;
  text: string;
  /** Optional: condition for this choice to be available */
  condition: string | null;
}
