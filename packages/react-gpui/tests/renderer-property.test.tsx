import React, { type ReactElement } from "react";
import { describe, expect, it } from "bun:test";
import { decodeWireForGolden, encodePayload, FrameDecoder } from "../src/protocol";
import { Image, MemoryTransport, Pressable, Text, View, createRoot, type HostNode } from "../src/index";
import type { StyleProp } from "../src/style";
import {
  UPDATE_ACCESSIBILITY,
  UPDATE_FOCUSABLE,
  UPDATE_LISTENER,
  UPDATE_POINTER_MOVE,
  UPDATE_PROPERTIES,
  UPDATE_SELECTABLE,
  UPDATE_STYLE,
  UPDATE_TEXT,
  UPDATE_TOOLTIP,
} from "../src/protocol";

const SEED_COUNT = 24;
const MUTATIONS_PER_SEED = 40;
const ALL_UPDATE_BITS =
  UPDATE_STYLE |
  UPDATE_TEXT |
  UPDATE_LISTENER |
  UPDATE_PROPERTIES |
  UPDATE_ACCESSIBILITY |
  UPDATE_FOCUSABLE |
  UPDATE_SELECTABLE |
  UPDATE_TOOLTIP |
  UPDATE_POINTER_MOVE;

const KIND_CODES = { View: 1, Text: 2, Pressable: 3, RawText: 4, Image: 7 } as const;
type HostKind = keyof typeof KIND_CODES;
type ModelHostKind = Exclude<HostKind, "RawText">;

type ModelNode = {
  readonly key: string;
  readonly kind: ModelHostKind;
  parent: ModelNode | null;
  children: ModelNode[];
  label: string | null;
  styleVariant: number;
  onPress: boolean;
};

const GENERAL_STYLES = [
  undefined,
  { width: 80, opacity: 0.5 },
  { padding: 2, backgroundColor: "#112233" },
  { marginTop: 1, color: "#abcdef" },
] as const;
const TEXT_STYLES = [
  undefined,
  { fontSize: 13, lineHeight: 18, color: "#334455" },
  { fontSize: 14, fontWeight: "bold", color: "#556677" },
  { fontSize: 12, textDecoration: "underline", color: "#778899" },
] as const;
const NESTED_TEXT_STYLES = [
  undefined,
  { color: "#3366cc", fontWeight: "bold" },
  { color: "#228855", fontStyle: "italic", textDecoration: "underline" },
] as const;

class XorShift64 {
  private state: bigint;

  constructor(seed: number) {
    this.state = BigInt(seed) + 0x9e3779b97f4a7c15n;
  }

  nextU64(): bigint {
    let value = this.state;
    value ^= value >> 12n;
    value ^= value << 25n;
    value ^= value >> 27n;
    this.state = value;
    return BigInt.asUintN(64, value * 0x2545f4914f6cdd1dn);
  }

  below(upper: number): number {
    if (upper <= 0) throw new RangeError("xorshift upper bound must be positive");
    return Number(this.nextU64() % BigInt(upper));
  }
}

function makeNode(
  key: string,
  kind: ModelHostKind,
  parent: ModelNode | null,
  styleVariant = 0,
  onPress = false,
  label: string | null = kind === "Text" ? `text-${key}` : null,
): ModelNode {
  const node: ModelNode = { key, kind, parent, children: [], label, styleVariant, onPress };
  parent?.children.push(node);
  return node;
}

function initialTree(): ModelNode {
  const root = makeNode("root", "View", null, 1);
  const left = makeNode("left", "View", root, 2);
  const paragraph = makeNode("paragraph", "Text", left, 1, false, "hello ");
  const run = makeNode("run", "Text", paragraph, 1, true, "nested link");
  const button = makeNode("button", "Pressable", left, 3, true);
  makeNode("button-label", "Text", button, 2, false, "button");
  const right = makeNode("right", "View", root, 0);
  makeNode("other-paragraph", "Text", right, 3, false, "another paragraph");
  makeNode("image", "Image", right, 1);
  return root;
}

