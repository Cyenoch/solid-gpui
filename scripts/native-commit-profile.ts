// Run with Bun's browser condition so this exercises Solid's client runtime.
import { strict as assert } from "node:assert";
import { createComponent, createSignal } from "../packages/solid-gpui/src/runtime";
import { mountApplication, MemoryTransport, Pressable, Text, TextInput, View } from "../packages/solid-gpui/src/index";
import { StdioTransport } from "../packages/solid-gpui/src/stdio";
import type { DisposableTransport } from "../packages/solid-gpui/src/transport";
import { Envelope } from "../packages/solid-gpui/src/protocol/generated/protocol";

const native = process.argv.includes("--native");
const hold = process.argv.includes("--hold");
const percentile = (values: number[], p: number) => {
  const sorted = [...values].sort((a, b) => a - b);
  return sorted[Math.floor((sorted.length - 1) * p)]!;
};

for (const rows of native ? [500] : [500, 2_500, 10_000]) {
  const connection = native ? new StdioTransport() : new MemoryTransport();
  let pending: number | undefined;
  let frameBytes = 0;
  let operations = 0;
  let appRuns = 0;
  let viewport: { width: number; height: number; scaleFactor?: number } | undefined;
  const toFrame: number[] = [];
  const reaction: number[] = [];
  const [count, setCount] = createSignal(0);
  const [text, setText] = createSignal("Edit here to verify native input");
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
  function App() {
    appRuns++;
    return createComponent(View, {
      style: { flexGrow: 1, flexDirection: "column", padding: 20, gap: 12, backgroundColor: "#18202c" },
      children: [
        createComponent(Text, { style: { color: "#ffffff", fontSize: 24 }, children: () => `Count: ${count()}` }),
        createComponent(Pressable, {
          style: { height: 36, backgroundColor: "#334866", padding: 8 },
          onPress: () => setCount((value) => value + 1),
          children: createComponent(Text, { style: { color: "#ffffff" }, children: "Increment" }),
        }),
        createComponent(TextInput, {
          style: { height: 36, padding: 6, backgroundColor: "#25344a", color: "#ffffff" },
          get value() {
            return text();
          },
          onChangeText: setText,
        }),
        createComponent(View, {
          style: { height: 0, flexGrow: 1, minHeight: 0, overflow: "scroll" },
          children: Array.from({ length: rows }, (_, index) =>
            createComponent(Text, {
              style: { color: "#d8e4f2", fontSize: 14, lineHeight: 24 },
              children: `Unchanged row ${index + 1}`,
            }),
          ),
        }),
      ],
    });
  }
  const application = mountApplication({
    transport: () => transport,
    setup: () => ({
      render: () => createComponent(App, {}),
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
    if (native && [90, 180, 270].includes(index)) {
      const width = index === 90 ? 560 : index === 180 ? 1280 : 800;
      await root.resize(width, 600);
      console.error(JSON.stringify({ phase: "resize", width }));
    }
  }
  if (native) {
    await root.getWindowSize();
    // Window bounds include platform chrome; compare the content viewport from resize events.
    assert.equal(viewport?.width, 800, "The platform changed the benchmark viewport; discard this run");
    assert.equal(viewport?.height, 600, "The platform changed the benchmark viewport; discard this run");
  }
  assert.equal(appRuns, 1);
  assert.equal(operations, iterations);
  const samples = toFrame.slice(16);
  console.error(
    JSON.stringify({
      mode: native ? "native-stdio" : "memory",
      rows,
      viewport,
      updates: iterations,
      appRuns,
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
