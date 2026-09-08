import { solidGpuiRouter } from "../../packages/solid-gpui-router/src/vite";
import { componentVariants } from "./component-variants";
import { solidGpui } from "../../packages/solid-gpui/src/vite/index.ts";
import { browserPreviewNames } from "./component-previews";
import { componentExamples } from "./component-examples";
import { componentCatalog } from "./component-catalog";
import { buildHighlights } from "./build-highlights";
import { format } from "oxfmt";
import { defineConfig } from "vite";
import { resolve } from "node:path";
import { readdirSync } from "node:fs";
import { transformJsx } from "../../packages/solid-gpui/src/vite/transform.ts";
const source = (path: string) => resolve(import.meta.dirname, "../../packages/solid-gpui/src", path);
export function websiteConfig(desktop = false, embedded = false) {
  return defineConfig({
    root: import.meta.dirname,
    base: process.env.PAGES_BASE_PATH ?? "/solid-gpui/",
    plugins: [
      solidGpuiRouter(),
      ...(desktop ? [solidGpui({ entry: embedded ? "src/quickjs.tsx" : "src/main.native.tsx" })] : []),
      {
        name: "component-documentation",
        resolveId(id) {
          if (id === "virtual:component-previews" || id.startsWith("virtual:component-preview/")) return "\0" + id;
          if (id === "virtual:component-catalog") return "\0component-catalog";
        },
        async load(id) {
          for (const file of readdirSync(resolve(import.meta.dirname, "component-recipes"))) {
            this.addWatchFile(resolve(import.meta.dirname, "component-recipes", file));
          }
          if (id === "\0virtual:component-previews") {
            this.addWatchFile(resolve(import.meta.dirname, "component-previews.ts"));
            this.addWatchFile(resolve(import.meta.dirname, "component-variants.ts"));
            const names = [
              ...new Set([
                ...browserPreviewNames,
                ...componentVariants
                  .filter((example) => browserPreviewNames.has(example.component))
                  .map((example) => example.id),
              ]),
            ];
            return (
              names
                .map((name, index) => `import Preview${index} from "virtual:component-preview/${name}";`)
                .join("\n") +
              `\nexport default {${names.map((name, index) => `${JSON.stringify(name)}: Preview${index}`).join(",")}};`
            );
          }
          if (id.startsWith("\0virtual:component-preview/")) {
            this.addWatchFile(resolve(import.meta.dirname, "component-examples.ts"));
            const name = id.slice(id.lastIndexOf("/") + 1);
            this.addWatchFile(resolve(import.meta.dirname, "component-variants.ts"));
            const example =
              componentVariants.find((example) => example.id === name) ??
              componentExamples.find((example) => example.names.includes(name));
            if (!example) throw new Error(`Missing preview example: ${name}`);
            return transformJsx(example.source, resolve(import.meta.dirname, `src/preview-${name}.tsx`));
          }
          if (id !== "\0component-catalog") return;
          this.addWatchFile(resolve(import.meta.dirname, "../../packages/solid-gpui/src/components.ts"));
          this.addWatchFile(resolve(import.meta.dirname, "component-examples.ts"));
          this.addWatchFile(resolve(import.meta.dirname, "src/examples.ts"));
          this.addWatchFile(resolve(import.meta.dirname, "src/snippets.ts"));
          this.addWatchFile(resolve(import.meta.dirname, "component-variants.ts"));
          this.addWatchFile(resolve(import.meta.dirname, "component-families.ts"));
          const catalog = componentCatalog();
          const formatted = new Map<string, string>();
          const formatSource = async (source: string) => {
            const cached = formatted.get(source);
            if (cached !== undefined) return cached;
            const result = await format("Example.tsx", source, { printWidth: 76 });
            if (result.errors.length) throw new Error("Invalid component example");
            formatted.set(source, result.code);
            return result.code;
          };
          for (const doc of catalog) {
            doc.source = await formatSource(doc.source);
            for (const example of doc.examples) example.source = await formatSource(example.source);
          }
          const highlights = await buildHighlights(catalog, (path) => this.addWatchFile(path));
          return `export default ${JSON.stringify(catalog)};\nexport const highlights = ${JSON.stringify(highlights)};`;
        },
      },
      ...(!desktop
        ? [
            {
              name: "solid-gpui-web-jsx",
              enforce: "pre",
              transform(code, id) {
                const filename = id.split("?")[0]!;
                if (/\.[jt]sx$/.test(filename)) return transformJsx(code, filename);
              },
            } as import("vite").Plugin,
          ]
        : []),
    ],
    resolve: {
      alias: [
        {
          find: "@solid-gpui/router",
          replacement: resolve(import.meta.dirname, "../../packages/solid-gpui-router/src/index.ts"),
        },
        ...(desktop
          ? [{ find: "./HeroVisual", replacement: resolve(import.meta.dirname, "src/HeroVisual.native.tsx") }]
          : []),
        { find: "@solid-gpui/core/stdio", replacement: source("stdio.ts") },
        { find: "@solid-gpui/core/embedded", replacement: source("embedded.ts") },
        { find: "assert", replacement: "assert/" },
        {
          find: "@solid-gpui/shiki",
          replacement: resolve(import.meta.dirname, "../../packages/solid-gpui-shiki/src/index.ts"),
        },
        { find: "@solid-gpui/core/native", replacement: source("native.ts") },
        { find: "@solid-gpui/core/components", replacement: source("components.ts") },
        { find: "@solid-gpui/core/runtime", replacement: source("runtime.ts") },
        { find: "@solid-gpui/core/web", replacement: source("web.ts") },
        { find: "@solid-gpui/core", replacement: source("index.ts") },
      ],
    },
    build: {
      outDir: desktop ? (embedded ? "dist-embedded" : "dist-native") : "dist",
      target: "esnext",
      rolldownOptions: {
        ...(embedded ? { platform: "neutral" as const } : {}),
        input: desktop
          ? resolve(import.meta.dirname, embedded ? "src/quickjs.tsx" : "src/main.native.tsx")
          : { main: resolve(import.meta.dirname, "index.html") },
      },
    },
  });
}
export default websiteConfig();
