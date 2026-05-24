import { useEffect } from "react";
import { useGameStateStore } from "../../features/runtime/world-state/index";
import { useAgentStore } from "../../shared/stores/agent.store";
import { useChatStore } from "../../shared/stores/chat.store";

export function useChatSwitchEffects() {
  useEffect(
    () =>
      useChatStore.subscribe(
        (state) => state.activeChatId,
        (activeChatId, previousChatId) => {
          if (activeChatId === previousChatId) return;
          useAgentStore.getState().reset();
          useGameStateStore.getState().setGameState(null);
        },
      ),
    [],
  );
}