function hostNodes(root: ModelNode): ModelNode[] {
  const result: ModelNode[] = [];
  const visit = (node: ModelNode): void => {
    result.push(node);
    for (const child of node.children) visit(child);
  };
  visit(root);
  return result;
}

function containsTextChild(node: ModelNode): boolean {
  return node.children.some((child) => child.kind === "Text" || containsTextChild(child));
}

function isInSubtree(node: ModelNode, possibleDescendant: ModelNode): boolean {
  for (const child of node.children) {
    if (child === possibleDescendant || isInSubtree(child, possibleDescendant)) return true;
  }
  return false;
}

function canAttach(node: ModelNode, parent: ModelNode): boolean {
  if (node === parent || isInSubtree(node, parent) || parent.kind === "Image") return false;
  if (parent.kind !== "Text") return true;
  if (node.kind !== "Text" || parent.parent?.kind === "Text") return false;
  return !containsTextChild(node);
}

function styleFor(node: ModelNode): StyleProp {
  if (node.kind === "Text") {
    if (node.parent?.kind === "Text") return NESTED_TEXT_STYLES[node.styleVariant % NESTED_TEXT_STYLES.length];
    return TEXT_STYLES[node.styleVariant % TEXT_STYLES.length];
  }
  return GENERAL_STYLES[node.styleVariant % GENERAL_STYLES.length];
}

const pressCallbacks = new Map<string, () => void>();
function pressCallback(key: string): () => void {
  const existing = pressCallbacks.get(key);
  if (existing !== undefined) return existing;
  const callback = (): void => undefined;
  pressCallbacks.set(key, callback);
  return callback;
}

function elementFor(node: ModelNode): ReactElement {
  const style = styleFor(node);
  if (node.kind === "View") {
    return (
      <View key={node.key} style={style}>
        {node.children.map((child) => elementFor(child))}
      </View>
    );
  }
  if (node.kind === "Pressable") {
    return (
      <Pressable key={node.key} style={style} onPress={node.onPress ? pressCallback(node.key) : undefined}>
        {node.children.map((child) => elementFor(child))}
      </Pressable>
    );
  }
  if (node.kind === "Image") {
    return <Image key={node.key} style={style} source={`/tmp/ts-prop-${node.key}.png`} />;
  }
  return (
    <Text key={node.key} style={style} onPress={node.onPress ? pressCallback(node.key) : undefined}>
      {[node.label ?? "", ...node.children.map((child) => elementFor(child))] as never}
    </Text>
  );
}

function renderTree(root: ModelNode): ReactElement {
  return elementFor(root);
}

type MutationKind = "add" | "delete" | "move" | "reorder" | "restyle" | "listener" | "text";

