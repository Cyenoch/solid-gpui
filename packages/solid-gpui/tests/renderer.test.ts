import { expect, test } from "bun:test";
import type { Setter } from "solid-js";
import type { ExtensionDescriptor } from "../src/renderer/extension";
import {
  Icon,
  MemoryTransport,
  Pressable,
  Text,
  TextInput,
  View,
  VirtualList,
  createExtensionElement,
  createHostElement,
  createRoot,
  createSurfaceHost,
} from "../src/index";
import { Envelope, type Body as WireBody } from "../src/protocol/generated/protocol";
import { COMMAND_INVOKE_NATIVE, MAX_NATIVE_CALL_BYTES, encodeFrame, type Event } from "../src/protocol";
import { batch, createComponent, createSignal, onCleanup } from "../src/runtime";
import { TransportTerminatedError, type Transport } from "../src/transport";
import { hostConfig, withRoot } from "../src/renderer/host-config";
import { RootContainer } from "../src/renderer/root-container";
import { CommandClient } from "../src/renderer/command-client";
function body(frame: Uint8Array): WireBody {
  const length = new DataView(frame.buffer, frame.byteOffset, 4).getUint32(0, true);
  expect(length).toBe(frame.byteLength - 4);
  return Envelope.decode(frame.subarray(4)).body!;
}

test("invokeNative returns opaque bytes through the surface command lifecycle", async () => {
  const transport = new MemoryTransport();
  const root = createRoot(transport, { surfaceId: 91, epoch: 3 });
  root.render(() => createComponent(Text, { children: "native" }));
  const moduleId = new Uint8Array(16).fill(7);
  const moduleDigest = new Uint8Array(32).fill(9);
  const args = Uint8Array.of(0, 255, 128);
  const result = root.invokeNative(moduleId, moduleDigest, 0xffff_ffff, args);
  const command = body(transport.submitted[1]!);
  expect(command.tag).toBe(4);
  if (command.tag !== 4) throw new Error("expected command");
  expect(command.value).toEqual({
    surfaceId: 91,
    epoch: 3,
    afterRevision: 1,
    requestId: 1,
    nodeId: 1,
    kind: COMMAND_INVOKE_NATIVE,
    payload: { tag: 12, value: { moduleId, moduleDigest, functionId: 0xffff_ffff, args } },
  });
  const bytes = Uint8Array.of(255, 0, 129);
  transport.push(
    encodeFrame({
      type: "event",
      surfaceId: 91,
      epoch: 3,
      revision: 1,
      sequence: 1,
      nodeId: 1,
      listenerId: 0,
      payload: {
        type: "command-result",
        result: {
          requestId: 1,
          command: COMMAND_INVOKE_NATIVE,
          nodeId: 1,
          success: true,
          error: null,
          value: { type: "bytes", value: bytes },
        },
      },
    }),
  );
  await expect(result).resolves.toEqual(bytes);
  const pending = root.invokeNative(moduleId, moduleDigest, 1, new Uint8Array());
  root.unmount();
  await expect(pending).rejects.toThrow();
  await expect(root.invokeNative(moduleId, moduleDigest, 1, args)).rejects.toThrow();
});

test("invokeNative rejects missing, confused, and out-of-bounds JavaScript arguments without sending", async () => {
  const transport = new MemoryTransport();
  const root = createRoot(transport);
  const id = new Uint8Array(16);
  const digest = new Uint8Array(32);
  const args = new Uint8Array();
  const calls: readonly unknown[][] = [
    [],
    [id, digest, 1],
    [undefined, digest, 1, args],
    [id, undefined, 1, args],
    [Array.from(id), digest, 1, args],
    [id, new Uint16Array(16), 1, args],
    [new Uint8Array(15), digest, 1, args],
    [id, new Uint8Array(33), 1, args],
    [id, digest, "1", args],
    [id, digest, 0, args],
    [id, digest, 0x1_0000_0000, args],
    [id, digest, 1.5, args],
    [id, digest, NaN, args],
    [id, digest, 1, []],
    [id, digest, 1, new Uint8Array(MAX_NATIVE_CALL_BYTES + 1)],
  ];
  for (const call of calls) await expect(Reflect.apply(root.invokeNative, root, call)).rejects.toThrow();
  expect(transport.submitted).toEqual([]);
  root.unmount();
});

