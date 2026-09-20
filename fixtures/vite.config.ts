import { defineConfig } from "vite";
import { resolve } from "node:path";
import { solidGpui } from "../packages/solid-gpui-vite/src/index.ts";
import { fixtureAliases, fixtureBuild, fixtureRoot } from "./vite.shared.ts";

const root = fixtureRoot;
const runtime = process.env.SOLID_GPUI_FIXTURE_RUNTIME ?? "quickjs";
if (runtime !== "bun" && runtime !== "quickjs") throw new Error("Unknown fixture runtime");
const output = resolve(process.env.SOLID_GPUI_FIXTURE_OUTPUT ?? resolve(root, ".scratch/vite-fixture/app.js"));

export default defineConfig({
  root,
  plugins: [
    solidGpui({
      entry: process.env.SOLID_GPUI_FIXTURE ?? "fixtures/quickjs-counter.tsx",
      runtime,
      ...(runtime === "quickjs"
        ? {
            native: {
              manifestPath: resolve(root, "Cargo.toml"),
              package: "solid-gpui",
              bin: "solid-gpui-host",
              features: ["quickjs"],
              output: resolve(root, ".scratch/vite-fixture/native.ts"),
            },
          }
        : {
            // This host is built separately; Vite exports its catalog before loading the application.
            host: {
              command: resolve(root, "target/debug/solid-gpui-host"),
              output: resolve(root, ".scratch/vite-fixture/native-bun.ts"),
            },
          }),
    }),
  ],
  resolve: { alias: fixtureAliases },
  build: fixtureBuild(output),
});
