import { resolve } from "node:path";
import type { Plugin } from "vite";
import { createGenerationSession } from "./generation-session";
import type { FileRouterOptions } from "./generator";

export type { FileRouterOptions } from "./generator";

/** Generate typed native routes before universal JSX compilation. */
export function solidGpuiRouter(options: FileRouterOptions = {}): Plugin {
  let session: ReturnType<typeof createGenerationSession> | undefined;
  let routesDirectory: string;
  return {
    name: "solid-gpui-router",
    enforce: "pre",
    async configResolved(config) {
      session = createGenerationSession({
        ...options,
        root: options.root ? resolve(config.root, options.root) : config.root,
      });
      try {
        routesDirectory = await session.run();
      } catch (error) {
        session.close();
        throw error;
      }
    },
    async buildStart() {
      this.addWatchFile(routesDirectory);
      await session!.run();
    },
    async watchChange(path, event) {
      await session!.run({ path, type: event.event });
    },
    closeBundle() {
      session?.close();
    },
  };
}