test("CommandClient rejects mismatched identities and inappropriate byte results", async () => {
  const client = new CommandClient({ submitFrame: () => true, getTerminationError: () => undefined });
  const command = {
    type: "command" as const,
    surfaceId: 1,
    epoch: 1,
    afterRevision: 0,
    requestId: 1,
    nodeId: 1,
    command: COMMAND_INVOKE_NATIVE,
    payload: {
      type: "invoke-native" as const,
      moduleId: new Uint8Array(16),
      moduleDigest: new Uint8Array(32),
      functionId: 1,
      args: new Uint8Array(),
    },
  };
  const result = {
    requestId: 1,
    command: COMMAND_INVOKE_NATIVE,
    nodeId: 1,
    success: true,
    error: null,
    value: { type: "bytes" as const, value: new Uint8Array() },
  };
  for (const invalid of [
    { ...result, command: 6 },
    { ...result, nodeId: 2 },
    { ...result, value: null },
    { ...result, value: { type: "text" as const, value: "" } },
    { ...result, value: { type: "bytes" as const, value: new Uint8Array(MAX_NATIVE_CALL_BYTES + 1) } },
  ]) {
    const pending = client.submit(command);
    client.resolve(invalid);
    await expect(pending).rejects.toThrow();
  }
  const ordinary = client.submit({ ...command, command: 6, payload: { type: "text", value: "title" } });
  client.resolve({ ...result, command: 6 });
  await expect(ordinary).rejects.toThrow();
  const failed = client.submit(command);
  client.resolve({ ...result, success: false, error: "native failure", value: null });
  await expect(failed).rejects.toThrow("native failure");
  const successful = client.submit(command);
  client.resolve(result);
  await expect(successful).resolves.toEqual(result.value);
});
test("Icon projects its allowlisted host properties", () => {
  const transport = new MemoryTransport();
  const root = createRoot(transport);
  root.render(() => createComponent(Icon, { name: "lucide:home", size: 18, color: "#112233" }));
  const snapshot = body(transport.submitted[0]!);
  expect(snapshot.tag).toBe(1);
  if (snapshot.tag === 1) {
    const icon = snapshot.value.nodes?.find((node) => node.kind === 9);
    expect(icon?.hostProperties?.tag).toBe(6);
    if (icon?.hostProperties?.tag === 6) {
      expect(icon.hostProperties.value.name).toBe("lucide:home");
      expect(icon.hostProperties.value.size).toBe(Math.fround(18));
      expect(icon.hostProperties.value.color).toBe(0x112233ff);
    }
  }
  root.unmount();
});

test("a Solid signal produces an incremental native patch", async () => {
  let setCount: Setter<number> | undefined;
  function App() {
    const [count, set] = createSignal(0);
    setCount = set;
    return createComponent(View, { children: createComponent(Text, { children: () => `Count: ${count()}` }) });
  }
  const transport = new MemoryTransport();
  const root = createRoot(transport, { surfaceId: 7, epoch: 2, onAction: () => undefined });
  root.render(() => createComponent(App, {}));
  expect(transport.submitted).toHaveLength(1);
  expect(body(transport.submitted[0]!).tag).toBe(1);
  setCount?.(1);
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
  expect(transport.submitted).toHaveLength(2);
  const patch = body(transport.submitted[1]!);
  expect(patch.tag).toBe(3);
  expect(patch.tag === 3 ? patch.value.operations?.length : 0).toBeGreaterThan(0);
  root.unmount();
});

test("created siblings are emitted in native child-index order", () => {
  const frames: WireBody[] = [];
  const container = new RootContainer({
    surfaceId: 8,
    epoch: 1,
    scheduleDispatch: (dispatch) => dispatch(),
    submitFrame: (frame) => {
      frames.push(body(frame));
      return true;
    },
  });
  const parent = withRoot(container.tree, () => {
    const node = hostConfig.createElement("View");
    hostConfig.insertNode(container.tree.syntheticRoot, node);
    container.tree.commit();
    return node;
  });
  withRoot(container.tree, () => {
    const first = hostConfig.createElement("View");
    const last = hostConfig.createElement("View");
    const middle = hostConfig.createElement("View");
    hostConfig.insertNode(parent, first);
    hostConfig.insertNode(parent, last);
    hostConfig.insertNode(parent, middle, last);
    container.tree.commit();
  });
  const patch = frames[1];
  expect(patch?.tag).toBe(3);
  if (patch?.tag === 3) {
    const siblingCreates = patch.value.operations
      ?.map((operation) => operation.operation)
      .filter((operation) => operation?.tag === 1 && operation.value.node?.parentId === parent.id)
      .map((operation) => (operation?.tag === 1 ? operation.value.node?.index : undefined));
    expect(siblingCreates).toEqual([0, 1, 2]);
  }
  container.dispose();
});

