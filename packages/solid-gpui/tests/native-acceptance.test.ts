import { expect, test } from "bun:test";
import { createRoot, Pressable, Text, TextInput, View, VirtualList } from "@solid-gpui/core";
import { createSignal } from "@solid-gpui/core/runtime";
import { NativeAcceptance } from "@solid-gpui/core/testing";
import { COMMAND_RESIZE_WINDOW, encodeFrame } from "../src/protocol";

const binary = process.env.SOLID_GPUI_ACCEPTANCE_BINARY;
const native = binary ? test : test.skip;

async function checkResizeFeedback(mode: "deterministic" | "gpu") {
  const host = await NativeAcceptance.launch({ command: [binary!], mode });
  let viewport: number[] = [];
  let scale: number | undefined;
  const root = createRoot(host.transport, {
    surfaceId: 1,
    onWindowResize: (width, height, scaleFactor) => {
      viewport = [width, height];
      scale = scaleFactor;
    },
  });
  try {
    root.render(() => View({ children: Text({ children: "Resize feedback" }) }));
    await host.flush();
    await host.resize(400, 300);
    expect(viewport).toEqual([400, 300]);
    expect((await host.locate({ text: "Resize feedback" })).bounds.x).toBeGreaterThanOrEqual(0);
    if (mode === "gpu") {
      if (scale === undefined) throw new Error("Native resize feedback omitted the display scale");
      const screenshot = await host.screenshot();
      expect(screenshot.width).toBe(400 * scale);
      expect(screenshot.height).toBe(300 * scale);
    }
  } finally {
    root.unmount();
    await host.close();
  }
}
native("native resize publishes production viewport feedback", () => checkResizeFeedback("deterministic"));
const gpu = binary && process.env.SOLID_GPUI_ACCEPTANCE_GPU === "1" ? test : test.skip;
gpu("macOS GPU resize completes the native bounds change before acknowledgement", () => checkResizeFeedback("gpu"));

native("a retired command reply cannot settle a new epoch request with the same ID", async () => {
  const host = await NativeAcceptance.launch({ command: [binary!], mode: "deterministic" });
  let root = createRoot(host.transport, { surfaceId: 1 });
  try {
    root.render(() => Text({ children: "Old epoch" }));
    await host.flush();
    root.unmount();
    root = createRoot(host.transport, { surfaceId: 1, epoch: 2 });
    root.render(() => Text({ children: "New epoch" }));
    host.transport.submit(
      encodeFrame({
        type: "command",
        surfaceId: 1,
        epoch: 1,
        afterRevision: 1,
        requestId: 1,
        nodeId: 1,
        command: COMMAND_RESIZE_WINDOW,
        payload: { type: "window-size", width: 400, height: 300 },
      }),
    );
    const current = root.resize(600, 400);
    await host.flush();
    await current;
  } finally {
    root.unmount();
    await host.close();
  }
});

