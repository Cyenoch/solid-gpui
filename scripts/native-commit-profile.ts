// Run with Bun's browser condition so this exercises Solid's client runtime.
import { strict as assert } from "node:assert";
import { createComponent } from "../packages/solid-gpui/src/runtime";
import { mountApplication, MemoryTransport } from "../packages/solid-gpui/src/index";
import { StdioTransport } from "../packages/solid-gpui/src/stdio";
import { createRegionWorkload } from "./native-region-workload";
import type { DisposableTransport } from "../packages/solid-gpui/src/transport";
import { Envelope } from "../packages/solid-gpui/src/protocol/generated/protocol";

const native = process.argv.includes("--native");
const hold = process.argv.includes("--hold");
const regions = process.argv.includes("--regions");
const resizeEachUpdate = process.argv.includes("--resize-each-update");
const percentile = (values: number[], p: number) => {
  const sorted = [...values].sort((a, b) => a - b);
  return sorted[Math.floor((sorted.length - 1) * p)]!;
};

for (const rows of native ? [500] : [500, 2_500, 10_000]) {
  const connection = native ? new StdioTransport() : new MemoryTransport();
  let pending: number | undefined;
  let frameBytes = 0;
  let operations = 0;
  let viewport: { width: number; height: number; scaleFactor?: number } | undefined;
  const toFrame: number[] = [];
  const reaction: number[] = [];
  const workload = createRegionWorkload(rows, regions);
  const { count, setCount } = workload;
  const transport: DisposableTransport = {
    submit(frame) {
      if (pending !== undefined) {
        toFrame.push(performance.now() - pending);
        pending = undefined;
        frameBytes += frame.byteLength;
        // Inspect after the timing boundary; never count unchanged output as a win.
        const body = Envelope.decode(frame.subarray(4)).body;
        assert(body?.tag === 3, "Expected an incremental patch");
        assert.equal(body.value.operations?.length, 1);
        const operation = body.value.operations![0]!.operation;
        assert(operation?.tag === 2, "Expected a text update");
        assert.equal(operation.value.text, `Count: ${count()}`);
        operations++;
      }
      return connection.submit(frame);
    },
    onData: (listener) => connection.onData(listener),
    onDrain: (listener) => connection.onDrain(listener),
    onTermination: (listener) => connection.onTermination(listener),
    dispose: () => connection.dispose(),
  };
  const application = mountApplication({
    transport: () => transport,
    setup: () => ({
      render: () => createComponent(workload.render, {}),
      rootOptions: {
        onWindowResize: (width, height, scaleFactor) => {
          viewport = { width, height, scaleFactor };
        },
      },
    }),
  });
  const root = application.root;
  assert(root);
  if (native) {
    await root.setTitle("Solid GPUI commit measurement");
    await root.resize(800, 600);
    await root.activateWindow();
  }
  const iterations = native ? 360 : 256;
  for (let index = 0; index < iterations; index++) {
    if (native) await new Promise((resolve) => setTimeout(resolve, 33));
    pending = performance.now();
    setCount((value) => value + 1);
    reaction.push(performance.now() - pending);
    await Promise.resolve();
    assert.equal(pending, undefined, "The scheduled update must produce bytes");
    if (native && resizeEachUpdate) await root.resize(800 + (index % 2), 600);
    if (native && [90, 180, 270].includes(index)) {
      const width = index === 90 ? 560 : index === 180 ? 1280 : 800;
      await root.resize(width, 600);
      console.error(JSON.stringify({ phase: "resize", width }));
    }
  }
  if (native) {
    if (resizeEachUpdate) await root.resize(800, 600);
    const [width, windowHeight] = await root.getWindowSize();
    // The query includes window chrome; resize feedback reports content height.
    assert(viewport, "Native resize feedback was not delivered");
    assert.equal(width, 800, "The platform changed the benchmark viewport; discard this run");
    assert.equal(viewport?.height, 600, "The platform changed the benchmark viewport; discard this run");
    assert(windowHeight >= viewport.height, "Native window height is smaller than its content viewport");
    viewport = { ...viewport, width };
  }
  assert.equal(workload.appRuns(), 1);
  assert.equal(operations, iterations);
  const samples = toFrame.slice(16);
  console.error(
    JSON.stringify({
      mode: native ? "native-stdio" : "memory",
      rows,
      regions,
      resizeEachUpdate,
      viewport,
      updates: iterations,
      appRuns: workload.appRuns(),
      operations,
      frameBytes,
      signal_to_frame_p50_ms: percentile(samples, 0.5),
      signal_to_frame_p95_ms: percentile(samples, 0.95),
      synchronous_reaction_p95_ms: percentile(reaction.slice(16), 0.95),
    }),
  );
  if (!hold) {
    if (native) application.quit();
    else application.dispose();
  }
}
