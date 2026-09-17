import { plugin } from "bun";
import { createHash } from "node:crypto";
import { existsSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { TEST_ENV, type TestLoadResponse } from "./test-options.ts";

/**
 * Test-process half of `@solid-gpui/vite/test`.
 *
 * Bun runs the tests and owns module execution, mocks and the Solid instance.
 * Resolution and transformation belong to the application's Vite config, which
 * lives in the runner process (test-pipeline.ts) because Vite's resolver cannot
 * run inside a Bun loader hook.
 *
 * Bun's `onResolve` cannot await, so it only decodes what Vite's own import
 * analysis already wrote - `/src/page.tsx`, `/@fs/...?inline`, `/@id/...` - into
 * real paths and claims the test files Bun was asked to run. Everything else
 * (bare packages, imports inside dependencies) is left to Bun, which is also
 * what keeps one Solid instance in the graph. Modules keep their real path and
 * no namespace, so their own imports - including bare packages - resolve from
 * where the file actually lives. `onLoad` may await, and that is where the Vite
 * pipeline answers.
 */
const root = process.env[TEST_ENV.root] ?? "";
const endpoint = process.env[TEST_ENV.endpoint] ?? "";
const token = process.env[TEST_ENV.token] ?? "";
if (!root || !endpoint || !token)
  throw new Error("@solid-gpui/vite/test: the test loader must be started by runTests() or `solid-gpui test`");

const PROTOCOL = /^[a-z][a-z0-9+.-]*:/i;
const JSX = /\.(?:tsx|jsx)$/;
const VITE_INTERNAL = "/@vite/";
const FS_PREFIX = "/@fs/";
const ID_PREFIX = "/@id/";
// Virtual modules have no file of their own, so they are given a path inside the
// project: bare imports in their code then resolve from the application's
// dependencies instead of from nowhere.
const VIRTUAL_DIRECTORY = join(root, "node_modules", ".solid-gpui-vite-test");

const virtual = new Map<string, string>();

function splitSpecifier(specifier: string): [string, string] {
  const index = specifier.indexOf("?");
  return index < 0 ? [specifier, ""] : [specifier.slice(0, index), specifier.slice(index)];
}

/** The real path of a Vite-written specifier, or undefined when Bun owns it. */
function bunPath(specifier: string, importer: string): string | undefined {
  if (specifier.length === 0 || PROTOCOL.test(specifier) || specifier.startsWith(VITE_INTERNAL)) return undefined;
  const [path, query] = splitSpecifier(specifier);
  if (path.startsWith(ID_PREFIX)) {
    const id = path.slice(ID_PREFIX.length);
    const key = join(VIRTUAL_DIRECTORY, createHash("sha1").update(id).digest("hex").slice(0, 16) + ".js");
    virtual.set(key, specifier);
    return key;
  }
  if (path.startsWith(FS_PREFIX)) {
    const fs = path.slice(FS_PREFIX.length);
    return (fs.startsWith("/") ? fs : "/" + fs) + query;
  }
  if (path.startsWith("/")) {
    const rooted = join(root, path);
    if (existsSync(rooted)) return rooted + query;
    // Absolute paths (test entries, dependencies already written as files) are real.
    if (existsSync(path)) return specifier;
    return undefined;
  }
  // CLI entries arrive with no importer; relative imports matter when a
  // dependency ships JSX sources that still need the application's transform.
  if (importer.length === 0) return specifier;
  if (path.startsWith(".") || JSX.test(path)) {
    const resolved = resolve(dirname(importer), path);
    if (existsSync(resolved)) return resolved + query;
  }
  return undefined;
}

async function call(route: string, body: unknown): Promise<TestLoadResponse> {
  const response = await fetch(`${endpoint}/${route}`, {
    method: "POST",
    headers: { authorization: `Bearer ${token}`, "content-type": "application/json" },
    body: JSON.stringify(body),
  });
  if (!response.ok) {
    const raw = await response.text();
    let message: string | undefined;
    try {
      const parsed: unknown = JSON.parse(raw);
      if (parsed !== null && typeof parsed === "object" && "message" in parsed && typeof parsed.message === "string")
        message = parsed.message;
    } catch {
      message = raw.slice(0, 400);
    }
    throw new Error(
      message || `@solid-gpui/vite/test: the Vite pipeline rejected ${route} with status ${response.status}`,
    );
  }
  return (await response.json()) as TestLoadResponse;
}

// Fail before any test file loads rather than on the first import.
const health = await fetch(`${endpoint}/health`, { headers: { authorization: `Bearer ${token}` } });
if (!health.ok) throw new Error(`@solid-gpui/vite/test: the Vite pipeline is unreachable (status ${health.status})`);

plugin({
  name: "solid-gpui-vite-test",
  setup(build) {
    build.onResolve({ filter: /.*/ }, (args) => {
      let path: string | undefined;
      let failure: string | undefined;
      try {
        path = bunPath(args.path, args.importer);
      } catch (error) {
        failure = error instanceof Error ? error.stack : String(error);
      }
      if (failure) throw new Error(failure);
      return path === undefined ? undefined : { path };
    });
    // Only paths with an extension are claimed, so Bun keeps loading its own
    // modules (and any module this loader passed through) itself.
    build.onLoad({ filter: /\.[a-z0-9]+(?:\?.*)?$/i }, async (args) => {
      const specifier = virtual.get(args.path) ?? args.path;
      const loaded = await call("load", { specifier });
      if (loaded.code !== undefined) return { contents: loaded.code, loader: "js" };
      // Not Vite's to transform: Bun reads the file with its own loader.
      if (existsSync(args.path)) return undefined;
      throw new Error(
        `@solid-gpui/vite/test: the project's Vite config produced no module for ${args.path}; check that the file exists and that its imports are valid`,
      );
    });
  },
});
