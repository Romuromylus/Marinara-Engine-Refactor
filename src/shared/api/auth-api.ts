// Web-target auth client. The endpoints live at /api/auth/<verb>, not behind
// the /api/invoke/<command> dispatcher, so this module talks fetch directly
// instead of going through tauri-client.ts. The Tauri build never calls into
// here — `bootstrapAuth` short-circuits via `platform.isWeb`.
import { ApiError } from "./api-errors";
import { useAuthStore, type AuthUser } from "../stores/auth.store";
import { platform } from "./tauri-client";

type MeResponse = {
  enforced: boolean;
  user: AuthUser | null;
  csrfToken: string | null;
};

type LoginResponse = {
  user: AuthUser;
  csrfToken: string;
};

async function fetchAuth<T>(path: string, init: RequestInit = {}): Promise<T> {
  let response: Response;
  try {
    response = await fetch(`/api/auth/${path}`, {
      credentials: "same-origin",
      headers: { "Content-Type": "application/json", ...(init.headers ?? {}) },
      ...init,
    });
  } catch (error) {
    throw new ApiError(
      error instanceof Error ? error.message : "Network error",
      0,
      error,
    );
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
      const record = payload as Record<string, unknown>;
      throw new ApiError(
        typeof record.message === "string" ? record.message : `HTTP ${response.status}`,
        response.status,
        record,
      );
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

export const authApi = {
  /**
   * Probe the server's auth state. Returns enforced=false when
   * MARINARA_AUTH_ENABLED is unset; returns the user + csrfToken when
   * the cookie maps to a live session; 401s when enforced and no session.
   */
  me: () => fetchAuth<MeResponse>("me"),
  login: (username: string, password: string) =>
    fetchAuth<LoginResponse>("login", {
      method: "POST",
      body: JSON.stringify({ username, password }),
    }),
  /**
   * Logout requires the X-CSRF-Token header (the middleware gates the
   * route). We read it from the in-memory store rather than asking the
   * caller to pass it, since the SPA only ever logs out the current
   * session.
   */
  logout: () => {
    const csrf = useAuthStore.getState().csrfToken;
    return fetchAuth<{ ok: boolean }>("logout", {
      method: "POST",
      headers: csrf ? { "X-CSRF-Token": csrf } : undefined,
    });
  },
};

/**
 * Resolve the auth store from the server on app boot. The result drives
 * what App.tsx renders: anonymous → LoginPage, anything else → AppShell.
 * Safe to call repeatedly; idempotent.
 *
 * Failure modes (fail-closed):
 * - On Tauri: skip the network call entirely, set disabled.
 * - 2xx with enforced=false: disabled.
 * - 2xx with enforced=true and user: authenticated.
 * - 401: anonymous (login form shown).
 * - Anything else: anonymous (treat as needing re-auth rather than
 *   silently letting the user through with stale state).
 */
export async function bootstrapAuth(): Promise<void> {
  const store = useAuthStore.getState();
  if (!platform.isWeb) {
    store.setDisabled();
    return;
  }
  store.setLoading();
  try {
    const response = await authApi.me();
    if (!response.enforced) {
      store.setDisabled();
      return;
    }
    if (response.user && response.csrfToken) {
      store.setAuthenticated(response.user, response.csrfToken);
      return;
    }
    store.setAnonymous();
  } catch (error) {
    if (error instanceof ApiError && error.status === 401) {
      store.setAnonymous();
      return;
    }
    // Fail-closed: if we can't reach /api/auth/me we'd rather show the
    // login form than let the user through with stale state. The login
    // request itself can still succeed (or also fail loud) and the user
    // gets a real error message either way.
    store.setAnonymous();
  }
}