native("public acceptance drives native paint, hit testing, Unicode edits, drag, wheel and teardown", async () => {
  const host = await NativeAcceptance.launch({ command: [binary!], mode: "deterministic" });
  let root = createRoot(host.transport, { surfaceId: 1 });
  const [value, setValue] = createSignal("");
  const [mounted, setMounted] = createSignal(true);
  let presses = 0;
  let moves = 0;
  try {
    root.render(() => {
      const content = View({
        style: { flexDirection: "column" },
        children: [
          Pressable({
            accessibilityLabel: "Apply",
            onPress: () => {
              presses++;
              setValue("painted");
            },
            style: { height: 40, width: 120 },
            children: Text({ children: "Apply" }),
          }),
          TextInput({
            accessibilityLabel: "Draft",
            get value() {
              return value();
            },
            onChangeText: setValue,
            style: { width: 220, height: 40 },
          }),
          View({ accessibilityLabel: "Drag", style: { width: 220, height: 40 }, onPointerMove: () => moves++ }),
          View({
            accessibilityLabel: "History",
            style: { height: 100, width: 220, overflow: "scroll" },
            children: Array.from({ length: 20 }, (_, i) =>
              Text({ children: `row ${i}`, style: { height: 30, flexShrink: 0 } }),
            ),
          }),
          VirtualList({
            data: Array.from({ length: 100 }, (_, i) => i),
            itemKey: (i) => i,
            estimatedItemSize: 30,
            renderItem: (i) => Text({ children: `virtual ${i}`, style: { height: 30, flexShrink: 0 } }),
            style: { width: 220, height: 100, flexShrink: 0 },
          }),
          Text({
            accessibilityLabel: "Select",
            selectable: true,
            children: "Native selection follows drag",
            style: { width: 300, height: 40 },
          }),
        ],
      });
      return View({
        get children() {
          return mounted() ? content : [];
        },
      });
    });
    const button = await host.locate({ label: "Apply" });
    expect(button.bounds.width).toBe(120);
    await host.click(button);
    expect(presses).toBe(1);
    const identity = await host.locate({ label: "Apply" });
    expect(identity.id).toBe(button.id);
    const draft = await host.locate({ label: "Draft" });
    expect(draft.text).toBe("painted");
    await host.click(draft);
    await host.key("secondary-a");
    await host.type("新值🙂");
    expect(value()).toBe("新值🙂");
    expect((await host.locate({ label: "Draft" })).text).toBe("新值🙂");
    expect((await host.locate({ label: "Draft" })).input).toMatchObject({
      value: "新值🙂",
      selectionStart: 4,
      selectionEnd: 4,
      editSeq: 3,
    });
    await host.key("secondary-a");
    await host.key("secondary-c");
    expect(await host.clipboardText()).toBe("新值🙂");
    const drag = await host.locate({ label: "Drag" });
    await host.drag(drag, { x: drag.bounds.x + 100, y: drag.bounds.y + 20 });
    expect(moves).toBeGreaterThan(0);
    const before = (await host.snapshot()).nodes.find((node) => node.text === "row 0")!;
    await host.wheel(await host.locate({ label: "History" }), { x: 0, y: -90 });
    const after = (await host.snapshot()).nodes.find((node) => node.id === before.id);
    expect(after === undefined || after.bounds.y < before.bounds.y).toBe(true);
    const list = (await host.snapshot()).nodes.find((node) => node.kind === 6)!;
    expect(list.scrollOffset).toBe(0);
    await host.wheel(list, { x: 0, y: -90 });
    expect((await host.locate({ id: list.id })).scrollOffset).toBeGreaterThan(0);
    const paragraph = await host.locate({ label: "Select" });
    await host.drag(
      paragraph,
      { x: paragraph.bounds.x + 140, y: paragraph.bounds.y + 10 },
      { from: { x: paragraph.bounds.x + 2, y: paragraph.bounds.y + 10 } },
    );
    expect((await host.locate({ label: "Select" })).selectedText?.length).toBeGreaterThan(0);
    await host.key("secondary-c");
    expect((await host.clipboardText())?.length).toBeGreaterThan(0);
    await expect(host.click(button)).rejects.toThrow("stale revision");
    await host.click(await host.locate({ label: "Apply" }));
    expect(presses).toBe(2);
    await expect(host.screenshot()).rejects.toThrow("unsupported");
    setMounted(false);
    await host.flush();
    await expect(host.click(button)).rejects.toThrow();
    expect(presses).toBe(2);
    root.unmount();
    root = createRoot(host.transport, { surfaceId: 1, epoch: 2 });
    root.render(() =>
      Pressable({ accessibilityLabel: "Apply", style: { width: 120, height: 40 }, onPress: () => presses++ }),
    );
    await host.flush();
    await expect(host.click(identity)).rejects.toThrow("stale epoch");
    await host.click(await host.locate({ label: "Apply" }));
    expect(presses).toBe(3);
  } finally {
    root.unmount();
    expect(await host.close()).toEqual({ surfaces: 0, windows: 0, popups: 0 });
  }
  await expect(host.snapshot()).rejects.toThrow("closed");
});

native("native clicks on painted text reach their Pressable ancestor through hit testing", async () => {
  const host = await NativeAcceptance.launch({ command: [binary!], mode: "deterministic" });
  const root = createRoot(host.transport, { surfaceId: 1 });
  let presses = 0;
  try {
    root.render(() =>
      View({
        style: { width: 200, height: 80 },
        children: [
          Pressable({
            onPress: () => presses++,
            style: { width: 160, height: 50 },
            children: Text({ children: "Child target" }),
          }),
        ],
      }),
    );
    const child = await host.locate({ text: "Child target" });
    expect(child.listenerId).toBe(0);
    await host.click(child);
    expect(presses).toBe(1);
  } finally {
    root.unmount();
    await host.close();
  }
});

gpu("macOS GPU acceptance captures changed production pixels", async () => {
  const host = await NativeAcceptance.launch({ command: [binary!], mode: "gpu" });
  const root = createRoot(host.transport, { surfaceId: 1 });
  const [color, setColor] = createSignal("#ff0000");
  try {
    root.render(() =>
      Pressable({
        accessibilityLabel: "Color",
        style: { width: 100, height: 100 },
        onPress: () => setColor("#0000ff"),
        children: View({
          get style() {
            return { width: 80, height: 80, backgroundColor: color() };
          },
        }),
      }),
    );
    await host.flush();
    const before = await host.screenshot();
    await host.click(await host.locate({ label: "Color" }));
    const after = await host.screenshot();
    expect(before.width).toBeGreaterThan(0);
    expect(before.png).not.toEqual(after.png);
  } finally {
    root.unmount();
    await host.close();
  }
});

native("acceptance executable requires explicit mode and rejects unsupported modes", async () => {
  const child = Bun.spawn([binary!], { stdout: "pipe", stderr: "pipe" });
  expect(await child.exited).not.toBe(0);
  expect(await new Response(child.stderr).text()).toContain("requires --native-acceptance");
  const unsupported = Bun.spawn([binary!, "--native-acceptance", "unknown"], { stdout: "pipe", stderr: "pipe" });
  expect(await unsupported.exited).not.toBe(0);
  expect(await new Response(unsupported.stderr).text()).toContain("unsupported acceptance mode");
  if (process.platform !== "darwin") {
    await expect(NativeAcceptance.launch({ command: [binary!], mode: "gpu" })).rejects.toThrow(
      "unsupported on this platform",
    );
  }
});
