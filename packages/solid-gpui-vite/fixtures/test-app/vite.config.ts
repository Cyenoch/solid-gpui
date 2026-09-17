import { defineConfig } from "vite";
import { resolve } from "node:path";
import { solidGpui } from "../../src/index.ts";

/** Mirrors an application config: the plugin owns JSX/native, the app owns #native. */
export default defineConfig({
  root: import.meta.dirname,
  plugins: [solidGpui({ entry: "src/counter.tsx" })],
  resolve: {
    // Exercise the published source condition against the SDK checkout.
    conditions: ["solid-gpui-source"],
    alias: [{ find: "#native", replacement: resolve(import.meta.dirname, "generated/native.ts") }],
  },
});
