import type { EventGateway, LlmGateway, StorageGateway } from "../capabilities";
import type { GenerationEvent } from "./generation-events";

export interface StartGenerationInput {
  chatId: string;
  connectionId?: string | null;
  message?: string;
  messages?: Array<{ role: "system" | "user" | "assistant"; content: string }>;
  parameters?: Record<string, unknown>;
}

export interface GenerationEngineDeps {
  storage: StorageGateway;
  llm: LlmGateway;
  events?: EventGateway;
}

export async function* startGeneration(
  deps: GenerationEngineDeps,
  input: StartGenerationInput,
): AsyncGenerator<GenerationEvent> {
  yield { type: "phase", data: "Preparing prompt..." };

  if (input.message?.trim()) {
    await deps.storage.call(`/chats/${input.chatId}/messages`, {
      role: "user",
      content: input.message.trim(),
    });
  }

  const promptMessages = input.messages?.length
    ? input.messages
    : ((await deps.storage.call(`/chats/${input.chatId}/messages`, null)) as Array<{
        role: "system" | "user" | "assistant";
        content: string;
      }>);

  yield { type: "phase", data: "Calling model..." };

  let content = "";
  for await (const chunk of deps.llm.stream({
    connectionId: input.connectionId,
    messages: promptMessages,
    parameters: input.parameters,
  })) {
    if (chunk.type === "token" && chunk.text) {
      content += chunk.text;
      yield { type: "token", data: chunk.text };
    }
  }

  if (content) {
    const message = await deps.storage.call(`/chats/${input.chatId}/messages`, {
      role: "assistant",
      content,
    });
    yield { type: "assistant_message", data: message };
  }

  yield { type: "done" };
}
