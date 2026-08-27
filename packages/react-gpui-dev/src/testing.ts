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
const EVENT_FOCUS = 4;
const EVENT_BLUR = 5;
const EVENT_VISIBLE_RANGE = 7;
const EVENT_KEY = 9;
const EVENT_POINTER = 10;
const EVENT_HOVER = 11;
const EVENT_SCROLL = 12;
const EVENT_SUBMIT = 13;
const EVENT_DRAG = 20;
const EVENT_POINTER_DOWN_OUTSIDE = 22;
const COMMAND_RESULT_EVENT = 6;
const KEY_DOWN = 1;
const KEY_REPEAT = 2;
const POINTER_DOWN = 1;
const POINTER_BUTTON_LEFT = 1;
const POINTER_BUTTON_RIGHT = 2;
const POINTER_BUTTON_MIDDLE = 3;
const POINTER_BUTTON_BACK = 4;
const POINTER_BUTTON_FORWARD = 5;
const SCROLL_PIXELS = 1;
const SCROLL_LINES = 2;
const DRAG_OVER = 1;
const DRAG_DROP = 2;
const INPUT_PAYLOAD = 1;

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

type TestAppLocator = string | TestNodePredicate | TestNodeHandle;
type TestAppKey = Pick<KeyEvent, "key" | "action"> & { readonly modifiers?: readonly string[] };
type TestAppScroll = {
  readonly dx: number;
  readonly dy: number;
  readonly deltaKind?: "pixels" | "lines";
  readonly x?: number;
  readonly y?: number;
  readonly modifiers?: readonly string[];
};
type TestAppPointer = {
  readonly action: "down" | "up";
  readonly button?: "left" | "right" | "middle" | "back" | "forward";
  readonly modifiers?: readonly string[];
  readonly clickCount?: number;
};

/** Consumer-facing behavior test facade over the real headless Root seam. */
export interface TestApp {
  readonly root: Root;
  readonly frames: readonly Uint8Array[];
  commits(): readonly unknown[];
  node(locator: TestAppLocator): TestNodeHandle;
  text(value: string | RegExp): TestNodeHandle;
  press(locator: TestAppLocator): readonly unknown[];
  hover(locator: TestAppLocator, hovered: boolean): readonly unknown[];
  key(locator: TestAppLocator, event: TestAppKey): readonly unknown[];
  input(locator: TestAppLocator, text: string): readonly unknown[];
  submit(locator: TestAppLocator, text?: string | null): readonly unknown[];
  scroll(locator: TestAppLocator, options: TestAppScroll): readonly unknown[];
  pointer(locator: TestAppLocator, options: TestAppPointer): readonly unknown[];
  dragOver(locator: TestAppLocator, dragType: string): readonly unknown[];
  drop(locator: TestAppLocator, dragType: string): readonly unknown[];
  pointerDownOutside(locator: TestAppLocator, point: { readonly x: number; readonly y: number }): readonly unknown[];
  focus(locator: TestAppLocator): readonly unknown[];
  blur(locator: TestAppLocator): readonly unknown[];
  visibleRange(locator: TestAppLocator, start: number, end: number): readonly unknown[];
  commandResult(options?: CommandResultOptions): readonly unknown[];
  commandResult(requestId: number, options?: CommandResultOptions): readonly unknown[];
  dispatchFrame(rawEvent: Uint8Array | ArrayBuffer): readonly unknown[];
  unmount(): void;
}

