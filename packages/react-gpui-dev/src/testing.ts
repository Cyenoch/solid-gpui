import { decode, encode } from "@msgpack/msgpack";
import {
  MemoryTransport,
  createRoot,
  type HostKind,
  type HostNode,
  type KeyEvent,
  type Root,
  type RootOptions,
} from "@react-gpui/core";
import type { ReactElement } from "react";

// These tags intentionally mirror the public v3 protocol constants in
// @react-gpui/core/src/protocol.ts:11-61. The testing package cannot import
// that non-exported module; every helper is exercised through a real core Root
// and MemoryTransport in testing.test.tsx, and the guard test reuses any
// constants later exposed by the core package entrypoint.
const PROTOCOL_VERSION = 3;
const EVENT_MESSAGE = 2;
const COMMAND_MESSAGE = 4;
const EVENT_PRESS = 1;
const EVENT_CHANGE = 2;
const EVENT_VISIBLE_RANGE = 7;
const EVENT_KEY = 9;
const EVENT_SUBMIT = 13;
const COMMAND_RESULT_EVENT = 6;
const KEY_DOWN = 1;

const HOST_KIND_BY_CODE: Record<number, HostKind> = {
  1: "View",
  2: "Text",
  3: "Pressable",
  4: "RawText",
  5: "TextInput",
  6: "VirtualList",
  7: "Image",
};

type WireNode = readonly unknown[];
type WirePayload = readonly unknown[];
/** A normalized retained Host Node used by test event helpers. */
export interface TestNode extends HostNode {
  readonly parentId: number;
  readonly index: number;
  readonly listenerId: number;
  readonly style: readonly unknown[] | null;
  readonly text: string | null;
  readonly hostProperties: readonly unknown[] | null;
  readonly accessibility: readonly unknown[] | null;
  readonly focusable: boolean;
  readonly raw: WireNode;
}

export type TestNodePredicate = (node: TestNode) => boolean;

export interface TestNodeHandle extends TestNode {}

export interface CommandResultOptions {
  readonly command?: number;
  readonly nodeId?: number;
  readonly success?: boolean;
  readonly error?: string | null;
  readonly value?: number | readonly [number, number] | readonly string[] | boolean | string | null;
}

export interface RenderOptions {
  readonly surfaceId?: number;
  readonly epoch?: number;
  readonly maxFrameSize?: number;
  readonly onWindowResize?: RootOptions["onWindowResize"];
  readonly onWindowActivation?: RootOptions["onWindowActivation"];
}

export interface RenderResult {
  readonly root: Root;
  readonly frames: readonly Uint8Array[];
  commits(): readonly unknown[];
  node(kind: HostKind, propsPredicate?: TestNodePredicate): TestNodeHandle;
  press(handle: TestNodeHandle): void;
  key(handle: TestNodeHandle, event: KeyEvent): void;
  input(handle: TestNodeHandle, text: string): void;
  submit(handle: TestNodeHandle, text?: string | null): void;
  visibleRange(handle: TestNodeHandle, start: number, end: number): void;
  commandResult(requestId: number, options?: CommandResultOptions): void;
  dispatchFrame(rawEvent: Uint8Array | ArrayBuffer): void;
  unmount(): void;
}

function frame(payload: unknown): Uint8Array {
  const encoded = encode(payload, { sortKeys: false, forceFloat32: true });
  const result = new Uint8Array(encoded.byteLength + 4);
  new DataView(result.buffer).setUint32(0, encoded.byteLength, true);
  result.set(encoded, 4);
  return result;
}

function asPayload(value: unknown): WirePayload | null {
  return Array.isArray(value) ? value : null;
}

function normalizedNode(raw: WireNode): TestNode | undefined {
  const kind = HOST_KIND_BY_CODE[Number(raw[3])];
  if (kind === undefined) return undefined;
  return {
    id: Number(raw[0]),
    kind,
    parentId: Number(raw[1]),
    index: Number(raw[2]),
    style: (raw[4] as readonly unknown[] | null) ?? null,
    text: (raw[5] as string | null) ?? null,
    listenerId: Number(raw[6]),
    hostProperties: (raw[7] as readonly unknown[] | null) ?? null,
    accessibility: (raw[8] as readonly unknown[] | null) ?? null,
    focusable: raw[9] === true,
    raw,
  };
}

