import { test } from "bun:test";
import { createRunnableDevEnvironment, createServer, isRunnableDevEnvironment } from "vite";
import { resolve } from "node:path";

test("documented previews render without DOM globals and component navigation retains the sidebar", async () => {
  const root = resolve(import.meta.dirname, "..");
  const server = await createServer({
    root,
    configFile: resolve(root, "vite.native.config.ts"),
    server: { middlewareMode: true, hmr: false, ws: false },
    environments: { ssr: { dev: { createEnvironment: createRunnableDevEnvironment } } },
    logLevel: "error",
  });
  try {
    const environment = server.environments.ssr;
    if (!environment || !isRunnableDevEnvironment(environment)) throw new Error("Missing native Vite environment");
    const fixture = await environment.runner.import(resolve(root, "tests/runtime.fixture.tsx"));
    await fixture.verifyRuntime();
  } finally {
    await server.close();
  }
}, 30000);
