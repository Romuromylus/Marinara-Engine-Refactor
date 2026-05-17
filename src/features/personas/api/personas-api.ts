function deferred(action: string): Promise<never> {
  return Promise.reject(new Error(`${action} is waiting for the Rust characters/personas backend slice.`));
}

export const api = {
  get: <T = unknown>(_path: string, _init?: RequestInit): Promise<T> => deferred("persona read") as Promise<T>,
  post: <T = unknown>(_path: string, _body?: unknown, _init?: RequestInit): Promise<T> =>
    deferred("persona mutation") as Promise<T>,
  patch: <T = unknown>(_path: string, _body?: unknown, _init?: RequestInit): Promise<T> =>
    deferred("persona update") as Promise<T>,
  delete: <T = unknown>(_path: string, _init?: RequestInit): Promise<T> => deferred("persona delete") as Promise<T>,
  put: <T = unknown>(_path: string, _body?: unknown, _init?: RequestInit): Promise<T> =>
    deferred("persona activation") as Promise<T>,
  downloadPost: (_path: string, _body?: unknown, _filename?: string): Promise<never> => deferred("persona export"),
};
