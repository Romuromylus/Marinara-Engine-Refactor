import { Component, type ErrorInfo, type ReactNode } from "react";

type Props = {
  children: ReactNode;
};

type State = {
  error: Error | null;
};

export class AppErrorBoundary extends Component<Props, State> {
  state: State = { error: null };

  static getDerivedStateFromError(error: Error): State {
    return { error };
  }

  componentDidCatch(error: Error, info: ErrorInfo): void {
    console.error("AppErrorBoundary caught:", error, info);
  }

  handleReload = () => {
    window.location.reload();
  };

  render() {
    if (!this.state.error) return this.props.children;
    return (
      <div
        style={{
          minHeight: "100vh",
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          justifyContent: "center",
          padding: "2rem",
          background: "#0d1117",
          color: "#e6edf3",
          fontFamily: "system-ui, sans-serif",
          gap: "1rem",
          textAlign: "center",
        }}
      >
        <h1 style={{ fontSize: "1.25rem", fontWeight: 600, margin: 0 }}>
          Marinara hit an unexpected error
        </h1>
        <p style={{ maxWidth: "32rem", lineHeight: 1.5, opacity: 0.8 }}>
          The UI failed to render. The server API (
          <code style={{ background: "#1f2937", padding: "0 0.3em", borderRadius: 4 }}>
            /api/*
          </code>
          ) is still reachable; this only affects the SPA. The error message below was logged
          to the browser console.
        </p>
        <pre
          style={{
            maxWidth: "48rem",
            maxHeight: "16rem",
            overflow: "auto",
            background: "#161b22",
            border: "1px solid #30363d",
            borderRadius: 6,
            padding: "0.75rem 1rem",
            fontSize: "0.8125rem",
            textAlign: "left",
            whiteSpace: "pre-wrap",
          }}
        >
          {this.state.error.stack ?? this.state.error.message}
        </pre>
        <button
          type="button"
          onClick={this.handleReload}
          style={{
            background: "#2da44e",
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