function applyUpdate(nodes: Map<number, WireNode>, operation: WireNode): void {
  const id = Number(operation[1]);
  const previous = nodes.get(id);
  if (previous === undefined) return;
  const next = [...previous];
  const mask = Number(operation[2]);
  if (mask & 1) next[4] = operation[3];
  if (mask & 2) next[5] = operation[4];
  if (mask & 4) next[6] = operation[5];
  if (mask & 8) next[7] = operation[6];
  if (mask & 16) next[8] = operation[7];
  if (mask & 32) next[9] = operation[8];
  nodes.set(id, next);
}

function applyDelete(nodes: Map<number, WireNode>, id: number): void {
  const removed = new Set<number>([id]);
  let changed = true;
  while (changed) {
    changed = false;
    for (const node of nodes.values()) {
      if (removed.has(Number(node[1])) && !removed.has(Number(node[0]))) {
        removed.add(Number(node[0]));
        changed = true;
      }
    }
  }
  for (const removedId of removed) nodes.delete(removedId);
}

function latestNodes(commits: readonly unknown[]): Map<number, WireNode> {
  const nodes = new Map<number, WireNode>();
  for (const commit of commits) {
    const message = asPayload(commit);
    if (message === null) continue;
    if (message[1] === 1) {
      const snapshotNodes = asPayload(message[6]);
      if (snapshotNodes === null) continue;
      nodes.clear();
      for (const node of snapshotNodes) {
        if (Array.isArray(node)) nodes.set(Number(node[0]), node);
      }
      continue;
    }
    if (message[1] !== 3) continue;
    const operations = asPayload(message[6]);
    if (operations === null) continue;
    for (const operation of operations) {
      if (!Array.isArray(operation)) continue;
      switch (Number(operation[0])) {
        case 1:
          nodes.set(Number(operation[1]), operation.slice(1));
          break;
        case 2:
          applyUpdate(nodes, operation);
          break;
        case 3: {
          const previous = nodes.get(Number(operation[1]));
          if (previous !== undefined) {
            const next = [...previous];
            next[1] = operation[2];
            next[2] = operation[3];
            nodes.set(Number(operation[1]), next);
          }
          break;
        }
        case 4:
          applyDelete(nodes, Number(operation[1]));
          break;
      }
    }
  }
  return nodes;
}

function requireListener(handle: TestNodeHandle): void {
  if (handle.listenerId === 0)
    throw new Error(`testing helper requires a listener on ${handle.kind} node ${handle.id}`);
}

function requireKind(handle: TestNodeHandle, kind: HostKind): void {
  if (handle.kind !== kind) throw new TypeError(`testing helper expected ${kind}, received ${handle.kind}`);
}

function u32(name: string, value: number): void {
  if (!Number.isInteger(value) || value < 0 || value > 0xffff_ffff) throw new RangeError(`${name} must be a u32`);
}

function eventFrame(
  surfaceId: number,
  epoch: number,
  revision: number,
  sequence: number,
  handle: TestNodeHandle,
  eventType: number,
  payload: unknown,
): Uint8Array {
  requireListener(handle);
  return frame([
    PROTOCOL_VERSION,
    EVENT_MESSAGE,
    surfaceId,
    epoch,
    revision,
    sequence,
    handle.id,
    handle.listenerId,
    eventType,
    payload,
  ]);
}

function commandValue(value: CommandResultOptions["value"]): WirePayload | null {
  if (value === undefined || value === null) return null;
  if (Array.isArray(value)) {
    return value.every((item) => typeof item === "string") ? [5, value] : [2, value];
  }
  if (typeof value === "number") return [1, value];
  if (typeof value === "boolean") return [3, value];
  return [4, value];
}

