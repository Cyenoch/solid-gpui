// Worker entry for fixtures/embedded-static-graph.tsx.
//
// It is built as an entry of its own and embedded by the serializer through
// `--workers`, so the packaged graph carries it at the graph root as
// `embedded-static-graph.worker.js`, which is the path the application loads.
// Nothing resolves this file at run time from disk.
//
// Bun's Worker runtime starts it with `workerData`, one message roundtrip, and
// its own shutdown; the Node-compatible `node:worker_threads` surface is used on
// both sides so the protocol is explicit rather than a browser Worker global.

import { parentPort, workerData } from "node:worker_threads";
import { checkNativeExtraction } from "./embedded-static-boundaries";

const seed: unknown = workerData?.seed;
if (typeof seed !== "number") throw new Error("embedded-static-graph.worker requires a numeric workerData.seed");
if (typeof workerData.resourcePath !== "string") throw new Error("Missing packaged resource path");
checkNativeExtraction(workerData.resourcePath);

const port = parentPort;
if (!port) throw new Error("embedded-static-graph.worker requires a parent port");

// Bun ends a Worker's event loop once nothing keeps it alive, so the Worker holds
// itself open until it has answered; otherwise the incremental message the
// application sends could arrive after the Worker is gone.
const keepAlive = setInterval(() => {}, 1_000);

port.on("message", (increment: unknown) => {
  if (typeof increment !== "number") throw new Error("embedded-static-graph.worker requires a numeric increment");
  clearInterval(keepAlive);
  port.postMessage(seed + increment);
});
