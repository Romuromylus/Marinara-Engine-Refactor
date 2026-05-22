import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import { fileURLToPath, URL } from "node:url";

const host = process.env.TAURI_DEV_HOST;

// Build a server-target image when the Dockerfile sets VITE_TARGET=web. The
// Tauri build leaves this unset and gets the desktop bundle.
const isWebTarget = process.env.VITE_TARGET === "web";

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: {
      "@": fileURLToPath(new URL("./src", import.meta.url)),
    },
    // Dedupe React so a stray transitive copy can't produce two dispatchers
    // (the classic source of "Invalid hook call" / React error #321).
    dedupe: ["react", "react-dom"],
  },

  // Production source maps for the server-target image so the browser shows
  // real stack traces instead of chunk-N:offset noise. The desktop Tauri
  // bundle skips them — they'd bloat the installer ~30% for users who can't
  // open them anyway.
  build: {
    sourcemap: isWebTarget,
  },

  // The dev server section below is Tauri-specific (port 1420 + HMR host
  // negotiation matches `tauri dev`). The build/resolve options above apply
  // to both targets.
  //
  // 1. prevent vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
}));
