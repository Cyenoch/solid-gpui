import { expect, test } from "bun:test";
import { mountApplication, MemoryTransport, Pressable, Text, type MountedApplication } from "../src/index";
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

test.each(["quit", "keep-alive"] as const)(
  "%s policy retains app ownership until host termination, including root close and reload",
  (lastWindowClose) => {
    const transport = new ObservedTransport();
    const seen: string[] = [];
    let cleaned = 0;
    const mount = () =>
      mountApplication({
        hotKey: `test:zero-window:${lastWindowClose}`,
        lastWindowClose,
        transport: () => transport,
        setup() {
          onCleanup(() => cleaned++);
          return {
            render: () => createComponent(Text, { children: "Workspace" }),
            onActivate: ({ reason }: { reason: string }) => seen.push(reason),
          };
        },
      });
    let app = mount();
    const activate = (epoch: number, sequence: number, targetSurfaceId: number) =>
      transport.push(
        encodeFrame({
          type: "event",
          surfaceId: 0,
          epoch,
          revision: 0,
          sequence,
          nodeId: 0,
          listenerId: 0,
          payload: { type: "application-activation", targetSurfaceId, reason: "reopen", urls: [] },
        }),
      );
    activate(1, 1, 1);
    transport.push(
      encodeFrame({
        type: "event",
        surfaceId: 1,
        epoch: 1,
        revision: 1,
        sequence: 1,
        nodeId: 0,
        listenerId: 0,
        payload: { type: "surface-closed" },
      }),
    );
    expect(app.root).toBeUndefined();
    expect(transport.subscriptions).toBe(1);
    expect(cleaned).toBe(0);
    const beforeReload = transport.submitted.length;
    app = mount();
    expect(app.root).toBeUndefined();
    expect(
      transport.submitted.slice(beforeReload).every((frame) => Envelope.decode(frame.subarray(4)).body?.tag === 4),
    ).toBe(true);
    expect(cleaned).toBe(1);
    activate(2, 2, 2);
    expect(app.root).toBeDefined();
    const snapshots = transport.submitted
      .map((frame) => Envelope.decode(frame.subarray(4)).body)
      .filter((body) => body?.tag === 1);
    expect(snapshots.map((body) => body?.tag === 1 && body.value.surfaceId)).toEqual([1, 2]);
    const afterReopen = transport.submitted.length;
    activate(2, 2, 2);
    activate(1, 3, 1);
    expect(transport.submitted).toHaveLength(afterReopen);
    expect(seen).toEqual(["reopen", "reopen"]);
    app.quit();
    expect(transport.subscriptions).toBe(0);
    expect(cleaned).toBe(2);
    const last = Envelope.decode(transport.submitted.at(-1)!.subarray(4)).body;
    expect(last?.tag === 4 && last.value.payload?.tag === 14 && last.value.payload.value.quit).toBe(true);
  },
);

test("a managed generation receives its state handoff without capturing the retired generation again", () => {
  const scope = globalThis as { __solidGpuiGeneration?: unknown };
  const previousGeneration = scope.__solidGpuiGeneration;
  const transport = new MemoryTransport();
  const hotKey = "test:application-managed-handoff";
  let captures = 0;
  const setup = (previous = 0) => {
    const [count] = createSignal(previous);
    return {
      render: () => createComponent(Text, { children: String(count()) }),
      captureState: () => {
        captures++;
        return count();
      },
    };
  };
  const first = mountApplication<number>({ hotKey, transport: () => transport, setup });
  scope.__solidGpuiGeneration = {
    epoch: 2,
    state: JSON.stringify({ state: [7], surfaceId: 1, open: true, activationSequence: 0 }),
    register() {},
  };
  let second: MountedApplication | undefined;
  try {
    second = mountApplication<number>({ hotKey, transport: () => transport, setup });
    // The handoff is authoritative: the retired generation is not asked again.
    expect(captures).toBe(0);
    const snapshots = transport.submitted
      .map((frame) => Envelope.decode(frame.subarray(4)).body)
      .filter((body) => body?.tag === 1);
    const snapshot = snapshots.at(-1);
    expect(snapshot?.tag === 1 && snapshot.value.epoch).toBe(2);
    expect(snapshot?.tag === 1 && snapshot.value.nodes?.some((node) => node.text === "7")).toBe(true);
  } finally {
    second?.dispose();
    first.dispose();
    if (previousGeneration) scope.__solidGpuiGeneration = previousGeneration;
    else delete scope.__solidGpuiGeneration;
  }
});

test("a Surface opened after activation publishes its first Snapshot or reports the failure", () => {
  const transport = new MemoryTransport();
  const hotKey = "test:application-reopened-surface";
  const seen: string[] = [];
  let empty = false;
  const mount = () =>
    mountApplication({
      hotKey,
      lastWindowClose: "keep-alive",
      transport: () => transport,
      setup: () => ({
        rootOptions: {
          onTransportTermination: (error) =>
            seen.push(error.cause && "detail" in error.cause ? error.cause.detail : error.message),
        },
        render: () => (empty ? null : createComponent(Text, { children: "Workspace" })),
      }),
    });
  const activate = (epoch: number, sequence: number, targetSurfaceId: number) =>
    transport.push(
      encodeFrame({
        type: "event",
        surfaceId: 0,
        epoch,
        revision: 0,
        sequence,
        nodeId: 0,
        listenerId: 0,
        payload: { type: "application-activation", targetSurfaceId, reason: "reopen", urls: [] },
      }),
    );
  let app = mount();
  activate(1, 1, 1);
  transport.push(
    encodeFrame({
      type: "event",
      surfaceId: 1,
      epoch: 1,
      revision: 1,
      sequence: 1,
      nodeId: 0,
      listenerId: 0,
      payload: { type: "surface-closed" },
    }),
  );
  expect(app.root).toBeUndefined();
  empty = true;
  app = mount();
  activate(2, 2, 2);
  // The reopened window is never acknowledged without its first Snapshot.
  expect(app.root).toBeUndefined();
  expect(seen).toEqual(["application must render a nonempty initial tree"]);
  expect(
    transport.submitted
      .map((frame) => Envelope.decode(frame.subarray(4)).body)
      .filter((body) => body?.tag === 1)
      .map((body) => body?.tag === 1 && body.value.surfaceId),
  ).toEqual([1]);
});
