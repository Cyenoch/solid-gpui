import { expect, test } from "bun:test";
import {
  createRoot,
  MemoryTransport,
  Pressable,
  Text,
  TextInput,
  View,
  VirtualList,
  type VirtualListHandle,
  type StyleProp,
} from "@solid-gpui/core";
import { batch, createComponent, createSignal, For, onCleanup } from "@solid-gpui/core/runtime";
import { Envelope } from "../src/protocol/generated/protocol";
import {
  createNativeClient,
  createNativeComponent,
  decodeNativeRequest,
  encodeJson,
  type NativeClientDescriptor,
} from "@solid-gpui/core/native";
import { TestHost, type TestSurface } from "@solid-gpui/core/testing";

const texts = (surface: TestSurface) => surface.nodes.flatMap((node) => (node.text === null ? [] : [node.text]));
test("surface commands follow the final commit of a synchronous Solid batch", async () => {
  const transport = new MemoryTransport();
  const root = createRoot(transport);
  const [value, setValue] = createSignal("initial");
  let pending: Promise<unknown> | undefined;
  try {
    root.render(() =>
      Text({
        get children() {
          return value();
        },
      }),
    );
    batch(() => {
      setValue("updated");
      pending = root.resize(400, 300).catch((error) => error);
      setValue("final");
    });
    await Promise.resolve();
    const bodies = transport.submitted.map((frame) => Envelope.decode(frame.subarray(4)).body!);
    expect(bodies.map((body) => body.tag)).toEqual([1, 3, 4]);
    const patch = bodies[1]!;
    const command = bodies[2]!;
    if (patch.tag !== 3 || command.tag !== 4) throw new Error("Expected patch followed by command");
    expect(command.value.afterRevision).toBe(patch.value.revision);
  } finally {
    root.unmount();
    await pending;
  }
});
test("surface commands reject before the first committed render without publishing an invalid request", async () => {
  const transport = new MemoryTransport();
  const root = createRoot(transport);
  try {
    await expect(root.resize(400, 300)).rejects.toThrow("initial committed render");
    await expect(root.setTitle("Loading")).rejects.toThrow("initial committed render");
    expect(transport.submitted).toHaveLength(0);
    root.render(() => Text({ children: "Ready" }));
    expect(transport.submitted).toHaveLength(1);
  } finally {
    root.unmount();
  }
});

const clientDescriptor: NativeClientDescriptor = {
  buildDigest: Array(32).fill(11),
  semanticVersion: "1.0.0",
  moduleId: Array(16).fill(7),
  moduleDigest: Array(32).fill(9),
  commands: [{ id: 1, name: "echo" }],
};

