#!/usr/bin/env -S bun --conditions=browser
import { createServer, isRunnableDevEnvironment, type Logger } from "vite";

// stdout belongs exclusively to the framed native protocol, including HMR logs.
const logger: Logger = {
  info: (message) => console.error(message),
  warn: (message) => console.error(message),
  warnOnce: (message) => console.error(message),
  error: (message) => console.error(message),
  clearScreen() {},
  hasErrorLogged: () => false,
  hasWarned: false,
};
export async function startDev(entry: string, configFile?: string): Promise<() => Promise<void>> {
  const server = await createServer({
    configFile,
    customLogger: logger,
    server: { middlewareMode: true, watch: { ignored: ["**/target/**", "**/dist*/**"] } },
  });
  const environment = server.environments.ssr;
  if (!environment || !isRunnableDevEnvironment(environment)) {
    await server.close();
    throw new Error("solid-gpui requires Vite's runnable SSR environment under Bun");
  }
  try {
    await environment.runner.import(entry);
  } catch (error) {
    await server.close();
    throw error;
  }
  return () => server.close();
}
if (import.meta.main) {
  const entry = process.argv[2];
  if (!entry) throw new Error("Usage: bun --conditions=browser @solid-gpui/core/vite/dev <entry> [config]");
  const close = await startDev(entry, process.argv[3]);
  for (const signal of ["SIGINT", "SIGTERM"] as const) {
    process.once(signal, () => {
      void close().finally(() => process.exit(0));
    });
  }
}
