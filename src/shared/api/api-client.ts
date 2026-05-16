export const ADMIN_SECRET_STORAGE_KEY = "marinara-admin-secret";

export function getAdminSecretHeader(): Record<string, string> {
  const secret = localStorage.getItem(ADMIN_SECRET_STORAGE_KEY);
  return secret ? { "x-admin-secret": secret } : {};
}

async function unavailable(): Promise<never> {
  throw new Error("This server-backed settings action is deferred until the matching Tauri command slice.");
}

export const api = {
  get: <T = unknown>(_path: string, _init?: RequestInit): Promise<T> => unavailable(),
  post: <T = unknown>(_path: string, _body?: unknown, _init?: RequestInit): Promise<T> => unavailable(),
  patch: <T = unknown>(_path: string, _body?: unknown, _init?: RequestInit): Promise<T> => unavailable(),
  delete: <T = unknown>(_path: string, _init?: RequestInit): Promise<T> => unavailable(),
  upload: <T = unknown>(_path: string, _body: FormData, _init?: RequestInit): Promise<T> => unavailable(),
};
