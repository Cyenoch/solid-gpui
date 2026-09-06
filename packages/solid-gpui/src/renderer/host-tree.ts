import {
  UPDATE_ACCESSIBILITY,
  UPDATE_LISTENER,
  UPDATE_PROPERTIES,
  UPDATE_STYLE,
  UPDATE_TEXT,
  type CommandKind,
  type CommandPayload,
  type CommandValue,
  type Patch,
  type PatchOperation,
  type Snapshot,
  type SnapshotNode,
} from "../protocol";
import { nextU32 } from "./props";
import { NodeGraph } from "./nodes";
import type { HostKind, HostNodeInternal, HostProps, RootOwner } from "./types";

export interface HostTreeOptions {
  readonly invokeNative: (
    moduleId: Uint8Array,
    moduleDigest: Uint8Array,
    functionId: number,
    args: Uint8Array,
  ) => Promise<Uint8Array>;
  readonly surfaceId: number;
  readonly epoch: number;
  readonly getRevision: () => number;
  readonly submitCommit: (commit: Snapshot | Patch) => boolean;
  readonly onCommitError: (error: unknown) => void;
  readonly submitCommand: (node: HostNodeInternal, kind: CommandKind, payload: CommandPayload) => Promise<void>;
  readonly submitCommandValue: (
    node: HostNodeInternal,
    kind: CommandKind,
    payload: CommandPayload,
  ) => Promise<CommandValue | null>;
}

export class HostTree implements RootOwner {
  readonly graph: NodeGraph;
  readonly syntheticRoot: HostNodeInternal;
  readonly children: HostNodeInternal[];
  invalid = false;
  validationError: Error | undefined;
  bootstrapped = false;
  private disposed = false;
  private transactionBootstrapped: boolean | undefined;

  constructor(private readonly options: HostTreeOptions) {
    this.graph = new NodeGraph(this);
    this.syntheticRoot = this.graph.syntheticRoot;
    this.children = this.graph.children;
  }
  isDisposed(): boolean {
    return this.disposed;
  }
  invokeNative(
    moduleId: Uint8Array,
    moduleDigest: Uint8Array,
    functionId: number,
    args: Uint8Array,
  ): Promise<Uint8Array> {
    return this.options.invokeNative(moduleId, moduleDigest, functionId, args);
  }
  allocateNode(kind: HostKind): HostNodeInternal {
    return this.graph.allocateNode(kind);
  }
  allocateListener(node: HostNodeInternal): number {
    return this.graph.allocateListener(node);
  }
  attachSubtree(node: HostNodeInternal): void {
    this.graph.attachSubtree(node, this.bootstrapped);
  }

  submitCommand(node: HostNodeInternal, kind: CommandKind, payload: CommandPayload): Promise<void> {
    return this.options.submitCommand(node, kind, payload);
  }

  submitCommandValue(node: HostNodeInternal, kind: CommandKind, payload: CommandPayload): Promise<CommandValue | null> {
    return this.options.submitCommandValue(node, kind, payload);
  }

  releaseDetachedFocus(node: HostNodeInternal): void {
    this.graph.releaseDetachedFocus(node);
  }
  markPropsDirty(node: HostNodeInternal, groups: number): void {
    this.graph.markPropsDirty(node, groups);
  }
  recordNodeMutation(node: HostNodeInternal): void {
    this.graph.recordNodeMutation(node);
  }
  setNodeProps(node: HostNodeInternal, props: HostProps): void {
    this.graph.setNodeProps(node, props);
  }
  updateNodeProps(node: HostNodeInternal, props: HostProps): number {
    return this.graph.updateNodeProps(node, props);
  }
  detachFromParent(node: HostNodeInternal): void {
    this.graph.detachFromParent(node);
  }
  refreshChildIndexes(parent: HostNodeInternal): void {
    this.graph.refreshChildIndexes(parent);
  }
  detachSubtree(node: HostNodeInternal): void {
    this.graph.detachSubtree(node);
  }
  markMoved(node: HostNodeInternal): void {
    this.graph.markMoved(node, this.bootstrapped);
  }
  markUpdated(node: HostNodeInternal, mask: number): void {
    this.graph.markUpdated(node, mask, this.bootstrapped);
  }
  markDeleted(node: HostNodeInternal): void {
    this.graph.markDeleted(node, this.bootstrapped);
  }

  beginRender(): void {
    this.graph.beginTransaction();
    this.transactionBootstrapped = this.bootstrapped;
    this.invalid = false;
    this.validationError = undefined;
    this.graph.clearMutations();
  }