function mutateTree(root: ModelNode, rng: XorShift64): MutationKind {
  const roll = rng.below(100);
  if (roll < 55) {
    const nodes = hostNodes(root);
    const node = nodes[rng.below(nodes.length)];
    const update = rng.below(3);
    if (update === 0) {
      node.styleVariant = (node.styleVariant + 1 + rng.below(3)) % 12;
      return "restyle";
    }
    if (update === 1 && (node.kind === "Text" || node.kind === "Pressable")) {
      node.onPress = !node.onPress;
      return "listener";
    }
    if (node.kind === "Text") {
      node.label = `${node.label ?? "text"}-${rng.below(1000)}`;
      return "text";
    }
    node.styleVariant = (node.styleVariant + 1) % 12;
    return "restyle";
  }

  if (roll < 75) {
    const parents = hostNodes(root).filter((node) => node.kind !== "Image");
    const parent = parents[rng.below(parents.length)];
    const kinds: ModelHostKind[] =
      parent.kind === "Text"
        ? parent.parent?.kind === "Text"
          ? []
          : ["Text"]
        : ["View", "Pressable", "Text", "Image"];
    if (kinds.length > 0) {
      const kind = kinds[rng.below(kinds.length)];
      const key = `added-${rng.nextU64().toString(16)}`;
      const child = makeNode(
        key,
        kind,
        null,
        rng.below(12),
        kind === "Text" || kind === "Pressable" ? rng.below(2) === 0 : false,
      );
      child.parent = parent;
      parent.children.splice(rng.below(parent.children.length + 1), 0, child);
      return "add";
    }
  }

  if (roll < 90) {
    const nodes = hostNodes(root).filter((node) => node !== root);
    if (nodes.length > 0) {
      const node = nodes[rng.below(nodes.length)];
      const parent = node.parent;
      if (parent === null) throw new Error(`node ${node.key} has no parent before delete`);
      const index = parent.children.indexOf(node);
      if (index < 0) throw new Error(`node ${node.key} is absent from parent before delete`);
      parent.children.splice(index, 1);
      node.parent = null;
      return "delete";
    }
  }

  const nodes = hostNodes(root).filter((node) => node !== root);
  const parents = hostNodes(root).filter((node) => node.kind !== "Image");
  for (let attempt = 0; attempt < Math.max(1, nodes.length * parents.length); attempt += 1) {
    if (nodes.length === 0) break;
    const node = nodes[rng.below(nodes.length)];
    const targets = parents.filter((parent) => parent !== node.parent && canAttach(node, parent));
    if (targets.length === 0) continue;
    const target = targets[rng.below(targets.length)];
    const oldParent = node.parent;
    if (oldParent === null) throw new Error(`node ${node.key} has no parent before move`);
    const oldIndex = oldParent.children.indexOf(node);
    if (oldIndex < 0) throw new Error(`node ${node.key} is absent from old parent before move`);
    oldParent.children.splice(oldIndex, 1);
    node.parent = target;
    target.children.splice(rng.below(target.children.length + 1), 0, node);
    return "move";
  }

  const reorderable = hostNodes(root).filter((node) => node.children.length >= 2);
  if (reorderable.length > 0) {
    const parent = reorderable[rng.below(reorderable.length)];
    const first = rng.below(parent.children.length);
    let second = rng.below(parent.children.length - 1);
    if (second >= first) second += 1;
    [parent.children[first], parent.children[second]] = [parent.children[second], parent.children[first]];
    return "reorder";
  }
  const node = hostNodes(root)[rng.below(hostNodes(root).length)];
  node.styleVariant = (node.styleVariant + 1) % 12;
  return "restyle";
}

type WireNode = {
  id: number;
  parent: WireNode | null;
  parentId: number;
  index: number;
  kind: number;
  style: unknown;
  text: string | null;
  listenerId: number;
  hostProperties: unknown;
  accessibility: unknown;
  focusable: boolean;
  selectable: boolean;
  tooltip: string | null;
  acceptsPointerMove: boolean;
  children: WireNode[];
};

function expectArray(value: unknown, label: string): asserts value is readonly unknown[] {
  expect(Array.isArray(value), label).toBe(true);
}

function expectInteger(value: unknown, label: string): asserts value is number {
  expect(typeof value, label).toBe("number");
  expect(Number.isInteger(value), label).toBe(true);
}

function parseTail(
  node: readonly unknown[],
  start: number,
): Pick<WireNode, "selectable" | "tooltip" | "acceptsPointerMove"> {
  expect(node.length).toBeGreaterThanOrEqual(start);
  expect(node.length).toBeLessThanOrEqual(start + 3);
  const selectable = node.length > start ? node[start] : false;
  const tooltip = node.length > start + 1 ? node[start + 1] : null;
  const acceptsPointerMove = node.length > start + 2 ? node[start + 2] : false;
  expect(typeof selectable).toBe("boolean");
  expect(tooltip === null || typeof tooltip === "string").toBe(true);
  expect(typeof acceptsPointerMove).toBe("boolean");
  return {
    selectable: selectable as boolean,
    tooltip: tooltip as string | null,
    acceptsPointerMove: acceptsPointerMove as boolean,
  };
}

