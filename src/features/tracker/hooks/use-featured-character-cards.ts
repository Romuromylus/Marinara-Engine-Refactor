import { useCallback, useEffect, useRef, useState } from "react";
import { useUpdateChatMetadata } from "../../chats/hooks/use-chats";
import { TRACKER_FEATURED_CHARACTER_META_KEY } from "../components/tracker-data-sidebar.constants";
import { normalizeStringArray } from "../components/tracker-metadata.helpers";

export function useFeaturedCharacterCards({
  activeChatId,
  chatMeta,
}: {
  activeChatId: string | null;
  chatMeta: Record<string, unknown>;
}) {
  const updateChatMetadata = useUpdateChatMetadata();
  const [featuredCharacterCards, setFeaturedCharacterCards] = useState<Set<string>>(() => new Set());
  const featuredCharacterCardsRef = useRef<Set<string>>(new Set());

  useEffect(() => {
    const next = new Set(normalizeStringArray(chatMeta[TRACKER_FEATURED_CHARACTER_META_KEY]));
    featuredCharacterCardsRef.current = next;
    setFeaturedCharacterCards(next);
  }, [activeChatId, chatMeta]);

  const persistFeaturedCharacterCards = useCallback(
    (next: Set<string>) => {
      featuredCharacterCardsRef.current = next;
      setFeaturedCharacterCards(next);
      if (!activeChatId) return;
      updateChatMetadata.mutate({
        id: activeChatId,
        [TRACKER_FEATURED_CHARACTER_META_KEY]: Array.from(next),
      });
    },
    [activeChatId, updateChatMetadata],
  );

  const removeFeaturedCharacterCard = useCallback(
    (key: string) => {
      if (!featuredCharacterCardsRef.current.has(key)) return;
      const next = new Set(featuredCharacterCardsRef.current);
      next.delete(key);
      persistFeaturedCharacterCards(next);
    },
    [persistFeaturedCharacterCards],
  );

  const toggleFeaturedCharacterCard = useCallback(
    (key: string) => {
      const next = new Set(featuredCharacterCardsRef.current);
      if (next.has(key)) {
        next.delete(key);
      } else {
        next.add(key);
      }
      persistFeaturedCharacterCards(next);
    },
    [persistFeaturedCharacterCards],
  );

  return {
    featuredCharacterCards,
    removeFeaturedCharacterCard,
    toggleFeaturedCharacterCard,
  };
}
