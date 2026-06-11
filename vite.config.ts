import { svelte } from "@sveltejs/vite-plugin-svelte";
import { defineConfig } from "vite";
import { svelteTesting } from "@testing-library/svelte/vite";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig(async () => ({
  plugins: [svelte(), svelteTesting()],
  clearScreen: false,
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
      ignored: ["**/src-tauri/**"],
    },
  },
  test: {
    include: ["src/**/*.test.ts"],
    environment: "jsdom",
    setupFiles: ["src/test/setup.ts"],
    // Parallel jsdom workers thrash low-memory machines; the suite is small,
    // so run files serially for reliability.
    fileParallelism: false,
  },
}));
