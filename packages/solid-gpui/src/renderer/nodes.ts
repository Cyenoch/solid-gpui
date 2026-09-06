import {
  COMMAND_BLUR,
  COMMAND_FOCUS,
  COMMAND_GET_FOCUS,
  COMMAND_GET_SCROLL_OFFSET,
  COMMAND_SCROLL_TO_END,
  COMMAND_SCROLL_TO_INDEX,
  COMMAND_SCROLL_TO_OFFSET,
  COMMAND_SET_SELECTION,
  UPDATE_ACCESSIBILITY,
  UPDATE_FOCUSABLE,
  UPDATE_SELECTABLE,
  UPDATE_LISTENER,
  UPDATE_STYLE,
  UPDATE_PROPERTIES,
  UPDATE_TOOLTIP,
  UPDATE_POINTER_MOVE,
  type SnapshotNode,
  type AccessibilityProperties,
  type HostProperties,
  type ExtensionValue,
} from "../protocol";
import type {
  HostKind,
  HostNodeInternal,
  HostProps,
  ListenerBinding,
  RootOwner,
  TextInputCallbackSources,
  TextInputCallbacks,
  TextInputProps,
} from "./types";
import { hostKindSpec } from "./facts";
import {
  accessibilityFor,
  dragFor,
  iconFor,
  imageFor,
  inputFor,
  nextU32,
  validateProps,
  virtualListFor,
} from "./props";
import { normalizeExtensionProperties } from "./extension";
interface ListenerSlot {
  current: ListenerBinding | null;
  previous: ListenerBinding | null;
}
type ListenerSlotSnapshot = ListenerSlot | undefined;
class ListenerRegistry {
  private readonly slots = new Map<number, ListenerSlot>();
  private readonly byId = new Map<number, ListenerSlot>();
  private readonly pendingNodes = new Set<number>();
  private publishedRevision = 0;

  bind(node: HostNodeInternal, listenerId: number, hasListener: boolean): void {
    const existing = this.slots.get(node.id);
    const slot = existing ?? { current: null, previous: null };
    const next = hasListener ? this.binding(node, listenerId) : null;
    if (next !== null && slot.current !== null && this.sameCallbacks(slot.current, next)) return;
    if (slot.current !== null && slot.current.revision <= this.publishedRevision) slot.previous = slot.current;
    slot.current = next;
    this.setSlot(node.id, slot);
  }

  remove(nodeId: number): void {
    this.setSlot(nodeId, { current: null, previous: null });
  }

  clear(): void {
    this.slots.clear();
    this.byId.clear();
    this.pendingNodes.clear();
  }

  current(nodeId: number): ListenerBinding | null {
    return this.slots.get(nodeId)?.current ?? null;
  }

  snapshotNode(nodeId: number): ListenerSlotSnapshot {
    const slot = this.slots.get(nodeId);
    if (slot === undefined) return undefined;
    return {
      current: slot.current === null ? null : this.cloneBinding(slot.current),
      previous: slot.previous === null ? null : this.cloneBinding(slot.previous),
    };
  }

  restoreNode(nodeId: number, snapshot: ListenerSlotSnapshot): void {
    if (snapshot === undefined) {
      this.setSlot(nodeId, { current: null, previous: null });
      return;
    }
    this.setSlot(nodeId, {
      current: snapshot.current === null ? null : this.cloneBinding(snapshot.current),
      previous: snapshot.previous === null ? null : this.cloneBinding(snapshot.previous),
    });
  }

  lookup(listenerId: number, revision: number): ListenerBinding | undefined {
    const entry = this.byId.get(listenerId);
    if (entry === undefined) return undefined;
    const current = entry.current;
    let match = current !== null && current.revision <= revision ? current : undefined;
    const previous = entry.previous;
    if (
      previous !== null &&
      previous.revision <= revision &&
      (match === undefined || previous.revision > match.revision)
    )
      match = previous;
    return match;
  }

  publishRevision(revision: number): void {
    this.publishedRevision = revision;
    for (const nodeId of this.pendingNodes) {
      const current = this.slots.get(nodeId)?.current;
      if (current?.pending) {
        current.revision = revision;
        current.pending = false;
      }
    }
    this.pendingNodes.clear();
  }

  private setSlot(nodeId: number, next: ListenerSlot): void {
    const previous = this.slots.get(nodeId);
    if (previous !== undefined) {
      this.removeById(previous.current);
      this.removeById(previous.previous);
    }
    if (next.current === null && next.previous === null) {
      this.slots.delete(nodeId);
      this.pendingNodes.delete(nodeId);
      return;
    }
    this.slots.set(nodeId, next);
    this.addById(next.current, true);
    this.addById(next.previous, false);
    if (next.current?.pending) this.pendingNodes.add(nodeId);
    else this.pendingNodes.delete(nodeId);
  }

  private addById(binding: ListenerBinding | null, current: boolean): void {
    if (binding === null) return;
    const entry = this.byId.get(binding.listenerId) ?? { current: null, previous: null };
    if (current) entry.current = binding;
    else entry.previous = binding;
    this.byId.set(binding.listenerId, entry);
  }

  private removeById(binding: ListenerBinding | null): void {
    if (binding === null) return;
    const entry = this.byId.get(binding.listenerId);
    if (entry === undefined) return;
    if (entry.current === binding) entry.current = null;
    if (entry.previous === binding) entry.previous = null;
    if (entry.current === null && entry.previous === null) this.byId.delete(binding.listenerId);
  }
  private binding(node: HostNodeInternal, listenerId: number): ListenerBinding {
    return {
      node,
      listenerId,
      revision: this.publishedRevision + 1,
      pending: true,
      listener: node.listener,
      keyListener: node.keyListener,
      pointerCallbacks: node.pointerCallbacks === null ? null : { ...node.pointerCallbacks },
      pointerMoveCallback: node.pointerMoveCallback,
      focusCallback: node.focusCallback,
      blurCallback: node.blurCallback,
      pointerDownOutsideCallback: node.pointerDownOutsideCallback,
      hoverCallback: node.hoverCallback,
      scrollCallback: node.scrollCallback,
      dragCallbacks: { ...node.dragCallbacks },
      layoutCallback: node.layoutCallback,
      inputCallbacks: node.inputCallbacks,
      visibleRangeCallback: node.visibleRangeCallback,
      animationCompleteCallback: node.animationCompleteCallback,
      extensionEventCallback: node.extensionEventCallback,
      extensionEventIds: [...node.extensionEventIds],
    };
  }

