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

  // Force Vite to pre-bundle the React family as a single unit. Without this,
  // Rollup ends up emitting react / react-dom / react-dom/client as three
  // separate CJS modules in the production bundle, each with its own internal
  // React dispatcher state. That triggers React error #321 because hooks
  // called from one bundle's React look at the other bundle's (un-set)
  // dispatcher and throw "Invalid hook call". Pre-bundling collapses them
  // into one ESM module so all hook dispatchers reference the same instance.
  optimizeDeps: {
    include: ["react", "react-dom", "react-dom/client", "react/jsx-runtime"],
  },

  // Production source maps for the server-target image so the browser shows
  // real stack traces instead of chunk-N:offset noise. The desktop Tauri
  // bundle skips them — they'd bloat the installer ~30% for users who can't
  // open them anyway.
  //
  // commonjsOptions: React 19 ships CJS-only. Without these settings,
  // Rollup's @rollup/plugin-commonjs emits two separate CJS wrappers for
  // react (one from the top-level `import "react"`, one from
  // `react/jsx-runtime`'s internal `require("react")`). Each wrapper has
  // its own dispatcher state (`j.H`), so when react-dom-client sets the
  // dispatcher on one, hooks called from code consuming the OTHER throw
  // React error #321 ("Invalid hook call"). `requireReturnsDefault: "auto"`
  // and `transformMixedEsModules: true` together tell the plugin to share
  // CJS module instances across importers, collapsing React to a single
  // module with one dispatcher.
  //
  // manualChunks groups the react family into one vendor chunk so the
  // production output mirrors what optimizeDeps does in dev.
  build: {
    sourcemap: isWebTarget,
    commonjsOptions: {
      transformMixedEsModules: true,
      requireReturnsDefault: "auto" as const,
      include: [/node_modules/],
    },
    rollupOptions: {
      output: {
        manualChunks: {
          "react-vendor": ["react", "react-dom", "react-dom/client", "react/jsx-runtime"],
        },
      },
    },
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
