import { describe, expect, it } from "bun:test";
import { decode } from "@msgpack/msgpack";
import { MemoryTransport, Pressable, Text, View, createRoot } from "../src/index";

function snapshot(transport: MemoryTransport): readonly unknown[] {
  return decode(transport.submitted[0].slice(4)) as readonly unknown[];
}

describe("accessibility metadata", () => {
  it("encodes expanded state and heading level in the optional accessibility tail", () => {
    const transport = new MemoryTransport();
    const root = createRoot(transport, { surfaceId: 71, epoch: 72 });
    root.render(
      <View>
        <Pressable accessibilityRole="button" accessibilityLabel="Actions" accessibilityExpanded={false} />
        <Text accessibilityRole="heading" accessibilityLabel="Section" accessibilityLevel={2}>
          Section
        </Text>
      </View>,
    );
    const nodes = snapshot(transport)[6] as readonly (readonly unknown[])[];
    expect(nodes.find((node) => (node[8] as readonly unknown[] | null)?.[1] === "Actions")?.[8]).toEqual([
      2,
      "Actions",
      null,
      false,
      null,
      null,
      null,
      false,
    ]);
    expect(nodes.find((node) => (node[8] as readonly unknown[] | null)?.[1] === "Section")?.[8]).toEqual([
      6,
      "Section",
      null,
      false,
      null,
      null,
      null,
      null,
      2,
    ]);
  });

  it("requires heading role for accessibilityLevel", () => {
    const transport = new MemoryTransport();
    const root = createRoot(transport, { surfaceId: 73, epoch: 74 });
    expect(() => root.render(<View accessibilityLevel={2} />)).toThrow(
      "accessibilityLevel requires accessibilityRole=heading",
    );
  });
});
