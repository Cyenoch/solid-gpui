import { decode } from "@msgpack/msgpack";
import { describe, expect, it } from "bun:test";
import React, { useState } from "react";

import {
  EVENT_DRAG,
  EVENT_LAYOUT,
  EVENT_SCROLL,
  EVENT_VISIBLE_RANGE,
  PROTOCOL_VERSION,
  SCROLL_DELTA_PIXELS,
  encodeFrame,
} from "../src/protocol";
import { MemoryTransport } from "../src/transport";
import { Text, View, VirtualList, createRoot, type Root } from "../src/index";

type Metric = {
  readonly name: string;
  readonly treeNodes: number;
  readonly rateHz: number;
  readonly durationMs: number;
  readonly events: number;
  readonly inputBytes: number;
  readonly commits: number;
  readonly bytes: number;
  readonly elapsedMs: number;
  readonly p50Ms: number;
  readonly p99Ms: number;
};

const TREE_SIZES = [143, 10_000] as const;
const RATES = [60, 120, 240] as const;
const DURATION_MS = 1_000;
const SURFACE_IDS = { scroll: 301, drag: 302, visible: 303, layout: 304 } as const;

function sleep(ms: number): Promise<void> {
  const { promise, resolve } = Promise.withResolvers<void>();
  setTimeout(resolve, ms);
  return promise;
}

function percentile(samples: readonly number[], quantile: number): number {
  const sorted = [...samples].sort((a, b) => a - b);
  const index = Math.min(sorted.length - 1, Math.max(0, Math.ceil(sorted.length * quantile) - 1));
  return sorted[index] ?? 0;
}

function snapshotNodes(transport: MemoryTransport): readonly unknown[][] {
  const message = decode(transport.submitted[0]?.slice(4) ?? new Uint8Array()) as readonly unknown[];
  return message[6] as readonly unknown[][];
}

function listenerNode(transport: MemoryTransport, kind?: number): readonly unknown[] {
  const node = snapshotNodes(transport).find(
    (candidate) => candidate[6] !== 0 && (kind === undefined || candidate[3] === kind),
  );
  if (node === undefined) throw new Error("expected event listener node");
  return node;
}

function messageKind(frame: Uint8Array): number {
  return (decode(frame.slice(4)) as readonly unknown[])[1] as number;
}

function makeTree(treeNodes: number, state: number, onScroll: (dy: number) => void): React.ReactElement {
  const children = Array.from({ length: Math.max(0, treeNodes - 3) }, (_, index) => <View key={index} />);
  return (
    <View onScroll={(event) => onScroll(event.dy)}>
      <Text>{state}</Text>
      {children}
    </View>
  );
}

function ScrollProbe({ treeNodes }: { readonly treeNodes: number }): React.ReactElement {
  const [delta, setDelta] = useState(0);
  return makeTree(treeNodes, delta, (dy) => setDelta(Math.round(dy)));
}
function NoopProbe({ treeNodes }: { readonly treeNodes: number }): React.ReactElement {
  return makeTree(treeNodes, 0, () => undefined);
}

function DragProbe({ treeNodes }: { readonly treeNodes: number }): React.ReactElement {
  const [dragType, setDragType] = useState("idle");
  const children = Array.from({ length: Math.max(0, treeNodes - 3) }, (_, index) => <View key={index} />);
  return (
    <View onDragOver={(type) => setDragType(type)}>
      <Text>{dragType}</Text>
      {children}
    </View>
  );
}
function LayoutProbe({ treeNodes }: { readonly treeNodes: number }): React.ReactElement {
  const [width, setWidth] = useState(0);
  const children = Array.from({ length: Math.max(0, treeNodes - 3) }, (_, index) => <View key={index} />);
  return (
    <View onLayout={(frame) => setWidth(Math.round(frame.x))}>
      <Text>{width}</Text>
      {children}
    </View>
  );
}

