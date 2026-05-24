import { useCallback, useEffect, useMemo, useState } from "react";
import type {
  CustomTrackerField,
  GameState,
  InventoryItem,
  PresentCharacter,
  QuestProgress,
} from "../../../../engine/contracts/types/game-state";
import type { TrackerStateController, TrackerStateSnapshot } from "../types";
import { worldStateApi } from "../api/world-state-api";
import { useGameStateStore } from "../stores/world-state.store";
import { useGameStatePatcher } from "./use-world-state-patcher";

function getTrackerStateSnapshot(gameState: GameState | null): TrackerStateSnapshot {
  const playerStats = gameState?.playerStats ?? null;
  return {
    gameState,
    playerStats,
    personaStats: gameState?.personaStats ?? [],
    presentCharacters: gameState?.presentCharacters ?? ([] as PresentCharacter[]),
    inventory: playerStats?.inventory ?? ([] as InventoryItem[]),
    quests: playerStats?.activeQuests ?? ([] as QuestProgress[]),
    customTrackerFields: playerStats?.customTrackerFields ?? ([] as CustomTrackerField[]),
  };
}

export function useTrackerStateController(chatId: string | null, registrationId?: string): TrackerStateController {
  const gameState = useGameStateStore((s) => (chatId && s.current?.chatId === chatId ? s.current : null));
  const gameStateRefreshing = useGameStateStore((s) => s.isRefreshing);
  const setGameState = useGameStateStore((s) => s.setGameState);
  const { patchField, patchPlayerStats, flushPatch } = useGameStatePatcher(chatId, registrationId);
  const [loadingGameState, setLoadingGameState] = useState(false);
  const getSnapshot = useCallback(() => {
    const current = useGameStateStore.getState().current;
    return getTrackerStateSnapshot(chatId && current?.chatId === chatId ? current : null);
  }, [chatId]);

  useEffect(() => {
    if (!chatId) {
      setLoadingGameState(false);
      return;
    }

    const existing = useGameStateStore.getState().current;
    if (existing?.chatId === chatId) {
      setLoadingGameState(false);
      return;
    }

    let cancelled = false;
    setLoadingGameState(true);
    worldStateApi
      .get(chatId)
      .then((state) => {
        if (!cancelled) setGameState(state ?? null);
      })
      .catch(() => {
        if (!cancelled) setGameState(null);
      })
      .finally(() => {
        if (!cancelled) setLoadingGameState(false);
      });

    return () => {
      cancelled = true;
    };
  }, [chatId, setGameState]);

  const snapshot = useMemo(
    () => ({
      ...getTrackerStateSnapshot(gameState as GameState | null),
      loadingGameState,
      gameStateRefreshing,
      isLoadingGameState: loadingGameState || gameStateRefreshing,
    }),
    [gameState, gameStateRefreshing, loadingGameState],
  );

  return {
    ...snapshot,
    getSnapshot,
    patchField,
    patchPlayerStats,
    flushPatch,
  };
}
