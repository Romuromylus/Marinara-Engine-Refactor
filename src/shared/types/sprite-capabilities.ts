export type SpriteCleanupEngine = "auto" | "builtin";

export interface SpriteCapabilities {
  imageProcessingAvailable: boolean;
  spriteGenerationAvailable: boolean;
  backgroundRemovalAvailable: boolean;
  reason: string | null;
  cleanupEngine?: {
    engine: SpriteCleanupEngine;
    installed: boolean;
    command: string | null;
    source: "env" | "local" | "path" | null;
    runtimeDir: string;
    reason: string | null;
  };
}
