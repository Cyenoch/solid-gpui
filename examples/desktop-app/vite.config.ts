import { defineConfig } from "vite";
import { solidGpui } from "@solid-gpui/vite";
import { solidGpuiSource } from "@solid-gpui/vite/source";
import { resolve } from "node:path";
const root = import.meta.dirname;
const source = (path: string) => resolve(root, "../..", path);
export default defineConfig({
  root,
  plugins: [
    solidGpuiSource({ root }),
    solidGpui({
      entry: "src/main.tsx",
      native: {
        manifestPath: source("examples/desktop-app/native/Cargo.toml"),
        output: source("examples/desktop-app/src/native.ts"),
      },
    }),
  ],
  build: { outDir: "dist" },
});
