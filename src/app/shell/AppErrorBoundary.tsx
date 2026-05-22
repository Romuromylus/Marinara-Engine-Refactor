import { Component, type ErrorInfo, type ReactNode } from "react";

type Props = {
  children: ReactNode;
};

type State = {
  error: Error | null;
  componentStack: string | null;
};

type Palette = {
  background: string;
  foreground: string;
  muted: string;
  codeBackground: string;
  border: string;
  surface: string;
  primary: string;
};

const DARK_PALETTE: Palette = {
  background: "#0d1117",
  foreground: "#e6edf3",
  muted: "rgba(230, 237, 243, 0.72)",
  codeBackground: "#1f2937",
  border: "#30363d",
  surface: "#161b22",
  primary: "#2da44e",
};

const LIGHT_PALETTE: Palette = {
  background: "#f6f8fa",
  foreground: "#1f2328",
  muted: "rgba(31, 35, 40, 0.72)",
  codeBackground: "#eef2f6",
  border: "#d0d7de",
  surface: "#ffffff",
  primary: "#1f883d",
};

function resolvePalette(): Palette {
  if (typeof window === "undefined" || typeof window.matchMedia !== "function") {
    return DARK_PALETTE;
  }
  return window.matchMedia("(prefers-color-scheme: dark)").matches
    ? DARK_PALETTE
    : LIGHT_PALETTE;
}

export class AppErrorBoundary extends Component<Props, State> {
  state: State = { error: null, componentStack: null };

  static getDerivedStateFromError(error: Error): Partial<State> {
    return { error };
  }

  componentDidCatch(error: Error, info: ErrorInfo): void {
    this.setState({ componentStack: info.componentStack ?? null });
    console.error(
      "AppErrorBoundary caught:",
      error,
      info.componentStack ?? "(no componentStack)",
    );
  }

  handleReload = () => {
    window.location.reload();
  };

  render() {
    if (!this.state.error) return this.props.children;
    const palette = resolvePalette();
    const stackBody =
      this.state.error.stack ?? this.state.error.message ?? "Unknown error";
    const componentStack = this.state.componentStack;

    return (
      <div
        role="alert"
        aria-live="assertive"
        style={{
          minHeight: "100vh",
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          justifyContent: "center",
          padding: "2rem",
          background: palette.background,
          color: palette.foreground,
          fontFamily: "system-ui, sans-serif",
          gap: "1rem",
          textAlign: "center",
        }}
      >
        <h1 style={{ fontSize: "1.25rem", fontWeight: 600, margin: 0 }}>
          Marinara hit an unexpected error
        </h1>
        <p
          style={{
            maxWidth: "32rem",
            lineHeight: 1.5,
            color: palette.muted,
          }}
        >
          The UI failed to render. The server API (
          <code
            style={{
              background: palette.codeBackground,
              padding: "0 0.3em",
              borderRadius: 4,
            }}
          >
            /api/*
          </code>
          ) is still reachable; this only affects the SPA. The error was logged
          to the browser console.
        </p>
        <pre
          style={{
            maxWidth: "48rem",
            maxHeight: "16rem",
            overflow: "auto",
            background: palette.surface,
            border: `1px solid ${palette.border}`,
            borderRadius: 6,
            padding: "0.75rem 1rem",
            fontSize: "0.8125rem",
            textAlign: "left",
            whiteSpace: "pre-wrap",
            color: palette.foreground,
          }}
        >
          {stackBody}
          {componentStack ? `\n\nComponent stack:${componentStack}` : ""}
        </pre>
        <button
          type="button"
          onClick={this.handleReload}
          style={{
            background: palette.primary,
            color: "white",
            border: 0,
            borderRadius: 6,
            padding: "0.5rem 1rem",
            fontSize: "0.875rem",
            cursor: "pointer",
          }}
        >
          Reload
        </button>
      </div>
    );
  }
}
