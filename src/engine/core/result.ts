export type EngineResult<T, E = EngineError> = { ok: true; value: T } | { ok: false; error: E };

export interface EngineError {
  code: string;
  message: string;
  details?: unknown;
}

export function ok<T>(value: T): EngineResult<T> {
  return { ok: true, value };
}

export function err(code: string, message: string, details?: unknown): EngineResult<never> {
  return { ok: false, error: { code, message, details } };
}

export function toEngineError(error: unknown, code = "engine_error"): EngineError {
  if (error && typeof error === "object" && "message" in error) {
    return { code, message: String((error as { message?: unknown }).message), details: error };
  }
  return { code, message: String(error ?? "Unknown engine error") };
}
