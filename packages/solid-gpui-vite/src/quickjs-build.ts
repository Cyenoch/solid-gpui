import { isBuiltin } from "node:module";
import { fileURLToPath } from "node:url";
import { extname } from "node:path";
import type { Plugin } from "vite";

export const quickJsEntry = "virtual:solid-gpui/quickjs-entry";
const resolvedEntry = "\0" + quickJsEntry;

/** QuickJS executes one ESM module and has no ambient Bun/Node module loader. */
export function quickJsBuild(entry: () => string): Pick<Plugin, "resolveId" | "load" | "generateBundle"> {
  return {
    resolveId(id) {
      if (id === quickJsEntry) return resolvedEntry;
      if (isBuiltin(id) || id === "bun" || id.startsWith("bun:") || id.startsWith("node:")) {
        throw new Error(`QuickJS cannot import ${JSON.stringify(id)}; expose host services through native commands`);
      }
    },
    load(id) {
      if (id === resolvedEntry)
        return `import ${JSON.stringify(fileURLToPath(new URL("./quickjs-platform" + extname(fileURLToPath(import.meta.url)), import.meta.url)))};\nimport ${JSON.stringify(entry())};`;
    },
    generateBundle(_options, bundle) {
      const outputs = Object.values(bundle).filter((output) => !output.fileName.endsWith(".map"));
      const output = outputs[0];
      if (
        outputs.length !== 1 ||
        output?.type !== "chunk" ||
        !output.isEntry ||
        output.imports.length ||
        output.dynamicImports.length
      ) {
        throw new Error("QuickJS requires one self-contained JavaScript module; load native assets through host APIs");
      }
      // A computed import can survive bundling without appearing in dynamicImports.
      const visit = (node: unknown): void => {
        if (!node || typeof node !== "object") return;
        if ("type" in node && (node.type === "ImportExpression" || node.type === "ImportDeclaration")) {
          throw new Error("QuickJS cannot load external modules at runtime; use statically bundled imports");
        }
        for (const value of Object.values(node)) {
          if (Array.isArray(value)) value.forEach(visit);
          else if (value && typeof value === "object") visit(value);
        }
      };
      visit(this.parse(output.code));
    },
  };
}