interface InternalRenderResult extends RenderResult {
  dispatchEvent(handle: TestNodeHandle, eventType: number, payload: unknown): void;
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
function latestCommit(commits: readonly unknown[]): readonly unknown[] {
  const commit = commits[commits.length - 1];
  if (!Array.isArray(commit)) throw new Error("testing app has no commit after dispatch");
  return commit;
}

function retainedNodes(commits: readonly unknown[]): TestNode[] {
  const nodes: TestNode[] = [];
  for (const raw of latestNodes(commits).values()) {
    const node = normalizedNode(raw);
    if (node !== undefined) nodes.push(node);
  }
  return nodes;
}

function isNodeHandle(value: unknown): value is TestNodeHandle {
  return typeof value === "object" && value !== null && "id" in value && "kind" in value;
}

function queryDescription(locator: TestAppLocator): string {
  if (typeof locator === "string") return `accessibility label ${JSON.stringify(locator)}`;
  if (typeof locator === "function") return "predicate";
  return `node id ${locator.id}`;
}

function locateNode(commits: readonly unknown[], locator: TestAppLocator): TestNodeHandle {
  const nodes = retainedNodes(commits);
  const match = isNodeHandle(locator)
    ? nodes.find((node) => node.id === locator.id)
    : typeof locator === "string"
      ? nodes.find((node) => node.accessibility?.[1] === locator)
      : nodes.find(locator);
  if (match !== undefined) return match;
  throw new Error(`testing app node not found for ${queryDescription(locator)}`);
}

function textMatches(value: string | RegExp, text: string): boolean {
  if (typeof value === "string") return text === value;
  value.lastIndex = 0;
  return value.test(text);
}

function locateText(commits: readonly unknown[], value: string | RegExp): TestNodeHandle {
  const nodes = retainedNodes(commits);
  const rawMatch = nodes.find((node) => node.kind === "RawText" && node.text !== null && textMatches(value, node.text));
  if (rawMatch !== undefined) return rawMatch;

  const textNodes = nodes.filter((node) => node.kind === "Text");
  for (const textNode of textNodes) {
    const children = nodes
      .filter((node) => node.parentId === textNode.id && node.kind === "RawText" && node.text !== null)
      .sort((left, right) => left.index - right.index);
    const text = children.map((child) => child.text ?? "").join("");
    if (textMatches(value, text)) return { ...textNode, text };
  }

  const description = typeof value === "string" ? JSON.stringify(value) : value.toString();
  throw new Error(`testing app text not found: ${description}`);
}

function finiteNumber(name: string, value: number): void {
  if (typeof value !== "number" || !Number.isFinite(value)) throw new RangeError(`${name} must be finite`);
}

function pointerButtonCode(button: NonNullable<TestAppPointer["button"]>): number {
  switch (button) {
    case "left":
      return POINTER_BUTTON_LEFT;
    case "right":
      return POINTER_BUTTON_RIGHT;
    case "middle":
      return POINTER_BUTTON_MIDDLE;
    case "back":
      return POINTER_BUTTON_BACK;
    case "forward":
      return POINTER_BUTTON_FORWARD;
  }
}

function inputEventPayload(handle: TestNodeHandle): readonly unknown[] {
  const properties = handle.hostProperties;
  if (properties === null || Number(properties[0]) !== INPUT_PAYLOAD) {
    return [INPUT_PAYLOAD, "", 0, 0, null, null, 0, false];
  }
  const text = typeof properties[1] === "string" ? properties[1] : "";
  const start = typeof properties[7] === "number" ? properties[7] : 0;
  const end = typeof properties[8] === "number" ? properties[8] : start;
  const markedStart = typeof properties[9] === "number" ? properties[9] : null;
  const markedEnd = typeof properties[10] === "number" ? properties[10] : null;
  const editSeq = typeof properties[6] === "number" ? properties[6] : 0;
  const reversed = properties[12] === true;
  return [INPUT_PAYLOAD, text, start, end, markedStart, markedEnd, editSeq, reversed];
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
  const dispatchEvent = (handle: TestNodeHandle, eventType: number, payload: unknown): void => {
    transport.push(eventFrame(surfaceId, epoch, 1, sequence++, handle, eventType, payload));
  };
  const result: InternalRenderResult = {
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
    dispatchEvent,
    press(handle) {
      dispatchEvent(handle, EVENT_PRESS, null);
    },
    key(handle, event) {
      dispatchEvent(handle, EVENT_KEY, [5, event.key, event.modifiers, actionCode(event.action)]);
    },
    input(handle, text) {
      requireKind(handle, "TextInput");
      requireListener(handle);
      const editSeq = (editSequences.get(handle.id) ?? 0) + 1;
      editSequences.set(handle.id, editSeq);
      const selection = text.length;
      dispatchEvent(handle, EVENT_CHANGE, [INPUT_PAYLOAD, text, selection, selection, null, null, editSeq, false]);
    },
    submit(handle, text = null) {
      requireKind(handle, "TextInput");
      requireListener(handle);
      dispatchEvent(handle, EVENT_SUBMIT, text ?? "");
    },
    visibleRange(handle, start, end) {
      requireKind(handle, "VirtualList");
      requireListener(handle);
      u32("start", start);
      u32("end", end);
      if (start > end) throw new RangeError("visible range start must not exceed end");
      dispatchEvent(handle, EVENT_VISIBLE_RANGE, [3, start, end]);
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
function dispatchFocus(renderResult: InternalRenderResult, handle: TestNodeHandle, eventType: number): void {
  if (handle.kind === "TextInput") {
    requireListener(handle);
    renderResult.dispatchEvent(handle, eventType, inputEventPayload(handle));
    return;
  }
  if (handle.kind !== "View" && handle.kind !== "Pressable") {
    throw new TypeError(`testing app focus requires View, Pressable, or TextInput; received ${handle.kind}`);
  }
  if (!handle.focusable) throw new TypeError(`testing app focus requires a focusable ${handle.kind} node`);
  renderResult.dispatchEvent(handle, eventType, null);
}

/** Render a component behind a locator- and interaction-oriented test facade. */
export function renderTestApp(element: ReactElement | null, options: RenderOptions = {}): TestApp {
  const renderResult = render(element, options) as InternalRenderResult;
  const hovered = new Map<number, boolean>();
  const resolve = (locator: TestAppLocator): TestNodeHandle => locateNode(renderResult.commits(), locator);
  const commit = (): readonly unknown[] => latestCommit(renderResult.commits());
  const app: TestApp = {
    root: renderResult.root,
    get frames(): readonly Uint8Array[] {
      return renderResult.frames;
    },
    commits: renderResult.commits,
    node: resolve,
    text(value) {
      return locateText(renderResult.commits(), value);
    },
    press(locator) {
      const node = resolve(locator);
      requireKind(node, "Pressable");
      renderResult.press(node);
      return commit();
    },
    hover(locator, nextHovered) {
      const node = resolve(locator);
      if (node.kind !== "View" && node.kind !== "Pressable") {
        throw new TypeError(`testing app hover requires View or Pressable; received ${node.kind}`);
      }
      requireListener(node);
      const currentHovered = hovered.get(node.id) ?? false;
      if (currentHovered !== nextHovered) {
        renderResult.dispatchEvent(node, EVENT_HOVER, null);
        hovered.set(node.id, nextHovered);
      }
      return commit();
    },
    key(locator, event) {
      const node = resolve(locator);
      renderResult.key(node, {
        key: event.key,
        modifiers: [...(event.modifiers ?? [])],
        action: event.action,
      });
      return commit();
    },
    input(locator, text) {
      const node = resolve(locator);
      renderResult.input(node, text);
      return commit();
    },
    submit(locator, text = null) {
      const node = resolve(locator);
      renderResult.submit(node, text);
      return commit();
    },
    scroll(locator, options) {
      const node = resolve(locator);
      requireKind(node, "View");
      requireListener(node);
      finiteNumber("scroll dx", options.dx);
      finiteNumber("scroll dy", options.dy);
      const x = options.x ?? 0;
      const y = options.y ?? 0;
      finiteNumber("scroll x", x);
      finiteNumber("scroll y", y);
      renderResult.dispatchEvent(node, EVENT_SCROLL, [
        7,
        options.deltaKind === "lines" ? SCROLL_LINES : SCROLL_PIXELS,
        options.dx,
        options.dy,
        x,
        y,
        [...(options.modifiers ?? [])],
      ]);
      return commit();
    },
    pointer(locator, options) {
      const node = resolve(locator);
      if (node.kind !== "View" && node.kind !== "Pressable") {
        throw new TypeError(`testing app pointer requires View or Pressable; received ${node.kind}`);
      }
      requireListener(node);
      const clickCount = options.clickCount ?? 1;
      u32("clickCount", clickCount);
      if (clickCount === 0) throw new RangeError("clickCount must be greater than zero");
      renderResult.dispatchEvent(node, EVENT_POINTER, [
        6,
        pointerButtonCode(options.button ?? "left"),
        [...(options.modifiers ?? [])],
        options.action === "down" ? POINTER_DOWN : 2,
        clickCount,
      ]);
      return commit();
    },
    dragOver(locator, dragType) {
      const node = resolve(locator);
      requireListener(node);
      renderResult.dispatchEvent(node, EVENT_DRAG, [DRAG_OVER, dragType]);
      return commit();
    },
    drop(locator, dragType) {
      const node = resolve(locator);
      requireListener(node);
      renderResult.dispatchEvent(node, EVENT_DRAG, [DRAG_DROP, dragType]);
      return commit();
    },
    pointerDownOutside(locator, point) {
      const node = resolve(locator);
      requireKind(node, "View");
      requireListener(node);
      finiteNumber("pointer-down-outside x", point.x);
      finiteNumber("pointer-down-outside y", point.y);
      renderResult.dispatchEvent(node, EVENT_POINTER_DOWN_OUTSIDE, [8, point.x, point.y]);
      return commit();
    },
    focus(locator) {
      dispatchFocus(renderResult, resolve(locator), EVENT_FOCUS);
      return commit();
    },
    blur(locator) {
      dispatchFocus(renderResult, resolve(locator), EVENT_BLUR);
      return commit();
    },
    visibleRange(locator, start, end) {
      const node = resolve(locator);
      renderResult.visibleRange(node, start, end);
      return commit();
    },
    commandResult(
      requestIdOrOptions: number | CommandResultOptions | undefined,
      resultOptions: CommandResultOptions = {},
    ) {
      let requestId: number;
      let optionsForResult: CommandResultOptions;
      if (typeof requestIdOrOptions === "number") {
        requestId = requestIdOrOptions;
        optionsForResult = resultOptions;
      } else {
        optionsForResult = requestIdOrOptions ?? {};
        const command = findLatestCommand(renderResult.commits());
        if (command === undefined) throw new Error("testing app command result has no captured command");
        requestId = Number(command[5]);
      }
      renderResult.commandResult(requestId, optionsForResult);
      return commit();
    },
    dispatchFrame(rawEvent) {
      renderResult.dispatchFrame(rawEvent);
      return commit();
    },
    unmount() {
      hovered.clear();
      renderResult.unmount();
    },
  };
  return app;
}

function actionCode(action: KeyEvent["action"]): number {
  if (action === "down") return KEY_DOWN;
  if (action === "repeat") return KEY_REPEAT;
  return 3;
}

function findCommand(commits: readonly unknown[], requestId: number): WirePayload | undefined {
  for (const commit of [...commits].reverse()) {
    const message = asPayload(commit);
    if (message !== null && message[1] === COMMAND_MESSAGE && message[5] === requestId) return message;
  }
  return undefined;
}

function findLatestCommand(commits: readonly unknown[]): WirePayload | undefined {
  for (const commit of [...commits].reverse()) {
    const message = asPayload(commit);
    if (message !== null && message[1] === COMMAND_MESSAGE) return message;
  }
  return undefined;
}
