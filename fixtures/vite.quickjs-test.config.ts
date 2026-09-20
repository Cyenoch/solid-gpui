import { resolve } from "node:path";
import { defineConfig } from "vite";
import { solidGpui } from "../packages/solid-gpui-vite/src/index.ts";
import { fixtureAliases, fixtureBuild, fixtureRoot } from "./vite.shared.ts";

// Rust VM tests own the real QuickJS host; this config only transforms their TSX fixture.
const entry = process.env.SOLID_GPUI_FIXTURE ?? "fixtures/quickjs-counter.tsx";
const output = resolve(process.env.SOLID_GPUI_FIXTURE_OUTPUT ?? resolve(fixtureRoot, ".scratch/vite-fixture/app.js"));

export default defineConfig({
  root: fixtureRoot,
  plugins: [solidGpui({ entry, runtime: "quickjs", host: false })],
  resolve: { alias: fixtureAliases },
  build: fixtureBuild(output),
});