test("TestHost drives virtual viewport ownership and explicit scroll replies through public APIs", async () => {
  const host = new TestHost();
  const root = createRoot(host.transport, { surfaceId: 47 });
  const cleaned: number[] = [];
  let list!: VirtualListHandle;
  let reached = 0;
  try {
    root.render(() =>
      VirtualList({
        data: [0, 1, 2, 3, 4],
        itemKey: (item) => item,
        initialNumToRender: 2,
        overscan: 0,
        estimatedItemSize: 20,
        renderItem: (item) => {
          onCleanup(() => cleaned.push(item));
          return Text({ children: String(item) });
        },
        ref: (handle) => {
          list = handle;
        },
        onEndReached: () => {
          reached++;
        },
        style: { height: 40, flexGrow: 1 },
      }),
    );
    const initial = host.surface(47)!.nodes.find((node) => node.kind === "VirtualList")!;
    expect(initial.virtualList).toMatchObject({ itemCount: 5, rangeStart: 0, rangeEnd: 2 });
    expect(initial.style).toMatchObject({ height: 40, flexGrow: 1 });
    host.dispatch(initial, { type: "visible-range", start: 3, end: 5 });
    expect(texts(host.surface(47)!)).toEqual(["3", "4"]);
    expect(cleaned.sort()).toEqual([0, 1]);
    expect(reached).toBe(1);
    let settled = false;
    const offset = list.getScrollOffset().then((value) => {
      settled = true;
      return value;
    });
    await Promise.resolve();
    expect(settled).toBe(false);
    const query = host.scrollCommands.find((command) => command.type === "get-scroll-offset")!;
    if (query.type !== "get-scroll-offset") throw new Error("missing offset query");
    host.replyScroll(query, 60);
    await expect(offset).resolves.toBe(60);
    const restore = list.scrollToOffset(20);
    await Promise.resolve();
    const restoreCommand = host.scrollCommands.at(-1)!;
    expect(restoreCommand).toMatchObject({ type: "scroll-to-offset", offset: 20 });
    if (restoreCommand.type === "get-scroll-offset") throw new Error("expected scroll action");
    host.replyScroll(restoreCommand);
    await restore;
    const index = list.scrollToIndex(1);
    await Promise.resolve();
    const indexCommand = host.scrollCommands.at(-1)!;
    expect(indexCommand).toMatchObject({ type: "scroll-to-index", index: 1 });
    if (indexCommand.type === "get-scroll-offset") throw new Error("expected scroll action");
    host.replyScroll(indexCommand);
    await index;
    const end = list.scrollToEnd();
    await Promise.resolve();
    host.reject(host.scrollCommands.at(-1)!, "scroll unavailable");
    await expect(end).rejects.toThrow("scroll unavailable");
    expect(() => host.replyScroll(query, 0)).toThrow("already been answered");
    const pending = list.scrollToEnd();
    await Promise.resolve();
    root.unmount();
    await expect(pending).rejects.toThrow();
  } finally {
    root.unmount();
  }
});

test("TestHost captures committed styles and accessibility and sequences layout, pointer and input", async () => {
  const host = new TestHost();
  const root = createRoot(host.transport, { surfaceId: 48 });
  const [height, setHeight] = createSignal<number | undefined>(80);
  const received: string[] = [];
  try {
    root.render(() =>
      View({
        get style(): StyleProp {
          return height() === undefined
            ? null
            : { height: height(), flexDirection: "column", backgroundColor: "#123456" };
        },
        accessibilityRole: "group",
        accessibilityLabel: "Crop",
        accessibilityDisabled: true,
        onLayout: (event) => received.push(`layout:${event.width}`),
        onPointerDown: (event) => received.push(`down:${event.x}`),
        onPointerMove: (event) => received.push(`move:${event.x}`),
        children: TextInput({ value: "", onChangeText: (text) => received.push(text) }),
      }),
    );
    const initial = host.surface(48)!.nodes.find((node) => node.accessibilityLabel === "Crop")!;
    expect(initial.style).toMatchObject({ height: 80, flexDirection: "column", backgroundColor: "#123456ff" });
    expect(initial.accessibility).toMatchObject({ accessibilityRole: "group", accessibilityDisabled: true });
    expect(initial.disabled).toBe(true);
    host.dispatch(initial, { type: "layout", x: 0, y: 0, width: 200, height: 80 });
    host.dispatch(initial, { type: "pointer", action: "down", x: 10, y: 20 });
    host.dispatch(initial, { type: "pointer-move", x: 30, y: 40 });
    host.dispatch(
      host.surface(48)!.nodes.find((node) => node.kind === "TextInput")!,
      { type: "input", text: "crop" },
    );
    expect(received).toEqual(["layout:200", "down:10", "move:30", "crop"]);
    const native = createNativeClient<{ echo(value: string): Promise<string> }>(root, clientDescriptor).echo("mixed");
    await Promise.resolve();
    host.reply(host.nativeCalls.at(-1)!, encodeJson("reply"));
    await expect(native).resolves.toBe("reply");
    host.dispatch(initial, { type: "pointer", action: "down", x: 50, y: 20 });
    expect(received.at(-1)).toBe("down:50");
    setHeight(40);
    await Promise.resolve();
    expect(host.surface(48)!.nodes.find((node) => node.id === initial.id)!.style?.height).toBe(40);
    setHeight(undefined);
    await Promise.resolve();
    expect(host.surface(48)!.nodes.find((node) => node.id === initial.id)!.style).toBe(null);
    expect(initial.style?.height).toBe(80);
  } finally {
    root.unmount();
  }
});

