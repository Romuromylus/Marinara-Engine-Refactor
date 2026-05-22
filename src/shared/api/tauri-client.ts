import { ApiError } from "./api-errors";

type InvokeArgs = Record<string, unknown> | undefined;

const VITE_TARGET = import.meta.env.VITE_TARGET ?? "tauri";
const IS_WEB_TARGET = VITE_TARGET === "web";

function normalize(error: unknown): ApiError {
  if (error instanceof ApiError) return error;
  if (error && typeof error === "object") {
    const record = error as Record<string, unknown>;
    const code = typeof record.code === "string" ? record.code : "";
    const status = code === "not_found" ? 404 : code === "invalid_input" ? 400 : 500;
    const message =
      typeof record.message === "string" ? record.message : "Command failed";
    return new ApiError(message, status, record);
  }
  return new ApiError(String(error ?? "Command failed"), 500, error);
}

async function invokeViaTauri<T>(command: string, args: InvokeArgs): Promise<T> {
  const { invoke } = await import("@tauri-apps/api/core");
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    throw normalize(error);
  }
}

async function invokeViaFetch<T>(command: string, args: InvokeArgs): Promise<T> {
  let response: Response;
  try {
    response = await fetch(`/api/invoke/${encodeURIComponent(command)}`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      credentials: "same-origin",
      body: JSON.stringify(args ?? {}),
    });
  } catch (error) {
    throw new ApiError(
      error instanceof Error ? error.message : "Network error",
      0,
      error,
    );
  }

  if (response.status === 204) {
    return undefined as T;
  }

  const text = await response.text();
  let payload: unknown = null;
  if (text.length > 0) {
    try {
      payload = JSON.parse(text);
    } catch {
      payload = text;
    }
  }

  if (!response.ok) {
    if (payload && typeof payload === "object") {
      throw normalize(payload);
    }
    throw new ApiError(
      typeof payload === "string" && payload.length > 0
        ? payload
        : `HTTP ${response.status}`,
      response.status,
      payload,
    );
  }

  return payload as T;
}

export async function invokeTauri<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  if (IS_WEB_TARGET) {
    return invokeViaFetch<T>(command, args);
  }
  return invokeViaTauri<T>(command, args);
}

export const platform = {
  target: VITE_TARGET,
  isWeb: IS_WEB_TARGET,
  isDesktop: !IS_WEB_TARGET,
};