  private cloneBinding(binding: ListenerBinding): ListenerBinding {
    return {
      ...binding,
      pointerCallbacks: binding.pointerCallbacks === null ? null : { ...binding.pointerCallbacks },
      dragCallbacks: { ...binding.dragCallbacks },
    };
  }

  private sameCallbacks(a: ListenerBinding, b: ListenerBinding): boolean {
    return (
      a.listenerId === b.listenerId &&
      a.listener === b.listener &&
      a.keyListener === b.keyListener &&
      a.pointerCallbacks?.down === b.pointerCallbacks?.down &&
      a.pointerCallbacks?.up === b.pointerCallbacks?.up &&
      a.pointerMoveCallback === b.pointerMoveCallback &&
      a.focusCallback === b.focusCallback &&
      a.blurCallback === b.blurCallback &&
      a.pointerDownOutsideCallback === b.pointerDownOutsideCallback &&
      a.hoverCallback === b.hoverCallback &&
      a.scrollCallback === b.scrollCallback &&
      a.dragCallbacks.over === b.dragCallbacks.over &&
      a.dragCallbacks.drop === b.dragCallbacks.drop &&
      a.inputCallbacks === b.inputCallbacks &&
      a.visibleRangeCallback === b.visibleRangeCallback &&
      a.animationCompleteCallback === b.animationCompleteCallback &&
      a.extensionEventCallback === b.extensionEventCallback &&
      a.extensionEventIds.length === b.extensionEventIds.length &&
      a.extensionEventIds.every((id, index) => id === b.extensionEventIds[index])
    );
  }
}
function equalTransition(
  a: NonNullable<HostNodeInternal["style"]>["transition"],
  b: NonNullable<HostNodeInternal["style"]>["transition"],
): boolean {
  if (a === b) return true;
  if (a === undefined || b === undefined) return false;
  if (
    a.durationMs !== b.durationMs ||
    a.delayMs !== b.delayMs ||
    a.easing !== b.easing ||
    a.onComplete !== b.onComplete ||
    a.properties?.length !== b.properties?.length
  )
    return false;
  if (a.properties === undefined || b.properties === undefined) return true;
  for (let index = 0; index < a.properties.length; index += 1) {
    if (a.properties[index] !== b.properties[index]) return false;
  }
  return true;
}

function equalBoxShadow(
  a: NonNullable<HostNodeInternal["style"]>["boxShadow"],
  b: NonNullable<HostNodeInternal["style"]>["boxShadow"],
): boolean {
  if (a === b) return true;
  if (a === undefined || b === undefined || Array.isArray(a) !== Array.isArray(b)) return false;
  const left = Array.isArray(a) ? a : [a];
  const right = Array.isArray(b) ? b : [b];
  if (left.length !== right.length) return false;
  for (let index = 0; index < left.length; index += 1) {
    const x = left[index];
    const y = right[index];
    if (
      x.offsetX !== y.offsetX ||
      x.offsetY !== y.offsetY ||
      x.blurRadius !== y.blurRadius ||
      x.spreadRadius !== y.spreadRadius ||
      x.color !== y.color ||
      x.inset !== y.inset
    )
      return false;
  }
  return true;
}

function equalStyle(a: HostNodeInternal["style"], b: HostNodeInternal["style"]): boolean {
  if (a === b) return true;
  if (a == null || b == null) return false;
  return (
    a.width === b.width &&
    a.height === b.height &&
    a.flexDirection === b.flexDirection &&
    a.flexGrow === b.flexGrow &&
    a.padding === b.padding &&
    a.gap === b.gap &&
    a.justifyContent === b.justifyContent &&
    a.alignItems === b.alignItems &&
    a.borderRadius === b.borderRadius &&
    a.borderWidth === b.borderWidth &&
    a.borderColor === b.borderColor &&
    a.fontSize === b.fontSize &&
    a.fontWeight === b.fontWeight &&
    a.overflow === b.overflow &&
    a.lineClamp === b.lineClamp &&
    a.textOverflow === b.textOverflow &&
    a.marginTop === b.marginTop &&
    a.marginRight === b.marginRight &&
    a.marginBottom === b.marginBottom &&
    a.marginLeft === b.marginLeft &&
    a.fontStyle === b.fontStyle &&
    a.textDecoration === b.textDecoration &&
    a.lineHeight === b.lineHeight &&
    a.minWidth === b.minWidth &&
    a.maxWidth === b.maxWidth &&
    a.minHeight === b.minHeight &&
    a.maxHeight === b.maxHeight &&
    a.flexShrink === b.flexShrink &&
    a.alignSelf === b.alignSelf &&
    a.position === b.position &&
    a.left === b.left &&
    a.top === b.top &&
    a.right === b.right &&
    a.bottom === b.bottom &&
    a.cursor === b.cursor &&
    a.textAlign === b.textAlign &&
    a.backgroundColor === b.backgroundColor &&
    a.color === b.color &&
    a.opacity === b.opacity &&
    a.fontFamily === b.fontFamily &&
    equalTransition(a.transition, b.transition) &&
    equalBoxShadow(a.boxShadow, b.boxShadow)
  );
}

