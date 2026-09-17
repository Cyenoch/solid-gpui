import { test } from "bun:test";
import { createRunnableDevEnvironment, createServer, isRunnableDevEnvironment } from "vite";
import { resolve } from "node:path";

test("documented previews render without DOM globals and component navigation retains the sidebar", async () => {
  const root = resolve(import.meta.dirname, "..");
  const server = await createServer({
    root,
    // MemoryTransport exercises shared rendering; native-ci verifies the actual host contracts.
    configFile: resolve(root, "vite.config.ts"),
    ssr: {
      // Share Bun's browser Solid instance with the surrounding test files.
      noExternal: [/^@solid-gpui\//],
      external: ["solid-js", "solid-js/universal", "solid-js/store"],
      resolve: {
        conditions: ["solid-gpui-source", "browser", "bun"],
        externalConditions: ["browser", "bun"],
      },
    },
    server: { middlewareMode: true, hmr: false, ws: false },
    environments: { ssr: { dev: { createEnvironment: createRunnableDevEnvironment } } },
    logLevel: "error",
  });
  try {
    const environment = server.environments.ssr;
    if (!environment || !isRunnableDevEnvironment(environment)) throw new Error("Missing runnable Vite environment");
    const fixture = await environment.runner.import(resolve(root, "tests/runtime.fixture.tsx"));
    await fixture.verifyRuntime();
  } finally {
    await server.close();
  }
}, 30000);
