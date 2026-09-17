import {
  UPDATE_ACCESSIBILITY,
  UPDATE_LISTENER,
  UPDATE_PROPERTIES,
  UPDATE_TEXT,
  UPDATE_TOOLTIP,
} from "../protocol/constants";
import { NodeKind, type Node as WireNode, type Patch, type Snapshot } from "../protocol/generated/protocol";
import type { HostKind } from "../renderer/types";
import type { TestNode, TestSurface } from "./types";

const KINDS: Partial<Record<NodeKind, HostKind>> = {
  [NodeKind.View]: "View",
  [NodeKind.Text]: "Text",
  [NodeKind.Pressable]: "Pressable",
  [NodeKind.RawText]: "RawText",
  [NodeKind.TextInput]: "TextInput",
  [NodeKind.VirtualList]: "VirtualList",
  [NodeKind.Image]: "Image",
  [NodeKind.Extension]: "Extension",
  [NodeKind.Icon]: "Icon",
};

export function required<T>(value: T | undefined, name: string): T {
  if (value === undefined) throw new Error(`TestHost received a frame without ${name}`);
  return value;
}

/** Wire records never escape this replay implementation. */
export class TestTree {
  readonly surfaceId: number;
  readonly epoch: number;
  revision: number;
  private readonly nodes = new Map<number, WireNode>();
  private readonly children = new Map<number, number[]>([[0, []]]);
  private cached: TestSurface | undefined;

  constructor(snapshot: Snapshot) {
    this.surfaceId = required(snapshot.surfaceId, "surfaceId");
    this.epoch = required(snapshot.epoch, "epoch");
    this.revision = required(snapshot.revision, "revision");
    for (const node of snapshot.nodes ?? []) this.insert(node);
  }

  apply(patch: Patch): void {
    if (patch.epoch !== this.epoch || patch.baseRevision !== this.revision)
      throw new Error("TestHost cannot replay a Patch without its matching Snapshot/revision");
    const revision = required(patch.revision, "revision");
    for (const entry of patch.operations ?? []) {
      const operation = required(entry.operation, "patch operation");
      switch (operation.tag) {
        case 1:
          this.insert(required(operation.value.node, "created node"));
          break;
        case 2: {
          const update = operation.value;
          const id = required(update.id, "updated node ID");
          const node = { ...this.node(id) };
          const mask = required(update.mask, "update mask");
          if (mask & UPDATE_TEXT) node.text = update.text;
          if (mask & UPDATE_LISTENER) node.listenerId = update.listenerId;
          if (mask & UPDATE_PROPERTIES) node.hostProperties = update.hostProperties;
          if (mask & UPDATE_ACCESSIBILITY) node.accessibility = update.accessibility;
          if (mask & UPDATE_TOOLTIP) node.tooltip = update.tooltip;
          this.nodes.set(id, node);
          break;
        }
        case 3: {
          const id = required(operation.value.id, "moved node ID");
          const node = this.node(id);
          const parentId = required(operation.value.parentId, "move parent ID");
          const index = required(operation.value.index, "move index");
          this.detach(node);
          this.attach(id, parentId, index);
          this.nodes.set(id, { ...node, parentId });
          break;
        }
        case 4: {
          const id = required(operation.value.id, "deleted node ID");
          this.detach(this.node(id));
          const removed = [id];
          while (removed.length > 0) {
            const child = removed.pop()!;
            for (const descendant of this.siblings(child)) removed.push(descendant);
            this.children.delete(child);
            this.nodes.delete(child);
          }
          break;
        }
      }
    }
    this.revision = revision;
    this.cached = undefined;
  }

  view(capture: (view: TestNode, wire: WireNode) => void): TestSurface {
    if (this.cached) return this.cached;
    const nodes: TestNode[] = [];
    const pending = [...this.siblings(0)].reverse();
    const visited = new Set<number>();
    while (pending.length > 0) {
      const id = pending.pop()!;
      if (visited.has(id)) throw new Error("TestHost received a cyclic tree");
      visited.add(id);
      const node = this.node(id);
      const kind = required(KINDS[required(node.kind, "node kind")], "supported node kind");
      const input = node.hostProperties?.tag === 1 ? node.hostProperties.value : undefined;
      const children = Object.freeze([...this.siblings(id)]);
      const view: TestNode = Object.freeze({
        id,
        kind,
        parentId: required(node.parentId, "parent ID"),
        children,
        text: node.text ?? null,
        inputValue: input?.value ?? null,
        placeholder: input?.placeholder ?? null,
        accessibilityLabel: node.accessibility?.label ?? null,
        tooltip: node.tooltip ?? null,
      });
      capture(view, node);
      nodes.push(view);
      for (let index = children.length - 1; index >= 0; index--) pending.push(children[index]!);
    }
    if (visited.size !== this.nodes.size) throw new Error("TestHost received a disconnected tree");
    this.cached = Object.freeze({
      surfaceId: this.surfaceId,
      epoch: this.epoch,
      revision: this.revision,
      nodes: Object.freeze(nodes),
    });
    return this.cached;
  }

  private node(id: number): WireNode {
    return required(this.nodes.get(id), `node ${id}`);
  }

  private siblings(parentId: number): number[] {
    return required(this.children.get(parentId), `parent ${parentId}`);
  }

  private attach(id: number, parentId: number, index: number): void {
    const siblings = this.siblings(parentId);
    if (index > siblings.length) throw new Error("TestHost received an out-of-range child index");
    siblings.splice(index, 0, id);
  }

  private detach(node: WireNode): void {
    const siblings = this.siblings(required(node.parentId, "parent ID"));
    const index = siblings.indexOf(required(node.id, "node ID"));
    if (index < 0) throw new Error("TestHost received a node missing from its parent");
    siblings.splice(index, 1);
  }

  private insert(node: WireNode): void {
    const id = required(node.id, "node ID");
    if (id === 0 || this.nodes.has(id)) throw new Error("TestHost received a duplicate/reserved node ID");
    this.attach(id, required(node.parentId, "parent ID"), required(node.index, "child index"));
    this.nodes.set(id, node);
    this.children.set(id, []);
  }
}
