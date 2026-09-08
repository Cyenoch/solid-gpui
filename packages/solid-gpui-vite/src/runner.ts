import { createWebSocketModuleRunnerTransport, ModuleRunner } from "vite/module-runner";
import { Console } from "node:console";

/** Runtime stdout is exclusively the native protocol, including during HMR. */
export function useProtocolStdio(): void {
  Object.assign(console, new Console({ stdout: process.stderr, stderr: process.stderr }));
}

if (import.meta.main) {
  useProtocolStdio();
  const [endpoint, entry] = process.argv.slice(2);
  if (!endpoint || !entry) throw new Error("The Vite module runner requires an endpoint and entry");
  const transport = createWebSocketModuleRunnerTransport({ createConnection: () => new WebSocket(endpoint) });
  const runner = new ModuleRunner({
    transport: {
      ...transport,
      connect: (handlers) =>
        transport.connect({
          ...handlers,
          onDisconnection: () => {
            handlers.onDisconnection();
            process.exit(0);
          },
        }),
    },
    hmr: { logger: { debug: (...args) => console.error(...args), error: (error) => console.error(error) } },
  });
  // Native host termination must also stop a runner with no mounted application.
  process.stdin.on("end", () => {
    void runner.close().finally(() => process.exit(0));
  });
  try {
    await runner.import(entry);
    await transport.send({ type: "custom", event: "solid-gpui:ready", data: null });
  } catch (error) {
    console.error(error);
    await transport.send({ type: "custom", event: "solid-gpui:error", data: String(error) });
    await runner.close();
    process.exitCode = 1;
  }
}