test("TestHost replays creation, moves and subtree deletion without mutating previous views", async () => {
  const transport = new MemoryTransport();
  const root = createRoot(transport, { surfaceId: 41 });
  const [items, setItems] = createSignal(["a", "b", "c"]);
  try {
    root.render(() =>
      createComponent(View, {
        get children() {
          return For({
            get each() {
              return items();
            },
            children: (item) => createComponent(View, { children: createComponent(Text, { children: item }) }),
          });
        },
      }),
    );
    // Attaching after the initial render must consume the existing Snapshot too.
    const host = new TestHost(transport);
    const initial = host.surface(41)!;
    const first = initial.nodes.find((node) => node.text === "a")!;
    expect(texts(initial)).toEqual(["a", "b", "c"]);
    setItems(["c", "d", "a"]);
    await Promise.resolve();
    const moved = host.surface(41)!;
    expect(texts(moved)).toEqual(["c", "d", "a"]);
    expect(moved.nodes.find((node) => node.text === "a")!.id).toBe(first.id);
    expect(texts(initial)).toEqual(["a", "b", "c"]);
    for (const node of moved.nodes) {
      expect(node.children).toEqual(moved.nodes.filter((child) => child.parentId === node.id).map((child) => child.id));
    }
    setItems([]);
    await Promise.resolve();
    expect(texts(host.surface(41)!)).toEqual([]);
    expect(host.commits.map((commit) => commit.type)).toEqual(["snapshot", "patch", "patch"]);
  } finally {
    root.unmount();
  }
});

test("TestHost delivers press and UTF-16 Unicode selection with controlled acknowledgements", () => {
  const host = new TestHost();
  const root = createRoot(host.transport, { surfaceId: 42 });
  const [value, setValue] = createSignal("before");
  const selections: { start: number; end: number }[] = [];
  try {
    root.render(() =>
      createComponent(View, {
        children: [
          createComponent(Pressable, {
            onPress: () => setValue("pressed"),
            children: createComponent(Text, { children: "Press" }),
          }),
          createComponent(TextInput, {
            get value() {
              return value();
            },
            onChangeText: setValue,
            onSelectionChange: (selection) => selections.push(selection),
          }),
          createComponent(Text, {
            get children() {
              return value();
            },
          }),
        ],
      }),
    );
    const initial = host.surface(42)!;
    host.dispatch(
      initial.nodes.find((node) => node.kind === "Pressable")!,
      { type: "press" },
    );
    expect(texts(host.surface(42)!)).toContain("pressed");
    host.dispatch(
      host.surface(42)!.nodes.find((node) => node.kind === "TextInput")!,
      { type: "input", text: "新值🙂" },
    );
    expect(value()).toBe("新值🙂");
    const input = host.surface(42)!.nodes.find((node) => node.kind === "TextInput")!;
    expect(input.inputValue).toBe("新值🙂");
    expect(selections).toMatchObject([{ start: 4, end: 4 }]);
    expect(input.inputState).toEqual({ ackEditSeq: 1, selectionStart: 4, selectionEnd: 4 });
    expect(texts(initial)).toContain("before");
  } finally {
    root.unmount();
  }
});