test("a root transaction does not suppress updates in another root", async () => {
  let actionCalled = false;
  let setOtherCount: Setter<number> | undefined;
  function OtherApp() {
    const [count, setCount] = createSignal(0);
    setOtherCount = setCount;
    return createComponent(Text, { children: () => `Other: ${count()}` });
  }
  const otherTransport = new MemoryTransport();
  const otherRoot = createRoot(otherTransport, { surfaceId: 12 });
  otherRoot.render(() => createComponent(OtherApp, {}));
  const eventTransport = new MemoryTransport();
  const eventRoot = createRoot(eventTransport, {
    surfaceId: 11,
    onAction: () => {
      actionCalled = true;
      setOtherCount?.(1);
    },
  });
  eventRoot.render(() => createComponent(Text, { children: "Event root" }));
  eventTransport.push(
    encodeFrame({
      type: "event",
      surfaceId: 11,
      epoch: 1,
      revision: 0,
      sequence: 1,
      nodeId: 1,
      listenerId: 0,
      payload: { type: "action", action: "update" },
    }),
  );
  expect(actionCalled).toBe(true);
  await Promise.resolve();
  expect(otherTransport.submitted).toHaveLength(2);
  expect(body(otherTransport.submitted[1]!).tag).toBe(3);
  eventRoot.unmount();
  otherRoot.unmount();
});

test("conditional nodes stay bound to their Solid owner root", async () => {
  let evaluations = 0;
  let setVisible: Setter<boolean> | undefined;
  function ConditionalApp() {
    const [visible, set] = createSignal(false);
    setVisible = set;
    return () => {
      evaluations += 1;
      return visible() ? createComponent(Text, { children: [1] }) : null;
    };
  }
  const conditionalTransport = new MemoryTransport();
  const conditionalRoot = createRoot(conditionalTransport, { surfaceId: 22 });
  conditionalRoot.render(() => createComponent(ConditionalApp, {}));
  expect(conditionalTransport.submitted).toHaveLength(0);
  const eventTransport = new MemoryTransport();
  const eventRoot = createRoot(eventTransport, { surfaceId: 21, onAction: () => setVisible?.(true) });
  eventRoot.render(() => createComponent(Text, { children: "Event root" }));
  eventTransport.push(
    encodeFrame({
      type: "event",
      surfaceId: 21,
      epoch: 1,
      revision: 0,
      sequence: 1,
      nodeId: 1,
      listenerId: 0,
      payload: { type: "action", action: "show" },
    }),
  );
  await Promise.resolve();
  await Promise.resolve();
  expect(evaluations).toBe(2);
  const snapshot = body(conditionalTransport.submitted[0]!);
  expect(snapshot.tag).toBe(1);
  expect(snapshot.tag === 1 && snapshot.value.nodes?.some((node) => node.kind === 4 && node.text === "1")).toBe(true);
  eventRoot.unmount();
  conditionalRoot.unmount();
});

test("VirtualList mounts after empty data and can remount", async () => {
  let setItems: Setter<string[]> | undefined;
  const transport = new MemoryTransport();
  const root = createRoot(transport, { surfaceId: 30 });
  root.render(() => {
    const [items, set] = createSignal<string[]>([]);
    setItems = set;
    return createComponent(VirtualList<string>, {
      get data() {
        return items();
      },
      itemKey: (item) => item,
      renderItem: (item) => createComponent(Text, { children: item }),
      estimatedItemSize: 20,
    });
  });
  expect(transport.submitted).toHaveLength(0);
  setItems?.(["a"]);
  await Promise.resolve();
  const initial = body(transport.submitted[0]!);
  expect(initial.tag).toBe(1);
  const list = initial.tag === 1 ? initial.value.nodes?.find((node) => node.kind === 6) : undefined;
  expect(list?.hostProperties?.tag).toBe(2);
  expect(list?.hostProperties?.tag === 2 ? list.hostProperties.value.itemCount : 0).toBe(1);
  expect(initial.tag === 1 && initial.value.nodes?.some((node) => node.text === "a")).toBe(true);
  setItems?.([]);
  await Promise.resolve();
  setItems?.(["b"]);
  await Promise.resolve();
  expect(transport.submitted).toHaveLength(3);
  const patch = body(transport.submitted[2]!);
  expect(patch.tag).toBe(3);
  expect(patch.tag === 3 && patch.value.operations?.some((operation) => operation.operation?.tag === 1)).toBe(true);
  root.unmount();
});

