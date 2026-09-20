import { basename, dirname, resolve } from "node:path";

export const fixtureRoot = resolve(import.meta.dirname, "..");

export const fixtureAliases = ["runtime", "stdio", "embedded", "native", "components"]
  .map((name) => ({
    find: `@solid-gpui/core/${name}`,
    replacement: resolve(fixtureRoot, `packages/solid-gpui/src/${name}.ts`),
  }))
  .concat([
    { find: "@solid-gpui/core", replacement: resolve(fixtureRoot, "packages/solid-gpui/src/index.ts") },
    {
      find: "@solid-gpui/router",
      replacement: resolve(fixtureRoot, "packages/solid-gpui-router/src/index.ts"),
    },
  ]);

export function fixtureBuild(output: string) {
  return {
    outDir: dirname(output),
    emptyOutDir: false,
    rolldownOptions: { output: { entryFileNames: basename(output) } },
  } as const;
}
