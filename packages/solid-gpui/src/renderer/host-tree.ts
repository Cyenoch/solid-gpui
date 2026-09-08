import type { NativeCallOptions } from "../native-call";
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
  type Snapshot,
  type SnapshotNode,
} from "../protocol";
import { nextU32 } from "./props";
import { NodeGraph } from "./nodes";
import { structuralPatch } from "./structural-patch";
import type { HostKind, HostNodeInternal, HostProps, RootOwner } from "./types";

export interface HostTreeOptions {
  readonly invokeNative: (
    moduleId: Uint8Array,
    moduleDigest: Uint8Array,
    functionId: number,
    args: Uint8Array,
    options?: NativeCallOptions,
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
    options?: NativeCallOptions,
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
    options?: NativeCallOptions,
  ): Promise<Uint8Array> {
    return this.options.invokeNative(moduleId, moduleDigest, functionId, args, options);
  }
  allocateNode(kind: HostKind): HostNodeInternal {
    return this.graph.allocateNode(kind);
  }
  insertNode(parent: HostNodeInternal, node: HostNodeInternal, anchor?: HostNodeInternal): void {
    this.graph.insertNode(parent, node, anchor, this.bootstrapped);
  }
  removeNode(parent: HostNodeInternal, node: HostNodeInternal): void {
    this.graph.removeNode(parent, node, this.bootstrapped);
  }

  submitCommand(node: HostNodeInternal, kind: CommandKind, payload: CommandPayload): Promise<void> {
    return this.options.submitCommand(node, kind, payload);
  }

  submitCommandValue(
    node: HostNodeInternal,
    kind: CommandKind,
    payload: CommandPayload,
    options?: NativeCallOptions,
  ): Promise<CommandValue | null> {
    return this.options.submitCommandValue(node, kind, payload, options);
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
  markUpdated(node: HostNodeInternal, mask: number): void {
    this.graph.markUpdated(node, mask, this.bootstrapped);
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
      if (commit === null) {
        // Suspense may construct detached content without changing the native tree.
        // Keep that host state for the later attachment transaction.
        this.graph.completeTransaction();
        this.transactionBootstrapped = undefined;
        this.graph.clearMutations();
        return;
      }
      if (!this.options.submitCommit(commit)) {
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
    const operations = structuralPatch(this.graph, (node) => this.snapshotNode(node));
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