test("TestHost inspects native DTO props and sends subscribed native events", () => {
  const Badge = createNativeComponent<{ label: string }, { onPress: (value: string) => void }, {}>({
    providerId: clientDescriptor.moduleId,
    catalogDigest: clientDescriptor.moduleDigest,
    buildDigest: clientDescriptor.buildDigest,
    semanticVersion: clientDescriptor.semanticVersion,
    entryId: 1,
    entryVersion: 1,
    props: ["label"],
    events: [{ id: 1, name: "press", prop: "onPress" }],
    commands: [],
    children: false,
    slots: [],
    controlled: null,
  });
  const host = new TestHost();
  const root = createRoot(host.transport, { surfaceId: 43 });
  const [label, setLabel] = createSignal("ready");
  try {
    root.render(() =>
      Badge({
        get label() {
          return label();
        },
        onPress: setLabel,
      }),
    );
    const initial = host.surface(43)!.nodes.find((node) => node.kind === "Extension")!;
    expect(host.nativeProps(initial)).toEqual({ label: "ready" });
    host.dispatch(initial, { type: "native", eventId: 1, value: "changed" });
    const changed = host.surface(43)!.nodes.find((node) => node.kind === "Extension")!;
    expect(host.nativeProps(changed)).toEqual({ label: "changed" });
    expect(host.nativeProps(initial)).toEqual({ label: "ready" });
    expect(() => host.dispatch(changed, { type: "native", eventId: 2, value: null })).toThrow("not subscribed");
  } finally {
    root.unmount();
  }
});

test("TestHost correlates native replies and errors across Surfaces without protocol imports", async () => {
  type Client = { echo(value: { label: string }): Promise<string> };
  const host = new TestHost();
  const first = createRoot(host.transport, { surfaceId: 44 });
  const second = createRoot(host.transport, { surfaceId: 45 });
  try {
    first.render(() => Text({ children: "first" }));
    second.render(() => Text({ children: "second" }));
    const one = createNativeClient<Client>(first, clientDescriptor).echo({ label: "one" });
    const two = createNativeClient<Client>(second, clientDescriptor).echo({ label: "two" });
    await Promise.resolve();
    const calls = host.nativeCalls;
    const firstCall = calls.find((call) => call.surfaceId === 44)!;
    const secondCall = calls.find((call) => call.surfaceId === 45)!;
    expect(decodeNativeRequest(firstCall.args).value).toEqual({ label: "one" });
    expect(decodeNativeRequest(secondCall.args).value).toEqual({ label: "two" });
    host.reply(secondCall, encodeJson("second result"));
    await expect(two).resolves.toBe("second result");
    host.reject(firstCall, "domain failure");
    await expect(one).rejects.toThrow("domain failure");
    expect(() => host.reply(firstCall, encodeJson("duplicate"))).toThrow("already been answered");
  } finally {
    first.unmount();
    second.unmount();
  }
});

test("TestHost keeps captured event revisions and does not retarget an old epoch", async () => {
  const host = new TestHost();
  let root = createRoot(host.transport, { surfaceId: 46, epoch: 1 });
  const calls: string[] = [];
  const [label, setLabel] = createSignal("first");
  try {
    root.render(() =>
      Pressable({
        get onPress() {
          const captured = label();
          return () => calls.push(captured);
        },
      }),
    );
    const previous = host.surface(46)!.nodes.find((node) => node.kind === "Pressable")!;
    setLabel("second");
    await Promise.resolve();
    host.dispatch(previous, { type: "press" });
    host.dispatch(
      host.surface(46)!.nodes.find((node) => node.kind === "Pressable")!,
      { type: "press" },
    );
    expect(calls).toEqual(["first", "second"]);
    root.unmount();
    root = createRoot(host.transport, { surfaceId: 46, epoch: 2 });
    root.render(() => Pressable({ onPress: () => calls.push("remounted") }));
    host.dispatch(previous, { type: "press" });
    expect(calls).toEqual(["first", "second"]);
    host.dispatch(
      host.surface(46)!.nodes.find((node) => node.kind === "Pressable")!,
      { type: "press" },
    );
    expect(calls).toEqual(["first", "second", "remounted"]);
  } finally {
    root.unmount();
  }
});