test("VirtualList scrolling retains overlapping rows and disposes rows that leave", async () => {
  const transport = new MemoryTransport();
  const root = createRoot(transport, { surfaceId: 32 });
  const created: number[] = [];
  const disposed: number[] = [];
  let setItems: Setter<number[]> | undefined;
  root.render(() => {
    const [items, set] = createSignal(Array.from({ length: 100 }, (_, i) => i));
    setItems = set;
    return createComponent(VirtualList<number>, {
      get data() {
        return items();
      },
      itemKey: (item) => item % 100,
      estimatedItemSize: 20,
      initialNumToRender: 10,
      renderItem: (item) => {
        created.push(item);
        onCleanup(() => disposed.push(item));
        return createComponent(Text, { children: String(item) });
      },
    });
  });
  const initial = body(transport.submitted[0]!);
  if (initial.tag !== 1) throw new Error("snapshot");
  const list = initial.value.nodes!.find((node) => node.kind === 6)!;
  if (list.id === undefined || list.listenerId === undefined) throw new Error("list identity");
  transport.push(
    encodeFrame({
      type: "event",
      surfaceId: 32,
      epoch: 1,
      revision: 1,
      sequence: 1,
      nodeId: list.id,
      listenerId: list.listenerId,
      payload: { type: "visible-range", start: 1, end: 11 },
    }),
  );
  await Promise.resolve();
  expect(created).toEqual(Array.from({ length: 11 }, (_, i) => i));
  expect(disposed).toEqual([0]);
  // The same key with a new value must update, not retain stale row content.
  setItems!((items) => items.map((item) => (item === 1 ? 101 : item)));
  await Promise.resolve();
  expect(created.at(-1)).toBe(101);
  expect(disposed).toEqual([0, 1]);
  root.unmount();
  expect(disposed.sort((a, b) => a - b)).toEqual([...created].sort((a, b) => a - b));
});

test("VirtualList filtering after scrolling keeps its committed range valid", async () => {
  let setItems: Setter<string[]> | undefined;
  const transport = new MemoryTransport();
  const root = createRoot(transport, { surfaceId: 31 });
  root.render(() => {
    const [items, set] = createSignal(Array.from({ length: 100 }, (_, i) => String(i)));
    setItems = set;
    return createComponent(VirtualList<string>, {
      get data() {
        return items();
      },
      itemKey: (item) => item,
      renderItem: (item) => createComponent(Text, { children: item }),
      estimatedItemSize: 20,
    });
  });
  const initial = body(transport.submitted[0]!);
  if (initial.tag !== 1) throw new Error("snapshot");
  const list = initial.value.nodes!.find((node) => node.kind === 6)!;
  if (list.id === undefined || list.listenerId === undefined) throw new Error("list event identity");
  transport.push(
    encodeFrame({
      type: "event",
      surfaceId: 31,
      epoch: 1,
      revision: 1,
      sequence: 1,
      nodeId: list.id,
      listenerId: list.listenerId,
      payload: { type: "visible-range", start: 90, end: 100 },
    }),
  );
  await Promise.resolve();
  setItems!(["filtered"]);
  await Promise.resolve();
  const patch = body(transport.submitted.at(-1)!);
  expect(patch.tag).toBe(3);
  if (patch.tag !== 3) throw new Error("patch");
  const properties = patch.value.operations?.flatMap((op) => {
    const operation = op.operation;
    return operation?.tag === 2 && operation.value.hostProperties?.tag === 2
      ? [operation.value.hostProperties.value]
      : [];
  });
  expect(properties).toContainEqual({ itemCount: 1, rangeStart: 0, rangeEnd: 1, estimatedItemSize: 20, overscan: 2 });
  root.unmount();
});

