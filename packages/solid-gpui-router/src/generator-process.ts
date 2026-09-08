import { createRouteGenerator } from "./generator-engine";
import type { GeneratorEvent } from "@tanstack/router-generator";

const generator = createRouteGenerator(JSON.parse(process.argv[2]!));
let requests = Promise.resolve();
process.on("message", (message: { id: number; event?: GeneratorEvent }) => {
  // A no-op upstream run can finish before draining events queued during its
  // cache check. Complete each request before giving it the next file event.
  requests = requests.then(async () => {
    try {
      await generator.run(message.event);
      process.send?.({ id: message.id, routesDirectory: generator.config.routesDirectory });
    } catch (error) {
      process.send?.({ id: message.id, error: error instanceof Error ? error.stack : String(error) });
    }
  });
});
process.on("disconnect", () => process.exit(0));
