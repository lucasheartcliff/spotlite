import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

// @tauri-apps/cli sets TAURI_DEV_HOST when running `tauri dev`.
const host = process.env.TAURI_DEV_HOST;

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [svelte()],

  // Prevent Vite from obscuring Rust errors.
  clearScreen: false,
  server: {
    // Tauri expects a fixed port and fails if it is unavailable.
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
      // Don't watch the Rust source tree.
      ignored: ["**/src-tauri/**"],
    },
  },

  // Produce a lean bundle — every KB counts for the "lightweight" goal.
  build: {
    target: "esnext",
    minify: "esbuild",
    sourcemap: false,
  },
});