function equalHostProperties(a: HostProperties | null, b: HostProperties | null): boolean {
  if (a === b) return true;
  if (a === null || b === null || a.type !== b.type) return false;
  if (a.type === "text-input" && b.type === "text-input") {
    const x = a.value;
    const y = b.value;
    return (
      x.value === y.value &&
      x.placeholder === y.placeholder &&
      x.multiline === y.multiline &&
      x.disabled === y.disabled &&
      x.controlled === y.controlled &&
      x.ackEditSeq === y.ackEditSeq &&
      x.selectionStart === y.selectionStart &&
      x.selectionEnd === y.selectionEnd &&
      x.markedStart === y.markedStart &&
      x.markedEnd === y.markedEnd &&
      x.maxLength === y.maxLength &&
      x.selectionReversed === y.selectionReversed
    );
  }
  if (a.type === "virtual-list" && b.type === "virtual-list") {
    return (
      a.value.itemCount === b.value.itemCount &&
      a.value.rangeStart === b.value.rangeStart &&
      a.value.rangeEnd === b.value.rangeEnd &&
      a.value.estimatedItemSize === b.value.estimatedItemSize &&
      a.value.overscan === b.value.overscan
    );
  }
  if (a.type === "image" && b.type === "image")
    return (
      a.value.source === b.value.source &&
      a.value.objectFit === b.value.objectFit &&
      a.value.fallbackSource === b.value.fallbackSource
    );
  if (a.type === "drag" && b.type === "drag") {
    if (
      a.value.dragType !== b.value.dragType ||
      a.value.acceptsDragOver !== b.value.acceptsDragOver ||
      a.value.acceptsDrop !== b.value.acceptsDrop ||
      a.value.exportFiles?.length !== b.value.exportFiles?.length
    )
      return false;
    if (a.value.exportFiles === null || b.value.exportFiles === null) return true;
    return a.value.exportFiles.every((path, index) => path === b.value.exportFiles?.[index]);
  }
  if (a.type === "extension" && b.type === "extension") {
    const x = a.value;
    const y = b.value;
    if (
      x.entryId !== y.entryId ||
      x.entryVersion !== y.entryVersion ||
      !sameBytes(x.providerId, y.providerId) ||
      !sameBytes(x.catalogDigest, y.catalogDigest) ||
      x.fields.length !== y.fields.length ||
      x.eventIds.length !== y.eventIds.length
    )
      return false;
    return (
      x.fields.every(
        (field, index) => field.id === y.fields[index]?.id && equalExtensionValue(field.value, y.fields[index]?.value),
      ) && x.eventIds.every((id, index) => id === y.eventIds[index])
    );
  }
  if (a.type === "icon" && b.type === "icon") {
    return a.value.name === b.value.name && a.value.size === b.value.size && a.value.color === b.value.color;
  }
  return false;
}
function sameBytes(a: Uint8Array, b: Uint8Array): boolean {
  return a.byteLength === b.byteLength && a.every((byte, index) => byte === b[index]);
}
function equalExtensionValue(a: ExtensionValue, b: ExtensionValue | undefined): boolean {
  if (b === undefined || a.type !== b.type) return false;
  if (a.type === "bytes" && b.type === "bytes") return sameBytes(a.value, b.value);
  return a.value === b.value;
}
function equalAccessibility(a: AccessibilityProperties | null, b: AccessibilityProperties | null): boolean {
  if (a === b) return true;
  if (a === null || b === null) return false;
  return (
    a.role === b.role &&
    a.label === b.label &&
    a.description === b.description &&
    a.disabled === b.disabled &&
    a.checked === b.checked &&
    a.selected === b.selected &&
    a.value === b.value &&
    a.expanded === b.expanded &&
    a.level === b.level
  );
}

interface NodeStateSnapshot {
  readonly node: HostNodeInternal;
  readonly parent: HostNodeInternal | null;
  readonly children: HostNodeInternal[];
  readonly index: number;
  readonly style: HostNodeInternal["style"];
  readonly text: string | null;
  readonly tooltip: string | null;
  readonly acceptsPointerMove: boolean;
  readonly keyListener: HostNodeInternal["keyListener"];
  readonly listenerId: number;
  readonly focusCallback: HostNodeInternal["focusCallback"];
  readonly blurCallback: HostNodeInternal["blurCallback"];
  readonly detachedFocusPending: boolean;
  readonly nativeFocused: boolean;
  readonly pointerDownOutsideCallback: HostNodeInternal["pointerDownOutsideCallback"];
  readonly hoverCallback: HostNodeInternal["hoverCallback"];
  readonly hovered: boolean;
  readonly scrollCallback: HostNodeInternal["scrollCallback"];
  readonly listener: HostNodeInternal["listener"];
  readonly focusable: boolean;
  readonly selectable: boolean;
  readonly disabled: boolean;
  readonly dragCallbacks: HostNodeInternal["dragCallbacks"];
  readonly hostProperties: HostNodeInternal["hostProperties"];
  readonly latestNativeText: string | null;
  readonly latestNativeEditSeq: number;
  readonly layoutCallback: HostNodeInternal["layoutCallback"];
  readonly latestNativeSelection: HostNodeInternal["latestNativeSelection"];
  readonly inputCallbacks: HostNodeInternal["inputCallbacks"];
  readonly inputCallbackSources: HostNodeInternal["inputCallbackSources"];
  readonly visibleRangeCallback: HostNodeInternal["visibleRangeCallback"];
  readonly animationCompleteCallback: HostNodeInternal["animationCompleteCallback"];
  readonly extensionEventCallback: HostNodeInternal["extensionEventCallback"];
  readonly extensionDescriptor: HostNodeInternal["extensionDescriptor"];
  readonly extensionEventIds: HostNodeInternal["extensionEventIds"];
  readonly lastAnimationGeneration: number | null;
  readonly accessibility: HostNodeInternal["accessibility"];
  readonly attached: boolean;
  readonly props: HostProps;
  readonly mounted: boolean;
}
interface TransactionJournal {
  readonly nodeStates: Map<HostNodeInternal, NodeStateSnapshot>;
  readonly newNodes: Set<HostNodeInternal>;
  readonly dirtyProps: Map<HostNodeInternal, number | undefined>;
  readonly nodesById: Map<number, HostNodeInternal | undefined>;
  readonly listeners: Map<number, HostNodeInternal | undefined>;
  readonly inputListeners: Map<number, TextInputCallbacks | undefined>;
  readonly createdIds: Map<number, boolean | undefined>;
  readonly deletedRoots: Map<number, boolean | undefined>;
  readonly movedIds: Map<number, boolean | undefined>;
  readonly updatedMasks: Map<number, number | undefined>;
  readonly registry: Map<number, ListenerSlotSnapshot>;
  readonly nextNodeId: number;
  readonly nextListenerId: number;
}