function parseSnapshotNode(raw: readonly unknown[]): WireNode {
  expect(raw.length).toBeGreaterThanOrEqual(10);
  expect(raw.length).toBeLessThanOrEqual(13);
  expectInteger(raw[0], "snapshot node id");
  expectInteger(raw[1], "snapshot node parent");
  expectInteger(raw[2], "snapshot node index");
  expectInteger(raw[3], "snapshot node kind");
  expect(raw[3]).toBeGreaterThanOrEqual(1);
  expect(raw[3]).toBeLessThanOrEqual(7);
  expect(raw[4] === null || Array.isArray(raw[4])).toBe(true);
  expect(raw[5] === null || typeof raw[5] === "string").toBe(true);
  expectInteger(raw[6], "snapshot node listener");
  expect(raw[6]).toBeGreaterThanOrEqual(0);
  expect(raw[7] === null || Array.isArray(raw[7])).toBe(true);
  expect(raw[8] === null || Array.isArray(raw[8])).toBe(true);
  expect(typeof raw[9]).toBe("boolean");
  const tail = parseTail(raw, 10);
  return {
    id: raw[0] as number,
    parent: null,
    parentId: raw[1] as number,
    index: raw[2] as number,
    kind: raw[3] as number,
    style: raw[4],
    text: raw[5] as string | null,
    listenerId: raw[6] as number,
    hostProperties: raw[7],
    accessibility: raw[8],
    focusable: raw[9] as boolean,
    ...tail,
    children: [],
  };
}

function parseCreate(raw: readonly unknown[]): WireNode {
  expect(raw.length).toBeGreaterThanOrEqual(11);
  expect(raw.length).toBeLessThanOrEqual(14);
  expect(raw[0]).toBe(1);
  return parseSnapshotNode(raw.slice(1));
}

function assertUpdateShape(raw: readonly unknown[]): void {
  expect(raw.length).toBeGreaterThanOrEqual(9);
  expect(raw.length).toBeLessThanOrEqual(12);
  expect(raw[0]).toBe(2);
  expectInteger(raw[1], "update node id");
  expectInteger(raw[2], "update mask");
  expect(raw[2]).toBeGreaterThanOrEqual(0);
  expect((raw[2] as number) & ~ALL_UPDATE_BITS).toBe(0);
  expect(raw[3] === null || Array.isArray(raw[3])).toBe(true);
  expect(raw[4] === null || typeof raw[4] === "string").toBe(true);
  expectInteger(raw[5], "update listener");
  expect(raw[5]).toBeGreaterThanOrEqual(0);
  expect(raw[6] === null || Array.isArray(raw[6])).toBe(true);
  expect(raw[7] === null || Array.isArray(raw[7])).toBe(true);
  expect(typeof raw[8]).toBe("boolean");
  const mask = raw[2] as number;
  if (mask & (UPDATE_TOOLTIP | UPDATE_POINTER_MOVE)) {
    expect(raw.length === 11 || raw.length === 12).toBe(true);
    parseTail(raw, 9);
  } else if (mask & UPDATE_SELECTABLE) {
    expect(raw.length === 9 || raw.length === 10).toBe(true);
    if (raw.length === 10) expect(typeof raw[9]).toBe("boolean");
  } else {
    expect(raw.length).toBe(9);
  }
}

class WireTree {
  readonly nodes = new Map<number, WireNode>();
  readonly revokedListenerIds = new Set<number>();
  revision = 0;