test("render errors publish no invalid frame and native surface close disposes the owner", () => {
  const transport = new MemoryTransport();
  const root = createRoot(transport, { surfaceId: 40 });
  expect(() =>
    root.render(() => {
      throw new Error("render failed");
    }),
  ).toThrow("render failed");
  expect(transport.submitted).toHaveLength(0);
  root.render(() => createComponent(Text, { children: "Recovered" }));
  expect(body(transport.submitted[0]!).tag).toBe(1);
  let cleaned = false;
  const closeTransport = new MemoryTransport();
  const closeRoot = createRoot(closeTransport, { surfaceId: 50 });
  closeRoot.render(() => {
    onCleanup(() => {
      cleaned = true;
    });
    return createComponent(Text, { children: "Closing" });
  });
  closeTransport.push(
    encodeFrame({
      type: "event",
      surfaceId: 50,
      epoch: 1,
      revision: 0,
      sequence: 1,
      nodeId: 0,
      listenerId: 0,
      payload: { type: "surface-closed" },
    }),
  );
  expect(cleaned).toBe(true);
});
test("SurfaceHost delivers the decoded semantic event directly to its root", () => {
  const transport = new MemoryTransport();
  let action: string | undefined;
  const host = createSurfaceHost(transport);
  const root = host.createRoot({ onAction: (value) => (action = value) });
  root.render(() => createComponent(Text, { children: "Root" }));
  transport.push(
    encodeFrame({
      type: "event",
      surfaceId: 1,
      epoch: 1,
      revision: 1,
      sequence: 1,
      nodeId: 1,
      listenerId: 0,
      payload: { type: "action", action: "open" },
    }),
  );
  expect(action).toBe("open");
  host.dispose();
});
test("previous-revision listener events invoke the callback that produced that revision", async () => {
  const transport = new MemoryTransport();
  const calls: string[] = [];
  let setPress: Setter<() => void> | undefined;
  function App() {
    const [press, set] = createSignal<() => void>(() => calls.push("old"));
    setPress = set;
    return createComponent(Pressable, {
      get onPress() {
        return press();
      },
      children: createComponent(Text, { children: "button" }),
    });
  }
  const root = createRoot(transport, { surfaceId: 60 });
  root.render(() => createComponent(App, {}));
  const snapshot = body(transport.submitted[0]!);
  expect(snapshot.tag).toBe(1);
  const node = snapshot.tag === 1 ? snapshot.value.nodes?.find((value) => value.kind === 3) : undefined;
  expect(node).toBeDefined();
  const listenerId = node?.listenerId ?? 0;
  setPress?.(() => () => calls.push("new"));
  await Promise.resolve();
  expect(body(transport.submitted[1]!).tag).toBe(3);
  transport.push(
    encodeFrame({
      type: "event",
      surfaceId: 60,
      epoch: 1,
      revision: 1,
      sequence: 1,
      nodeId: node?.id ?? 0,
      listenerId,
      payload: { type: "press" },
    }),
  );
  expect(calls).toEqual(["old"]);
  root.unmount();
});

test("layout, file-drop, and input callbacks retain the generation that owns each event", async () => {
  const cases: readonly {
    kind: "View" | "TextInput";
    prop: string;
    payload: Event["payload"];
  }[] = [
    { kind: "View", prop: "onLayout", payload: { type: "layout", x: 0, y: 0, width: 20, height: 20 } },
    { kind: "View", prop: "onExternalFileDrop", payload: { type: "external-file-drop", paths: ["/tmp/item"] } },
    { kind: "TextInput", prop: "onSubmitEditing", payload: { type: "submit", text: "value" } },
    {
      kind: "TextInput",
      prop: "onChangeText",
      payload: {
        type: "change",
        data: {
          text: "value",
          selectionStart: 5,
          selectionEnd: 5,
          reversed: false,
          markedStart: null,
          markedEnd: null,
          editSeq: 1,
        },
      },
    },
  ];
  for (const { kind, prop, payload } of cases) {
    const transport = new MemoryTransport();
    const root = createRoot(transport, { surfaceId: 66 });
    const calls: string[] = [];
    let setCallback!: Setter<() => void>;
    root.render(() => {
      const [callback, set] = createSignal<() => void>(() => calls.push("old"));
      setCallback = set;
      return createHostElement(kind, {
        get [prop]() {
          return callback();
        },
      });
    });
    try {
      const snapshot = body(transport.submitted[0]!);
      if (snapshot.tag !== 1) throw new Error("expected snapshot");
      const node = snapshot.value.nodes!.find((value) => value.listenerId !== 0)!;
      setCallback(() => () => calls.push("new"));
      await Promise.resolve();
      expect(transport.submitted).toHaveLength(2);
      for (const revision of [1, 2]) {
        transport.push(
          encodeFrame({
            type: "event",
            surfaceId: 66,
            epoch: 1,
            revision,
            sequence: revision,
            nodeId: node.id!,
            listenerId: node.listenerId!,
            payload,
          }),
        );
      }
      expect(calls).toEqual(["old", "new"]);
    } finally {
      root.unmount();
    }
  }
});

