import type {
  CharacterStat,
  CustomTrackerField,
  GameState,
  InventoryItem,
  PlayerStats,
  PresentCharacter,
  QuestProgress,
} from "../../engine/contracts/types/game-state";

export type GameStatePatchField =
  | "date"
  | "time"
  | "location"
  | "weather"
  | "temperature"
  | "presentCharacters"
  | "playerStats"
  | "personaStats";

export type WorldTemperatureUnit = "celsius" | "fahrenheit";

export interface TrackerStateController {
  gameState: GameState | null;
  playerStats: PlayerStats | null;
  personaStats: CharacterStat[];
  presentCharacters: PresentCharacter[];
  inventory: InventoryItem[];
  quests: QuestProgress[];
  customTrackerFields: CustomTrackerField[];
  loadingGameState: boolean;
  gameStateRefreshing: boolean;
  isLoadingGameState: boolean;
  patchField: (field: GameStatePatchField, value: unknown) => void;
  patchPlayerStats: (field: keyof PlayerStats, value: unknown) => void;
  flushPatch: () => Promise<void>;
}
