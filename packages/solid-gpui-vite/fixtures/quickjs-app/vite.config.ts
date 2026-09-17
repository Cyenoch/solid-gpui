import { defineConfig } from "vite";
import { solidGpui } from "../../src/index.ts";

/**
 * A QuickJS application: the runtime has no ambient environment, so the plugin
 * configures the SSR environment to inline `process.env` for the production
 * bundle. Tests run in real Bun with the caller's environment and must not
 * inherit that inlining.
 */
export default defineConfig({
  root: import.meta.dirname,
  plugins: [solidGpui({ entry: "src/main.tsx", runtime: "quickjs" })],
});
