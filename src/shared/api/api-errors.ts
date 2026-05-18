export class ApiError extends Error {
  constructor(
    message: string,
    public status = 0,
    public details?: unknown,
  ) {
    super(message);
    this.name = "ApiError";
  }

  get payload() {
    return this.details;
  }
}

export interface JsonRepairRequest {
  id?: string;
  title?: string;
  endpoint?: string;
  rawJson?: string;
  applyEndpoint: string;
  applyBody?: Record<string, unknown>;
  payload?: unknown;
  error?: string;
  [key: string]: unknown;
}

export function isJsonRepairApiError(error: unknown): boolean {
  return error instanceof ApiError && !!getJsonRepairRequest(error);
}

export function getJsonRepairRequest(error: unknown): JsonRepairRequest | null {
  if (!(error instanceof ApiError)) return null;
  const details = error.details;
  if (!details || typeof details !== "object") return null;
  const request = (details as { jsonRepair?: unknown }).jsonRepair;
  if (!request || typeof request !== "object") return null;
  if (typeof (request as { applyEndpoint?: unknown }).applyEndpoint !== "string") return null;
  return request as JsonRepairRequest;
}
