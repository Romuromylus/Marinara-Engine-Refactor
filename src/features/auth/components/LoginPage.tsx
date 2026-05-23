import { useState, type FormEvent } from "react";
import { LogIn } from "lucide-react";
import { authApi } from "../../../shared/api/auth-api";
import { useAuthStore } from "../../../shared/stores/auth.store";
import { ApiError } from "../../../shared/api/api-errors";

export function LoginPage() {
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (submitting) return;
    setSubmitting(true);
    setError(null);
    try {
      const response = await authApi.login(username, password);
      useAuthStore.getState().setAuthenticated(response.user, response.csrfToken);
    } catch (err) {
      const message =
        err instanceof ApiError
          ? err.message
          : err instanceof Error
            ? err.message
            : "Login failed";
      setError(message);
      setSubmitting(false);
    }
  }

  return (
    <div
      className="flex min-h-screen items-center justify-center px-4"
      style={{ background: "var(--background)", color: "var(--foreground)" }}
    >
      <div
        className="w-full max-w-sm rounded-lg border p-8 shadow-lg"
        style={{ background: "var(--card)", borderColor: "var(--border)" }}
      >
        <div className="mb-6 flex items-center justify-center">
          <div
            className="flex h-12 w-12 items-center justify-center rounded-full"
            style={{ background: "var(--primary)", color: "var(--primary-foreground)" }}
          >
            <LogIn size={20} />
          </div>
        </div>
        <h1 className="mb-1 text-center text-xl font-semibold">Sign in to Marinara</h1>
        <p
          className="mb-6 text-center text-sm"
          style={{ color: "var(--muted-foreground)" }}
        >
          Enter your credentials to continue.
        </p>

        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label
              htmlFor="login-username"
              className="mb-1 block text-sm font-medium"
            >
              Username
            </label>
            <input
              id="login-username"
              type="text"
              autoComplete="username"
              required
              autoFocus
              value={username}
              onChange={(event) => setUsername(event.target.value)}
              disabled={submitting}
              className="w-full rounded-md border px-3 py-2 text-sm outline-none focus:ring-2 disabled:opacity-50"
              style={{
                background: "var(--background)",
                borderColor: "var(--border)",
                color: "var(--foreground)",
              }}
            />
          </div>

          <div>
            <label
              htmlFor="login-password"
              className="mb-1 block text-sm font-medium"
            >
              Password
            </label>
            <input
              id="login-password"
              type="password"
              autoComplete="current-password"
              required
              value={password}
              onChange={(event) => setPassword(event.target.value)}
              disabled={submitting}
              className="w-full rounded-md border px-3 py-2 text-sm outline-none focus:ring-2 disabled:opacity-50"
              style={{
                background: "var(--background)",
                borderColor: "var(--border)",
                color: "var(--foreground)",
              }}
            />
          </div>

          {error ? (
            <div
              role="alert"
              aria-live="polite"
              className="rounded-md border px-3 py-2 text-sm"
              style={{
                background: "var(--destructive-background, var(--card))",
                borderColor: "var(--destructive, #ef4444)",
                color: "var(--destructive, #ef4444)",
              }}
            >
              {error}
            </div>
          ) : null}

          <button
            type="submit"
            disabled={submitting || !username || !password}
            className="w-full rounded-md px-4 py-2 text-sm font-medium transition disabled:opacity-50"
            style={{
              background: "var(--primary)",
              color: "var(--primary-foreground)",
            }}
          >
            {submitting ? "Signing in…" : "Sign in"}
          </button>
        </form>
      </div>
    </div>
  );
}