  applyFrame(value: readonly unknown[]): void {
    expect(value.length).toBe(7);
    expect(value[0]).toBe(3);
    expectInteger(value[2], "surface id");
    expectInteger(value[3], "epoch");
    expectInteger(value[4], "base revision");
    expectInteger(value[5], "revision");
    expect(value[5]).toBeGreaterThan(value[4] as number);
    if (value[1] === 1) this.applySnapshot(value);
    else {
      expect(value[1]).toBe(3);
      expect(value[4]).toBe(this.revision);
      this.applyPatch(value);
    }
    this.revision = value[5] as number;
    this.assertStructure();
  }

  private applySnapshot(value: readonly unknown[]): void {
    expect(value[4]).toBe(0);
    this.nodes.clear();
    expectArray(value[6], "snapshot nodes");
    for (const raw of value[6]) {
      expectArray(raw, "snapshot node");
      const node = parseSnapshotNode(raw);
      expect(this.nodes.has(node.id)).toBe(false);
      this.nodes.set(node.id, node);
    }
    this.rebuildParents();
  }

  private applyPatch(value: readonly unknown[]): void {
    expectArray(value[6], "patch operations");
    for (const raw of value[6]) {
      expectArray(raw, "patch operation");
      const operation = raw[0];
      if (operation === 1) this.applyCreate(raw);
      else if (operation === 2) this.applyUpdate(raw);
      else if (operation === 3) this.applyMove(raw);
      else if (operation === 4) this.applyDelete(raw);
      else throw new Error(`unknown patch operation ${String(operation)}`);
      this.assertStructure();
    }
  }

  private applyCreate(raw: readonly unknown[]): void {
    const node = parseCreate(raw);
    expect(this.nodes.has(node.id)).toBe(false);
    expect(this.nodes.has(node.parentId)).toBe(true);
    this.nodes.set(node.id, node);
    this.rebuildParents();
    const parent = this.nodes.get(node.parentId);
    if (parent === undefined) throw new Error(`create parent ${node.parentId} disappeared`);
    const current = parent.children.filter((child) => child.id !== node.id);
    expect(node.index).toBeGreaterThanOrEqual(0);
    expect(node.index).toBeLessThanOrEqual(current.length);
    current.splice(node.index, 0, node);
    parent.children = current;
    current.forEach((child, index) => {
      child.index = index;
    });
  }

  private applyUpdate(raw: readonly unknown[]): void {
    assertUpdateShape(raw);
    const id = raw[1] as number;
    const node = this.nodes.get(id);
    expect(node, `update references node ${id}`).toBeDefined();
    if (node === undefined) return;
    const mask = raw[2] as number;
    if (mask & UPDATE_STYLE) node.style = raw[3];
    if (mask & UPDATE_TEXT) node.text = raw[4] as string | null;
    if (mask & UPDATE_LISTENER) {
      const previousListenerId = node.listenerId;
      node.listenerId = raw[5] as number;
      if (previousListenerId !== 0 && previousListenerId !== node.listenerId)
        this.revokedListenerIds.add(previousListenerId);
    }
    if (mask & UPDATE_PROPERTIES) node.hostProperties = raw[6];
    if (mask & UPDATE_ACCESSIBILITY) node.accessibility = raw[7];
    if (mask & UPDATE_FOCUSABLE) node.focusable = raw[8] as boolean;
    if (mask & (UPDATE_TOOLTIP | UPDATE_POINTER_MOVE)) {
      const tail = parseTail(raw, 9);
      node.selectable = tail.selectable;
      node.tooltip = tail.tooltip;
      node.acceptsPointerMove = tail.acceptsPointerMove;
    } else if (mask & UPDATE_SELECTABLE) {
      node.selectable = raw.length === 10 && raw[9] === true;
    }
  }