function VisibleRangeProbe(): React.ReactElement {
  const data = Array.from({ length: 256 }, (_, index) => index);
  return (
    <VirtualList
      data={data}
      itemKey={(item) => item}
      initialNumToRender={8}
      estimatedItemSize={24}
      renderItem={(item) => <Text>{item}</Text>}
    />
  );
}

async function drive(
  name: string,
  rateHz: number,
  treeNodes: number,
  render: (transport: MemoryTransport) => Root,
  event: (index: number, node: readonly unknown[]) => Uint8Array,
  paced = true,
): Promise<Metric> {
  const transport = new MemoryTransport();
  const root = render(transport);
  const node = listenerNode(transport);
  const initialFrameCount = transport.submitted.length;
  const count = Math.max(1, Math.round((rateHz * DURATION_MS) / 1_000));
  const periodMs = 1_000 / rateHz;
  const samples: number[] = [];
  let inputBytes = 0;
  const started = performance.now();
  for (let index = 0; index < count; index += 1) {
    if (paced) {
      const due = started + index * periodMs;
      const waitMs = due - performance.now();
      if (waitMs > 0) await sleep(waitMs);
    }
    const frame = event(index, node);
    inputBytes += frame.byteLength;
    const eventStarted = performance.now();
    transport.push(frame);
    samples.push(performance.now() - eventStarted);
  }
  const elapsedMs = performance.now() - started;
  const outputFrames = transport.submitted.slice(initialFrameCount);
  const commits = outputFrames.filter((frame) => messageKind(frame) === 3).length;
  const bytes = outputFrames.reduce((total, frame) => total + frame.byteLength, 0);
  root.unmount();
  return {
    name,
    treeNodes,
    rateHz,
    durationMs: elapsedMs,
    events: count,
    inputBytes,
    commits,
    bytes,
    elapsedMs,
    p50Ms: percentile(samples, 0.5),
    p99Ms: percentile(samples, 0.99),
  };
}

function eventFrame(
  surfaceId: number,
  eventType: number,
  sequence: number,
  node: readonly unknown[],
  payload: unknown,
): Uint8Array {
  return encodeFrame([
    PROTOCOL_VERSION,
    2,
    surfaceId,
    1,
    1,
    sequence,
    node[0] as number,
    node[6] as number,
    eventType,
    payload,
  ] as never);
}

