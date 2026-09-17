import { expect, test } from "bun:test";
import type { Setter } from "solid-js";
import { Column, MemoryTransport, Row, View, createRoot } from "../src/index";
import { Envelope, type Body as WireBody } from "../src/protocol/generated/protocol";
import { encodeFrame } from "../src/protocol";
import { createComponent, createSignal } from "../src/runtime";

// Wire flex-direction codes, from the style encoder.
const ROW = 1;
const COLUMN = 2;
const ROW_REVERSE = 3;
function body(frame: Uint8Array): WireBody {
  const length = new DataView(frame.buffer, frame.byteOffset, 4).getUint32(0, true);
  expect(length).toBe(frame.byteLength - 4);
  return Envelope.decode(frame.subarray(4)).body!;
}
/// The style the renderer published for the one styled View of a fresh tree.
function styledView(transport: MemoryTransport) {
  const snapshot = body(transport.submitted[0]!);
  if (snapshot.tag !== 1) throw new Error("expected a snapshot");
  const node = snapshot.value.nodes?.find((candidate) => candidate.style);
  if (!node?.style) throw new Error("expected a styled View");
  return node.style;
}

test("Row states its own axis rather than letting gap imply one", () => {
  const transport = new MemoryTransport();
  const root = createRoot(transport);
  root.render(() => createComponent(Row, { gap: 8 }));
  const style = styledView(transport);
  expect(style.flexDirection).toBe(ROW);
  expect(style.gap).toBe(Math.fround(8));
  root.unmount();
});

test("Column maps every container shorthand onto the native style", () => {
  const transport = new MemoryTransport();
  const root = createRoot(transport);
  root.render(() => createComponent(Column, { gap: 4, align: "center", justify: "space-between", padding: 12 }));
  const style = styledView(transport);
  expect(style.flexDirection).toBe(COLUMN);
  expect(style.gap).toBe(Math.fround(4));
  expect(style.alignItems).toBe(2);
  expect(style.justifyContent).toBe(4);
  expect(style.padding).toBe(Math.fround(12));
  root.unmount();
});

test("a style object keeps the raw View's implicit layout", () => {
  const transport = new MemoryTransport();
  const root = createRoot(transport);
  root.render(() => createComponent(View, { style: { gap: 8 } }));
  const style = styledView(transport);
  expect(style.gap).toBe(Math.fround(8));
  expect(style.flexDirection).toBeUndefined();
  root.unmount();
});

test("an explicit style.flexDirection wins over the helper's axis", () => {
  const transport = new MemoryTransport();
  const root = createRoot(transport);
  root.render(() => createComponent(Row, { style: { flexDirection: "row-reverse" } }));
  expect(styledView(transport).flexDirection).toBe(ROW_REVERSE);
  root.unmount();
});

test("a reactive gap republishes the axis too", async () => {
  let setGap: Setter<number> | undefined;
  function App() {
    const [gap, set] = createSignal(8);
    setGap = set;
    return createComponent(Row, {
      get gap() {
        return gap();
      },
    });
  }
  const transport = new MemoryTransport();
  const root = createRoot(transport, { surfaceId: 7, epoch: 2, onAction: () => undefined });
  root.render(() => createComponent(App, {}));
  expect(styledView(transport).flexDirection).toBe(ROW);
  setGap?.(20);
  transport.push(
    encodeFrame({
      type: "event",
      surfaceId: 7,
      epoch: 2,
      revision: 0,
      sequence: 1,
      nodeId: 1,
      listenerId: 0,
      payload: { type: "action", action: "noop" },
    }),
  );
  await Promise.resolve();
  const patch = body(transport.submitted[1]!);
  expect(patch.tag).toBe(3);
  const operation = patch.tag === 3 ? patch.value.operations?.find((entry) => entry.operation?.tag === 2) : undefined;
  expect(operation?.operation?.tag === 2 ? operation.operation.value.style : undefined).toMatchObject({
    flexDirection: ROW,
    gap: Math.fround(20),
  });
  root.unmount();
});
