import { expect, test } from "bun:test";
import { mountApplication, MemoryTransport, Pressable, Text } from "../src/index";
import { createComponent, createSignal, onCleanup } from "../src/runtime";
import { encodeFrame } from "../src/protocol";
import { Envelope } from "../src/protocol/generated/protocol";
import type { TransportListener } from "../src/transport";

class ObservedTransport extends MemoryTransport {
  subscriptions = 0;
  override onData(listener: TransportListener): () => void {
    this.subscriptions++;
    const off = super.onData(listener);
    return () => {
      this.subscriptions--;
      off();
    };
  }
}

test("application reload stages candidates, preserves explicit state, isolates epochs and releases owners", () => {
  const transport = new ObservedTransport();
  const hotKey = "test:application-lifecycle";
  const cleaned: number[] = [];
  let attempts = 0;
  let current = mount();
  function mount(fail = false) {
    return mountApplication<number>({
      hotKey,
      transport: () => transport,
      setup(previous = 0) {
        const attempt = ++attempts;
        onCleanup(() => cleaned.push(attempt));
        const [count, setCount] = createSignal(previous);
        return {
          render: () => {
            if (fail) throw new Error("broken render");
            return createComponent(Pressable, {
              onPress: () => setCount((value) => value + 1),
              get children() {
                return createComponent(Text, {
                  get children() {
                    return String(count());
                  },
                });
              },
            });
          },
          captureState: count,
        };
      },
    });
  }
  function latestSnapshot() {
    for (const frame of [...transport.submitted].reverse()) {
      const body = Envelope.decode(frame.subarray(4)).body;
      if (body?.tag === 1) return body.value;
    }
    throw new Error("missing snapshot");
  }
  const first = latestSnapshot();
  const button = first.nodes!.find((node) => node.listenerId)!;
  function press(epoch: number, sequence: number) {
    transport.push(
      encodeFrame({
        type: "event",
        surfaceId: 1,
        epoch,
        revision: 1,
        sequence,
        nodeId: button.id!,
        listenerId: button.listenerId!,
        payload: { type: "press" },
      }),
    );
  }
  try {
    press(1, 1);
    const beforeFailure = transport.submitted.length;
    expect(() => mount(true)).toThrow("broken render");
    expect(transport.submitted).toHaveLength(beforeFailure);
    expect(cleaned).toEqual([2]);
    expect(transport.subscriptions).toBe(1);
    current = mount();
    expect(latestSnapshot().epoch).toBe(2);
    expect(latestSnapshot().nodes!.some((node) => node.text === "1")).toBe(true);
    expect(cleaned).toEqual([2, 1]);
    const beforeStale = transport.submitted.length;
    press(1, 2);
    expect(transport.submitted).toHaveLength(beforeStale);
    for (let epoch = 3; epoch <= 5; epoch++) {
      current = mount();
      expect(latestSnapshot().epoch).toBe(epoch);
      expect(transport.subscriptions).toBe(1);
    }
  } finally {
    current.dispose();
  }
  expect(transport.subscriptions).toBe(0);
  expect(cleaned.sort((a, b) => a - b)).toEqual([1, 2, 3, 4, 5, 6]);
});