/** Render a component against MemoryTransport with public event/command helpers. */
export function render(element: ReactElement | null, options: RenderOptions = {}): RenderResult {
  const transport = new MemoryTransport();
  const surfaceId = options.surfaceId ?? 1;
  const epoch = options.epoch ?? 1;
  const root = createRoot(transport, {
    surfaceId,
    epoch,
    maxFrameSize: options.maxFrameSize,
    onWindowResize: options.onWindowResize,
    onWindowActivation: options.onWindowActivation,
  });
  root.render(element);
  let sequence = 1;
  const editSequences = new Map<number, number>();
  let unmounted = false;

  const commits = (): readonly unknown[] =>
    transport.submitted.map((submitted) => decode(submitted.slice(4), { useBigInt64: false }));
  const result: RenderResult = {
    root,
    get frames(): readonly Uint8Array[] {
      return transport.submitted;
    },
    commits,
    node(kind, propsPredicate) {
      const nodes = latestNodes(commits());
      for (const raw of nodes.values()) {
        const node = normalizedNode(raw);
        if (node?.kind === kind && (propsPredicate === undefined || propsPredicate(node))) return node;
      }
      throw new Error(`testing node not found: ${kind}`);
    },
    press(handle) {
      requireListener(handle);
      transport.push(eventFrame(surfaceId, epoch, 1, sequence++, handle, EVENT_PRESS, null));
    },
    key(handle, event) {
      requireListener(handle);
      transport.push(
        eventFrame(surfaceId, epoch, 1, sequence++, handle, EVENT_KEY, [
          5,
          event.key,
          event.modifiers,
          actionCode(event.action),
        ]),
      );
    },
    input(handle, text) {
      requireKind(handle, "TextInput");
      requireListener(handle);
      const editSeq = (editSequences.get(handle.id) ?? 0) + 1;
      editSequences.set(handle.id, editSeq);
      const selection = text.length;
      transport.push(
        eventFrame(surfaceId, epoch, 1, sequence++, handle, EVENT_CHANGE, [
          1,
          text,
          selection,
          selection,
          null,
          null,
          editSeq,
        ]),
      );
    },
    submit(handle, text = null) {
      requireKind(handle, "TextInput");
      requireListener(handle);
      transport.push(eventFrame(surfaceId, epoch, 1, sequence++, handle, EVENT_SUBMIT, text));
    },
    visibleRange(handle, start, end) {
      requireKind(handle, "VirtualList");
      requireListener(handle);
      u32("start", start);
      u32("end", end);
      if (start > end) throw new RangeError("visible range start must not exceed end");
      transport.push(eventFrame(surfaceId, epoch, 1, sequence++, handle, EVENT_VISIBLE_RANGE, [3, start, end]));
    },
    commandResult(requestId, resultOptions = {}) {
      const command = findCommand(commits(), requestId);
      const commandKind = resultOptions.command ?? command?.[7];
      const nodeId = resultOptions.nodeId ?? command?.[6];
      if (commandKind === undefined || nodeId === undefined) {
        throw new Error(`testing command result has no captured command for request ${requestId}`);
      }
      transport.push(
        frame([
          PROTOCOL_VERSION,
          EVENT_MESSAGE,
          surfaceId,
          epoch,
          1,
          sequence++,
          Number(nodeId),
          0,
          COMMAND_RESULT_EVENT,
          [
            2,
            requestId,
            Number(commandKind),
            Number(nodeId),
            resultOptions.success ?? true,
            resultOptions.error ?? null,
            commandValue(resultOptions.value),
          ],
        ]),
      );
    },
    dispatchFrame(rawEvent) {
      transport.push(rawEvent);
    },
    unmount() {
      if (unmounted) return;
      unmounted = true;
      root.unmount();
    },
  };
  return result;
}

function actionCode(action: KeyEvent["action"]): number {
  if (action === "down") return KEY_DOWN;
  if (action === "repeat") return 2;
  return 3;
}

function findCommand(commits: readonly unknown[], requestId: number): WirePayload | undefined {
  for (const commit of [...commits].reverse()) {
    const message = asPayload(commit);
    if (message !== null && message[1] === COMMAND_MESSAGE && message[5] === requestId) return message;
  }
  return undefined;
}
