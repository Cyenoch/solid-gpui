import { resolve } from "node:path";
import { defineConfig } from "vite";
import { solidGpui } from "../../src/index.ts";

/**
 * Every mode-dependent knob an application has: the entry the plugin plans for, a
 * resolution alias and a definition. Each one records the mode Vite resolved, so a
 * test can prove which mode the test runner asked for.
 */
export default defineConfig(({ mode }) => {
  const staging = mode === "staging";
  const variant = staging ? "staging" : "development";
  return {
    root: import.meta.dirname,
    plugins: [solidGpui({ entry: `src/entry-${variant}.ts` })],
    define: { __SOLID_GPUI_FIXTURE_MODE__: JSON.stringify(mode) },
    resolve: {
      alias: [{ find: "#mode", replacement: resolve(import.meta.dirname, `src/mode-${variant}.ts`) }],
    },
  };
});