test("failed host updates roll the private graph back before a later commit", async () => {
  const transport = new MemoryTransport();
  const root = createRoot(transport, { surfaceId: 61 });
  let setValue: Setter<string> | undefined;
  function App() {
    const [value, set] = createSignal("stable");
    setValue = set;
    return createComponent(View, {
      children: createComponent(Text, {
        children: () => {
          const next = value();
          if (next === "boom") throw new Error("host update failed");
          return next;
        },
      }),
    });
  }
  root.render(() => createComponent(App, {}));
  expect(transport.submitted).toHaveLength(1);
  expect(() => setValue?.("boom")).toThrow("host update failed");
  setValue?.("recovered");
  await Promise.resolve();
  const frame = body(transport.submitted.at(-1)!);
  expect(frame.tag).toBe(3);
  if (frame.tag === 3) {
    expect(frame.value.operations?.every((operation) => operation.operation?.tag === 2)).toBe(true);
    const update = frame.value.operations?.find((operation) => operation.operation?.tag === 2);
    expect(update?.operation?.tag === 2 ? update.operation.value.text : undefined).toBe("recovered");
  }
  root.unmount();
});
test("Extension getter props emit host property patches and failed encoding rolls back", async () => {
  let setLabel: Setter<string> | undefined;
  let node: ReturnType<typeof createExtensionElement> | undefined;
  const descriptor: ExtensionDescriptor<{ readonly label: string }> = {
    providerId: new Uint8Array(16),
    catalogDigest: new Uint8Array(32),
    entryId: 1,
    entryVersion: 1,
    eventIds: [],
    encodeProps: (props) => {
      if (props.label === "boom") throw new Error("extension encoding failed");
      return [{ id: 1, value: { type: "text", value: props.label } }];
    },
  };
  const transport = new MemoryTransport();
  const root = createRoot(transport, { surfaceId: 64 });
  function App() {
    const [label, set] = createSignal("stable");
    setLabel = set;
    const props = {
      get label() {
        return label();
      },
      onPress: () => undefined,
    };
    return (node = createExtensionElement(descriptor, props));
  }
  root.render(() => createComponent(App, {}));
  expect(transport.submitted).toHaveLength(1);
  setLabel?.("changed");
  await Promise.resolve();
  expect(transport.submitted).toHaveLength(2);
  const changed = body(transport.submitted[1]!);
  expect(changed.tag).toBe(3);
  if (changed.tag === 3) {
    const update = changed.value.operations?.find((operation) => operation.operation?.tag === 2);
    expect(update?.operation?.tag === 2 ? update.operation.value.hostProperties?.tag : undefined).toBe(5);
    if (update?.operation?.tag === 2 && update.operation.value.hostProperties?.tag === 5) {
      expect(update.operation.value.hostProperties.value.fields).toHaveLength(1);
      expect(update.operation.value.hostProperties.value.fields?.[0]?.value?.tag).toBe(5);
    }
  }
  setLabel?.("boom");
  await Promise.resolve();
  expect(transport.submitted).toHaveLength(2);
  expect(node?.hostProperties?.type).toBe("extension");
  if (node?.hostProperties?.type === "extension") {
    expect(node.hostProperties.value.fields[0]?.value).toEqual({ type: "text", value: "changed" });
  }
  setLabel?.("recovered");
  await Promise.resolve();
  expect(transport.submitted).toHaveLength(3);
  root.unmount();
});
test("Extension events honor subscribed IDs and current/previous listener revisions", async () => {
  const calls: string[] = [];
  let setHandler: Setter<(event: { readonly eventId: number }) => void> | undefined;
  const descriptor: ExtensionDescriptor<{ readonly label: string }> = {
    providerId: new Uint8Array(16),
    catalogDigest: new Uint8Array(32),
    entryId: 2,
    entryVersion: 1,
    eventIds: [1, 2],
    encodeProps: () => [],
  };
  const transport = new MemoryTransport();
  const root = createRoot(transport, { surfaceId: 65 });
  function App() {
    const [handler, set] = createSignal<(event: { readonly eventId: number }) => void>((event) => {
      calls.push(`old:${event.eventId}`);
    });
    setHandler = set;
    return createExtensionElement(descriptor, {
      label: "value",
      get onExtensionEvent() {
        return handler();
      },
    });
  }
  root.render(() => createComponent(App, {}));
  const snapshot = body(transport.submitted[0]!);
  expect(snapshot.tag).toBe(1);
  const node = snapshot.tag === 1 ? snapshot.value.nodes?.find((value) => value.kind === 8) : undefined;
  expect(node?.listenerId).toBeGreaterThan(0);
  const listenerId = node?.listenerId ?? 0;
  setHandler?.(() => (event: { readonly eventId: number }) => calls.push(`new:${event.eventId}`));
  await Promise.resolve();
  transport.push(
    encodeFrame({
      type: "event",
      surfaceId: 65,
      epoch: 1,
      revision: 1,
      sequence: 1,
      nodeId: node?.id ?? 0,
      listenerId,
      payload: { type: "extension", eventId: 1, fields: [] },
    }),
  );
  transport.push(
    encodeFrame({
      type: "event",
      surfaceId: 65,
      epoch: 1,
      revision: 2,
      sequence: 2,
      nodeId: node?.id ?? 0,
      listenerId,
      payload: { type: "extension", eventId: 2, fields: [] },
    }),
  );
  transport.push(
    encodeFrame({
      type: "event",
      surfaceId: 65,
      epoch: 1,
      revision: 2,
      sequence: 3,
      nodeId: node?.id ?? 0,
      listenerId,
      payload: { type: "extension", eventId: 99, fields: [] },
    }),
  );
  expect(calls).toEqual(["old:1", "new:2"]);
  root.unmount();
});

