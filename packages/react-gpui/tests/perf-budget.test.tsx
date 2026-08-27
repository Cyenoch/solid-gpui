import { decode } from "@msgpack/msgpack";
import { describe, expect, it } from "bun:test";

import {
  EVENT_DRAG,
  EVENT_PRESS,
  EVENT_SCROLL,
  EVENT_WINDOW_RESIZE,
  PROTOCOL_VERSION,
  SCROLL_DELTA_PIXELS,
  encodeFrame,
} from "../src/protocol";
import { createSurfaceHost } from "../src/surface-host";
import { MemoryTransport } from "../src/transport";
import { Pressable, View, createRoot } from "../src/index";

const EVENT_COUNT = 2_000;
const ROUTE_ITERATIONS = 1_000;

// Budgets are measured * 10 on 2026-08-26 after the final local run:
// surface route 2/8: 5.821/9.229 ms; press/scroll/drag: 1.240/2.724/3.171 ms.
const SURFACE_2_BUDGET_MS = 60;
const SURFACE_8_BUDGET_MS = 100;
const PRESS_BUDGET_MS = 15;
const SCROLL_BUDGET_MS = 30;
const DRAG_BUDGET_MS = 35;
function concat(frames: readonly Uint8Array[]): Uint8Array {
  const size = frames.reduce((total, frame) => total + frame.byteLength, 0);
  const output = new Uint8Array(size);
  let offset = 0;
  for (const frame of frames) {
    output.set(frame, offset);
    offset += frame.byteLength;
  }
  return output;
}

function firstHostNode(transport: MemoryTransport): readonly unknown[] {
  const snapshot = decode(transport.submitted[0]?.slice(4) ?? new Uint8Array()) as readonly unknown[];
  return (snapshot[6] as readonly (readonly unknown[])[]).find((node) => node[0] !== 1) as readonly unknown[];
}

function assertMeasured(name: string, elapsedMs: number, budgetMs: number, count: number): void {
  console.log(`perf_budget: ${name}: ${elapsedMs.toFixed(3)} ms / ${count} events (budget ${budgetMs} ms)`);
  expect(elapsedMs).toBeLessThan(budgetMs);
}

describe("performance budgets", () => {
  it("routes 2 and 8 surfaces within the routing budget", () => {
    const measurements: Array<[number, number]> = [];
    for (const surfaceCount of [2, 8]) {
      const transport = new MemoryTransport();
      const host = createSurfaceHost(transport);
      const roots = Array.from({ length: surfaceCount }, (_, index) => {
        const surfaceId = index + 1;
        const root = host.createRoot({ surfaceId, epoch: 1 });
        root.render(null);
        return root;
      });
      const frames = roots.map((_, index) =>
        encodeFrame([PROTOCOL_VERSION, 2, index + 1, 1, 1, 1, 1, 0, EVENT_WINDOW_RESIZE, [800, 600, 1]]),
      );
      const batch = concat(frames);
      const started = performance.now();
      for (let iteration = 0; iteration < ROUTE_ITERATIONS; iteration += 1) transport.push(batch);
      const elapsedMs = performance.now() - started;
      measurements.push([surfaceCount, elapsedMs]);
      for (const root of roots) root.unmount();
      host.dispose();
    }
    for (const [surfaceCount, elapsedMs] of measurements) {
      assertMeasured(
        `surface route ${surfaceCount}`,
        elapsedMs,
        surfaceCount === 2 ? SURFACE_2_BUDGET_MS : SURFACE_8_BUDGET_MS,
        ROUTE_ITERATIONS * surfaceCount,
      );
    }
  });

  it("decodes and dispatches press, scroll, and drag hot-path events", () => {
    const scenarios = [
      {
        name: "press",
        eventType: EVENT_PRESS,
        payload: null,
        render: (transport: MemoryTransport) => {
          const root = createRoot(transport, { surfaceId: 21, epoch: 1 });
          root.render(<Pressable onPress={() => undefined} />);
          return root;
        },
        budgetMs: PRESS_BUDGET_MS,
      },
      {
        name: "scroll",
        eventType: EVENT_SCROLL,
        payload: [7, SCROLL_DELTA_PIXELS, 1, 2, 3, 4, []],
        render: (transport: MemoryTransport) => {
          const root = createRoot(transport, { surfaceId: 22, epoch: 1 });
          root.render(<View onScroll={() => undefined} />);
          return root;
        },
        budgetMs: SCROLL_BUDGET_MS,
      },
      {
        name: "drag",
        eventType: EVENT_DRAG,
        payload: [1, "card"],
        render: (transport: MemoryTransport) => {
          const root = createRoot(transport, { surfaceId: 23, epoch: 1 });
          root.render(<View onDragOver={() => undefined} />);
          return root;
        },
        budgetMs: DRAG_BUDGET_MS,
      },
    ] as const;
    for (const scenario of scenarios) {
      const transport = new MemoryTransport();
      const root = scenario.render(transport);
      const node = firstHostNode(transport);
      const nodeId = node[0] as number;
      const listenerId = node[6] as number;
      const frames = Array.from({ length: EVENT_COUNT }, (_, index) =>
        encodeFrame([
          PROTOCOL_VERSION,
          2,
          scenario.name === "press" ? 21 : scenario.name === "scroll" ? 22 : 23,
          1,
          1,
          index + 1,
          nodeId,
          listenerId,
          scenario.eventType,
          scenario.payload,
        ] as never),
      );
      const started = performance.now();
      for (const frame of frames) transport.push(frame);
      const elapsedMs = performance.now() - started;
      assertMeasured(`${scenario.name} decode+dispatch`, elapsedMs, scenario.budgetMs, EVENT_COUNT);
      root.unmount();
    }
  });
});
