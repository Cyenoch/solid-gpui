import { expect, test } from "bun:test";
import { createComponent } from "../src/runtime";
import { createRoot, MemoryTransport, View, Icon, registerIconNames } from "../src/index";
import { validateStyle, type Style } from "../src/style";
import { Envelope } from "../src/protocol/generated/protocol";

test("migration styles survive producer validation and the wire with explicit zero overrides", () => {
  const style: Style = {
    padding: 12,
    paddingLeft: 88,
    paddingTop: 0,
    borderRadius: 8,
    borderTopRightRadius: 0,
    borderBottomRightRadius: 0,
    borderWidth: 0,
    borderBottomWidth: 1,
    borderBottomColor: "#D4688C",
    widthPercent: 50,
    minWidth: 300,
    flexWrap: "wrap",
    linearGradient: {
      angle: 180,
      stops: [
        { color: "#13121700", position: 0.2 },
        { color: "#131217FF", position: 1 },
      ],
    },
  };
  const transport = new MemoryTransport();
  const root = createRoot(transport);
  root.render(() => createComponent(View, { style }));
  const body = Envelope.decode(transport.submitted[0]!.subarray(4)).body!;
  if (body.tag !== 1) throw new Error("expected Snapshot");
  const result = body.value.nodes!.find((node) => node.style?.paddingLeft === 88)!.style!;
  expect(result.paddingTop).toBe(0);
  expect(result.borderTopRightRadius).toBe(0);
  expect(result.borderBottomWidth).toBe(1);
  expect(result.borderBottomColor).toBe(0xd4688cff);
  expect(result.flexWrap).toBe(1);
  expect(result.widthPercent).toBe(50);
  expect(result.linearGradient).toEqual({
    angle: 180,
    startColor: 0x13121700,
    startPosition: Math.fround(0.2),
    endColor: 0x131217ff,
    endPosition: 1,
  });
  root.unmount();
});

test("migration styles reject ambiguous sizes and malformed gradients before publication", () => {
  expect(() => validateStyle({ width: 100, widthPercent: 50 })).toThrow("mutually exclusive");
  expect(() => validateStyle({ paddingLeft: -1 })).toThrow();
  expect(() =>
    validateStyle({
      linearGradient: {
        angle: 180,
        stops: [
          { color: "#000000", position: 1 },
          { color: "#ffffff", position: 0 },
        ],
      },
    }),
  ).toThrow("increasing");
});

test("application icons remain explicitly registered and bounded", () => {
  const [name] = registerIconNames(["migration-test:brand"] as const);
  const transport = new MemoryTransport();
  const root = createRoot(transport);
  root.render(() => createComponent(Icon, { name }));
  expect(transport.submitted.length).toBe(1);
  root.unmount();
  expect(() => registerIconNames(["../escape:icon"])).toThrow();
  expect(() => registerIconNames(["lucide:play"])).toThrow("reserved");
});
