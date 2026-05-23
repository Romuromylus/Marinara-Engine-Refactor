// In-memory Zustand store for the web target's auth state. NEVER persisted
// to localStorage — that would leak the CSRF token onto disk and undermine
// the HttpOnly-cookie session model. On boot, the store re-bootstraps from
// `/api/auth/me` (web) or short-circuits to "disabled" (Tauri).
import { create } from "zustand";

export type AuthUser = {
  id: string;
  username: string;
  role: string;
};

/**
 * unknown — initial. The bootstrap call has not run yet.
 * loading — bootstrap call in flight.
 * disabled — server reports `enforced: false` (Phase 6a default-off prod)
 *            OR we're running on Tauri where auth doesn't apply. The app
 *            renders normally; no login form, no CSRF header on writes.
 * authenticated — valid session cookie + csrfToken loaded. Renders normally
 *                 and the fetch transport echoes the token on mutating
 *                 requests.
 * anonymous — auth is enforced and we have no session (cookie missing or
 *             expired). App renders the login page.
 */
export type AuthStatus =
  | "unknown"
  | "loading"
  | "disabled"
  | "authenticated"
  | "anonymous";

type AuthState = {
  status: AuthStatus;
  enforced: boolean;
  user: AuthUser | null;
  csrfToken: string | null;
  setLoading: () => void;
  setDisabled: () => void;
  setAuthenticated: (user: AuthUser, csrfToken: string) => void;
  setAnonymous: () => void;
};

export const useAuthStore = create<AuthState>((set) => ({
  status: "unknown",
  enforced: false,
  user: null,
  csrfToken: null,
  setLoading: () => set({ status: "loading" }),
  setDisabled: () =>
    set({ status: "disabled", enforced: false, user: null, csrfToken: null }),
  setAuthenticated: (user, csrfToken) =>
    set({ status: "authenticated", enforced: true, user, csrfToken }),
  setAnonymous: () =>
    set({ status: "anonymous", enforced: true, user: null, csrfToken: null }),
}));
