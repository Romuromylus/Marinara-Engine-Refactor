import { useMutation } from "@tanstack/react-query";

type ChatMode = "roleplay" | "conversation";

interface StartChatFromCharacterOptions {
  characterId: string;
  characterName: string;
  mode: ChatMode;
  firstMessage?: string;
  alternateGreetings?: string[];
}

async function deferredStartChat(_options: StartChatFromCharacterOptions): Promise<never> {
  throw new Error("Starting chats from characters is waiting for the Rust chats backend slice.");
}

export function useStartChatFromCharacter() {
  const mutation = useMutation({
    mutationFn: deferredStartChat,
  });

  return {
    startChatFromCharacter: (options: StartChatFromCharacterOptions) => mutation.mutate(options),
    isStartingChat: mutation.isPending,
  };
}