export class NodeGraph {
  readonly children: HostNodeInternal[];
  readonly syntheticRoot: HostNodeInternal;
  readonly listeners = new Map<number, HostNodeInternal>();
  readonly nodesById = new Map<number, HostNodeInternal>();
  readonly inputListeners = new Map<number, TextInputCallbacks>();
  readonly listenerRegistry = new ListenerRegistry();
  readonly allNodes = new Set<HostNodeInternal>();
  readonly createdIds = new Set<number>();
  readonly deletedRoots = new Set<number>();
  readonly movedIds = new Set<number>();
  readonly updatedMasks = new Map<number, number>();
  readonly dirtyProps = new Map<HostNodeInternal, number>();
  private nextNodeId = 2;
  private nextListenerId = 1;
  private transaction: TransactionJournal | undefined;
  constructor(private readonly owner: RootOwner) {
    this.syntheticRoot = {
      id: 1,
      kind: "View",
      root: owner,
      parent: null,
      children: [],
      index: 0,
      style: null,
      text: null,
      listenerId: 0,
      listener: undefined,
      dragCallbacks: {},
      layoutCallback: undefined,
      focusable: false,
      selectable: false,
      disabled: false,
      keyListener: undefined,
      pointerCallbacks: null,
      pointerMoveCallback: undefined,
      focusCallback: undefined,
      blurCallback: undefined,
      detachedFocusPending: false,
      nativeFocused: false,
      pointerDownOutsideCallback: undefined,
      scrollCallback: undefined,
      hovered: false,
      hoverCallback: undefined,
      hostProperties: null,
      tooltip: null,
      acceptsPointerMove: false,
      latestNativeText: null,
      latestNativeEditSeq: 0,
      latestNativeSelection: null,
      inputCallbacks: null,
      inputCallbackSources: null,
      extensionEventCallback: undefined,
      extensionDescriptor: undefined,
      extensionEventIds: [],
      lastAnimationGeneration: null,
      accessibility: null,
      attached: true,
      props: {},
      mounted: true,
    };
    this.nodesById.set(1, this.syntheticRoot);
    this.allNodes.add(this.syntheticRoot);
    this.children = this.syntheticRoot.children;
  }
  allocateNode(kind: HostKind): HostNodeInternal {
    const node: HostNodeInternal = {
      id: this.nextNodeId,
      kind,
      root: this.owner,
      parent: null,
      children: [],
      index: 0,
      style: null,
      text: null,
      listenerId: 0,
      listener: undefined,
      dragCallbacks: {},
      layoutCallback: undefined,
      focusable: false,
      selectable: false,
      disabled: false,
      keyListener: undefined,
      pointerCallbacks: null,
      pointerMoveCallback: undefined,
      focusCallback: undefined,
      blurCallback: undefined,
      detachedFocusPending: false,
      nativeFocused: false,
      pointerDownOutsideCallback: undefined,
      hoverCallback: undefined,
      scrollCallback: undefined,
      hovered: false,
      hostProperties: null,
      tooltip: null,
      acceptsPointerMove: false,
      latestNativeText: null,
      latestNativeEditSeq: 0,
      latestNativeSelection: null,
      inputCallbacks: null,
      inputCallbackSources: null,
      extensionDescriptor: undefined,
      extensionEventIds: [],
      lastAnimationGeneration: null,
      accessibility: null,
      attached: false,
      props: {},
      mounted: false,
    };
    this.nextNodeId = nextU32(this.nextNodeId, "node id");
    this.allNodes.add(node);
    this.transaction?.newNodes.add(node);
    if (kind === "TextInput") {
      node.focus = () => this.owner.submitCommand(node, COMMAND_FOCUS, null);
      node.blur = () => this.owner.submitCommand(node, COMMAND_BLUR, null);
      node.setSelection = (start, end) =>
        this.owner.submitCommand(node, COMMAND_SET_SELECTION, { type: "selection", start, end });
    }
    if (kind === "View") {
      node.focus = () => this.owner.submitCommand(node, COMMAND_FOCUS, null);
      node.isFocused = () =>
        this.owner.submitCommandValue(node, COMMAND_GET_FOCUS, null).then((value) => {
          if (value === null || value.type !== "boolean")
            throw new Error(
              `native getFocus returned an invalid value for View node ${node.id}; update the host binary and renderer package together`,
            );
          return value.value;
        });
      node.blur = () => this.owner.submitCommand(node, COMMAND_BLUR, null);
    }
    if (kind === "VirtualList") {
      node.scrollToIndex = (index) =>
        this.owner.submitCommand(node, COMMAND_SCROLL_TO_INDEX, { type: "scroll-index", index, alignment: 0 });
      node.scrollToEnd = () => this.owner.submitCommand(node, COMMAND_SCROLL_TO_END, null);
      node.getScrollOffset = () =>
        this.owner.submitCommandValue(node, COMMAND_GET_SCROLL_OFFSET, null).then((value) => {
          if (value === null || value.type !== "scroll-offset")
            throw new Error(
              `native getScrollOffset returned an invalid value for VirtualList node ${node.id}; update the host binary and renderer package together`,
            );
          return value.value;
        });
      node.scrollToOffset = (offset) => {
        if (typeof offset !== "number" || !Number.isFinite(offset) || offset < 0)
          return Promise.reject(
            new TypeError(
              `scroll offset is invalid for VirtualList node ${node.id}; provide a finite non-negative number`,
            ),
          );
        return this.owner.submitCommand(node, COMMAND_SCROLL_TO_OFFSET, { type: "number", value: offset });
      };
    }
    return node;
  }

  allocateListener(_node: HostNodeInternal): number {
    const listenerId = this.nextListenerId;
    this.nextListenerId = nextU32(this.nextListenerId, "listener id");
    return listenerId;
  }
  private journalNode(node: HostNodeInternal): void {
    const transaction = this.transaction;
    if (transaction !== undefined && !transaction.nodeStates.has(node)) {
      transaction.nodeStates.set(node, this.captureNode(node));
    }
  }

