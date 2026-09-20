import { expect, test } from "bun:test";
import type { Setter } from "solid-js";
import { createStore } from "solid-js/store";
import { MemoryTransport, Text, VirtualList, createRoot } from "../src/index";
import { Envelope, type Body as WireBody } from "../src/protocol/generated/protocol";
import type { VirtualListProperties as WireListProperties } from "../src/protocol/generated/protocol";
import { encodeFrame } from "../src/protocol";
import { RootContainer } from "../src/renderer/root-container";
import { hostConfig, withRoot } from "../src/renderer/host-config";
import { createComponent, createSignal } from "../src/runtime";

function body(frame: Uint8Array): WireBody {
  const length = new DataView(frame.buffer, frame.byteOffset, 4).getUint32(0, true);
  expect(length).toBe(frame.byteLength - 4);
  return Envelope.decode(frame.subarray(4)).body!;
}

function snapshotOf(frame: Uint8Array) {
  const decoded = body(frame);
  if (decoded.tag !== 1) throw new Error("expected a snapshot frame");
  return decoded.value;
}

function patchOf(frame: Uint8Array) {
  const decoded = body(frame);
  if (decoded.tag !== 3) throw new Error("expected a patch frame");
  return decoded.value;
}

function wireListPropertiesFromSnapshot(frame: Uint8Array): WireListProperties {
  const snapshot = snapshotOf(frame);
  const list = (snapshot.nodes ?? []).find((node) => node.kind === 6);
  const hostProperties = list?.hostProperties;
  if (hostProperties?.tag !== 2) throw new Error("expected VirtualList host properties");
  return hostProperties.value;
}

function wireListPropertiesFromPatch(frame: Uint8Array): WireListProperties | undefined {
  for (const entry of patchOf(frame).operations ?? []) {
    const operation = entry.operation;
    if (operation?.tag !== 2) continue;
    const hostProperties = operation.value.hostProperties;
    if (hostProperties?.tag === 2) return hostProperties.value;
  }
  return undefined;
}

function wireListNodeId(frame: Uint8Array): { id: number; listenerId: number } {
  const snapshot = snapshotOf(frame);
  const list = (snapshot.nodes ?? []).find((node) => node.kind === 6);
  if (list?.id === undefined || list.listenerId === undefined) throw new Error("list identity");
  return { id: list.id, listenerId: list.listenerId };
}

function visibleRangeFrame(
  surfaceId: number,
  list: { id: number; listenerId: number },
  start: number,
  end: number,
): Uint8Array {
  return encodeFrame({
    type: "event",
    surfaceId,
    epoch: 1,
    revision: 1,
    sequence: 1,
    nodeId: list.id,
    listenerId: list.listenerId,
    payload: { type: "visible-range", start, end },
  });
}

function mountList(
  surfaceId: number,
  initial: number[],
  options: { initialNumToRender?: number; estimatedItemSize?: number } = {},
) {
  const transport = new MemoryTransport();
  const root = createRoot(transport, { surfaceId });
  const rendered: number[] = [];
  let setItems: Setter<number[]> | undefined;
  let setSize: Setter<number> | undefined;
  root.render(() => {
    const [items, set] = createSignal<number[]>(initial, { equals: false });
    const [estimated, setEstimated] = createSignal(options.estimatedItemSize ?? 20);
    setItems = set;
    setSize = setEstimated;
    return createComponent(VirtualList<number>, {
      get data() {
        return items();
      },
      itemKey: (item) => item,
      renderItem: (item) => {
        rendered.push(item);
        return createComponent(Text, { children: String(item) });
      },
      get estimatedItemSize() {
        return estimated();
      },
      initialNumToRender: options.initialNumToRender ?? 10,
    });
  });
  return { transport, root, rendered, setItems: setItems!, setSize: setSize! };
}

test("append publishes one bounded tail edit without materializing new rows", async () => {
  const { transport, root, rendered, setItems } = mountList(
    40,
    Array.from({ length: 10_000 }, (_, i) => i),
  );

  const initial = wireListPropertiesFromSnapshot(transport.submitted[0]!);
  expect(initial.itemCount).toBe(10_000);
  expect(initial.dataRevision).toBe(0);
  expect(initial.dataEdit).toBeUndefined();
  expect(rendered).toEqual(Array.from({ length: 10 }, (_, index) => index));

  const framesBefore = transport.submitted.length;
  setItems((items) => [...items, 10_000]);
  await Promise.resolve();

  expect(transport.submitted.length).toBe(framesBefore + 1);
  const operations = patchOf(transport.submitted.at(-1)!).operations ?? [];
  // Only the list's properties change: the appended row sits outside the
  // committed window, so no row nodes are created, moved, or removed.
  expect(operations).toHaveLength(1);
  expect(rendered).toEqual(Array.from({ length: 10 }, (_, index) => index));
  const properties = wireListPropertiesFromPatch(transport.submitted.at(-1)!);
  expect(properties?.itemCount).toBe(10_001);
  expect(properties?.dataRevision).toBe(1);
  expect(properties?.dataEdit).toEqual({
    baseRevision: 0,
    start: 10_000,
    oldCount: 0,
    newCount: 1,
  });
  root.unmount();
});

