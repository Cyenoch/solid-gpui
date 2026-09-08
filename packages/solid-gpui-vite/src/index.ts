import { resolve } from "node:path";
import { builtinModules } from "node:module";
import { createRunnableDevEnvironment, normalizePath, type Plugin } from "vite";
import { buildNativeHost, exportNativeBindings, type NativeExportOptions } from "./native-export.ts";
import { createStdioEnvironment, NativeDevEnvironment, type NativeHostOptions } from "./environment.ts";
import { transformJsx } from "./transform.ts";
import { quickJsBuild, quickJsEntry } from "./quickjs-build.ts";
import { QuickJsDevEnvironment } from "./quickjs-dev.ts";
import type { StdioConfig } from "./environment.ts";

export interface SolidGpuiOptions {
  readonly entry: string;
  readonly runtime?: "bun" | "quickjs";
  readonly native?: Omit<NativeExportOptions, "check" | "output"> & { readonly output?: string };
  /** An existing host, or false for an application-owned module runner. */
  readonly host?: NativeHostOptions | false;
}
export type { NativeHostOptions } from "./environment.ts";

/** Vite development and production builds for Solid GPUI applications. */
export function solidGpui(options: SolidGpuiOptions | { readonly target: "web" }): Plugin {
  let entry: string;
  let prepare: () => Promise<NativeHostOptions>;
  const web = "target" in options;
  const quickjs = !web && options.runtime === "quickjs";
  return {
    name: "solid-gpui",
    enforce: "pre",
    perEnvironmentStartEndDuringDev: true,
    ...(quickjs ? quickJsBuild(() => entry) : {}),
    async config(config, environment) {
      if (web) return;
      const stdio = (config as StdioConfig).__solidGpuiStdio;
      if (options.runtime && options.runtime !== "bun" && options.runtime !== "quickjs")
        throw new Error("Runtime must be bun or quickjs");
      if (quickjs && stdio)
        throw new Error(
          "Rust's Vite process adapter runs Bun; start QuickJS development with vite and a QuickJS-enabled host",
        );
      if (
        quickjs &&
        environment.command === "serve" &&
        options.host === false &&
        !config.environments?.ssr?.dev?.createEnvironment
      )
        throw new Error("QuickJS development requires a native host or an application-owned Vite environment");
      if (options.native && options.host)
        throw new Error("Choose native or host; native already selects the host executable");
      const root = resolve(config.root ?? process.cwd());
      entry = normalizePath(resolve(root, options.entry));
      const output = config.build?.rolldownOptions?.output;
      if (quickjs && Array.isArray(output)) throw new Error("QuickJS requires a single Vite output configuration");
      const native = options.native && normalizePath(resolve(root, options.native.output ?? ".generated/native.ts"));
      let preparation: Promise<NativeHostOptions> | undefined;
      prepare = () =>
        (preparation ??= (async () => {
          if (!options.native) return options.host || { command: "solid-gpui-host" };
          const executable = stdio?.nativeHost ?? (await buildNativeHost(options.native, root));
          await exportNativeBindings({ ...options.native, output: native! }, root, executable);
          return { command: executable };
        })());
      // During restart, dev preparation waits until the previous environment closes.
      if (environment.command === "build") await prepare();
      return {
        ...(native ? { resolve: { alias: [{ find: "#native", replacement: native }] } } : {}),
        appType: "custom",
        server: {
          open: false,
          watch: { ignored: ["**/target/**", `${normalizePath(resolve(root, config.build?.outDir ?? "dist"))}/**`] },
        },
        optimizeDeps: { noDiscovery: true, include: [] },
        ssr: {
          noExternal: quickjs || environment.command === "build" ? true : [/^@solid-gpui\//],
          external:
            quickjs || environment.command === "build" ? [] : ["solid-js", "solid-js/universal", "solid-js/store"],
          resolve: {
            externalConditions: ["bun", "browser"],
            conditions: [...(quickjs ? [] : ["bun"]), "browser", "module", "development|production"],
          },
        },
        build: {
          ssr: quickjs ? true : entry,
          target: "esnext",
          ...(quickjs
            ? {
                emitAssets: true,
                rolldownOptions: {
                  input: quickJsEntry,
                  platform: "neutral",
                  output: {
                    codeSplitting: false,
                    entryFileNames: (Array.isArray(output) ? undefined : output?.entryFileNames) ?? "app.js",
                  },
                },
              }
            : {}),
        },
        ...(environment.command === "build" ? { define: { "process.env.NODE_ENV": '"production"' } } : {}),
        environments: {
          ssr: {
            keepProcessEnv: !quickjs,
            resolve: { builtins: quickjs ? [] : [...builtinModules, /^node:/, "bun", /^bun:/] },
            dev: {
              preTransformRequests: false,
              createEnvironment:
                config.environments?.ssr?.dev?.createEnvironment ??
                ((name, config) =>
                  stdio
                    ? createStdioEnvironment(name, config, entry, prepare)
                    : options.host === false
                      ? createRunnableDevEnvironment(name, config)
                      : quickjs
                        ? new QuickJsDevEnvironment(name, config, prepare)
                        : new NativeDevEnvironment(name, config, entry, prepare)),
            },
          },
        },
      };
    },
    configResolved(config) {
      if (config.plugins.filter((plugin) => plugin.name === "solid-gpui").length !== 1) {
        throw new Error("Configure solidGpui once per Vite application");
      }
    },
    async buildStart() {
      if (!web && this.environment.name === "ssr") await prepare();
    },
    transform(code, id) {
      const filename = normalizePath(id.split("?")[0]!);
      let map;
      if (/\.[jt]sx$/.test(filename)) {
        const result = transformJsx(code, filename);
        code = result.code;
        map = result.map;
      } else if (filename !== entry) return;
      if (!quickjs && filename === entry) code += "\nif (import.meta.hot) import.meta.hot.accept();\n";
      return { code, map };
    },
  };
}