  private applyMove(raw: readonly unknown[]): void {
    expect(raw.length).toBe(4);
    expect(raw[0]).toBe(3);
    expectInteger(raw[1], "move node id");
    expectInteger(raw[2], "move parent id");
    expectInteger(raw[3], "move index");
    const node = this.nodes.get(raw[1] as number);
    const parent = this.nodes.get(raw[2] as number);
    expect(node, `move references node ${String(raw[1])}`).toBeDefined();
    expect(parent, `move references parent ${String(raw[2])}`).toBeDefined();
    if (node === undefined || parent === undefined) return;
    const oldParent = node.parent;
    expect(oldParent).not.toBeNull();
    if (oldParent === null) return;
    const oldIndex = oldParent.children.indexOf(node);
    expect(oldIndex).toBeGreaterThanOrEqual(0);
    oldParent.children.splice(oldIndex, 1);
    const index = raw[3] as number;
    expect(index).toBeGreaterThanOrEqual(0);
    expect(index).toBeLessThanOrEqual(parent.children.length);
    parent.children.splice(index, 0, node);
    node.parent = parent;
    node.parentId = parent.id;
    this.refreshIndexes(oldParent);
    if (parent !== oldParent) this.refreshIndexes(parent);
  }

  private applyDelete(raw: readonly unknown[]): void {
    expect(raw.length).toBe(2);
    expect(raw[0]).toBe(4);
    expectInteger(raw[1], "delete node id");
    const node = this.nodes.get(raw[1] as number);
    expect(node, `delete references node ${String(raw[1])}`).toBeDefined();
    if (node === undefined) return;
    const parent = node.parent;
    expect(parent).not.toBeNull();
    if (parent !== null) {
      const index = parent.children.indexOf(node);
      expect(index).toBeGreaterThanOrEqual(0);
      parent.children.splice(index, 1);
      this.refreshIndexes(parent);
    }
    const remove = (current: WireNode): void => {
      if (current.listenerId !== 0) this.revokedListenerIds.add(current.listenerId);
      for (const child of current.children) remove(child);
      this.nodes.delete(current.id);
    };
    remove(node);
  }

  private rebuildParents(): void {
    for (const node of this.nodes.values()) {
      node.parent = null;
      node.children = [];
    }
    for (const node of this.nodes.values()) {
      if (node.parentId === 0) continue;
      const parent = this.nodes.get(node.parentId);
      expect(parent, `node ${node.id} references missing parent ${node.parentId}`).toBeDefined();
      if (parent !== undefined) {
        node.parent = parent;
        parent.children.push(node);
      }
    }
    for (const parent of this.nodes.values()) this.refreshIndexes(parent);
  }

  private refreshIndexes(parent: WireNode): void {
    parent.children.sort((a, b) => a.index - b.index || a.id - b.id);
    parent.children.forEach((child, index) => {
      child.index = index;
      child.parent = parent;
      child.parentId = parent.id;
    });
  }

  private assertStructure(): void {
    const syntheticRoot = this.nodes.get(1);
    expect(syntheticRoot, "synthetic root").toBeDefined();
    if (syntheticRoot === undefined) return;
    expect(syntheticRoot.kind).toBe(KIND_CODES.View);
    expect(syntheticRoot.parentId).toBe(0);
    expect(syntheticRoot.index).toBe(0);
    const seen = new Set<number>();
    const visit = (node: WireNode, textDepth: number): void => {
      expect(seen.has(node.id), `duplicate node ${node.id}`).toBe(false);
      seen.add(node.id);
      expect(this.nodes.get(node.id)).toBe(node);
      expect(node.parentId === 0 || this.nodes.has(node.parentId)).toBe(true);
      expect(node.kind).toBeGreaterThanOrEqual(1);
      expect(node.kind).toBeLessThanOrEqual(7);
      expect(node.kind === KIND_CODES.RawText ? typeof node.text === "string" : node.text === null).toBe(true);
      if (node.kind === KIND_CODES.RawText) expect(node.parent?.kind).toBe(KIND_CODES.Text);
      if (node.kind === KIND_CODES.Image || node.kind === KIND_CODES.RawText) expect(node.children).toHaveLength(0);
      if (node.kind === KIND_CODES.Text) {
        for (const child of node.children) {
          expect(child.kind === KIND_CODES.RawText || child.kind === KIND_CODES.Text).toBe(true);
          if (child.kind === KIND_CODES.Text) expect(textDepth).toBe(0);
        }
      }
      const ids = new Set<number>();
      node.children.forEach((child, index) => {
        expect(ids.has(child.id)).toBe(false);
        ids.add(child.id);
        expect(child.parent).toBe(node);
        expect(child.parentId).toBe(node.id);
        expect(child.index).toBe(index);
        visit(child, textDepth + (node.kind === KIND_CODES.Text ? 1 : 0));
      });
    };
    visit(syntheticRoot, 0);
    expect(seen.size).toBe(this.nodes.size);
    for (const node of this.nodes.values()) {
      expect(node.listenerId === 0 || node.listenerId > 0).toBe(true);
      expect(this.revokedListenerIds.has(node.listenerId)).toBe(false);
    }
  }