test("prepend publishes a head edit so retained rows keep their identity", async () => {
  const { transport, root, setItems } = mountList(41, [10, 11, 12, 13, 14, 15]);
  const framesBefore = transport.submitted.length;

  setItems((items) => [9, ...items]);
  await Promise.resolve();

  const properties = wireListPropertiesFromPatch(transport.submitted.at(-1)!);
  expect(properties?.itemCount).toBe(7);
  expect(properties?.dataRevision).toBe(1);
  expect(properties?.dataEdit).toEqual({ baseRevision: 0, start: 0, oldCount: 0, newCount: 1 });
  expect(transport.submitted.length).toBe(framesBefore + 1);
  root.unmount();
});

test("deletes while scrolled publish the replaced span and keep the range valid", async () => {
  const { transport, root, setItems } = mountList(
    42,
    Array.from({ length: 100 }, (_, i) => i),
  );
  const list = wireListNodeId(transport.submitted[0]!);

  transport.push(visibleRangeFrame(42, list, 40, 50));
  await Promise.resolve();

  setItems((items) => items.filter((item) => item % 2 === 0));
  await Promise.resolve();

  const properties = wireListPropertiesFromPatch(transport.submitted.at(-1)!);
  expect(properties?.itemCount).toBe(50);
  expect(properties?.dataRevision).toBe(1);
  expect(properties?.dataEdit).toEqual({ baseRevision: 0, start: 1, oldCount: 99, newCount: 49 });
  // The committed window stays inside the shrunken item space.
  expect(properties!.rangeEnd!).toBeLessThanOrEqual(50);
  root.unmount();
});

test("equal-count reorder still bumps the data revision", async () => {
  const { transport, root, setItems } = mountList(43, [0, 1, 2, 3, 4, 5]);
  const framesBefore = transport.submitted.length;

  setItems((items) => [...items].reverse());
  await Promise.resolve();

  const properties = wireListPropertiesFromPatch(transport.submitted.at(-1)!);
  expect(properties?.itemCount).toBe(6);
  expect(properties?.dataRevision).toBe(1);
  expect(properties?.dataEdit).toEqual({ baseRevision: 0, start: 0, oldCount: 6, newCount: 6 });
  expect(transport.submitted.length).toBe(framesBefore + 1);
  root.unmount();
});

test("batched data updates diff once against the published baseline", async () => {
  const { transport, root, setItems } = mountList(
    44,
    Array.from({ length: 100 }, (_, i) => i),
  );
  const framesBefore = transport.submitted.length;

  setItems((items) => [...items, 100, 101]);
  setItems((items) => [...items, 102]);
  await Promise.resolve();

  // Exactly one commit, computed against the published 100-item list, not the
  // intermediate two-append state that was never sent.
  expect(transport.submitted.length).toBe(framesBefore + 1);
  const properties = wireListPropertiesFromPatch(transport.submitted.at(-1)!);
  expect(properties?.itemCount).toBe(103);
  expect(properties?.dataRevision).toBe(1);
  expect(properties?.dataEdit).toEqual({ baseRevision: 0, start: 100, oldCount: 0, newCount: 3 });
  root.unmount();
});

test("viewport-only commits retain the published edit verbatim", async () => {
  const { transport, root, setItems } = mountList(
    45,
    Array.from({ length: 100 }, (_, i) => i),
  );
  const list = wireListNodeId(transport.submitted[0]!);

  setItems((items) => [...items, 100]);
  await Promise.resolve();
  const afterEdit = wireListPropertiesFromPatch(transport.submitted.at(-1)!);
  expect(afterEdit?.dataRevision).toBe(1);

  transport.push(visibleRangeFrame(45, list, 4, 14));
  await Promise.resolve();
  const afterScroll = wireListPropertiesFromPatch(transport.submitted.at(-1)!);
  // Equal revision must republish the SAME edit, never an absent one.
  expect(afterScroll?.dataRevision).toBe(1);
  expect(afterScroll?.dataEdit).toEqual(afterEdit?.dataEdit);
  root.unmount();
});

test("estimate changes retain the data revision and edit", async () => {
  const { transport, root, setSize } = mountList(46, [0, 1, 2, 3], { estimatedItemSize: 20 });
  const framesBefore = transport.submitted.length;

  setSize(32);
  await Promise.resolve();

  const properties = wireListPropertiesFromPatch(transport.submitted.at(-1)!);
  expect(properties?.itemCount).toBe(4);
  expect(properties?.dataRevision).toBe(0);
  expect(properties?.dataEdit).toBeUndefined();
  expect(transport.submitted.length).toBe(framesBefore + 1);
  root.unmount();
});

