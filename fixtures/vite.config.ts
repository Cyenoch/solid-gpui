import { defineConfig } from "vite";
import { basename, dirname, resolve } from "node:path";
import { solidGpui } from "../packages/solid-gpui-vite/src/index.ts";

const root = resolve(import.meta.dirname, "..");
const runtime = process.env.SOLID_GPUI_FIXTURE_RUNTIME ?? "quickjs";
if (runtime !== "bun" && runtime !== "quickjs") throw new Error("Unknown fixture runtime");
const output = resolve(process.env.SOLID_GPUI_FIXTURE_OUTPUT ?? resolve(root, ".scratch/vite-fixture/app.js"));

export const fixtureAliases = ["runtime", "stdio", "embedded", "native", "components"]
  .map((name) => ({
    find: `@solid-gpui/core/${name}`,
    replacement: resolve(root, `packages/solid-gpui/src/${name}.ts`),
  }))
  .concat([
    { find: "@solid-gpui/core", replacement: resolve(root, "packages/solid-gpui/src/index.ts") },
    { find: "@solid-gpui/router", replacement: resolve(root, "packages/solid-gpui-router/src/index.ts") },
  ]);

export default defineConfig({
  root,
  plugins: [
    solidGpui({
      entry: process.env.SOLID_GPUI_FIXTURE ?? "fixtures/quickjs-counter.tsx",
      runtime,
      host: { command: resolve(root, "target/debug/solid-gpui-host") },
    }),
  ],
  resolve: { alias: fixtureAliases },
  build: {
    outDir: dirname(output),
    emptyOutDir: false,
    rolldownOptions: { output: { entryFileNames: basename(output) } },
  },
});
