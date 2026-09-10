import { defineConfig } from "vite";
import { solidGpui } from "@solid-gpui/vite";
import { resolve } from "node:path";
const root = import.meta.dirname;
const source = (path: string) => resolve(root, "../..", path);
export default defineConfig({
  root,
  plugins: [
    solidGpui({
      entry: "src/main.tsx",
      native: {
        manifestPath: source("examples/native-migration/native/Cargo.toml"),
        output: source("examples/native-migration/src/native.ts"),
      },
    }),
  ],
  resolve: {
    alias: [
      { find: "@solid-gpui/core/stdio", replacement: source("packages/solid-gpui/src/stdio.ts") },
      { find: "@solid-gpui/core/runtime", replacement: source("packages/solid-gpui/src/runtime.ts") },
      { find: "@solid-gpui/core/native", replacement: source("packages/solid-gpui/src/native.ts") },
      { find: "@solid-gpui/core/motion", replacement: source("packages/solid-gpui/src/motion.ts") },
      { find: "@solid-gpui/core/components", replacement: source("packages/solid-gpui/src/components.ts") },
      { find: "@solid-gpui/core", replacement: source("packages/solid-gpui/src/index.ts") },
      { find: "@solid-gpui/router", replacement: source("packages/solid-gpui-router/src/index.ts") },
    ],
  },
  build: { outDir: "dist" },
});
