import type { LlmChunk, LlmGateway, LlmRequest } from "../../engine/capabilities/llm";
import { Channel } from "@tauri-apps/api/core";
import { invokeTauri, platform } from "./tauri-client";
import { useAuthStore } from "../stores/auth.store";

function createStreamId(): string {
  if (typeof crypto !== "undefined" && "randomUUID" in crypto) return crypto.randomUUID();
  return `llm-stream-${Date.now()}-${Math.random().toString(36).slice(2)}`;
}

function normalizeChunk(event: LlmChunk): LlmChunk {
  const text =
    typeof event.text === "string"
      ? event.text
      : typeof event.data === "string"
        ? event.data
        : undefined;
  return text === undefined ? event : { ...event, text };
}

/// Phase 4b web target: POST the LLM request body and parse the SSE response
/// stream by hand. EventSource isn't usable because it only does GET — the
/// request envelope (messages + tools + parameters) is too big for a query
/// string. The server-side route at `/api/stream/llm` emits the same event
/// payloads (`{type:"start"|"token"|"tool_call"|"done"|"error", ...}`) the
/// Tauri Channel sees, just framed as `data: <json>\n\n` blocks.
async function* streamViaSse(
  request: LlmRequest,
  streamId: string,
  signal?: AbortSignal,
): AsyncGenerator<LlmChunk> {
  const controller = new AbortController();
  const forwardAbort = () => {
    controller.abort();
    void invokeTauri("llm_stream_cancel", { streamId }).catch(() => undefined);
  };
  if (signal?.aborted) forwardAbort();
  signal?.addEventListener("abort", forwardAbort, { once: true });

  // The SSE path doesn't go through invokeViaFetch, so we have to do its
  // CSRF echo + 401 interception by hand. Without these, enabling auth in
  // Phase 6c would silently 401 every streaming reply with no UI feedback.
  const headers: Record<string, string> = { "Content-Type": "application/json" };
  const csrf = useAuthStore.getState().csrfToken;
  if (csrf) {
    headers["X-CSRF-Token"] = csrf;
  }

  let response: Response;
  try {
    response = await fetch("/api/stream/llm", {
      method: "POST",
      headers,
      credentials: "same-origin",
      body: JSON.stringify({ streamId, request }),
      signal: controller.signal,
    });
  } finally {
    signal?.removeEventListener("abort", forwardAbort);
  }

  if (response.status === 401) {
    useAuthStore.getState().setAnonymous();
  }

  if (!response.ok || !response.body) {
    const text = await response.text().catch(() => "");
    throw new Error(text || `LLM stream failed with HTTP ${response.status}`);
  }

  const reader = response.body.getReader();
  const decoder = new TextDecoder();
  let buffer = "";
  try {
    while (true) {
      const { value, done } = await reader.read();
      if (done) break;
      buffer += decoder.decode(value, { stream: true });

      // SSE frames are separated by blank lines. A frame is a sequence of
      // `field: value` lines; we only care about `data:`. Multi-line `data:`
      // values are joined by newlines per the spec, but the server emits
      // single-line JSON payloads so there's almost always one data line.
      let separatorIdx = buffer.indexOf("\n\n");
      while (separatorIdx >= 0) {
        const frame = buffer.slice(0, separatorIdx);
        buffer = buffer.slice(separatorIdx + 2);
        const dataLines = frame
          .split("\n")
          .filter((line) => line.startsWith("data:"))
          .map((line) => line.slice(5).replace(/^\s/, ""));
        if (dataLines.length > 0) {
          const payload = dataLines.join("\n");
          try {
            const event = JSON.parse(payload) as LlmChunk;
            const normalized = normalizeChunk(event);
            if (normalized.type === "error") {
              throw new Error(
                String(normalized.text ?? normalized.data ?? "LLM stream failed"),
              );
            }
            yield normalized;
            if (normalized.type === "done") return;
          } catch (parseError) {
            if (parseError instanceof Error && parseError.message.startsWith("LLM stream"))
              throw parseError;
            // Skip a malformed frame rather than aborting the whole stream.
          }
        }
        separatorIdx = buffer.indexOf("\n\n");
      }
    }
  } finally {
    try {
      reader.releaseLock();
    } catch {
      // Ignored; only matters when the stream was already cancelled.
    }
  }
}

async function* streamViaChannel(
  request: LlmRequest,
  streamId: string,
  signal?: AbortSignal,
): AsyncGenerator<LlmChunk> {
  const queue: LlmChunk[] = [];
  let completed = false;
  let failure: unknown = null;
  let wake: (() => void) | null = null;

  const notify = () => {
    wake?.();
    wake = null;
  };
  const abort = () => {
    failure = new DOMException("The operation was aborted.", "AbortError");
    void invokeTauri("llm_stream_cancel", { streamId }).catch(() => undefined);
    notify();
  };

  if (signal?.aborted) abort();
  signal?.addEventListener("abort", abort, { once: true });

  const onEvent = new Channel<LlmChunk>((event) => {
    const normalized = normalizeChunk(event);
    if (normalized.type === "done" || normalized.type === "error") completed = true;
    queue.push(normalized);
    notify();
  });

  const command = invokeTauri<void>("llm_stream_channel", {
    streamId,
    request,
    onEvent,
  }).catch((error) => {
    failure = error;
    completed = true;
    notify();
  });

  try {
    while (!completed || queue.length > 0) {
      if (failure) throw failure;
      if (queue.length === 0) {
        await new Promise<void>((resolve) => {
          wake = resolve;
        });
        continue;
      }
      const event = queue.shift()!;
      if (event.type === "error") throw new Error(String(event.text ?? event.data ?? "LLM stream failed"));
      yield event;
    }
    await command;
    if (failure) throw failure;
  } finally {
    signal?.removeEventListener("abort", abort);
  }
}

export const llmApi: LlmGateway = {
  complete: (request: LlmRequest) =>
    invokeTauri("llm_complete", {
      request,
    }),
  stream: function (request: LlmRequest, signal?: AbortSignal): AsyncGenerator<LlmChunk> {
    const streamId = createStreamId();
    return platform.isWeb
      ? streamViaSse(request, streamId, signal)
      : streamViaChannel(request, streamId, signal);
  },
  listModels: (connectionId?: string | null) =>
    invokeTauri("llm_list_models", {
      connectionId: connectionId ?? null,
    }),
};
