import { expect, test } from "bun:test";
import { createRoot, MemoryTransport, View, Pressable, createStyleSheet } from "../src/index";
import { createComponent, createSignal } from "../src/runtime";
import { Envelope } from "../src/protocol/generated/protocol";
import { validateStyle, type Style } from "../src/style";

test("style conveniences lower with side over axis over all precedence and retain explicit units", () => {
  const transport = new MemoryTransport();
  const root = createRoot(transport);
  root.render(() =>
    createComponent(View, {
      style: {
        padding: 12,
        paddingX: 8,
        paddingY: 4,
        paddingLeft: 0,
        margin: 9,
        marginX: 3,
        marginTop: 0,
        width: { unit: "percent", value: 50 },
        height: "auto",
        minWidth: { unit: "rem", value: 2 },
        flexBasis: { unit: "px", value: 40 },
        overflow: "hidden",
        overflowY: "scroll",
      },
    }),
  );
  const body = Envelope.decode(transport.submitted[0]!.subarray(4)).body!;
  if (body.tag !== 1) throw new Error("expected Snapshot");
  expect(body.value.nodes!.find((node) => node.style?.paddingLeft === 0)!.style).toMatchObject({
    paddingTop: 4,
    paddingRight: 8,
    paddingBottom: 4,
    paddingLeft: 0,
    marginTop: 0,
    marginRight: 3,
    marginBottom: 9,
    marginLeft: 3,
    width: 50,
    widthUnit: 2,
    height: 0,
    heightUnit: 3,
    minWidth: 2,
    minWidthUnit: 1,
    flexBasis: { unit: 0, value: 40 },
    overflowX: 2,
    overflowY: 3,
  });
  root.unmount();
});

test("unsupported lengths and interaction styles fail before publication", () => {
  const invalid: readonly unknown[] = [
    { width: "50%" },
    { width: { unit: "em", value: 2 } },
    { width: { unit: "px", value: Infinity } },
    { paddingX: -1 },
    { hover: { width: 12 } },
    { active: { hover: { opacity: 0.5 } } },
    { transition: { durationMs: 100 }, width: { unit: "rem", value: 2 } },
    { aspectRatio: 0 },
    { toString: 3 },
  ];
  for (const style of invalid) expect(() => validateStyle(style as Style)).toThrow();
  const transport = new MemoryTransport();
  const root = createRoot(transport);
  expect(() =>
    root.render(() =>
      createComponent(View, {
        style: { focusVisible: { borderColor: "#00ff00" } },
      }),
    ),
  ).toThrow("focusable");
  expect(transport.submitted).toHaveLength(0);
  root.unmount();
});

test("hover notification demand is independent of native paint and publishes callback removal", async () => {
  const transport = new MemoryTransport();
  const root = createRoot(transport);
  const [observe, setObserve] = createSignal(false);
  const onHoverChange = () => {};
  root.render(() =>
    createComponent(Pressable, {
      onPress: () => {},
      style: { hover: { opacity: 0.8 } },
      get onHoverChange() {
        return observe() ? onHoverChange : undefined;
      },
    }),
  );
  const snapshot = Envelope.decode(transport.submitted[0]!.subarray(4)).body!;
  if (snapshot.tag !== 1) throw new Error("expected Snapshot");
  expect(snapshot.value.nodes!.find((v) => v.style?.hover)!.observesHover).toBe(false);
  for (const enabled of [true, false]) {
    setObserve(enabled);
    await Promise.resolve();
    const patch = Envelope.decode(transport.submitted.at(-1)!.subarray(4)).body!;
    if (patch.tag !== 3) throw new Error("expected Patch");
    const update = patch.value.operations!.find((v) => v.operation?.tag === 2)!.operation!;
    expect(update.tag === 2 && update.value.observesHover).toBe(enabled);
    expect(update.tag === 2 && (update.value.mask! & 1024) !== 0).toBe(true);
  }
  root.unmount();
});

test("disabled controls retain an explicit native disabled state and immutable refinements", () => {
  const hover = { backgroundColor: "#112233" };
  const styles = createStyleSheet({ action: { hover } });
  hover.backgroundColor = "#445566";
  expect(styles.action.hover.backgroundColor).toBe("#112233");
  const transport = new MemoryTransport();
  const root = createRoot(transport);
  root.render(() =>
    createComponent(Pressable, {
      disabled: true,
      focusable: true,
      style: {
        ...styles.action,
        focusVisible: { borderColor: "#00ff00" },
      },
    }),
  );
  const body = Envelope.decode(transport.submitted[0]!.subarray(4)).body!;
  if (body.tag !== 1) throw new Error("expected Snapshot");
  const node = body.value.nodes!.find((v) => v.style?.hover)!;
  expect(node.accessibility?.disabled).toBe(true);
  expect(node.focusable).toBe(false);
  root.unmount();
});

test("changing only a native refinement publishes the style update", async () => {
  const transport = new MemoryTransport();
  const root = createRoot(transport);
  const [color, setColor] = createSignal("#112233");
  root.render(() =>
    createComponent(Pressable, {
      get style() {
        return { hover: { backgroundColor: color() } };
      },
    }),
  );
  setColor("#445566");
  await Promise.resolve();
  const body = Envelope.decode(transport.submitted[1]!.subarray(4)).body!;
  if (body.tag !== 3) throw new Error("expected Patch");
  const update = body.value.operations!.find((operation) => operation.operation?.tag === 2)!.operation!;
  expect(update.tag === 2 && update.value.style?.hover?.backgroundColor).toBe(0x445566ff);
  root.unmount();
});
