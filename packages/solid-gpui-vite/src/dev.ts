import { createLogger, createServer, type ViteDevServer } from "vite";
import { resolve } from "node:path";
import type { StdioConfig } from "./environment.ts";
import { useProtocolStdio } from "./runner.ts";

export interface DevOptions {
  readonly root?: string;
  readonly configFile?: string;
  /** Export bindings from the running Rust host instead of rebuilding it. */
  readonly nativeHost?: string;
}

/** Run a Vite project inside a Rust-owned Bun process with protocol stdio. */
export async function startDev(options: DevOptions = {}): Promise<ViteDevServer> {
  if (typeof Bun === "undefined") throw new Error("Native Vite development requires Bun");
  useProtocolStdio();
  const config: StdioConfig = {
    root: options.root && resolve(options.root),
    configFile: options.configFile,
    __solidGpuiStdio: { nativeHost: options.nativeHost },
    customLogger: createLogger("info", { console }),
    clearScreen: false,
  };
  const server = await createServer({ ...config, server: { middlewareMode: true } });
  try {
    if (!server.config.plugins.some((plugin) => plugin.name === "solid-gpui") || !server.config.build.ssr) {
      throw new Error("Native development requires solidGpui({ entry }) in the Vite config");
    }
    return server;
  } catch (error) {
    await server.close();
    throw error;
  }
}
