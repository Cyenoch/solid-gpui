import { describe, expect, it } from "bun:test";

import {
  MAX_FRAME_SIZE,
  PROTOCOL_VERSION,
  SNAPSHOT_KIND,
  encodeFrame,
  type Snapshot,
  type SnapshotNode,
} from "../src/protocol";
import { NodeGraph } from "../src/renderer/nodes";
import type { HostKind, HostNodeInternal, HostProps, RootOwner } from "../src/renderer/types";
import { MemoryTransport } from "../src/transport";

type Samples = {
  readonly p50Ms: number;
  readonly p99Ms: number;
};

type Measurement = {
  readonly size: number;
  readonly nodeCount: number;
  readonly construction: Samples;
  readonly encode: Samples;
  readonly total: Samples;
  readonly frameBytes: number;
  readonly payloadBytes: number;
};

const TREE_SIZES = [1_000, 5_000, 20_000] as const;
const RUNS = 8;
const SURFACE_ID = 701;
const EPOCH = 1;

const APP_STYLE = {
  flexDirection: "column",
  gap: 8,
  padding: 6,
  backgroundColor: "#18202aff",
  color: "#e8edf2ff",
} as const;
const TEXT_STYLE = { fontSize: 14, color: "#d7e2f0ff" } as const;
const RUN_STYLE = { fontSize: 13, color: "#95b8e8ff" } as const;
const ROW_STYLE = { flexDirection: "row", gap: 4, padding: 2 } as const;

function percentile(samples: readonly number[], quantile: number): number {
  const sorted = [...samples].sort((a, b) => a - b);
  const index = Math.min(sorted.length - 1, Math.max(0, Math.ceil(sorted.length * quantile) - 1));
  return sorted[index] ?? 0;
}

function summarize(samples: readonly number[]): Samples {
  return { p50Ms: percentile(samples, 0.5), p99Ms: percentile(samples, 0.99) };
}

function createGraph(targetNodes: number): NodeGraph {
  let graph: NodeGraph;
  const owner: RootOwner = {
    invalid: false,
    validationError: undefined,
    submitCommand: async () => undefined,
    submitCommandValue: async () => undefined,
    releaseDetachedFocus: (node) => graph.releaseDetachedFocus(node),
    setNodeProps: (node, props) => graph.setNodeProps(node, props),
    updateNodeProps: (node, props) => graph.updateNodeProps(node, props),
    detachFromParent: (node) => graph.detachFromParent(node),
    refreshChildIndexes: (parent) => graph.refreshChildIndexes(parent),
    detachSubtree: (node) => graph.detachSubtree(node),
    markMoved: (node) => graph.markMoved(node, false),
    markUpdated: (node, mask) => graph.markUpdated(node, mask, false),
    markDeleted: (node) => graph.markDeleted(node, false),
  };
  graph = new NodeGraph(owner);

  const append = (parent: HostNodeInternal, kind: HostKind, props: HostProps = {}): HostNodeInternal => {
    const node = graph.allocateNode(kind);
    graph.setNodeProps(node, props);
    node.parent = parent;
    node.index = parent.children.length;
    parent.children.push(node);
    return node;
  };
  const appendRawText = (parent: HostNodeInternal, text: string): HostNodeInternal => {
    const node = append(parent, "RawText");
    node.text = text;
    return node;
  };

  const app = append(graph.syntheticRoot, "View", {
    style: APP_STYLE,
    focusable: true,
    onScroll: () => undefined,
    accessibilityRole: "generic",
    accessibilityLabel: "synthetic snapshot root",
  });

  const paragraph = append(app, "Text", { style: TEXT_STYLE, selectable: true, onPress: () => undefined });
  appendRawText(paragraph, "A mixed synthetic paragraph with ");
  const emphasizedRun = append(paragraph, "Text", { style: RUN_STYLE });
  appendRawText(emphasizedRun, "nested text runs");
  appendRawText(paragraph, " for snapshot assembly.");

  append(app, "TextInput", {
    value: "search query",
    placeholder: "Filter rows",
    multiline: false,
    onChangeText: () => undefined,
    onSelectionChange: () => undefined,
  });

  const list = append(app, "VirtualList", {
    style: { height: 400, flexGrow: 0 },
    __itemCount: 100_000,
    __rangeStart: 0,
    __rangeEnd: 16,
    __estimatedItemSize: 24,
    __overscan: 3,
    __onVisibleRange: () => undefined,
  });
  for (let row = 0; row < 16; row += 1) {
    const rowView = append(list, "View", { style: row % 2 === 0 ? ROW_STYLE : undefined });
    const rowText = append(rowView, "Text", { style: TEXT_STYLE });
    appendRawText(rowText, `Virtual row ${row}`);
  }

  let fillerIndex = 0;
  while (graph.nodesById.size < targetNodes) {
    const remaining = targetNodes - graph.nodesById.size;
    if (remaining >= 2 && fillerIndex % 7 === 0) {
      const text = append(app, "Text", {
        style: fillerIndex % 14 === 0 ? TEXT_STYLE : undefined,
        selectable: fillerIndex % 21 === 0,
      });
      appendRawText(text, `item ${fillerIndex}`);
    } else if (fillerIndex % 17 === 0) {
      append(app, "Pressable", {
        focusable: true,
        onPress: () => undefined,
        tooltip: `item ${fillerIndex}`,
      });
    } else {
      append(app, "View", { style: fillerIndex % 23 === 0 ? ROW_STYLE : undefined });
    }
    fillerIndex += 1;
  }

  return graph;
}

