import { fork } from "node:child_process";
import type { GeneratorEvent } from "@tanstack/router-generator";
import type { FileRouterOptions } from "./generator-engine";

/** Keep Node build dependencies outside the native runtime's browser conditions. */
export function createGenerationSession(options: FileRouterOptions) {
  const entry = new URL(
    import.meta.url.endsWith(".ts") ? "./generator-process.ts" : "./generator-process.js",
    import.meta.url,
  );
  const child = fork(entry, [JSON.stringify(options)], { execArgv: [], stdio: ["ignore", "pipe", "pipe", "ipc"] });
  child.stdout?.pipe(process.stderr, { end: false });
  child.stderr?.pipe(process.stderr, { end: false });
  let nextId = 0;
  let failure: Error | undefined;
  const pending = new Map<number, { resolve: (directory: string) => void; reject: (error: Error) => void }>();
  const fail = (error: Error) => {
    failure = error;
    for (const request of pending.values()) request.reject(error);
    pending.clear();
  };
  child.on("error", fail);
  child.on("exit", (code, signal) => fail(new Error(`Route generator exited (${signal ?? code})`)));
  child.on("message", (message: { id: number; error?: string; routesDirectory: string }) => {
    const request = pending.get(message.id);
    if (!request) return;
    pending.delete(message.id);
    if (message.error) request.reject(new Error(message.error));
    else request.resolve(message.routesDirectory);
  });
  return {
    run(event?: GeneratorEvent): Promise<string> {
      if (failure) return Promise.reject(failure);
      return new Promise((resolve, reject) => {
        const id = ++nextId;
        pending.set(id, { resolve, reject });
        child.send({ id, event }, (error) => {
          if (error) fail(error);
        });
      });
    },
    close() {
      fail(new Error("Route generator session closed"));
      if (child.connected) child.disconnect();
      child.kill();
    },
  };
}