test("reference churn with identical content does not republish data", async () => {
  const { transport, root, setItems } = mountList(47, [0, 1, 2, 3]);
  const framesBefore = transport.submitted.length;

  setItems((items) => [...items]);
  await Promise.resolve();

  expect(transport.submitted.length).toBe(framesBefore);
  root.unmount();
});

test("a rejected commit re-chains its retry against the published list", async () => {
  const submitted: Uint8Array[] = [];
  let accept = true;
  const container = new RootContainer({
    surfaceId: 51,
    epoch: 1,
    scheduleDispatch: (dispatch) => dispatch(),
    submitFrame: (frame) => {
      if (accept) submitted.push(frame);
      return accept;
    },
  });
  try {
    const list = withRoot(container.tree, () => hostConfig.createElement("VirtualList"));
    for (const [name, value] of Object.entries({
      __data: [0, 1, 2],
      __itemCount: 3,
      __rangeStart: 0,
      __rangeEnd: 0,
      __estimatedItemSize: 20,
    }))
      hostConfig.setProperty(list, name, value);
    hostConfig.insertNode(container.tree.syntheticRoot, list);
    await Promise.resolve();
    expect(wireListPropertiesFromSnapshot(submitted[0]!).dataRevision).toBe(0);

    accept = false;
    hostConfig.setProperty(list, "__data", [0, 1, 2, 3]);
    hostConfig.setProperty(list, "__itemCount", 4);
    await Promise.resolve();
    expect(submitted).toHaveLength(1);

    accept = true;
    hostConfig.setProperty(list, "__data", [0, 1, 2, 3, 4]);
    hostConfig.setProperty(list, "__itemCount", 5);
    await Promise.resolve();
    const retry = wireListPropertiesFromPatch(submitted[1]!);
    expect(retry?.dataRevision).toBe(1);
    expect(retry?.dataEdit).toEqual({ baseRevision: 0, start: 3, oldCount: 0, newCount: 2 });
  } finally {
    container.dispose();
  }
});

test("republishing a mutable array preserves earlier identity snapshots", async () => {
  const data = Array.from({ length: 30 }, (_, index) => index);
  const { transport, root, setItems } = mountList(52, data);
  try {
    data[15] = 900;
    setItems(data);
    await Promise.resolve();
    const replacement = wireListPropertiesFromPatch(transport.submitted.at(-1)!);
    expect(replacement?.dataRevision).toBe(1);
    expect(replacement?.dataEdit).toEqual({ baseRevision: 0, start: 15, oldCount: 1, newCount: 1 });

    data.push(30);
    setItems(data);
    await Promise.resolve();
    const append = wireListPropertiesFromPatch(transport.submitted.at(-1)!);
    expect(append?.dataRevision).toBe(2);
    expect(append?.dataEdit).toEqual({ baseRevision: 1, start: 30, oldCount: 0, newCount: 1 });
  } finally {
    root.unmount();
  }
});

test("reactive store edits outside the mounted window invalidate data identities", async () => {
  const transport = new MemoryTransport();
  const root = createRoot(transport, { surfaceId: 53 });
  const [data, setData] = createStore(Array.from({ length: 30 }, (_, index) => index));
  try {
    root.render(() =>
      createComponent(VirtualList<number>, {
        data,
        itemKey: (item) => item,
        renderItem: (item) => createComponent(Text, { children: String(item) }),
        estimatedItemSize: 20,
        initialNumToRender: 10,
      }),
    );
    setData(20, 900);
    await Promise.resolve();
    const edit = wireListPropertiesFromPatch(transport.submitted.at(-1)!);
    expect(edit?.dataRevision).toBe(1);
    expect(edit?.dataEdit).toEqual({ baseRevision: 0, start: 20, oldCount: 1, newCount: 1 });
  } finally {
    root.unmount();
  }
});

test("restoring an emptied list starts a fresh native data revision", async () => {
  const { transport, root, setItems } = mountList(54, [0, 1, 2]);
  try {
    setItems([0, 1, 2, 3]);
    await Promise.resolve();
    expect(wireListPropertiesFromPatch(transport.submitted.at(-1)!)?.dataRevision).toBe(1);
    setItems([]);
    await Promise.resolve();
    setItems([10, 11, 12]);
    await Promise.resolve();
    const creates = patchOf(transport.submitted.at(-1)!).operations?.flatMap((entry) => {
      const op = entry.operation;
      if (op?.tag !== 1 || op.value.node?.hostProperties?.tag !== 2) return [];
      return [op.value.node.hostProperties.value];
    });
    expect(creates).toHaveLength(1);
    expect(creates![0]!.itemCount).toBe(3);
    expect(creates![0]!.dataRevision).toBe(0);
    expect(creates![0]!.dataEdit).toBeUndefined();
    setItems([10, 11, 12, 13]);
    await Promise.resolve();
    const edit = wireListPropertiesFromPatch(transport.submitted.at(-1)!);
    expect(edit?.dataRevision).toBe(1);
    expect(edit?.dataEdit).toEqual({ baseRevision: 0, start: 3, oldCount: 0, newCount: 1 });
  } finally {
    root.unmount();
  }
});
