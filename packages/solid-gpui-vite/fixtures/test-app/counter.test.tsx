import { beforeEach, expect, mock, test } from "bun:test";
import { createSignal } from "solid-js";
import { MemoryTransport, createRoot } from "@solid-gpui/core";
import { encodeJson, type NativeInvoker } from "@solid-gpui/core/native";
import { counterDescriptor, createCounterClient } from "#native";
import pixel from "./src/pixel.svg?inline";
import { Counter } from "./src/counter";
// A fixture may inspect frames exactly as the framework's own tests do.
import { Envelope } from "../../../../packages/solid-gpui/src/protocol/generated/protocol";

function body(frame: Uint8Array) {
  const length = new DataView(frame.buffer, frame.byteOffset, 4).getUint32(0, true);
  expect(length).toBe(frame.byteLength - 4);
  return Envelope.decode(frame.subarray(4)).body!;
}

/** Every text a frame creates or updates, whichever way the renderer sent it. */
function texts(frame: Uint8Array): string[] {
  const decoded = body(frame);
  if (decoded.tag === 1)
    return (decoded.value.nodes ?? []).flatMap((node) => (node.text === undefined ? [] : [node.text]));
  if (decoded.tag !== 3) return [];
  return (decoded.value.operations ?? []).flatMap(({ operation }) => {
    if (operation?.tag === 1) return operation.value.node?.text === undefined ? [] : [operation.value.node.text];
    if (operation?.tag === 2) return operation.value.text === undefined ? [] : [operation.value.text];
    return [];
  });
}

const invokeNative = mock((moduleId: Uint8Array, moduleDigest: Uint8Array, functionId: number, args: Uint8Array) => {
  expect(moduleId).toHaveLength(16);
  expect(moduleDigest).toHaveLength(32);
  expect(functionId).toBe(1);
  expect(new TextDecoder().decode(args)).toBe("1");
  return Promise.resolve(encodeJson(42));
});

beforeEach(() => invokeNative.mockClear());

test("renders real TSX through MemoryTransport and patches reactively", async () => {
  const [count, setCount] = createSignal(41);
  const transport = new MemoryTransport();
  const root = createRoot(transport, { surfaceId: 7 });
  try {
    root.render(() => <Counter count={count} />);
    const snapshot = body(transport.submitted[0]!);
    expect(snapshot.tag).toBe(1);
    expect(texts(transport.submitted[0]!).join(" ")).toContain("count 41");
    transport.submitted.length = 0;

    // The signal comes from `solid-js` directly: only the renderer observing the
    // same Solid instance can turn this write into a patch.
    setCount(42);
    await Promise.resolve();
    expect(transport.submitted.length).toBeGreaterThan(0);
    expect(transport.submitted.every((frame) => body(frame).tag === 3)).toBe(true);
    const patched = transport.submitted.flatMap(texts).join(" ");
    expect(patched).toContain("count 42");
    expect(patched).toContain("ready");
  } finally {
    root.unmount();
  }
});

test("imports inline assets and generated native bindings", async () => {
  expect(pixel.startsWith("data:image/svg+xml")).toBe(true);
  expect(counterDescriptor.commands.map(({ name }) => name)).toEqual(["increment"]);
  const client = createCounterClient({ invokeNative } as NativeInvoker);
  expect(await client.increment(1)).toBe(42);
  expect(invokeNative).toHaveBeenCalledTimes(1);
});