  recordNodeMutation(node: HostNodeInternal): void {
    this.journalNode(node);
  }
  markPropsDirty(node: HostNodeInternal, groups: number): void {
    this.journalNode(node);
    const transaction = this.transaction;
    if (transaction !== undefined && !transaction.dirtyProps.has(node)) {
      transaction.dirtyProps.set(node, this.dirtyProps.get(node));
    }
    this.dirtyProps.set(node, (this.dirtyProps.get(node) ?? 0) | groups);
  }
  finalizeDirtyProps(): void {
    for (const [node] of this.dirtyProps) {
      const mask = this.updateNodeProps(node, node.props);
      if (node.attached) this.markUpdated(node, mask, true);
    }
    this.dirtyProps.clear();
  }

  private journalListenerMaps(listenerId: number): void {
    const transaction = this.transaction;
    if (transaction === undefined) return;
    if (!transaction.listeners.has(listenerId)) transaction.listeners.set(listenerId, this.listeners.get(listenerId));
    if (!transaction.inputListeners.has(listenerId))
      transaction.inputListeners.set(listenerId, this.inputListeners.get(listenerId));
  }
  private journalMap<K, V>(changes: Map<K, V | undefined> | undefined, map: Map<K, V>, key: K): void {
    if (changes !== undefined && !changes.has(key)) changes.set(key, map.get(key));
  }

  private journalSet(changes: Map<number, boolean | undefined> | undefined, set: Set<number>, key: number): void {
    if (changes !== undefined && !changes.has(key)) changes.set(key, set.has(key) ? true : undefined);
  }

  private journalRegistry(nodeId: number): void {
    if (this.transaction !== undefined && !this.transaction.registry.has(nodeId)) {
      this.transaction.registry.set(nodeId, this.listenerRegistry.snapshotNode(nodeId));
    }
  }

  private syncListenerId(listenerId: number): void {
    if (listenerId === 0) return;
    const binding = this.listenerRegistry.lookup(listenerId, 0xffff_ffff);
    this.journalListenerMaps(listenerId);
    if (binding !== undefined && (binding.node.attached || binding.node.detachedFocusPending)) {
      this.listeners.set(listenerId, binding.node);
      if (binding.inputCallbacks !== null) this.inputListeners.set(listenerId, binding.inputCallbacks);
      else this.inputListeners.delete(listenerId);
    } else {
      this.listeners.delete(listenerId);
      this.inputListeners.delete(listenerId);
    }
  }

  findListener(listenerId: number, revision: number): ListenerBinding | undefined {
    return this.listenerRegistry.lookup(listenerId, revision);
  }

  publishRevision(revision: number): void {
    this.listenerRegistry.publishRevision(revision);
  }

  beginTransaction(): void {
    if (this.transaction !== undefined) return;
    this.transaction = {
      nodeStates: new Map(),
      newNodes: new Set(),
      dirtyProps: new Map(),
      nodesById: new Map(),
      listeners: new Map(),
      inputListeners: new Map(),
      createdIds: new Map(),
      deletedRoots: new Map(),
      movedIds: new Map(),
      updatedMasks: new Map(),
      registry: new Map(),
      nextNodeId: this.nextNodeId,
      nextListenerId: this.nextListenerId,
    };
  }
  dispose(): void {
    this.transaction = undefined;
    this.dirtyProps.clear();
    this.nodesById.clear();
    this.listeners.clear();
    this.inputListeners.clear();
    this.createdIds.clear();
    this.deletedRoots.clear();
    this.movedIds.clear();
    this.updatedMasks.clear();
    this.listenerRegistry.clear();
    this.children.length = 0;
    this.allNodes.clear();
  }

  completeTransaction(): void {
    this.transaction = undefined;
  }

  rollbackTransaction(): void {
    const transaction = this.transaction;
    if (transaction === undefined) return;
    for (const node of transaction.newNodes) {
      this.resetNode(node);
      this.allNodes.delete(node);
    }
    for (const [node, value] of transaction.dirtyProps) {
      if (value === undefined) this.dirtyProps.delete(node);
      else this.dirtyProps.set(node, value);
    }
    for (const [node, state] of transaction.nodeStates) this.restoreNode(state);
    for (const [key, value] of transaction.nodesById) {
      if (value === undefined) this.nodesById.delete(key);
      else this.nodesById.set(key, value);
    }
    for (const [key, value] of transaction.listeners) {
      if (value === undefined) this.listeners.delete(key);
      else this.listeners.set(key, value);
    }
    for (const [key, value] of transaction.inputListeners) {
      if (value === undefined) this.inputListeners.delete(key);
      else this.inputListeners.set(key, value);
    }
    for (const [key, value] of transaction.createdIds) {
      if (value === undefined) this.createdIds.delete(key);
      else this.createdIds.add(key);
    }
    for (const [key, value] of transaction.deletedRoots) {
      if (value === undefined) this.deletedRoots.delete(key);
      else this.deletedRoots.add(key);
    }
    for (const [key, value] of transaction.movedIds) {
      if (value === undefined) this.movedIds.delete(key);
      else this.movedIds.add(key);
    }
    for (const [key, value] of transaction.updatedMasks) {
      if (value === undefined) this.updatedMasks.delete(key);
      else this.updatedMasks.set(key, value);
    }
    for (const [nodeId, snapshot] of transaction.registry) this.listenerRegistry.restoreNode(nodeId, snapshot);
    this.nextNodeId = transaction.nextNodeId;
    this.nextListenerId = transaction.nextListenerId;
    this.transaction = undefined;
  }