function snapshot(nodes: readonly SnapshotNode[]): Snapshot {
  return [PROTOCOL_VERSION, SNAPSHOT_KIND, SURFACE_ID, EPOCH, 0, 1, nodes];
}

function measure(size: number): Measurement {
  const graph = createGraph(size);
  expect(graph.nodesById.size).toBe(size);

  // Warm up MessagePack/JIT and style encoding; each measured operation still assembles
  // a fresh wire-node array, matching RootContainer.commit's initial snapshot path.
  const warmup = graph.snapshotNodes();
  encodeFrame(snapshot(warmup));

  const constructionSamples: number[] = [];
  const encodeSamples: number[] = [];
  const totalSamples: number[] = [];
  const transport = new MemoryTransport();
  let frameBytes = 0;
  let payloadBytes = 0;

  for (let run = 0; run < RUNS; run += 1) {
    let started = performance.now();
    const nodes = graph.snapshotNodes();
    constructionSamples.push(performance.now() - started);

    const wireSnapshot = snapshot(nodes);
    started = performance.now();
    const frame = encodeFrame(wireSnapshot);
    encodeSamples.push(performance.now() - started);
    frameBytes = frame.byteLength;
    payloadBytes = frame.byteLength - 4;

    started = performance.now();
    const totalFrame = encodeFrame(snapshot(graph.snapshotNodes()));
    transport.submit(totalFrame);
    totalSamples.push(performance.now() - started);
  }

  expect(transport.submitted).toHaveLength(RUNS);
  expect(frameBytes).toBeGreaterThan(4);
  expect(payloadBytes).toBeLessThanOrEqual(MAX_FRAME_SIZE);
  return {
    size,
    nodeCount: graph.nodesById.size,
    construction: summarize(constructionSamples),
    encode: summarize(encodeSamples),
    total: summarize(totalSamples),
    frameBytes,
    payloadBytes,
  };
}

function formatMeasurement(measurement: Measurement): string {
  return [
    `perf_snapshot_encode: tree=${measurement.size} nodes=${measurement.nodeCount}`,
    `construction=${measurement.construction.p50Ms.toFixed(3)}/${measurement.construction.p99Ms.toFixed(3)}ms`,
    `encode=${measurement.encode.p50Ms.toFixed(3)}/${measurement.encode.p99Ms.toFixed(3)}ms`,
    `total=${measurement.total.p50Ms.toFixed(3)}/${measurement.total.p99Ms.toFixed(3)}ms`,
    `frameBytes=${measurement.frameBytes} payloadBytes=${measurement.payloadBytes}`,
  ].join(" ");
}

describe("snapshot encode measurements", () => {
  it("measures mixed snapshot construction, frame encoding, and transport wire size", () => {
    const measurements = TREE_SIZES.map(measure);
    for (const measurement of measurements) console.log(formatMeasurement(measurement));

    const largest = measurements[measurements.length - 1];
    expect(largest).toBeDefined();
    // This is a sanity guard, not a benchmark claim: the first local run was
    // measured before choosing this 3x-headroom bound for CI variability.
    expect(largest!.total.p50Ms).toBeLessThan(150);
  }, 45_000);
});
