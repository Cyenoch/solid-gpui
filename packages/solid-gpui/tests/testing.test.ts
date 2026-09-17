import { expect, test } from "bun:test";
import { createRoot, MemoryTransport, Pressable, Text, TextInput, View } from "@solid-gpui/core";
import { createComponent, createSignal, For } from "@solid-gpui/core/runtime";
import {
  createNativeClient,
  createNativeComponent,
  decodeJson,
  encodeJson,
  type NativeClientDescriptor,
} from "@solid-gpui/core/native";
import { TestHost, type TestSurface } from "@solid-gpui/core/testing";

const texts = (surface: TestSurface) => surface.nodes.flatMap((node) => (node.text === null ? [] : [node.text]));
const clientDescriptor: NativeClientDescriptor = {
  moduleId: Array(16).fill(7),
  moduleDigest: Array(32).fill(9),
  commands: [{ id: 1, name: "echo" }],
};

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

test("TestHost delivers press and Unicode input through real listener revisions", () => {
  const host = new TestHost();
  const root = createRoot(host.transport, { surfaceId: 42 });
  const [value, setValue] = createSignal("before");
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
      { type: "input", text: "新值" },
    );
    expect(value()).toBe("新值");
    expect(host.surface(42)!.nodes.find((node) => node.kind === "TextInput")!.inputValue).toBe("新值");
    expect(texts(initial)).toContain("before");
  } finally {
    root.unmount();
  }
});

test("TestHost inspects native DTO props and sends subscribed native events", () => {
  const Badge = createNativeComponent<{ label: string }, { onPress: (value: string) => void }, {}>({
    providerId: clientDescriptor.moduleId,
    catalogDigest: clientDescriptor.moduleDigest,
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
    expect(decodeJson(firstCall.args)).toEqual({ label: "one" });
    expect(decodeJson(secondCall.args)).toEqual({ label: "two" });
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
