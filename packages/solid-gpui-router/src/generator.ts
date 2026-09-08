import { createGenerationSession } from "./generation-session";
import type { FileRouterOptions } from "./generator-engine";

export type { FileRouterOptions } from "./generator-engine";

/** Generate the route tree before type checking or using a non-Vite bundler. */
export async function generateRoutes(options: FileRouterOptions = {}): Promise<void> {
  const session = createGenerationSession(options);
  try {
    await session.run();
  } finally {
    session.close();
  }
}