describe("event storm measurements", () => {
  it("measures scroll, drag, and visible-range commits at native event rates", async () => {
    const metrics: Metric[] = [];
    for (const treeNodes of TREE_SIZES) {
      for (const rateHz of RATES) {
        metrics.push(
          await drive(
            "scroll",
            rateHz,
            treeNodes,
            (transport) => {
              const root = createRoot(transport, { surfaceId: SURFACE_IDS.scroll, epoch: 1 });
              root.render(<ScrollProbe treeNodes={treeNodes} />);
              return root;
            },
            (index, node) =>
              eventFrame(SURFACE_IDS.scroll, EVENT_SCROLL, index + 1, node, [
                7,
                SCROLL_DELTA_PIXELS,
                index + 1,
                index + 1,
                0,
                0,
                [],
              ]),
          ),
        );
        metrics.push(
          await drive(
            "drag-over",
            rateHz,
            treeNodes,
            (transport) => {
              const root = createRoot(transport, { surfaceId: SURFACE_IDS.drag, epoch: 1 });
              root.render(<DragProbe treeNodes={treeNodes} />);
              return root;
            },
            (index, node) => eventFrame(SURFACE_IDS.drag, EVENT_DRAG, index + 1, node, [1, `card:${index}`]),
          ),
        );
      }
    }
    for (const rateHz of [120, 240]) {
      metrics.push(
        await drive(
          "scroll-burst",
          rateHz,
          10_000,
          (transport) => {
            const root = createRoot(transport, { surfaceId: SURFACE_IDS.scroll, epoch: 1 });
            root.render(<ScrollProbe treeNodes={10_000} />);
            return root;
          },
          (index, node) =>
            eventFrame(SURFACE_IDS.scroll, EVENT_SCROLL, index + 1, node, [
              7,
              SCROLL_DELTA_PIXELS,
              index + 1,
              index + 1,
              0,
              0,
              [],
            ]),
          false,
        ),
      );
      metrics.push(
        await drive(
          "drag-over-burst",
          rateHz,
          10_000,
          (transport) => {
            const root = createRoot(transport, { surfaceId: SURFACE_IDS.drag, epoch: 1 });
            root.render(<DragProbe treeNodes={10_000} />);
            return root;
          },
          (index, node) => eventFrame(SURFACE_IDS.drag, EVENT_DRAG, index + 1, node, [1, `card:${index}`]),
          false,
        ),
      );
    }
    metrics.push(
      await drive(
        "scroll-noop",
        240,
        10_000,
        (transport) => {
          const root = createRoot(transport, { surfaceId: SURFACE_IDS.scroll, epoch: 1 });
          root.render(<NoopProbe treeNodes={10_000} />);
          return root;
        },
        (index, node) =>
          eventFrame(SURFACE_IDS.scroll, EVENT_SCROLL, index + 1, node, [
            7,
            SCROLL_DELTA_PIXELS,
            index + 1,
            index + 1,
            0,
            0,
            [],
          ]),
        false,
      ),
    );
    metrics.push(
      await drive(
        "layout",
        240,
        143,
        (transport) => {
          const root = createRoot(transport, { surfaceId: SURFACE_IDS.layout, epoch: 1 });
          root.render(<LayoutProbe treeNodes={143} />);
          return root;
        },
        (index, node) => eventFrame(SURFACE_IDS.layout, EVENT_LAYOUT, index + 1, node, [index, 1, 100, 48]),
      ),
    );
    metrics.push(
      await drive(
        "visible-range",
        240,
        143,
        (transport) => {
          const root = createRoot(transport, { surfaceId: SURFACE_IDS.visible, epoch: 1 });
          root.render(<VisibleRangeProbe />);
          return root;
        },
        (index, node) => {
          const start = index % 32;
          return eventFrame(SURFACE_IDS.visible, EVENT_VISIBLE_RANGE, index + 1, node, [3, start, start + 8]);
        },
      ),
    );

    for (const metric of metrics) {
      const commitsPerSecond = (metric.commits * 1_000) / metric.elapsedMs;
      const inputBytesPerSecond = (metric.inputBytes * 1_000) / metric.elapsedMs;
      const bytesPerSecond = (metric.bytes * 1_000) / metric.elapsedMs;
      console.log(
        `perf_event_storm: ${metric.name} tree=${metric.treeNodes} rate=${metric.rateHz}Hz events=${metric.events} commits=${metric.commits} ` +
          `inputBytes=${metric.inputBytes} bytes=${metric.bytes} elapsed=${metric.elapsedMs.toFixed(1)}ms ` +
          `events/s=${((metric.events * 1_000) / metric.elapsedMs).toFixed(1)} commits/s=${commitsPerSecond.toFixed(1)} ` +
          `inputBytes/s=${inputBytesPerSecond.toFixed(0)} bytes/s=${bytesPerSecond.toFixed(0)} ` +
          `p50=${metric.p50Ms.toFixed(3)}ms p99=${metric.p99Ms.toFixed(3)}ms`,
      );
      if (metric.name === "scroll-noop") {
        expect(metric.commits).toBe(0);
      } else {
        expect(metric.commits).toBeGreaterThanOrEqual(metric.events - 1);
        expect(metric.commits).toBeLessThanOrEqual(metric.events);
      }
      if (metric.name !== "scroll-noop") {
        expect(metric.bytes).toBeGreaterThan(metric.commits * 4);
      }
      expect(metric.p99Ms).toBeLessThan(100);
    }
  }, 45_000);
});