  commit(): void {
    if (this.disposed) {
      this.graph.rollbackTransaction();
      return;
    }
    if (this.invalid) {
      this.abortTransaction();
      return;
    }
    try {
      this.graph.finalizeDirtyProps();
      const baseRevision = this.options.getRevision();
      const revision = nextU32(baseRevision, "revision");
      if (!this.bootstrapped && this.children.length === 0) {
        this.graph.completeTransaction();
        this.transactionBootstrapped = undefined;
        this.graph.clearMutations();
        return;
      }
      const commit = this.buildCommit(baseRevision, revision);
      if (commit === null || !this.options.submitCommit(commit)) {
        this.abortTransaction();
        return;
      }
      this.graph.publishRevision(revision);
      this.graph.completeTransaction();
      this.transactionBootstrapped = undefined;
      this.graph.clearMutations();
      this.bootstrapped = true;
    } catch (error) {
      this.abortTransaction();
      this.options.onCommitError(error);
    }
  }

  dispose(): void {
    if (this.disposed) return;
    this.disposed = true;
    this.graph.dispose();
    this.transactionBootstrapped = undefined;
    this.invalid = false;
    this.validationError = undefined;
  }

  findListener(listenerId: number, revision: number) {
    return this.graph.findListener(listenerId, revision);
  }

  private abortTransaction(): void {
    this.graph.rollbackTransaction();
    this.bootstrapped = this.transactionBootstrapped ?? this.bootstrapped;
    this.transactionBootstrapped = undefined;
    this.invalid = false;
    this.graph.clearMutations();
  }

  private buildCommit(baseRevision: number, revision: number): Snapshot | Patch | null {
    if (!this.bootstrapped) {
      return {
        type: "snapshot",
        surfaceId: this.options.surfaceId,
        epoch: this.options.epoch,
        baseRevision,
        revision,
        nodes: this.graph.snapshotNodes(),
      };
    }
    const operations: PatchOperation[] = [];
    const created = [...this.graph.createdIds]
      .map((id) => this.graph.nodesById.get(id))
      .filter((node): node is HostNodeInternal => node !== undefined)
      .sort(
        (a, b) =>
          this.graph.nodeDepth(a) - this.graph.nodeDepth(b) ||
          this.graph.nativeParentId(a) - this.graph.nativeParentId(b) ||
          a.index - b.index ||
          a.id - b.id,
      );
    for (const node of created) operations.push({ type: "create", node: this.snapshotNode(node) });
    const moved = [...this.graph.movedIds]
      .map((id) => this.graph.nodesById.get(id))
      .filter((node): node is HostNodeInternal => node !== undefined && !this.graph.createdIds.has(node.id))
      .sort((a, b) => this.graph.nodeDepth(a) - this.graph.nodeDepth(b) || a.id - b.id);
    for (const node of moved) {
      operations.push({ type: "move", id: node.id, parentId: this.graph.nativeParentId(node), index: node.index });
    }
    for (const [id, mask] of [...this.graph.updatedMasks.entries()].sort(([a], [b]) => a - b)) {
      const node = this.graph.nodesById.get(id);
      if (node === undefined) continue;
      operations.push({
        type: "update",
        id,
        mask,
        style: mask & UPDATE_STYLE ? node.style : null,
        text: mask & UPDATE_TEXT && node.kind === "RawText" ? node.text : null,
        listenerId: mask & UPDATE_LISTENER ? node.listenerId : 0,
        hostProperties: mask & UPDATE_PROPERTIES ? node.hostProperties : null,
        accessibility: mask & UPDATE_ACCESSIBILITY ? node.accessibility : null,
        focusable: node.focusable,
        selectable: node.selectable,
        tooltip: node.tooltip,
        acceptsPointerMove: node.acceptsPointerMove,
      });
    }
    for (const id of this.graph.deletedNodeIds()) operations.push({ type: "delete", id });
    if (operations.length === 0) return null;
    return {
      type: "patch",
      surfaceId: this.options.surfaceId,
      epoch: this.options.epoch,
      baseRevision,
      revision,
      operations,
    };
  }

  private snapshotNode(node: HostNodeInternal): SnapshotNode {
    return {
      id: node.id,
      parentId: this.graph.nativeParentId(node),
      index: node.index,
      kind: node.kind,
      style: node.style,
      text: node.text,
      listenerId: node.listenerId,
      hostProperties: node.hostProperties,
      accessibility: node.accessibility,
      focusable: node.focusable,
      selectable: node.selectable,
      tooltip: node.tooltip,
      acceptsPointerMove: node.acceptsPointerMove,
    };
  }
}