  private captureNode(node: HostNodeInternal): NodeStateSnapshot {
    return {
      node,
      parent: node.parent,
      children: [...node.children],
      index: node.index,
      style: node.style,
      text: node.text,
      tooltip: node.tooltip,
      acceptsPointerMove: node.acceptsPointerMove,
      keyListener: node.keyListener,
      listenerId: node.listenerId,
      focusCallback: node.focusCallback,
      blurCallback: node.blurCallback,
      detachedFocusPending: node.detachedFocusPending,
      nativeFocused: node.nativeFocused,
      pointerDownOutsideCallback: node.pointerDownOutsideCallback,
      hoverCallback: node.hoverCallback,
      hovered: node.hovered,
      scrollCallback: node.scrollCallback,
      listener: node.listener,
      focusable: node.focusable,
      selectable: node.selectable,
      disabled: node.disabled,
      dragCallbacks: { ...node.dragCallbacks },
      hostProperties: node.hostProperties,
      latestNativeText: node.latestNativeText,
      latestNativeEditSeq: node.latestNativeEditSeq,
      layoutCallback: node.layoutCallback,
      latestNativeSelection: node.latestNativeSelection === null ? null : { ...node.latestNativeSelection },
      inputCallbacks: node.inputCallbacks,
      inputCallbackSources: node.inputCallbackSources,
      visibleRangeCallback: node.visibleRangeCallback,
      animationCompleteCallback: node.animationCompleteCallback,
      extensionEventCallback: node.extensionEventCallback,
      extensionDescriptor: node.extensionDescriptor,
      extensionEventIds: [...node.extensionEventIds],
      lastAnimationGeneration: node.lastAnimationGeneration,
      accessibility: node.accessibility,
      attached: node.attached,
      props: { ...node.props },
      mounted: node.mounted,
    };
  }

  private restoreNode(state: NodeStateSnapshot): void {
    const node = state.node;
    node.parent = state.parent;
    node.children.splice(0, node.children.length, ...state.children);
    node.index = state.index;
    node.style = state.style;
    node.text = state.text;
    node.tooltip = state.tooltip;
    node.acceptsPointerMove = state.acceptsPointerMove;
    node.keyListener = state.keyListener;
    node.listenerId = state.listenerId;
    node.focusCallback = state.focusCallback;
    node.blurCallback = state.blurCallback;
    node.detachedFocusPending = state.detachedFocusPending;
    node.nativeFocused = state.nativeFocused;
    node.pointerDownOutsideCallback = state.pointerDownOutsideCallback;
    node.hoverCallback = state.hoverCallback;
    node.hovered = state.hovered;
    node.scrollCallback = state.scrollCallback;
    node.listener = state.listener;
    node.focusable = state.focusable;
    node.selectable = state.selectable;
    node.disabled = state.disabled;
    node.dragCallbacks = { ...state.dragCallbacks };
    node.hostProperties = state.hostProperties;
    node.latestNativeText = state.latestNativeText;
    node.latestNativeEditSeq = state.latestNativeEditSeq;
    node.layoutCallback = state.layoutCallback;
    node.latestNativeSelection = state.latestNativeSelection === null ? null : { ...state.latestNativeSelection };
    node.inputCallbacks = state.inputCallbacks;
    node.inputCallbackSources = state.inputCallbackSources;
    node.visibleRangeCallback = state.visibleRangeCallback;
    node.animationCompleteCallback = state.animationCompleteCallback;
    node.extensionEventCallback = state.extensionEventCallback;
    node.extensionDescriptor = state.extensionDescriptor;
    node.extensionEventIds = [...state.extensionEventIds];
    node.lastAnimationGeneration = state.lastAnimationGeneration;
    node.accessibility = state.accessibility;
    node.attached = state.attached;
    const props = node.props as Record<string, unknown>;
    for (const key of Object.keys(props)) delete props[key];
    Object.assign(props, state.props);
    node.mounted = state.mounted;
  }

  private resetNode(node: HostNodeInternal): void {
    node.parent = null;
    node.children.length = 0;
    node.index = 0;
    node.style = null;
    node.text = null;
    node.tooltip = null;
    node.acceptsPointerMove = false;
    node.keyListener = undefined;
    node.listenerId = 0;
    node.focusCallback = undefined;
    node.blurCallback = undefined;
    node.detachedFocusPending = false;
    node.nativeFocused = false;
    node.pointerDownOutsideCallback = undefined;
    node.hoverCallback = undefined;
    node.hovered = false;
    node.scrollCallback = undefined;
    node.listener = undefined;
    node.focusable = false;
    node.selectable = false;
    node.disabled = false;
    node.dragCallbacks = {};
    node.hostProperties = null;
    node.latestNativeText = null;
    node.latestNativeEditSeq = 0;
    node.layoutCallback = undefined;
    node.latestNativeSelection = null;
    node.inputCallbacks = null;
    node.inputCallbackSources = null;
    node.visibleRangeCallback = undefined;
    node.animationCompleteCallback = undefined;
    node.extensionEventCallback = undefined;
    node.extensionDescriptor = undefined;
    node.extensionEventIds = [];
    node.lastAnimationGeneration = null;
    node.accessibility = null;
    node.attached = false;
    const props = node.props as Record<string, unknown>;
    for (const key of Object.keys(props)) delete props[key];
    node.mounted = false;
  }

