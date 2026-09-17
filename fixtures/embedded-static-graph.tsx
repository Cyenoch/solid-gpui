// Packaged Embedded Bun qualification fixture.
//
// Vite compiles Solid JSX and emits an import()-reachable chunk. The pinned Bun
// serializer can fold that chunk into the entry source; it need not remain a
// separate final graph module. Explicit --workers and --assets entries supply
// the Worker and resource. All three results reach the first Snapshot, and each
// asynchronous step is bounded before the counter input/restart probe begins.

import { mountApplication, Pressable, Text, View } from "@solid-gpui/core";
import { EmbeddedTransport } from "@solid-gpui/core/embedded";
import { createSignal } from "@solid-gpui/core/runtime";
import { Worker } from "node:worker_threads";
import { checkChildInterpreter, checkNativeExtraction } from "./embedded-static-boundaries";

/**
 * The serializer includes this module through `--workers`. Resolve its path
 * against this module's virtual directory, not the process working directory.
 * A string avoids Vite's separately deployed browser Worker bundle.
 */
const WORKER_ENTRY = `${import.meta.dirname}/embedded-static-graph.worker.js`;

/** Basename of the file the serializer embeds through `--assets`. */
const RESOURCE_NAME = "embedded-static-graph-resource.txt";
const resourcePath = `${import.meta.dirname}/${RESOURCE_NAME}`;
/** Matched instead of one exact name in case the asset keeps a content hash. */
const RESOURCE_STEM = "embedded-static-graph-resource";

/** Seed carried to the Worker as `workerData`; the Worker answers `seed + 1`. */
const WORKER_SEED = 41;
/** Increment the application sends once; the Worker's answer is `WORKER_SEED + 1`. */
const WORKER_INCREMENT = 1;

/**
 * One deadline for every packaged step: a stuck step must fail rather than hang.
 * Eight seconds leaves margin over the measured 3.86-second Windows x64 Worker
 * roundtrip under ARM64 emulation. The host still bounds initial commits at 30s.
 */
const STEP_DEADLINE_MS = 8_000;

/** Rejects with the step name when the work settles late, fails, or never settles. */
function deadline<T>(step: string, work: Promise<T>): Promise<T> {
  return new Promise<T>((resolve, reject) => {
    const timer = setTimeout(
      () => reject(new Error(`${step} did not finish within ${STEP_DEADLINE_MS}ms`)),
      STEP_DEADLINE_MS,
    );
    work.then(
      (value) => {
        clearTimeout(timer);
        resolve(value);
      },
      (error: unknown) => {
        clearTimeout(timer);
        reject(
          new Error(`${step} failed: ${error instanceof Error ? error.message : String(error)}`, { cause: error }),
        );
      },
    );
  });
}

/**
 * One Worker roundtrip: the Worker receives its seed through `workerData`,
 * receives one increment as a message, and answers with the sum. The Worker is
 * terminated before the value is returned, so mounting starts no thread that the
 * session would still have to wait for.
 */
async function workerRoundtrip(): Promise<number> {
  const worker = new Worker(WORKER_ENTRY, { workerData: { seed: WORKER_SEED, resourcePath } });
  try {
    return await deadline(
      "Worker roundtrip",
      new Promise<number>((resolve, reject) => {
        worker.on("message", (value: number) => resolve(value));
        // A Worker that fails to start, throws, or exits early must reject here
        // instead of leaving the roundtrip pending until the deadline.
        worker.on("error", (error: Error) => reject(error));
        worker.on("exit", (code: number) => reject(new Error(`Worker exited with code ${code} before answering`)));
        worker.postMessage(WORKER_INCREMENT);
      }),
    );
  } finally {
    // Termination is awaited: the packaged session owns one runtime and must be
    // able to shut down without a Worker that is still running.
    await worker.terminate();
  }
}

/** Reads the embedded file the serializer attached through `--assets`. */
async function readEmbeddedResource(): Promise<string> {
  const embedded = Bun.embeddedFiles;
  const matches = embedded.filter(
    (file) =>
      "name" in file &&
      typeof file.name === "string" &&
      file.name.startsWith(RESOURCE_STEM) &&
      file.name.endsWith(".txt"),
  );
  if (matches.length !== 1) {
    const names = embedded.map((file) => ("name" in file ? String(file.name) : "<unnamed>")).join(", ");
    throw new Error(
      `expected exactly one embedded ${RESOURCE_NAME}, found ${matches.length} ` +
        `(embedded files: ${names.length > 0 ? names : "none"})`,
    );
  }
  const contents = await matches[0].text();
  return contents.trim();
}

// A rejected step rejects the entry module, which the embedded runtime reports and
// ends as a failed session: the packaged application never mounts without them.
const dynamicModule = await deadline("dynamic import", import("./embedded-static-graph-dynamic"));
checkNativeExtraction(resourcePath);
checkChildInterpreter();
const workerValue = await workerRoundtrip();
const resourceValue = await deadline("embedded resource", readEmbeddedResource());

mountApplication<number>({
  transport: () => new EmbeddedTransport(),
  setup(previous = 0) {
    const [count, setCount] = createSignal(previous);
    return {
      render: () => (
        // Explicit foreground and background: the default host theme is light, so
        // every text node states its own color instead of relying on it.
        <View style={{ padding: 24, gap: 12, backgroundColor: "#111827" }}>
          <Text style={{ color: "#E5E7EB", fontSize: 16 }}>{`Dynamic: ${dynamicModule.dynamicMarker}`}</Text>
          <Text style={{ color: "#E5E7EB", fontSize: 16 }}>{`Worker: ${workerValue}`}</Text>
          <Text style={{ color: "#E5E7EB", fontSize: 16 }}>{`Resource: ${resourceValue}`}</Text>
          <Text style={{ color: "#E5E7EB", fontSize: 16 }}>Native extraction: blocked</Text>
          <Text style={{ color: "#E5E7EB", fontSize: 16 }}>Child interpreter: guarded</Text>
          <Text style={{ color: "#F9FAFB", fontSize: 28, fontWeight: "semibold" }}>{`Count: ${count()}`}</Text>
          <Pressable
            style={{ padding: 12, backgroundColor: "#2563EB", borderRadius: 8 }}
            accessibilityRole="button"
            onPress={() => setCount((value: number) => value + 1)}
          >
            <Text style={{ color: "#FFFFFF", fontWeight: "medium" }}>Increment</Text>
          </Pressable>
        </View>
      ),
      captureState: count,
    };
  },
});
