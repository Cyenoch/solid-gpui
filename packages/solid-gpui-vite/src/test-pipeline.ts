import { builtinModules } from "node:module";
import { existsSync, readFileSync } from "node:fs";
import { join, relative, resolve } from "node:path";
import { createServer, DevEnvironment, type ResolvedConfig } from "vite";
import type { StdioConfig } from "./environment.ts";
import type { TestLoadRequest, TestLoadResponse, TestPipelineOptions } from "./test-options.ts";

/**
 * The application's Vite pipeline, hosted by the process that started the test
 * run.
 *
 * Vite's resolver imports its own modules, so running it inside a Bun loader
 * hook deadlocks the resolver against the loader it feeds. Worse, Bun's
 * `onResolve` cannot await: it rejects pending promises outright. The pipeline
 * therefore lives here, and the test process asks it only for module *contents*
 * over an authenticated loopback channel - every specifier it is asked about is
 * already a resolved URL written by Vite's own import analysis.
 */
export interface TestPipeline {
  readonly config: ResolvedConfig;
  readonly endpoint: string;
  readonly token: string;
  /**
   * Conditions the test process must run with, so bare specifiers Bun resolves
   * itself (for example `solid-gpui-source`) select the same files Vite did.
   */
  readonly conditions: readonly string[];
  close(): Promise<void>;
}

