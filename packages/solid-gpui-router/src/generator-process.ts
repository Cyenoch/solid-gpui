import { createRouteGenerator } from "./generator-engine";
import type { GeneratorEvent } from "@tanstack/router-generator";

const generator = createRouteGenerator(JSON.parse(process.argv[2]!));
process.on("message", async (message: { id: number; event?: GeneratorEvent }) => {
  try {
    await generator.run(message.event);
    process.send?.({ id: message.id, routesDirectory: generator.config.routesDirectory });
  } catch (error) {
    process.send?.({ id: message.id, error: error instanceof Error ? error.stack : String(error) });
  }
});
process.on("disconnect", () => process.exit(0));
