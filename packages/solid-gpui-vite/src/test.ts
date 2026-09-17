import { spawn } from "node:child_process";
import { existsSync } from "node:fs";
import { extname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { startTestPipeline } from "./test-pipeline.ts";
import { collectConditions, TEST_ENV, type RunTestsOptions } from "./test-options.ts";

export type { RunTestsOptions } from "./test-options.ts";

const source = fileURLToPath(import.meta.url);
const preload = fileURLToPath(new URL(`./test-preload${extname(source)}`, import.meta.url));

/**
 * Run the application's `bun:test` suite with the project's Vite pipeline.
 *
 * Tests execute in a real Bun test process, so `bun:test` hooks, mocks and
 * assertions behave normally, and Bun owns module execution. Resolution and
 * transformation come from the application's own Vite config: the universal
 * Solid JSX transform, aliases such as `#native`, inline/raw assets and the
 * project's native host contract all apply, and Solid's client build is forced
 * so signals drive the native renderer instead of resolving to Solid's server
 * build. Vite resolves Solid through the project root for every importer, so
 * the application, linked SDK sources and renderer share one reactive graph.
 *
 * Tests keep the environment of the calling process. A QuickJS application's
 * config inlines `process.env` for its production bundle
 * (`environments.ssr.keepProcessEnv: false`), which in a test run would hide
 * `PATH` and every variable a test reads or spawns with, so the test server
 * resolves the application's config with that inlining switched off. Nothing but
 * this server is affected: the application's own dev and build configs keep it.
 *
 * @returns Bun's exit code: 0 when the suite passed.
 */
export async function runTests(options: RunTestsOptions = {}): Promise<number> {
  if (typeof Bun === "undefined")
    throw new Error("@solid-gpui/vite/test requires Bun; run it with `bun run solid-gpui test` or `bun --bun ...`");
  const root = resolve(options.root ?? process.cwd());
  if (!existsSync(preload))
    throw new Error(`@solid-gpui/vite: the test preload is missing at ${preload}; reinstall or rebuild the package`);
  const configFile = options.configFile && resolve(root, options.configFile);
  const conditions = collectConditions(process.execArgv);
  const pipeline = await startTestPipeline({
    root,
    configFile,
    ...(options.mode ? { mode: options.mode } : {}),
    conditions,
  });
  const child = spawn(
    process.execPath,
    [
      "test",
      "--preload",
      preload,
      ...pipeline.conditions.map((condition) => `--conditions=${condition}`),
      ...(options.args ?? []),
    ],
    {
      cwd: root,
      stdio: "inherit",
      env: {
        ...process.env,
        // Plugin-provided modules carry absolute paths from the project being
        // tested, and Bun's runtime transpiler cache survives across runs. A
        // second project with identical sources would otherwise replay the first
        // project's transformed output and import files that no longer exist.
        BUN_RUNTIME_TRANSPILER_CACHE_PATH: "0",
        [TEST_ENV.root]: root,
        [TEST_ENV.endpoint]: pipeline.endpoint,
        [TEST_ENV.token]: pipeline.token,
      },
    },
  );
  // The runner owns the child: an interrupted test run must not leave Vite or
  // the pipeline server behind.
  const interrupt = (signal: NodeJS.Signals) => () => child.kill(signal);
  const forward = { SIGINT: interrupt("SIGINT"), SIGTERM: interrupt("SIGTERM") };
  for (const [signal, handler] of Object.entries(forward)) process.once(signal as NodeJS.Signals, handler);
  try {
    return await new Promise<number>((settle, fail) => {
      child.once("error", fail);
      child.once("exit", (code, signal) => settle(code ?? (signal ? 1 : 0)));
    });
  } finally {
    for (const [signal, handler] of Object.entries(forward)) process.off(signal as NodeJS.Signals, handler);
    await pipeline.close();
  }
}