const FS_PREFIX = "/@fs/";
const ID_PREFIX = "/@id/";
const NULL_BYTE = "\0";
const NULL_BYTE_PLACEHOLDER = "__x00__";
const SOURCE = /\.(?:[cm]?[jt]sx?)$/;
// Solid is processed by Vite rather than left external, so one resolution -
// deduped from the project root - serves every importer: application code, a
// linked SDK checkout that carries its own solid-js, and a packed SDK dist. Left
// external, each importer's own node_modules copy would win and the application
// and the renderer would end up with separate reactive graphs. The SDK itself is
// processed too, so aliases, `#native` bindings and the host contract apply.
const NO_EXTERNAL = [/^@solid-gpui\//, /^@solidjs\//, "solid-js", /^solid-js\//];
const MODULE_CONDITIONS = ["bun", "browser", "module", "development|production"];
// Conditions Vite decides for itself. Anything else a config declares (for
// example `solid-gpui-source`) has to reach both the SSR environment and Bun.
const BUILTIN_CONDITIONS = new Set([
  "module",
  "browser",
  "development",
  "production",
  "import",
  "default",
  "node",
  "require",
  "bun",
  "worker",
  "deno",
  "react-native",
]);

function customConditions(declared: readonly string[] | undefined, known: readonly string[]): string[] {
  return (declared ?? []).filter((condition) => !BUILTIN_CONDITIONS.has(condition) && !known.includes(condition));
}

function splitSpecifier(specifier: string): [string, string] {
  const index = specifier.indexOf("?");
  return index < 0 ? [specifier, ""] : [specifier.slice(0, index), specifier.slice(index)];
}

function normalize(path: string): string {
  return path.split("\\").join("/");
}

/** Turn a Vite id into the URL Vite uses for it in its own module graph. */
function toViteUrl(id: string, rootPrefix: string): string {
  if (id.startsWith(NULL_BYTE)) return ID_PREFIX + id.split(NULL_BYTE).join(NULL_BYTE_PLACEHOLDER);
  const path = normalize(id);
  if (path.startsWith(rootPrefix)) return "/" + relative(rootPrefix, path);
  return FS_PREFIX + path;
}

function attachSourceMap(code: string, map: { readonly mappings: string } | null): string {
  if (!map?.mappings) return code;
  // Bun stops sending a module's imports through loader plugins when it carries a
  // base64 data-URL sourcemap, so the map is inlined percent-encoded instead.
  return `${code}\n//# sourceMappingURL=data:application/json;charset=utf-8,${encodeURIComponent(JSON.stringify(map))}`;
}

export async function startTestPipeline(options: TestPipelineOptions): Promise<TestPipeline> {
  const root = resolve(options.root);
  const configFile = options.configFile ? resolve(root, options.configFile) : undefined;
  const conditions = [...new Set([...MODULE_CONDITIONS, ...options.conditions])];
  const config: StdioConfig = {
    root,
    ...(configFile ? { configFile } : {}),
    // Vite's own default stands when no mode is asked for: a test run must not
    // invent one. The mode only chooses how the config resolves; the runner adds
    // no `NODE_ENV` of its own, because tests keep the environment they were
    // started with.
    ...(options.mode ? { mode: options.mode } : {}),
    logLevel: "warn",
    clearScreen: false,
    appType: "custom",
    // `createServer` also stands up the client hot channel, which binds its own
    // port even in middleware mode and collides when several test runs share a
    // process. Tests need no HMR at all, so the channel is switched off.
    server: { middlewareMode: true, hmr: false, ws: false, watch: null },
    optimizeDeps: { noDiscovery: true, include: [] },
    resolve: { conditions: [...options.conditions], dedupe: ["solid-js"] },
    environments: {
      ssr: {
        // `createServer` listens on every dev environment, and a plain
        // environment transforms modules without starting the application's
        // host session: tests must never build or spawn a host.
        dev: { createEnvironment: (name, resolved) => new DevEnvironment(name, resolved, { hot: false }) },
        resolve: {
          conditions,
          builtins: [...builtinModules, /^node:/, "bun", /^bun:/],
          // Solid must not be externalized: see NO_EXTERNAL above.
          external: [],
          noExternal: NO_EXTERNAL,
        },
      },
    },
    // Tests never start the application's native host, so preparation must not
    // build one while the test server is created.
    __solidGpuiPreparedHost: { command: process.execPath },
    plugins: [
      {
        name: "solid-gpui-vite-test-conditions",
        // Runs after the application's own plugins, so their declared conditions
        // are visible: Vite drops root `resolve.conditions` for server
        // environments, and tests transform modules there.
        enforce: "post",
        config(resolved) {
          const conditions = customConditions(resolved.resolve?.conditions, options.conditions);
          return {
            // The application's SSR environment is configured for its production bundle, where a
            // QuickJS runtime has no ambient environment: `keepProcessEnv: false` makes Vite's
            // define pass rewrite `process.env` (and `process.env.NODE_ENV`) to `{}` and a static
            // value, in dev transforms too. A test runs in real Bun with the real environment -
            // it reads that environment and spawns subprocesses that inherit it - so this server
            // resolves the application's config with the rewrite switched off. Only this test
            // server is affected; the application's own dev and build configs keep their setting.
            environments: { ssr: { keepProcessEnv: true } },
            ...(conditions.length === 0 ? {} : { ssr: { resolve: { conditions } } }),
          };
        },
      },
    ],
  };
  const server = await createServer(config);
  const settings = server.config.configFile ?? "the Vite config";
  const environment = server.environments.ssr;
  if (!environment) {
    await server.close();
    throw new Error(`@solid-gpui/vite/test: ${settings} has no ssr environment to transform tests with`);
  }
  if (!server.config.plugins.some((entry) => entry.name === "solid-gpui")) {
    await server.close();
    throw new Error(
      `@solid-gpui/vite/test: ${settings} does not register the solidGpui plugin, so tests would not use the application's JSX transform, native bindings or host contract`,
    );
  }
  const rootPrefix = normalize(server.config.root).replace(/\/+$/, "") + "/";

  /** Vite URL or absolute path -> the id its own plugins expect. */
  function toId(specifier: string): { id: string; query: string } {
    const [raw, query] = splitSpecifier(specifier);
    if (raw.startsWith(FS_PREFIX)) {
      const path = decodeURIComponent(raw.slice(FS_PREFIX.length));
      return { id: path.startsWith("/") ? path : "/" + path, query };
    }
    if (raw.startsWith(ID_PREFIX))
      return {
        id: decodeURIComponent(raw.slice(ID_PREFIX.length)).split(NULL_BYTE_PLACEHOLDER).join(NULL_BYTE),
        query,
      };
    // Absolute paths are real files (test entries, dependencies); a specifier
    // that only looks absolute is a root-relative URL, which is how Vite writes
    // application modules and assets.
    if (raw.startsWith("/")) return { id: existsSync(raw) ? raw : join(server.config.root, raw), query };
    return { id: raw, query };
  }

  async function loadModule(specifier: string): Promise<TestLoadResponse> {
    const { id, query } = toId(specifier);
    await environment.moduleGraph.ensureEntryFromUrl(toViteUrl(id, rootPrefix) + query);
    const loaded = await environment.pluginContainer.load(id + query);
    let source: string;
    let map: { readonly mappings: string } | null = null;
    if (loaded == null) {
      // Vite's dev pipeline reads plain source files itself; only plugins that
      // own a specifier (assets, JSON, CSS, virtual modules) answer `load`.
      if (query || !SOURCE.test(id) || !existsSync(id)) return {};
      source = readFileSync(id, "utf8");
    } else {
      source = typeof loaded === "string" ? loaded : loaded.code;
      map = typeof loaded === "object" && loaded.map && typeof loaded.map === "object" ? loaded.map : null;
    }
    const result = await environment.pluginContainer.transform(source, id + query, { ...(map ? { inMap: map } : {}) });
    return { code: attachSourceMap(result.code, result.map) };
  }

  const token = crypto.randomUUID();
  const endpoint = Bun.serve({
    hostname: "127.0.0.1",
    port: 0,
    async fetch(request) {
      const url = new URL(request.url);
      if (request.headers.get("authorization") !== `Bearer ${token}`) return new Response(null, { status: 403 });
      if (url.pathname === "/health") return Response.json({ root: server.config.root });
      try {
        if (url.pathname === "/load") {
          const body = (await request.json()) as TestLoadRequest;
          return Response.json(await loadModule(body.specifier));
        }
        return new Response(null, { status: 404 });
      } catch (error) {
        return Response.json(
          {
            message: error instanceof Error ? error.message : String(error),
            stack: error instanceof Error ? error.stack : undefined,
          },
          { status: 500 },
        );
      }
    },
  });
  // Bare specifiers inside natively loaded files (dependencies, SDK sources
  // reached by Bun) are resolved by Bun itself, so it needs the same custom
  // conditions Vite applied - otherwise one graph mixes sources and dist.
  const childConditions = [
    ...options.conditions,
    ...customConditions(server.config.resolve.conditions, options.conditions),
  ];
  return {
    config: server.config,
    endpoint: `http://127.0.0.1:${endpoint.port}`,
    token,
    conditions: childConditions,
    close: async () => {
      await endpoint.stop(true);
      await server.close();
    },
  };
}
