import { useCallback, useRef, useState } from "react";
import type { ChangeEvent } from "react";
import type { InventoryItem, PresentCharacter, QuestProgress } from "../../../engine/contracts/types/game-state";
import type { TrackerStateController } from "../../world-state/types";
import { useTrackerCharacterAvatarActions } from "../../world-state/hooks/use-tracker-character-avatar-actions";
import {
  appendTrackerListItem,
  createManualCharacterStat,
  createManualInventoryItem,
  createManualPresentCharacter,
  createManualQuest,
  removeTrackerListItem,
  replaceTrackerListItem,
} from "../../world-state/lib/tracker-state-edits";
import { getCharacterFeatureKey } from "../components/tracker-character.helpers";

export function useTrackerMutations({
  activeChatId,
  agentConfigLookupEnabled,
  inventory,
  personaStats,
  presentCharacters,
  quests,
  patchField,
  patchPlayerStats,
  removeFeaturedCharacterCard,
}: {
  activeChatId: string | null;
  agentConfigLookupEnabled: boolean;
  inventory: InventoryItem[];
  personaStats: TrackerStateController["personaStats"];
  presentCharacters: PresentCharacter[];
  quests: QuestProgress[];
  patchField: TrackerStateController["patchField"];
  patchPlayerStats: TrackerStateController["patchPlayerStats"];
  removeFeaturedCharacterCard: (key: string) => void;
}) {
  const [avatarUploadIndex, setAvatarUploadIndex] = useState<number | null>(null);
  const avatarFileInputRef = useRef<HTMLInputElement>(null);
  const updatePresentCharacters = useCallback(
    (characters: PresentCharacter[]) => patchField("presentCharacters", characters),
    [patchField],
  );
  const {
    autoGenerateCharacterAvatars,
    canToggleAutoGenerateCharacterAvatars,
    isUpdatingAutoGenerateCharacterAvatars,
    toggleAutoGenerateCharacterAvatars,
    uploadCharacterAvatar,
  } = useTrackerCharacterAvatarActions({
    chatId: activeChatId,
    characters: presentCharacters,
    onUpdateCharacters: updatePresentCharacters,
    agentConfigLookupEnabled,
  });

  const openAvatarUpload = useCallback((index: number) => {
    setAvatarUploadIndex(index);
    avatarFileInputRef.current?.click();
  }, []);

  const handleAvatarFileInputChange = useCallback(
    (event: ChangeEvent<HTMLInputElement>) => {
      const file = event.target.files?.[0];
      const index = avatarUploadIndex;
      setAvatarUploadIndex(null);
      if (file && index !== null) {
        void uploadCharacterAvatar(index, file);
      }
      event.target.value = "";
    },
    [avatarUploadIndex, uploadCharacterAvatar],
  );

  const updateCharacter = useCallback(
    (index: number, character: PresentCharacter) => {
      updatePresentCharacters(replaceTrackerListItem(presentCharacters, index, character));
    },
    [presentCharacters, updatePresentCharacters],
  );

  const removeCharacter = useCallback(
    (index: number) => {
      const removed = presentCharacters[index];
      if (removed) removeFeaturedCharacterCard(getCharacterFeatureKey(removed, index));
      updatePresentCharacters(removeTrackerListItem(presentCharacters, index));
    },
    [presentCharacters, removeFeaturedCharacterCard, updatePresentCharacters],
  );

  const addCharacter = useCallback(() => {
    updatePresentCharacters(appendTrackerListItem(presentCharacters, createManualPresentCharacter()));
  }, [presentCharacters, updatePresentCharacters]);

  const updateInventory = useCallback(
    (items: InventoryItem[]) => patchPlayerStats("inventory", items),
    [patchPlayerStats],
  );

  const updateInventoryItem = useCallback(
    (index: number, item: InventoryItem) => {
      updateInventory(replaceTrackerListItem(inventory, index, item));
    },
    [inventory, updateInventory],
  );

  const removeInventoryItem = useCallback(
    (index: number) => {
      updateInventory(removeTrackerListItem(inventory, index));
    },
    [inventory, updateInventory],
  );

  const addInventoryItem = useCallback(() => {
    updateInventory(appendTrackerListItem(inventory, createManualInventoryItem()));
  }, [inventory, updateInventory]);

  const updateQuests = useCallback(
    (nextQuests: QuestProgress[]) => patchPlayerStats("activeQuests", nextQuests),
    [patchPlayerStats],
  );

  const updateQuest = useCallback(
    (index: number, quest: QuestProgress) => {
      updateQuests(replaceTrackerListItem(quests, index, quest));
    },
    [quests, updateQuests],
  );

  const removeQuest = useCallback(
    (index: number) => {
      updateQuests(removeTrackerListItem(quests, index));
    },
    [quests, updateQuests],
  );

  const addQuest = useCallback(() => {
    updateQuests(appendTrackerListItem(quests, createManualQuest()));
  }, [quests, updateQuests]);

  return {
    addCharacter,
    addInventoryItem,
    addPersonaStat: () => patchField("personaStats", appendTrackerListItem(personaStats, createManualCharacterStat())),
    addQuest,
    autoGenerateCharacterAvatars,
    avatarFileInputRef,
    canToggleAutoGenerateCharacterAvatars,
    handleAvatarFileInputChange,
    isUpdatingAutoGenerateCharacterAvatars,
    openAvatarUpload,
    removeCharacter,
    removeInventoryItem,
    removeQuest,
    toggleAutoGenerateCharacterAvatars,
    updateCharacter,
    updateCustomFields: (fields: TrackerStateController["customTrackerFields"]) =>
      patchPlayerStats("customTrackerFields", fields),
    updateInventoryItem,
    updatePersonaStats: (stats: TrackerStateController["personaStats"]) => patchField("personaStats", stats),
    updateQuest,
    savePersonaStatus: (status: string) => patchPlayerStats("status", status),
  };
}
