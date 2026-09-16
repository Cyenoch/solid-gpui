import { expect, test } from "bun:test";
import type { Setter } from "solid-js";
import { MemoryTransport, Text, View, createRoot } from "../src/index";
import { Envelope } from "../src/protocol/generated/protocol";
import { For, Show, createComponent, createSignal, onCleanup } from "../src/runtime";

function body(frame: Uint8Array) {
  const length = new DataView(frame.buffer, frame.byteOffset, 4).getUint32(0, true);
  expect(length).toBe(frame.byteLength - 4);
  return Envelope.decode(frame.subarray(4)).body!;
}

function snapshotNodes(frame: Uint8Array) {
  const snapshot = body(frame);
  expect(snapshot.tag).toBe(1);
  if (snapshot.tag !== 1) throw new Error("expected Snapshot");
  return (snapshot.value.nodes ?? []).map((node) => {
    if (node.id === undefined || node.parentId === undefined) throw new Error("snapshot node lacks identity");
    return { id: node.id, parentId: node.parentId, text: node.text };
  });
}

function patchChanges(frame: Uint8Array) {
  const patch = body(frame);
  expect(patch.tag).toBe(3);
  if (patch.tag !== 3) throw new Error("expected Patch");
  const operations = patch.value.operations ?? [];
  return {
    created: operations.flatMap((entry) => {
      const node = entry.operation?.tag === 1 ? entry.operation.value.node : undefined;
      return node?.text === undefined ? [] : [node.text];
    }),
    deleted: operations.flatMap((entry) => (entry.operation?.tag === 4 ? [entry.operation.value.id] : [])),
  };
}

test("For reconciles and disposes native children through the public runtime export", async () => {
  const disposed: string[] = [];
  let setItems: Setter<string[]> | undefined;
  const transport = new MemoryTransport();
  const root = createRoot(transport, { surfaceId: 41 });
  root.render(() => {
    const [items, set] = createSignal(["a", "b"]);
    setItems = set;
    return createComponent(View, {
      get children() {
        return For({
          get each() {
            return items();
          },
          fallback: createComponent(Text, { children: "empty" }),
          children: (item, index) => {
            onCleanup(() => disposed.push(item));
            return createComponent(Text, { children: `${item}:${index()}` });
          },
        });
      },
    });
  });
  const initial = snapshotNodes(transport.submitted[0]!);
  expect(initial.flatMap((node) => (node.text === undefined ? [] : [node.text]))).toEqual(["a:0", "b:1"]);
  const removedItem = initial.find((node) => node.text === "a:0")!;

  setItems?.(["b", "c"]);
  await Promise.resolve();
  const reconciled = patchChanges(transport.submitted[1]!);
  expect(disposed).toEqual(["a"]);
  expect(reconciled.deleted).toEqual([removedItem.parentId]);
  expect(reconciled.created).toEqual(["c:1"]);

  setItems?.([]);
  await Promise.resolve();
  expect(disposed).toEqual(["a", "b", "c"]);
  expect(patchChanges(transport.submitted[2]!).created).toEqual(["empty"]);
  root.unmount();
});

test("Show swaps native children and their fallback", async () => {
  let setOpen: Setter<boolean> | undefined;
  const transport = new MemoryTransport();
  const root = createRoot(transport, { surfaceId: 42 });
  root.render(() => {
    const [open, set] = createSignal(false);
    setOpen = set;
    return createComponent(View, {
      get children() {
        return Show({
          get when() {
            return open();
          },
          fallback: createComponent(Text, { children: "closed" }),
          children: createComponent(Text, { children: "open" }),
        });
      },
    });
  });
  const initial = snapshotNodes(transport.submitted[0]!);
  expect(initial.flatMap((node) => (node.text === undefined ? [] : [node.text]))).toEqual(["closed"]);
  const fallback = initial.find((node) => node.text === "closed")!;

  setOpen?.(true);
  await Promise.resolve();
  const opened = patchChanges(transport.submitted[1]!);
  expect(opened.deleted).toEqual([fallback.parentId]);
  expect(opened.created).toEqual(["open"]);
  root.unmount();
});