  setNodeProps(node: HostNodeInternal, props: HostProps): void {
    const previousListenerId = node.listenerId;
    this.journalNode(node);
    this.journalRegistry(node.id);
    try {
      validateProps(node.kind, props);
    } catch (error) {
      node.root.invalid = true;
      node.root.validationError = error instanceof Error ? error : new Error(String(error));
      throw error;
    }
    const spec = hostKindSpec(node.kind);
    node.style = node.kind === "RawText" ? null : props.style;
    node.tooltip =
      node.kind === "View" || node.kind === "Pressable" ? ((props.tooltip as string | undefined) ?? null) : null;
    node.extensionDescriptor = node.kind === "Extension" ? props.__extensionDescriptor : undefined;
    node.extensionEventIds = node.extensionDescriptor?.eventIds ?? [];
    node.extensionEventCallback =
      node.kind === "Extension" && typeof props.onExtensionEvent === "function"
        ? (props.onExtensionEvent as HostNodeInternal["extensionEventCallback"])
        : undefined;
    node.hostProperties =
      node.kind === "Extension" && node.extensionDescriptor !== undefined
        ? {
            type: "extension",
            value: normalizeExtensionProperties(
              node.extensionDescriptor,
              (props.__extensionProps ?? props) as Record<string, unknown>,
            ),
          }
        : spec.projector === "input"
          ? inputFor(node, props)
          : spec.projector === "virtualList"
            ? virtualListFor(node, props)
            : spec.projector === "image"
              ? imageFor(node, props)
              : spec.projector === "icon"
                ? iconFor(node, props)
                : spec.projector === "drag"
                  ? dragFor(node, props)
                  : null;
    node.accessibility = accessibilityFor(node.kind, props);
    node.disabled = (node.kind === "Pressable" || node.kind === "TextInput") && props.disabled === true;
    node.focusable =
      !node.disabled &&
      (node.kind === "View" || node.kind === "Pressable"
        ? (props.focusable ?? false)
        : node.kind === "Text" && props.onPress !== undefined);
    node.selectable = node.kind === "Text" && props.selectable === true;
    node.keyListener =
      !node.disabled && (node.kind === "View" || node.kind === "Pressable" || node.kind === "TextInput")
        ? props.onKeyDown
        : undefined;
    node.listener = (node.kind === "Pressable" || node.kind === "Text") && !node.disabled ? props.onPress : undefined;
    node.pointerCallbacks =
      !node.disabled && (node.kind === "View" || node.kind === "Pressable")
        ? { down: props.onPointerDown, up: props.onPointerUp }
        : null;
    node.pointerMoveCallback =
      !node.disabled && (node.kind === "View" || node.kind === "Pressable") ? props.onPointerMove : undefined;
    node.acceptsPointerMove = node.pointerMoveCallback !== undefined;
    node.focusCallback =
      !node.disabled && (node.kind === "View" || node.kind === "Pressable" || node.kind === "Text")
        ? props.onFocus
        : undefined;
    node.blurCallback =
      !node.disabled && (node.kind === "View" || node.kind === "Pressable" || node.kind === "Text")
        ? props.onBlur
        : undefined;
    node.pointerDownOutsideCallback = !node.disabled && node.kind === "View" ? props.onPointerDownOutside : undefined;
    node.hoverCallback =
      !node.disabled && (node.kind === "View" || node.kind === "Pressable") ? props.onHoverChange : undefined;
    node.scrollCallback = node.kind === "View" ? props.onScroll : undefined;
    node.dragCallbacks =
      !node.disabled && (node.kind === "View" || node.kind === "Pressable")
        ? {
            over: props.onDragOver,
            drop: props.onDrop,
            externalFileDrop: props.onExternalFileDrop,
          }
        : {};
    node.layoutCallback = spec.allowsLayout ? props.onLayout : undefined;
    if (node.hoverCallback === undefined) node.hovered = false;
    const previousInputCallbacks = node.inputCallbacks;
    const previousInputSources = node.inputCallbackSources;
    node.inputCallbacks = null;
    node.inputCallbackSources = null;
    node.visibleRangeCallback = node.kind === "VirtualList" ? props.__onVisibleRange : undefined;
    node.animationCompleteCallback = node.style?.transition?.onComplete;
    if (node.kind === "TextInput") {
      const input = props as TextInputProps;
      const inputSources: TextInputCallbackSources = {
        change: input.onChangeText,
        selection: input.onSelectionChange,
        focus: input.onFocus,
        blur: input.onBlur,
        submit: input.onSubmitEditing,
      };
      const sameSources =
        previousInputSources?.change === inputSources.change &&
        previousInputSources?.selection === inputSources.selection &&
        previousInputSources?.focus === inputSources.focus &&
        previousInputSources?.blur === inputSources.blur &&
        previousInputSources?.submit === inputSources.submit;
      node.inputCallbackSources = inputSources;
      node.inputCallbacks =
        sameSources && previousInputCallbacks !== null
          ? previousInputCallbacks
          : {
              change: input.onChangeText ? (event) => input.onChangeText?.(event.text) : undefined,
              selection: input.onSelectionChange ? (event) => input.onSelectionChange?.(event.selection) : undefined,
              focus: input.onFocus ? () => input.onFocus?.() : undefined,
              blur: input.onBlur ? () => input.onBlur?.() : undefined,
              submit: input.onSubmitEditing ? (value) => input.onSubmitEditing?.(value) : undefined,
            };
    }
    const hasListener =
      node.listener !== undefined ||
      node.keyListener !== undefined ||
      node.acceptsPointerMove ||
      (node.pointerCallbacks !== null &&
        (node.pointerCallbacks.down !== undefined || node.pointerCallbacks.up !== undefined)) ||
      node.focusCallback !== undefined ||
      node.blurCallback !== undefined ||
      node.pointerDownOutsideCallback !== undefined ||
      node.hoverCallback !== undefined ||
      node.scrollCallback !== undefined ||
      node.layoutCallback !== undefined ||
      Object.values(node.dragCallbacks).some((callback) => callback !== undefined) ||
      Object.values(node.inputCallbacks ?? {}).some((callback) => callback !== undefined) ||
      node.visibleRangeCallback !== undefined ||
      node.animationCompleteCallback !== undefined ||
      (node.kind === "Extension" && node.extensionEventCallback !== undefined && node.extensionEventIds.length > 0);
    if (hasListener && node.listenerId === 0) node.listenerId = this.allocateListener(node);
    if (!hasListener) node.listenerId = 0;
    this.listenerRegistry.bind(node, node.listenerId, hasListener);
    this.syncListenerId(previousListenerId);
    this.syncListenerId(node.listenerId);
  }

