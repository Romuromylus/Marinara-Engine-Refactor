import type {
  CharacterStat,
  CustomTrackerField,
  GameState,
  InventoryItem,
  PlayerStats,
  PresentCharacter,
  QuestProgress,
} from "../../engine/contracts/types/game-state";
import type { TemperatureUnit } from "../../shared/lib/temperature-units";

export type GameStatePatchField =
  | "date"
  | "time"
  | "location"
  | "weather"
  | "temperature"
  | "presentCharacters"
  | "playerStats"
  | "personaStats";

export type WorldStatePatchField = Extract<
  GameStatePatchField,
  "date" | "time" | "location" | "weather" | "temperature"
>;

export type WorldTemperatureUnit = TemperatureUnit;

export interface GameStatePatchValue {
  date: GameState["date"];
  time: GameState["time"];
  location: GameState["location"];
  weather: GameState["weather"];
  temperature: GameState["temperature"];
  presentCharacters: GameState["presentCharacters"];
  playerStats: GameState["playerStats"];
  personaStats: GameState["personaStats"];
}

export interface TrackerStateSnapshot {
  gameState: GameState | null;
  playerStats: PlayerStats | null;
  personaStats: CharacterStat[];
  presentCharacters: PresentCharacter[];
  inventory: InventoryItem[];
  quests: QuestProgress[];
  customTrackerFields: CustomTrackerField[];
}

export interface TrackerStateController extends TrackerStateSnapshot {
  loadingGameState: boolean;
  gameStateRefreshing: boolean;
  isLoadingGameState: boolean;
  getSnapshot: () => TrackerStateSnapshot;
  patchField: <K extends GameStatePatchField>(field: K, value: GameStatePatchValue[K]) => void;
  patchPlayerStats: <K extends keyof PlayerStats>(field: K, value: PlayerStats[K]) => void;
  flushPatch: () => Promise<void>;
}