test("accessibility props can be removed from an existing host node", () => {
  const transport = new MemoryTransport();
  const root = createRoot(transport, { surfaceId: 62 });
  root.render(() => createComponent(View, { accessibilityLabel: "before" }));
  root.render(() => createComponent(View, {}));
  const frame = body(transport.submitted.at(-1)!);
  expect(frame.tag).toBe(3);
  if (frame.tag === 3) {
    const update = frame.value.operations?.find((operation) => operation.operation?.tag === 2);
    expect(update?.operation?.tag === 2 ? update.operation.value.accessibility : undefined).toBeUndefined();
  }
  root.unmount();
});

test("focusable View commands do not require an event listener", () => {
  const transport = new MemoryTransport();
  const handle: { current?: { focus(): Promise<void> } } = { current: undefined };
  const root = createRoot(transport, { surfaceId: 63 });
  root.render(() => createComponent(View, { focusable: true, ref: handle }));
  const snapshot = body(transport.submitted[0]!);
  expect(snapshot.tag).toBe(1);
  const node = snapshot.tag === 1 ? snapshot.value.nodes?.find((value) => value.id !== 1) : undefined;
  expect(node?.focusable).toBe(true);
  expect(node?.listenerId).toBe(0);
  expect(handle.current).toBeDefined();
  root.unmount();
});

test("host prop runtime validation covers Pressable, TextInput, and accessibility scalars", () => {
  const root = createRoot(new MemoryTransport(), { surfaceId: 64 });
  expect(() => root.render(() => createComponent(Pressable, { onPress: "bad" as never }))).toThrow(
    "Pressable onPress must be a function",
  );
  expect(() => root.render(() => createComponent(TextInput, { multiline: "bad" as never }))).toThrow(
    "TextInput multiline must be a boolean",
  );
  expect(() => root.render(() => createComponent(TextInput, { onChangeText: "bad" as never }))).toThrow(
    "TextInput onChangeText must be a function",
  );
  expect(() => root.render(() => createComponent(TextInput, { disabled: "bad" as never }))).toThrow(
    "TextInput disabled must be a boolean",
  );
  expect(() => root.render(() => createComponent(TextInput, { maxLength: -1 }))).toThrow(
    "TextInput maxLength must be a non-negative u32",
  );
  expect(() => root.render(() => createComponent(View, { accessibilityRole: "bad" as never }))).toThrow(
    "accessibilityRole is invalid",
  );
  expect(() => root.render(() => createComponent(View, { toString: 1 } as never))).toThrow(
    "Unsupported View prop: toString",
  );
  expect(() => root.render(() => createComponent(View, { tooltip: "hidden\u0085control" }))).toThrow(
    "View tooltip must be a non-empty safe string",
  );
  root.unmount();
});

test("prop updates finalize atomically across cross-field accessibility validation", async () => {
  const transport = new MemoryTransport();
  let setRole: Setter<"generic" | "checkbox"> | undefined;
  let setChecked: Setter<boolean | undefined> | undefined;
  function App() {
    const [role, setNextRole] = createSignal<"generic" | "checkbox">("generic");
    const [checked, setNextChecked] = createSignal<boolean | undefined>(undefined);
    setRole = setNextRole;
    setChecked = setNextChecked;
    return createComponent(View, {
      get accessibilityChecked() {
        return checked();
      },
      get accessibilityRole() {
        return role();
      },
    });
  }
  const root = createRoot(transport, { surfaceId: 65 });
  root.render(() => createComponent(App, {}));
  expect(() =>
    batch(() => {
      setChecked?.(true);
      setRole?.("checkbox");
    }),
  ).not.toThrow();
  await Promise.resolve();
  expect(transport.submitted).toHaveLength(2);
  root.unmount();
});

test("a single incoming chunk batches semantic events into one Solid transaction", async () => {
  const transport = new MemoryTransport();
  let setCount: Setter<number> | undefined;
  function App() {
    const [count, set] = createSignal(0);
    setCount = set;
    return createComponent(Text, { children: () => String(count()) });
  }
  const host = createSurfaceHost(transport);
  const root = host.createRoot({ onAction: () => setCount?.((value) => value + 1) });
  root.render(() => createComponent(App, {}));
  const event = (sequence: number) =>
    encodeFrame({
      type: "event" as const,
      surfaceId: 1,
      epoch: 1,
      revision: 1,
      sequence,
      nodeId: 1,
      listenerId: 0,
      payload: { type: "action" as const, action: "increment" },
    });
  const first = event(1);
  const second = event(2);
  const chunk = new Uint8Array(first.byteLength + second.byteLength);
  chunk.set(first);
  chunk.set(second, first.byteLength);
  transport.push(chunk);
  await Promise.resolve();
  expect(transport.submitted).toHaveLength(2);
  root.unmount();
  host.dispose();
});

