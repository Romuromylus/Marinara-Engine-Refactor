function deferred(action: string): Promise<never> {
  return Promise.reject(new Error(`${action} is waiting for the Rust characters/personas backend slice.`));
}

export const api = {
  get: <T = unknown>(_path: string, _init?: RequestInit): Promise<T> => deferred("character read") as Promise<T>,
  post: <T = unknown>(path: string, _body?: unknown, _init?: RequestInit): Promise<T> =>
    deferred(path.startsWith("/chats/") ? "chat message mutation from character UI" : "character mutation") as Promise<T>,
  patch: <T = unknown>(_path: string, _body?: unknown, _init?: RequestInit): Promise<T> =>
    deferred("character update") as Promise<T>,
  delete: <T = unknown>(_path: string, _init?: RequestInit): Promise<T> => deferred("character delete") as Promise<T>,
  downloadPost: (_path: string, _body?: unknown, _filename?: string): Promise<never> => deferred("character export"),
};