  assertListeners(expectedCount: number): void {
    const listenerNodes = [...this.nodes.values()].filter((node) => node.listenerId !== 0);
    expect(listenerNodes.every((node) => node.kind === KIND_CODES.Text || node.kind === KIND_CODES.Pressable)).toBe(
      true,
    );
    expect(listenerNodes).toHaveLength(expectedCount);
    const ids = new Set(listenerNodes.map((node) => node.listenerId));
    expect(ids.size).toBe(listenerNodes.length);
    for (const id of ids) expect(this.revokedListenerIds.has(id)).toBe(false);
  }
}

function expectedListenerCount(root: ModelNode): number {
  return hostNodes(root).filter((node) => node.onPress && (node.kind === "Text" || node.kind === "Pressable")).length;
}

function assertOutboundFrame(frame: Uint8Array, tree: WireTree): void {
  expect(frame.byteLength).toBeGreaterThanOrEqual(4);
  const declaredLength = new DataView(frame.buffer, frame.byteOffset, 4).getUint32(0, true);
  expect(declaredLength).toBe(frame.byteLength - 4);
  const decoder = new FrameDecoder();
  const payloads = decoder.push(frame);
  expect(payloads).toHaveLength(1);
  const decoded = decodeWireForGolden(payloads[0]);
  expect(decoded).not.toBeNull();
  expect(Array.isArray(decoded)).toBe(true);
  const roundTrip = decodeWireForGolden(encodePayload(decoded as never));
  expect(roundTrip).toEqual(decoded);
  tree.applyFrame(decoded as readonly unknown[]);
}

describe("renderer commit emission properties", () => {
  it("keeps decoded patches structurally valid across deterministic random trees", () => {
    const seen = new Set<MutationKind>();
    for (let seed = 0; seed < SEED_COUNT; seed += 1) {
      const transport = new MemoryTransport();
      const root = createRoot(transport, { surfaceId: 700 + seed, epoch: 900 + seed });
      const model = initialTree();
      const tree = new WireTree();
      root.render(renderTree(model));
      expect(transport.submitted).toHaveLength(1);
      assertOutboundFrame(transport.submitted[0], tree);
      tree.assertListeners(expectedListenerCount(model));

      const rng = new XorShift64(seed);
      for (let step = 0; step < MUTATIONS_PER_SEED; step += 1) {
        const mutation = mutateTree(model, rng);
        seen.add(mutation);
        const before = transport.submitted.length;
        root.render(renderTree(model));
        expect(transport.submitted).toHaveLength(before + 1);
        assertOutboundFrame(transport.submitted[before], tree);
        tree.assertListeners(expectedListenerCount(model));
      }
    }
    expect(seen).toEqual(new Set(["add", "delete", "move", "reorder", "restyle", "listener", "text"]));
  });
});