test("createRoot tolerates synchronous transport termination registration", () => {
  const termination = new TransportTerminatedError("sync termination", { kind: "shutdown" });
  const transport: Transport = {
    submit: () => undefined,
    onData: () => () => undefined,
    onTermination(listener) {
      listener(termination);
      return () => undefined;
    },
  };
  expect(() => createRoot(transport)).not.toThrow();
});

test("SurfaceHost releases a root even when unmount cleanup throws", () => {
  const host = createSurfaceHost(new MemoryTransport());
  const root = host.createRoot({ surfaceId: 70 });
  root.render(() => {
    onCleanup(() => {
      throw new Error("cleanup failed");
    });
    return createComponent(Text, { children: "cleanup" });
  });
  expect(() => root.unmount()).toThrow("cleanup failed");
  expect(() => host.createRoot({ surfaceId: 70 })).toThrow("already closed");
  host.dispose();
});

test("transport disposal rejects pending commands and notifies termination once", async () => {
  let notify: ((error: TransportTerminatedError) => void) | undefined;
  let notifications = 0;
  const transport: Transport = {
    submit: () => undefined,
    onData: () => () => undefined,
    onTermination(listener) {
      notify = listener;
      return () => undefined;
    },
  };
  const root = createRoot(transport, {
    onTransportTermination: () => {
      notifications += 1;
    },
  });
  const pending = root.setTitle("pending");
  const error = new TransportTerminatedError("terminated", { kind: "shutdown" });
  notify?.(error);
  await expect(pending).rejects.toBe(error);
  notify?.(error);
  expect(notifications).toBe(1);
  expect(() => root.setTitle("after")).toThrow();
});

test("HostTree is the only host-config owner and RootContainer hides tree bookkeeping", () => {
  const container = new RootContainer({
    surfaceId: 80,
    epoch: 1,
    scheduleDispatch: (dispatch) => dispatch(),
    submitFrame: () => true,
  });
  const node = withRoot(container.tree, () => hostConfig.createElement("View"));
  expect(node.root).toBe(container.tree);
  const internals = container as unknown as Record<string, unknown>;
  expect(internals.nodes).toBeUndefined();
  expect(internals.children).toBeUndefined();
  expect(internals.listeners).toBeUndefined();
  expect(internals.nodesById).toBeUndefined();
  expect(internals.createdIds).toBeUndefined();
  expect("transport" in (node.root as object)).toBe(false);
  container.dispose();
});

test("batched subtree removals use published ancestry, including moves through a temporary parent", async () => {
  for (const scenario of ["child then parent", "temporary parent", "surviving child"] as const) {
    const submitted: Uint8Array[] = [];
    const container = new RootContainer({
      surfaceId: 81,
      epoch: 1,
      scheduleDispatch: (dispatch) => dispatch(),
      submitFrame(frame) {
        submitted.push(frame);
        return true;
      },
    });
    const tree = container.tree;
    const make = () => withRoot(tree, () => hostConfig.createElement("View"));
    const parent = make();
    const child = make();
    hostConfig.insertNode(parent, child);
    hostConfig.insertNode(tree.syntheticRoot, parent);
    await Promise.resolve();
    if (scenario === "child then parent") {
      hostConfig.removeNode(parent, child);
      hostConfig.removeNode(tree.syntheticRoot, parent);
    } else if (scenario === "temporary parent") {
      const temporary = make();
      hostConfig.insertNode(tree.syntheticRoot, temporary);
      hostConfig.insertNode(temporary, parent);
      hostConfig.removeNode(tree.syntheticRoot, temporary);
    } else {
      hostConfig.insertNode(tree.syntheticRoot, child);
      hostConfig.removeNode(tree.syntheticRoot, parent);
    }
    await Promise.resolve();
    expect(submitted).toHaveLength(2);
    const patch = body(submitted[1]!);
    if (patch.tag !== 3) throw new Error("expected removal patch");
    const ops = patch.value.operations!.map((item) => item.operation!);
    expect(ops.filter((op) => op.tag === 4).map((op) => op.value.id)).toEqual([parent.id]);
    expect(ops.filter((op) => op.tag === 1)).toEqual([]);
    expect(ops.filter((op) => op.tag === 3).map((op) => op.value.id)).toEqual(
      scenario === "surviving child" ? [child.id] : [],
    );
    container.dispose();
  }
});
