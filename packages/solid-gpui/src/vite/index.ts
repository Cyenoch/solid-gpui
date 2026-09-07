import { resolve } from "node:path";
import type { Plugin } from "vite";
import { exportNativeBindings, type NativeExportOptions } from "./native-export";
import { transformJsx } from "./transform";

export interface SolidGpuiOptions {
  readonly entry: string;
  readonly native?: Omit<NativeExportOptions, "check" | "output"> & { readonly output?: string };
}

/** Native universal JSX, shared by development and Bun production bundles. */
export function solidGpui(options: SolidGpuiOptions): Plugin {
  let entry: string;
  return {
    name: "solid-gpui",
    enforce: "pre",
    async config(config) {
      const root = config.root ?? process.cwd();
      entry = resolve(root, options.entry);
      const native =
        options.native &&
        (await exportNativeBindings(
          {
            ...options.native,
            output: options.native.output ?? ".generated/native.ts",
          },
          root,
        ));
      return {
        ...(native ? { resolve: { alias: [{ find: "#native", replacement: native }] } } : {}),
        appType: "custom",
        optimizeDeps: { noDiscovery: true, include: [] },
        ssr: {
          noExternal: [/^@solid-gpui\//],
          external: ["solid-js", "solid-js/universal", "solid-js/store"],
          resolve: { externalConditions: ["browser"], conditions: ["browser", "module", "development|production"] },
        },
        build: { ssr: entry, target: "esnext" },
      };
    },
    transform(code, id) {
      const filename = id.split("?")[0]!;
      let map;
      if (/\.[jt]sx$/.test(filename)) {
        const result = transformJsx(code, filename);
        code = result.code;
        map = result.map;
      } else if (filename !== entry) return;
      if (filename === entry) code += "\nif (import.meta.hot) import.meta.hot.accept();\n";
      return { code, map };
    },
  };
}
