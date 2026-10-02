import { defineConfig } from "vite";
import { solidGpui } from "../../packages/solid-gpui-vite/src/index.ts";
import { solidGpuiSource } from "../../packages/solid-gpui-vite/src/source.ts";

// This shares the Showcase modules and existing stock host; no alternate renderer or native catalog.
export default defineConfig({
  root: import.meta.dirname,
  plugins: [
    solidGpuiSource({ root: import.meta.dirname, exclude: [] }),
    solidGpui({ entry: "tests/reference-studio.entry.tsx", host: false }),
  ],
  build: { outDir: "dist-reference", rolldownOptions: { input: "tests/reference-studio.entry.tsx" } },
});