  updateNodeProps(node: HostNodeInternal, props: HostProps): number {
    const previousStyle = node.style;
    const previousListenerId = node.listenerId;
    const previousListener = this.listenerRegistry.current(node.id);
    const previousFocusable = node.focusable;
    const previousSelectable = node.selectable;
    const previousProperties = node.hostProperties;
    const previousAccessibility = node.accessibility;
    const previousTooltip = node.tooltip;
    const previousAcceptsPointerMove = node.acceptsPointerMove;
    this.setNodeProps(node, props);
    let mask = 0;
    if (!equalStyle(previousStyle, node.style)) mask |= UPDATE_STYLE;
    if (previousListenerId !== node.listenerId || previousListener !== this.listenerRegistry.current(node.id))
      mask |= UPDATE_LISTENER;
    if (previousFocusable !== node.focusable) mask |= UPDATE_FOCUSABLE;
    if (previousSelectable !== node.selectable) mask |= UPDATE_SELECTABLE;
    if (previousTooltip !== node.tooltip) mask |= UPDATE_TOOLTIP;
    if (previousAcceptsPointerMove !== node.acceptsPointerMove) mask |= UPDATE_POINTER_MOVE;
    if (!equalHostProperties(previousProperties, node.hostProperties)) mask |= UPDATE_PROPERTIES;
    if (!equalAccessibility(previousAccessibility, node.accessibility)) mask |= UPDATE_ACCESSIBILITY;
    return mask;
  }

  detachFromParent(node: HostNodeInternal): void {
    const parent = node.parent ?? this.syntheticRoot;
    const siblings = parent.children;
    this.journalNode(parent);
    this.journalNode(node);
    const index = siblings.indexOf(node);
    if (index >= 0) siblings.splice(index, 1);
    node.parent = null;
    this.refreshChildIndexes(parent);
  }

  attachSubtree(node: HostNodeInternal, bootstrapped: boolean): void {
    const wasCreated = this.createdIds.has(node.id);
    this.journalSet(this.transaction?.deletedRoots, this.deletedRoots, node.id);
    const wasDeleted = this.deletedRoots.delete(node.id);
    const attach = (current: HostNodeInternal): void => {
      this.journalNode(current);
      current.attached = true;
      current.detachedFocusPending = false;
      this.allNodes.add(current);
      this.journalMap(this.transaction?.nodesById, this.nodesById, current.id);
      this.nodesById.set(current.id, current);
      if (current.listenerId !== 0) this.syncListenerId(current.listenerId);
      for (const child of current.children) attach(child);
    };
    attach(node);
    if (!bootstrapped || wasCreated) return;
    if (wasDeleted) {
      this.journalSet(this.transaction?.movedIds, this.movedIds, node.id);
      this.movedIds.add(node.id);
      return;
    }
    const markCreated = (current: HostNodeInternal): void => {
      this.journalSet(this.transaction?.createdIds, this.createdIds, current.id);
      this.createdIds.add(current.id);
      for (const child of current.children) markCreated(child);
    };
    markCreated(node);
  }

  detachSubtree(node: HostNodeInternal): void {
    const retainFocusRouting = node.nativeFocused;
    const listenerId = node.listenerId;
    this.journalNode(node);
    node.attached = false;
    node.detachedFocusPending = retainFocusRouting;
    this.journalMap(this.transaction?.nodesById, this.nodesById, node.id);
    this.nodesById.delete(node.id);
    if (!retainFocusRouting) {
      this.journalRegistry(node.id);
      this.listenerRegistry.remove(node.id);
    }
    this.syncListenerId(listenerId);
    for (const child of node.children) this.detachSubtree(child);
  }

  releaseDetachedFocus(node: HostNodeInternal): void {
    if (node.attached || !node.detachedFocusPending) return;
    const listenerId = node.listenerId;
    this.journalNode(node);
    node.detachedFocusPending = false;
    this.journalRegistry(node.id);
    this.listenerRegistry.remove(node.id);
    this.syncListenerId(listenerId);
    node.nativeFocused = false;
  }

  refreshChildIndexes(parent: HostNodeInternal): void {
    this.journalNode(parent);
    parent.children.forEach((child, index) => {
      this.journalNode(child);
      child.index = index;
    });
  }

  markMoved(node: HostNodeInternal, bootstrapped: boolean): void {
    if (bootstrapped && !this.createdIds.has(node.id)) {
      this.journalSet(this.transaction?.movedIds, this.movedIds, node.id);
      this.movedIds.add(node.id);
    }
  }

  markUpdated(node: HostNodeInternal, mask: number, bootstrapped: boolean): void {
    if (bootstrapped && mask !== 0 && !this.createdIds.has(node.id)) {
      if (this.transaction !== undefined && !this.transaction.updatedMasks.has(node.id)) {
        this.transaction.updatedMasks.set(node.id, this.updatedMasks.get(node.id));
      }
      this.updatedMasks.set(node.id, (this.updatedMasks.get(node.id) ?? 0) | mask);
    }
  }

  markDeleted(node: HostNodeInternal, bootstrapped: boolean): void {
    if (bootstrapped) {
      this.journalSet(this.transaction?.deletedRoots, this.deletedRoots, node.id);
      this.deletedRoots.add(node.id);
    }
  }

  clearMutations(): void {
    for (const id of this.createdIds) this.journalSet(this.transaction?.createdIds, this.createdIds, id);
    for (const id of this.deletedRoots) this.journalSet(this.transaction?.deletedRoots, this.deletedRoots, id);
    for (const id of this.movedIds) this.journalSet(this.transaction?.movedIds, this.movedIds, id);
    for (const id of this.updatedMasks.keys()) {
      if (this.transaction !== undefined && !this.transaction.updatedMasks.has(id)) {
        this.transaction.updatedMasks.set(id, this.updatedMasks.get(id));
      }
    }
    this.createdIds.clear();
    this.deletedRoots.clear();
    this.movedIds.clear();
    this.updatedMasks.clear();
  }

  snapshotNodes(): SnapshotNode[] {
    const nodes: SnapshotNode[] = [];
    const visit = (node: HostNodeInternal, parentId: number, index: number): void => {
      nodes.push({
        id: node.id,
        parentId,
        index,
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
      });
      node.children.forEach((child, childIndex) => visit(child, node.id, childIndex));
    };
    visit(this.syntheticRoot, 0, 0);
    return nodes;
  }

  nodeDepth(node: HostNodeInternal): number {
    let depth = 0;
    let parent = node.parent;
    while (parent !== null) {
      depth += 1;
      parent = parent.parent;
    }
    return depth;
  }

  nativeParentId(node: HostNodeInternal): number {
    return node.parent?.id ?? this.syntheticRoot.id;
  }
}
