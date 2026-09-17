import { builtinModules } from "node:module";
import { resolve } from "node:path";
import { createRunnableDevEnvironment, normalizePath, type Plugin } from "vite";
import {
  createStdioEnvironment,
  NativeDevEnvironment,
  type NativeHostOptions,
  type StdioConfig,
} from "./environment.ts";
import type { NativeBuildReporter } from "./native-export.ts";
import { transformJsx } from "./transform.ts";
import { quickJsBuild, quickJsEntry } from "./quickjs-build.ts";
import { QuickJsDevEnvironment } from "./quickjs-dev.ts";
import {
  assertHostRunsLocally,
  createSolidGpuiProject,
  publishProject,
  registerSolidGpuiProject,
  type NativePreparation,
  type SolidGpuiOptions,
  type SolidGpuiProject,
} from "./project.ts";
import { writeNativeArtifacts, type NativeArtifacts } from "./artifacts.ts";

export type { SolidGpuiOptions } from "./project.ts";
export type { NativeHostOptions } from "./environment.ts";
// Source consumption is configured from the same import as the plugin it modifies.
export {
  SOURCE_CONDITION,
  SDK_PACKAGES,
  pluginSourceMappings,
  sdkPackage,
  sdkSource,
  solidGpuiSource,
  type SdkPackageInfo,
  type SdkSource,
  type SdkSourceOptions,
} from "./source.ts";

/** Vite development and production builds for Solid GPUI applications. */
export function solidGpui(options: SolidGpuiOptions | { readonly target: "web" }): Plugin {
  let entry: string;
  let prepare: () => Promise<NativeHostOptions>;
  // The production build's own record, so the emitted bundle can correct it after output.
  let production: { readonly project: SolidGpuiProject; readonly artifacts: NativeArtifacts } | undefined;
  const web = "target" in options;
  const quickjs = !web && options.runtime === "quickjs";
  return registerSolidGpuiProject(
    {
      name: "solid-gpui",
      enforce: "pre",
      perEnvironmentStartEndDuringDev: true,
      ...(quickjs ? quickJsBuild(() => entry) : {}),
      async config(config, environment) {
        if (web) return;
        const stdio = (config as StdioConfig).__solidGpuiStdio;
        const preparedHost = (config as StdioConfig).__solidGpuiPreparedHost;
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
        // The plugin and the prepare CLI must resolve paths, aliases and the bundle from one plan.
        const project = createSolidGpuiProject(config, options, { preparedHost, plugins: config.plugins ?? [] });
        if (!project.entry) throw new Error("solidGpui({ entry }) requires an application entry");
        entry = project.entry;
        let preparation: Promise<NativePreparation> | undefined;
        const ready = () => (preparation ??= project.prepare());
        prepare = async () => (await ready()).host;
        // A production build publishes the same generated files the prepare CLI does, so packaging
        // scripts can read authoritative paths after either one. QuickJS development re-bundles
        // through this same hook into a temporary directory, and that nested build must not publish
        // (or verify) anything: its output is not the production bundle.
        const nestedDevelopment = preparedHost !== undefined;
        const check = project.native?.check === true;
        if (environment.command === "build" && !nestedDevelopment) {
          const { artifacts } = await publishProject(project, { preparation: await ready(), check });
          if (!check) production = { project, artifacts };
        }
        const sessionOptions = {
          native: project.native,
          bindings: project.bindings,
          // A fresh attempt per session: native changes must rebuild the host and its bindings.
          prepare: async (signal: AbortSignal, log: NativeBuildReporter) => {
            const preparation = await project.prepare(signal, { log });
            // Development launches the host, so a cross artifact must be refused by name.
            assertHostRunsLocally(preparation, "development");
            return preparation.host;
          },
        };
        return {
          resolve: {
            ...(project.aliases.length > 0 ? { alias: [...project.aliases] } : {}),
            // The SDK and the application must share one reactive graph, dist or source.
            dedupe: ["solid-js"],
          },
          appType: "custom",
          server: {
            open: false,
            watch: {
              ignored: [
                "**/target/**",
                `${normalizePath(resolve(project.root, config.build?.outDir ?? "dist"))}/**`,
                // Prepared files are inputs to editors, never to the application build.
                `${normalizePath(resolve(project.root, ".solid-gpui"))}/**`,
              ],
            },
          },
          optimizeDeps: { noDiscovery: true, include: [] },
          ssr: {
            noExternal: quickjs || environment.command === "build" ? true : [/^@solid-gpui\//],
            external:
              quickjs || environment.command === "build" ? [] : ["solid-js", "solid-js/universal", "solid-js/store"],
            resolve: {
              externalConditions: ["bun", "browser"],
              // An ssr list replaces the root conditions, so anything a plugin selected for the
              // runtime — source mode in particular — must be repeated here or the runtime and the
              // generated tsconfig stop agreeing.
              conditions: [
                ...new Set([
                  ...(quickjs ? [] : ["bun"]),
                  "browser",
                  ...(config.resolve?.conditions ?? []),
                  ...project.customConditions,
                  "module",
                  "development|production",
                ]),
              ],
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
                      // A configured pattern or function wins; the record is corrected after the
                      // build from the file Vite actually emitted.
                      entryFileNames:
                        (Array.isArray(config.build?.rolldownOptions?.output)
                          ? undefined
                          : config.build?.rolldownOptions?.output?.entryFileNames) ?? "app.js",
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
                          ? new QuickJsDevEnvironment(name, config, sessionOptions)
                          : new NativeDevEnvironment(name, config, entry, sessionOptions)),
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
        if (
          !web &&
          this.environment.name === "ssr" &&
          !(this.environment instanceof NativeDevEnvironment) &&
          !(this.environment instanceof QuickJsDevEnvironment)
        )
          await prepare();
      },
      /**
       * Vite decides the final file name, so the record is only authoritative once the bundle
       * exists: a configured function or pattern cannot be derived before the build runs.
       */
      async writeBundle(_options, bundle) {
        if (!production || this.environment.name !== "ssr") return;
        const entry = Object.values(bundle).find((item) => item.type === "chunk" && item.isEntry);
        if (!entry) return;
        // Vite reports the output directory relative to the environment root, like the user config.
        const outDir = resolve(this.environment.config.root, this.environment.config.build.outDir);
        await writeNativeArtifacts({
          ...production.artifacts,
          outDir,
          bundle: normalizePath(resolve(outDir, entry.fileName)),
        });
      },
      hotUpdate({ file }) {
        const environment = this.environment;
        if (
          (environment instanceof NativeDevEnvironment || environment instanceof QuickJsDevEnvironment) &&
          environment.session?.suppressUpdate(file)
        )
          return [];
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
    },
    options,
  );
}
