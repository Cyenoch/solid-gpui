import { decode } from "@msgpack/msgpack";
import { describe, expect, it } from "bun:test";
import { MemoryTransport, Text, View, createAppearanceStore, createRoot, useAppearance } from "../src/index";
import { EVENT_WINDOW_APPEARANCE, PROTOCOL_VERSION, decodeEvent, encodeFrame } from "../src/protocol";

function appearanceFrame(
  surfaceId: number,
  epoch: number,
  sequence: number,
  nodeId: number,
  value: unknown,
): Uint8Array {
  return encodeFrame([
    PROTOCOL_VERSION,
    2,
    surfaceId,
    epoch,
    1,
    sequence,
    nodeId,
    0,
    EVENT_WINDOW_APPEARANCE,
    value,
  ] as never);
}

function textInSubmitted(transport: MemoryTransport, value: string): boolean {
  return transport.submitted.some((frame) => JSON.stringify(decode(frame.slice(4))).includes(value));
}

describe("window appearance observation", () => {
  it("routes valid root appearance events and rejects invalid payloads", () => {
    const transport = new MemoryTransport();
    const observed: string[] = [];
    const root = createRoot(transport, {
      surfaceId: 41,
      epoch: 42,
      onAppearance: (appearance) => observed.push(appearance),
    });
    root.render(<View />);
    transport.push(appearanceFrame(41, 42, 1, 1, "dark"));
    transport.push(appearanceFrame(41, 42, 2, 1, "light"));
    expect(observed).toEqual(["dark", "light"]);
    expect(decodeEvent(appearanceFrame(41, 42, 3, 2, "dark").slice(4))).toBeNull();
    expect(decodeEvent(appearanceFrame(41, 42, 4, 1, true).slice(4))).toBeNull();
    root.unmount();
  });

  it("bridges RootOptions.onAppearance through createAppearanceStore/useAppearance", () => {
    const transport = new MemoryTransport();
    const store = createAppearanceStore();
    const root = createRoot(transport, {
      surfaceId: 43,
      epoch: 44,
      onAppearance: (appearance) => store.set(appearance),
    });
    function Probe() {
      return <Text>{useAppearance(store)}</Text>;
    }
    root.render(<Probe />);
    expect(textInSubmitted(transport, "light")).toBe(true);
    transport.push(appearanceFrame(43, 44, 1, 1, "dark"));
    expect(textInSubmitted(transport, "dark")).toBe(true);
    root.unmount();
  });
});
