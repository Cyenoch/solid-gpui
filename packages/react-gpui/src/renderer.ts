import type { ReactNode } from "react";
import Reconciler from "react-reconciler";
import { DefaultEventPriority, LegacyRoot } from "react-reconciler/constants";
import {
  COMMAND_BLUR,
  COMMAND_FOCUS,
  COMMAND_KIND,
  COMMAND_SCROLL_TO_END,
  COMMAND_SCROLL_TO_INDEX,
  COMMAND_SET_SELECTION,
  decodeEvent,
  encodeFrame,
  EVENT_ANIMATION_COMPLETE,
  EVENT_BLUR,
  EVENT_CHANGE,
  EVENT_COMMAND_RESULT,
  EVENT_FOCUS,
  EVENT_PRESS,
  EVENT_SELECTION,
  EVENT_VISIBLE_RANGE,
  FrameDecoder,
  PROTOCOL_VERSION,
  type Command,
  type HostPropertiesWire,
  type Patch,
  type PatchOperation,
  type PressEventFrame,
  type SnapshotNode,
  type TextInputEventPayload,
  UPDATE_ACCESSIBILITY,
  UPDATE_LISTENER,
  UPDATE_PROPERTIES,
  UPDATE_STYLE,
  UPDATE_TEXT,
} from "./protocol";
import { encodeStyle, validateStyle, type StyleProp, type Transition } from "./style";
import type { Transport } from "./transport";

export type HostKind = "View" | "Text" | "Pressable" | "TextInput" | "RawText" | "VirtualList";
export type PressEventType = "press";

export interface HostNode {
  readonly id: number;
  readonly kind: HostKind;
}
export interface AccessibilityProps {
  readonly accessibilityRole?: "button" | "text" | "textbox" | "checkbox" | "heading" | "generic";
  readonly accessibilityLabel?: string;
  readonly accessibilityDescription?: string;
  readonly accessibilityDisabled?: boolean;
  readonly accessibilityChecked?: boolean;
  readonly accessibilitySelected?: boolean;
  readonly accessibilityValue?: string;
}

export interface TextInputProps extends AccessibilityProps {
  readonly value?: string;
  readonly defaultValue?: string;
  readonly placeholder?: string;
  readonly onChangeText?: (value: string) => void;
  readonly onSelectionChange?: (selection: { start: number; end: number; composing?: { start: number; end: number } | null }) => void;
  readonly onFocus?: () => void;
  readonly onBlur?: () => void;
  readonly multiline?: boolean;
  readonly disabled?: boolean;
  readonly style?: StyleProp;
  readonly children?: ReactNode;
}
export interface TextInputEvent {
  readonly type: "change" | "selection" | "focus" | "blur";
  readonly text: string;
  readonly selection: { start: number; end: number; composing: { start: number; end: number } | null };
  readonly editSeq: number;
  readonly target: HostNode;
}
export interface TextInputHandle extends HostNode {
  focus(): Promise<void>;
  blur(): Promise<void>;
  setSelection(start: number, end: number): Promise<void>;
}

export interface VirtualListProps<T> extends AccessibilityProps {
  readonly data: readonly T[];
  readonly itemKey: (item: T, index: number) => string | number;
  readonly renderItem: (item: T, index: number) => ReactNode;
  readonly estimatedItemSize: number;
  readonly overscan?: number;
  readonly initialNumToRender?: number;
  readonly onEndReached?: () => void;
  readonly style?: StyleProp;
}
export interface VirtualListHandle extends HostNode {
  scrollToIndex(index: number): Promise<void>;
  scrollToEnd(): Promise<void>;
}
export interface AnimationCompleteEvent {
  readonly generation: number;
  readonly target: HostNode;
}

/** A press is a semantic notification; native cancellation is intentionally unavailable. */
export interface PressEvent {
  readonly type: PressEventType;
  readonly surfaceId: number;
  readonly epoch: number;
  readonly revision: number;
  readonly sequence: number;
  readonly target: HostNode;
}

export type PressHandler = (event: PressEvent) => void;

export interface ViewProps extends AccessibilityProps {
  readonly style?: StyleProp;
  readonly children?: ReactNode;
}
export interface TextProps extends AccessibilityProps {
  readonly style?: StyleProp;
  readonly children?: ReactNode;
}
export interface PressableProps extends AccessibilityProps {
  readonly style?: StyleProp;
  readonly onPress?: PressHandler;
  readonly children?: ReactNode;
}
interface HostProps extends AccessibilityProps {
  readonly style?: StyleProp;
  readonly onPress?: PressHandler;
  readonly children?: ReactNode;
  readonly __itemCount?: number;
  readonly __rangeStart?: number;
  readonly __rangeEnd?: number;
  readonly __estimatedItemSize?: number;
  readonly __overscan?: number;
  readonly __onVisibleRange?: (start: number, end: number) => void;
  readonly __onAnimationComplete?: (generation: number) => void;
  readonly [key: string]: unknown;
}
interface TextInputWire {
  readonly value: string;
  readonly placeholder: string | null;
  readonly multiline: boolean;
  readonly disabled: boolean;
  readonly controlled: boolean;
  readonly ackEditSeq: number;
  readonly selectionStart: number;
  readonly selectionEnd: number;
  readonly markedStart: number | null;
  readonly markedEnd: number | null;
}
interface VirtualListWire {
  readonly itemCount: number;
  readonly rangeStart: number;
  readonly rangeEnd: number;
  readonly estimatedItemSize: number;
  readonly overscan: number;
}
interface AccessibilityWire {
  readonly role: number;
  readonly label: string | null;
  readonly description: string | null;
  readonly disabled: boolean;
  readonly checked: boolean | null;
  readonly selected: boolean | null;
  readonly value: string | null;
}
interface PendingCommand {
  readonly resolve: () => void;
  readonly reject: (error: Error) => void;
}
interface TextInputCallbacks {
  readonly change?: (event: TextInputEvent) => void;
  readonly selection?: (event: TextInputEvent) => void;
  readonly focus?: (event: TextInputEvent) => void;
  readonly blur?: (event: TextInputEvent) => void;
}
interface HostNodeInternal extends HostNode {
  readonly kind: HostKind;
  readonly root: RootContainer;
  parent: HostNodeInternal | null;
  children: HostNodeInternal[];
  index: number;
  style: StyleProp;
  text: string | null;
  listenerId: number;
  listener: PressHandler | undefined;
  hostProperties: TextInputWire | VirtualListWire | null;
  latestNativeText: string | null;
  latestNativeEditSeq: number;
  latestNativeSelection: { start: number; end: number; markedStart: number | null; markedEnd: number | null } | null;
  focus?: () => Promise<void>;
  blur?: () => Promise<void>;
  setSelection?: (start: number, end: number) => Promise<void>;
  scrollToIndex?: (index: number) => Promise<void>;
  scrollToEnd?: () => Promise<void>;
  inputCallbacks: TextInputCallbacks | null;
  visibleRangeCallback?: (start: number, end: number) => void;
  animationCompleteCallback?: (generation: number) => void;
  lastAnimationGeneration: number | null;
  accessibility: AccessibilityWire | null;
  attached: boolean;
}
interface HostContext {
  readonly root: RootContainer;
  readonly parentKind: HostKind | null;
}

function assertChildKind(parentKind: HostKind | null, childKind: HostKind): void {
  if (parentKind === null) return;
  if (parentKind === "Text" && childKind !== "RawText") {
    throw new TypeError("Text children must be raw text; use a separate Text node for nested content");
  }
  if (childKind === "RawText" && parentKind !== "Text") {
    throw new TypeError("Raw text is only valid directly under Text");
  }
}

function assertChildForRoot(root: RootContainer, parentKind: HostKind | null, childKind: HostKind): void {
  try {
    assertChildKind(parentKind, childKind);
  } catch (error) {
    root.invalid = true;
    throw error;
  }
}
const KIND_CODES: Record<HostKind, 1 | 2 | 3 | 4 | 5 | 6> = {
  View: 1,
  Text: 2,
  Pressable: 3,
  TextInput: 5,
  RawText: 4,
  VirtualList: 6,
};
const VALID_HOST_TYPES: Record<string, true> = { View: true, Text: true, Pressable: true, TextInput: true, VirtualList: true };
const ACCESSIBILITY_PROPS: Record<string, true> = {
  accessibilityRole: true,
  accessibilityLabel: true,
  accessibilityDescription: true,
  accessibilityDisabled: true,
  accessibilityChecked: true,
  accessibilitySelected: true,
  accessibilityValue: true,
};
const ALLOWED_PROPS: Record<HostKind, Record<string, true>> = {
  View: { style: true, children: true, ref: true, ...ACCESSIBILITY_PROPS },
  Text: { style: true, children: true, ref: true, ...ACCESSIBILITY_PROPS },
  Pressable: { style: true, onPress: true, children: true, ref: true, ...ACCESSIBILITY_PROPS },
  TextInput: { style: true, children: true, ref: true, value: true, defaultValue: true, placeholder: true, onChangeText: true, onSelectionChange: true, onFocus: true, onBlur: true, multiline: true, disabled: true, ...ACCESSIBILITY_PROPS },
  VirtualList: { style: true, children: true, ref: true, __itemCount: true, __rangeStart: true, __rangeEnd: true, __estimatedItemSize: true, __overscan: true, __onVisibleRange: true, __onAnimationComplete: true, ...ACCESSIBILITY_PROPS },
  RawText: { children: true, ref: true },
};
const ROLE_CODES: Record<NonNullable<AccessibilityProps["accessibilityRole"]>, number> = {
  generic: 1,
  button: 2,
  text: 3,
  textbox: 4,
  checkbox: 5,
  heading: 6,
};

function accessibilityFor(kind: HostKind, props: HostProps): AccessibilityWire | null {
  const role = props.accessibilityRole ?? (kind === "TextInput" ? "textbox" : kind === "Pressable" ? "button" : undefined);
  const checked = props.accessibilityChecked;
  const input = props as TextInputProps;
  if (checked !== undefined && role !== "checkbox") throw new TypeError("accessibilityChecked requires accessibilityRole=checkbox");
  if (role === undefined && Object.keys(props).every((key) => !ACCESSIBILITY_PROPS[key])) return null;
  return {
    role: role === undefined ? 0 : ROLE_CODES[role],
    label: props.accessibilityLabel ?? null,
    description: props.accessibilityDescription ?? null,
    disabled: props.accessibilityDisabled ?? input.disabled ?? false,
    checked: checked ?? null,
    selected: props.accessibilitySelected ?? null,
    value: props.accessibilityValue ?? null,
  };
}
function inputFor(node: HostNodeInternal, props: HostProps): TextInputWire | null {
  if (node.kind !== "TextInput") return null;
  const input = props as TextInputProps;
  const previous = node.hostProperties;
  const previousInput = previous && "value" in previous ? previous : null;
  return {
    value: input.value ?? node.latestNativeText ?? previousInput?.value ?? input.defaultValue ?? "",
    placeholder: input.placeholder ?? null,
    multiline: input.multiline ?? false,
    disabled: input.disabled ?? false,
    controlled: input.value !== undefined,
    ackEditSeq: node.latestNativeEditSeq || previousInput?.ackEditSeq || 0,
    selectionStart: node.latestNativeSelection?.start ?? previousInput?.selectionStart ?? 0,
    selectionEnd: node.latestNativeSelection?.end ?? previousInput?.selectionEnd ?? 0,
    markedStart: node.latestNativeSelection?.markedStart ?? previousInput?.markedStart ?? null,
    markedEnd: node.latestNativeSelection?.markedEnd ?? previousInput?.markedEnd ?? null,
  };
}
function virtualListFor(node: HostNodeInternal, props: HostProps): VirtualListWire | null {
  if (node.kind !== "VirtualList") return null;
  const itemCount = props.__itemCount;
  const rangeStart = props.__rangeStart;
  const rangeEnd = props.__rangeEnd;
  const estimatedItemSize = props.__estimatedItemSize;
  const overscan = props.__overscan ?? 0;
  if (itemCount === undefined || rangeStart === undefined || rangeEnd === undefined || estimatedItemSize === undefined) {
    throw new TypeError("VirtualList host properties are incomplete");
  }
  if (![itemCount, rangeStart, rangeEnd, overscan].every(Number.isInteger) || itemCount < 0 || rangeStart < 0 || rangeEnd < rangeStart || rangeEnd > itemCount || overscan < 0 || estimatedItemSize <= 0 || !Number.isFinite(estimatedItemSize)) {
    throw new RangeError("VirtualList host properties are invalid");
  }
  assertU32Option("VirtualList itemCount", itemCount);
  assertU32Option("VirtualList rangeStart", rangeStart);
  assertU32Option("VirtualList rangeEnd", rangeEnd);
  assertU32Option("VirtualList overscan", overscan);
  return { itemCount, rangeStart, rangeEnd, estimatedItemSize, overscan };
}
function hostPropertiesWire(value: TextInputWire | VirtualListWire | null): HostPropertiesWire | null {
  if (value === null) return null;
  if ("value" in value) return [1, value.value, value.placeholder, value.multiline, value.disabled, value.controlled, value.ackEditSeq, value.selectionStart, value.selectionEnd, value.markedStart, value.markedEnd];
  return [2, value.itemCount, value.rangeStart, value.rangeEnd, value.estimatedItemSize, value.overscan];
}
function accessibilityWire(value: AccessibilityWire | null): readonly unknown[] | null {
  return value === null ? null : [value.role, value.label, value.description, value.disabled, value.checked, value.selected, value.value];
}

let nextSurfaceId = 1;
let currentUpdatePriority = DefaultEventPriority;

function nextU32(value: number, name: string): number {
  if (value >= 0xffff_ffff) throw new RangeError(`${name} exhausted u32 range`);
  return value + 1;
}

function assertU32Option(name: string, value: number): number {
  if (!Number.isInteger(value) || value < 0 || value > 0xffff_ffff) throw new RangeError(`${name} must be a u32`);
  return value;
}
function validateProps(kind: HostKind, props: HostProps): void {
  for (const key of Object.keys(props)) {
    if (!ALLOWED_PROPS[kind][key]) throw new TypeError(`Unsupported ${kind} prop: ${key}`);
  }
  if (kind !== "RawText") validateStyle(props.style);
  if (kind === "Pressable" && props.onPress !== undefined && typeof props.onPress !== "function") {
    throw new TypeError("Pressable onPress must be a function");
  }
  if (kind === "VirtualList") {
    virtualListFor({ kind, hostProperties: null } as HostNodeInternal, props);
    if (props.__onVisibleRange !== undefined && typeof props.__onVisibleRange !== "function") throw new TypeError("VirtualList range callback must be a function");
  }
}

function detachFromParent(node: HostNodeInternal): void {
  const parent = node.parent;
  const siblings = parent ? parent.children : node.root.children;
  const index = siblings.indexOf(node);
  if (index >= 0) siblings.splice(index, 1);
  node.parent = null;
  refreshChildIndexes(parent ?? node.root.syntheticRoot);
}

function detachSubtree(node: HostNodeInternal): void {
  node.attached = false;
  node.root.nodesById.delete(node.id);
  node.root.listeners.delete(node.listenerId);
  node.root.inputListeners.delete(node.listenerId);
  node.listener = undefined;
  node.inputCallbacks = null;
  for (const child of node.children) detachSubtree(child);
}
function refreshChildIndexes(parent: HostNodeInternal): void {
  parent.children.forEach((child, index) => {
    child.index = index;
  });
}

function setNodeProps(node: HostNodeInternal, props: HostProps): void {
  validateProps(node.kind, props);
  node.style = node.kind === "RawText" ? null : props.style;
  node.hostProperties = node.kind === "TextInput" ? inputFor(node, props) : node.kind === "VirtualList" ? virtualListFor(node, props) : null;
  node.accessibility = accessibilityFor(node.kind, props);
  node.listener = node.kind === "Pressable" ? props.onPress : undefined;
  node.inputCallbacks = null;
  node.visibleRangeCallback = node.kind === "VirtualList" ? props.__onVisibleRange : undefined;
  node.animationCompleteCallback = node.style?.transition?.onComplete;
  if (node.kind === "TextInput") {
    const input = props as TextInputProps;
    node.inputCallbacks = {
      change: input.onChangeText ? (event) => input.onChangeText?.(event.text) : undefined,
      selection: input.onSelectionChange ? (event) => input.onSelectionChange?.(event.selection) : undefined,
      focus: input.onFocus ? () => input.onFocus?.() : undefined,
      blur: input.onBlur ? () => input.onBlur?.() : undefined,
    };
  }
  const hasListener = node.listener !== undefined
    || Object.values(node.inputCallbacks ?? {}).some((callback) => callback !== undefined)
    || node.visibleRangeCallback !== undefined
    || node.animationCompleteCallback !== undefined;
  if (hasListener && node.listenerId === 0) node.listenerId = node.root.allocateListener(node);
  if (!hasListener && node.listenerId !== 0) {
    node.root.listeners.delete(node.listenerId);
    node.root.inputListeners.delete(node.listenerId);
    node.listenerId = 0;
  }
  if (node.listenerId !== 0) {
    node.root.listeners.set(node.listenerId, node);
    if (node.inputCallbacks !== null) node.root.inputListeners.set(node.listenerId, node.inputCallbacks);
  }
}

function updateNodeProps(node: HostNodeInternal, props: HostProps): number {
  const previousStyle = node.style;
  const previousListenerId = node.listenerId;
  const previousProperties = node.hostProperties;
  const previousAccessibility = node.accessibility;
  setNodeProps(node, props);
  let mask = 0;
  if (previousStyle !== node.style) mask |= UPDATE_STYLE;
  if (previousListenerId !== node.listenerId) mask |= UPDATE_LISTENER;
  if (JSON.stringify(previousProperties) !== JSON.stringify(node.hostProperties)) mask |= UPDATE_PROPERTIES;
  if (JSON.stringify(previousAccessibility) !== JSON.stringify(node.accessibility)) mask |= UPDATE_ACCESSIBILITY;
  return mask;
}
// The protocol always has one parentId=0 View; this synthetic node remains in the unmount snapshot.
class RootContainer {
  readonly children: HostNodeInternal[];
  readonly syntheticRoot: HostNodeInternal;
  readonly listeners = new Map<number, HostNodeInternal>();
  readonly nodesById = new Map<number, HostNodeInternal>();
  readonly surfaceId: number;
  private readonly pendingCommands = new Map<number, PendingCommand>();
  private nextRequestId = 1;
  readonly epoch: number;
  revision = 0;
  bootstrapped = false;
  readonly inputListeners = new Map<number, TextInputCallbacks>();
  private readonly createdIds = new Set<number>();
  private readonly deletedRoots = new Set<number>();
  private readonly movedIds = new Set<number>();
  private readonly updatedMasks = new Map<number, number>();
  invalid = false;
  unmounted = false;
  private nextNodeId = 2;
  private nextListenerId = 1;
  private lastEventSequence = 0;
  private hasEventSequence = false;
  private readonly decoder: FrameDecoder;
  private readonly unsubscribe: () => void;

  constructor(
    readonly transport: Transport,
    surfaceId: number,
    epoch: number,
    maxFrameSize: number,
  ) {
    this.surfaceId = assertU32Option("surfaceId", surfaceId);
    this.epoch = assertU32Option("epoch", epoch);
    this.syntheticRoot = {
      id: 1,
      kind: "View",
      root: this,
      parent: null,
      children: [],
      index: 0,
      style: null,
      text: null,
      listenerId: 0,
      listener: undefined,
      hostProperties: null,
      latestNativeText: null,
      latestNativeEditSeq: 0,
      latestNativeSelection: null,
      inputCallbacks: null,
      lastAnimationGeneration: null,
      accessibility: null,
      attached: true,
    };
    this.nodesById.set(1, this.syntheticRoot);
    this.children = this.syntheticRoot.children;
    this.decoder = new FrameDecoder(maxFrameSize);
    this.unsubscribe = transport.onData((chunk) => this.receive(chunk));

  }
  allocateNode(kind: HostKind): HostNodeInternal {
    const node: HostNodeInternal = {
      id: this.nextNodeId,
      kind,
      root: this,
      parent: null,
      children: [],
      index: 0,
      style: null,
      text: null,
      listenerId: 0,
      listener: undefined,
      hostProperties: null,
      latestNativeText: null,
      latestNativeEditSeq: 0,
      latestNativeSelection: null,
      inputCallbacks: null,
      lastAnimationGeneration: null,
      accessibility: null,
      attached: true,
    };
    this.nextNodeId = nextU32(this.nextNodeId, "node id");
    this.nodesById.set(node.id, node);
    if (kind === "TextInput") {
      node.focus = () => this.submitCommand(node, COMMAND_FOCUS, null);
      node.blur = () => this.submitCommand(node, COMMAND_BLUR, null);
      node.setSelection = (start, end) => this.submitCommand(node, COMMAND_SET_SELECTION, [start, end]);
    }
    if (kind === "VirtualList") {
      node.scrollToIndex = (index) => this.submitCommand(node, COMMAND_SCROLL_TO_INDEX, [index, 0]);
      node.scrollToEnd = () => this.submitCommand(node, COMMAND_SCROLL_TO_END, null);
    }
    if (this.bootstrapped) this.createdIds.add(node.id);
    return node;
  }

  allocateListener(node: HostNodeInternal): number {
    const listenerId = this.nextListenerId;
    this.nextListenerId = nextU32(this.nextListenerId, "listener id");
    this.listeners.set(listenerId, node);
    return listenerId;
  }
  submitCommand(node: HostNodeInternal, kind: number, payload: readonly [number, number] | null): Promise<void> {
    if (this.unmounted || !node.attached) return Promise.reject(new Error("host node is unavailable"));
    const isInput = node.kind === "TextInput";
    const isList = node.kind === "VirtualList";
    if (!isInput && !isList) return Promise.reject(new Error("host node does not support commands"));
    if (isInput && node.hostProperties && "disabled" in node.hostProperties && node.hostProperties.disabled) {
      return Promise.reject(new Error("TextInput is disabled"));
    }
    if (isInput && !([COMMAND_FOCUS, COMMAND_BLUR, COMMAND_SET_SELECTION] as number[]).includes(kind)) return Promise.reject(new Error("unknown TextInput command"));
    if (isList && !([COMMAND_SCROLL_TO_INDEX, COMMAND_SCROLL_TO_END] as number[]).includes(kind)) return Promise.reject(new Error("unknown VirtualList command"));
    if (kind === COMMAND_SET_SELECTION && (payload === null || !payload.every((value) => Number.isInteger(value) && value >= 0 && value <= 0xffff_ffff) || payload[0] > payload[1])) return Promise.reject(new Error("invalid UTF-16 selection"));
    if (kind === COMMAND_SCROLL_TO_INDEX && (payload === null || !Number.isInteger(payload[0]) || payload[0] < 0 || payload[0] > 0xffff_ffff || (node.hostProperties && "itemCount" in node.hostProperties && payload[0] >= node.hostProperties.itemCount))) return Promise.reject(new Error("VirtualList index is out of range"));
    let requestId: number;
    try {
      requestId = this.nextRequestId;
      this.nextRequestId = nextU32(this.nextRequestId, "command request id");
    } catch (error) {
      return Promise.reject(error);
    }
    const command: Command = [PROTOCOL_VERSION, COMMAND_KIND, this.surfaceId, this.epoch, this.revision, requestId, node.id, kind as 1 | 2 | 3 | 4 | 5, payload];
    return new Promise<void>((resolve, reject) => {
      this.pendingCommands.set(requestId, { resolve, reject });
      try {
        this.transport.submit(encodeFrame(command));
      } catch (error) {
        this.pendingCommands.delete(requestId);
        reject(error instanceof Error ? error : new Error(String(error)));
      }
    });
  }

  beginRender(): void {
    this.invalid = false;
    this.clearMutations();
  }

  markMoved(node: HostNodeInternal): void {
    if (this.bootstrapped && !this.createdIds.has(node.id)) this.movedIds.add(node.id);
  }

  markUpdated(node: HostNodeInternal, mask: number): void {
    if (this.bootstrapped && mask !== 0 && !this.createdIds.has(node.id)) {
      this.updatedMasks.set(node.id, (this.updatedMasks.get(node.id) ?? 0) | mask);
    }
  }

  markDeleted(node: HostNodeInternal): void {
    if (this.bootstrapped) this.deletedRoots.add(node.id);
  }

  private clearMutations(): void {
    this.createdIds.clear();
    this.deletedRoots.clear();
    this.movedIds.clear();
    this.updatedMasks.clear();
  }
  private snapshotNodes(): SnapshotNode[] {
    const nodes: SnapshotNode[] = [];
    const visit = (node: HostNodeInternal, parentId: number, index: number): void => {
      nodes.push([
        node.id,
        parentId,
        index,
        KIND_CODES[node.kind],
        encodeStyle(node.style),
        node.kind === "RawText" ? node.text : null,
        node.listenerId,
        hostPropertiesWire(node.hostProperties),
        accessibilityWire(node.accessibility),
      ]);
      node.children.forEach((child, childIndex) => visit(child, node.id, childIndex));
    };
    visit(this.syntheticRoot, 0, 0);
    return nodes;
  }

  private nodeDepth(node: HostNodeInternal): number {
    let depth = 0;
    let parent = node.parent;
    while (parent !== null) {
      depth += 1;
      parent = parent.parent;
    }
    return depth;
  }

  private nativeParentId(node: HostNodeInternal): number {
    return node.parent?.id ?? this.syntheticRoot.id;
  }

  commit(): void {
    if (this.invalid) {
      this.invalid = false;
      this.clearMutations();
      return;
    }
    const baseRevision = this.revision;
    const revision = nextU32(baseRevision, "revision");
    if (!this.bootstrapped) {
      this.transport.submit(encodeFrame([PROTOCOL_VERSION, 1, this.surfaceId, this.epoch, baseRevision, revision, this.snapshotNodes()]));
    } else {
      const operations: PatchOperation[] = [];
      const created = [...this.createdIds]
        .map((id) => this.nodesById.get(id))
        .filter((node): node is HostNodeInternal => node !== undefined)
        .sort((a, b) => this.nodeDepth(a) - this.nodeDepth(b) || a.id - b.id);
      for (const node of created) {
        operations.push([
          1,
          node.id,
          this.nativeParentId(node),
          node.index,
          KIND_CODES[node.kind],
          encodeStyle(node.style),
          node.kind === "RawText" ? node.text : null,
          node.listenerId,
          hostPropertiesWire(node.hostProperties),
          accessibilityWire(node.accessibility),
        ]);
      }
      const moved = [...this.movedIds]
        .map((id) => this.nodesById.get(id))
        .filter((node): node is HostNodeInternal => node !== undefined && !this.createdIds.has(node.id))
        .sort((a, b) => this.nodeDepth(a) - this.nodeDepth(b) || a.id - b.id);
      for (const node of moved) {
        operations.push([3, node.id, this.nativeParentId(node), node.index]);
      }
      for (const [id, mask] of [...this.updatedMasks.entries()].sort(([a], [b]) => a - b)) {
        const node = this.nodesById.get(id);
        if (node === undefined || this.createdIds.has(id)) continue;
        operations.push([
          2,
          id,
          mask,
          mask & UPDATE_STYLE ? encodeStyle(node.style) : null,
          mask & UPDATE_TEXT && node.kind === "RawText" ? node.text : null,
          mask & UPDATE_LISTENER ? node.listenerId : 0,
          mask & UPDATE_PROPERTIES ? hostPropertiesWire(node.hostProperties) : null,
          mask & UPDATE_ACCESSIBILITY ? accessibilityWire(node.accessibility) : null,
        ]);
      }
      for (const id of [...this.deletedRoots].sort((a, b) => a - b)) {
        if (!this.createdIds.has(id) && !this.nodesById.has(id)) operations.push([4, id]);
      }
      const patch: Patch = [PROTOCOL_VERSION, 3, this.surfaceId, this.epoch, baseRevision, revision, operations];
      this.transport.submit(encodeFrame(patch));
    }
    this.revision = revision;
    this.bootstrapped = true;
    this.clearMutations();
  }

  receive(chunk: Uint8Array | ArrayBuffer): void {
    let payloads: Uint8Array[];
    try {
      payloads = this.decoder.push(chunk);
    } catch {
      return;
    }
    if (payloads.length === 0) return;
    const dispatch = (): void => {
      for (const payload of payloads) this.dispatch(decodeEvent(payload));
    };
    const batch = renderer.batchedUpdates;
    const dispatchBatch = (): void => {
      if (typeof batch === "function") batch(dispatch);
      else dispatch();
    };
    const flush = renderer.flushSyncFromReconciler;
    if (typeof flush === "function") flush(dispatchBatch);

    else dispatchBatch();
  }
  private dispatch(event: PressEventFrame | null): void {
    if (
      event === null
      || event[2] !== this.surfaceId
      || event[3] !== this.epoch
      || event[4] > this.revision
      || (this.hasEventSequence && event[5] <= this.lastEventSequence)
    ) return;
    this.lastEventSequence = event[5];
    this.hasEventSequence = true;
    const payload = event[9];
    if (event[8] === EVENT_COMMAND_RESULT) {
      if (!Array.isArray(payload) || payload[0] !== 2) return;
      const pending = this.pendingCommands.get(payload[1]);
      if (pending === undefined) return;
      this.pendingCommands.delete(payload[1]);
      if (payload[4] === true) pending.resolve();
      else pending.reject(new Error(String(payload[5] ?? "native command failed")));
      return;
    }
    if (event[8] === EVENT_PRESS) {
      const node = this.listeners.get(event[7]);
      if (node === undefined || !node.attached || node.kind !== "Pressable" || node.id !== event[6] || node.listenerId !== event[7] || node.listener === undefined) return;
      node.listener({
        type: "press",
        surfaceId: event[2],
        epoch: event[3],
        revision: event[4],
        sequence: event[5],
        target: node,
      });
      return;
    }
    const node = this.nodesById.get(event[6]);
    if (node === undefined || !node.attached || node.listenerId !== event[7]) return;
    if (event[8] === EVENT_VISIBLE_RANGE) {
      if (node.kind !== "VirtualList" || !Array.isArray(payload) || payload[0] !== 3) return;
      node.visibleRangeCallback?.(payload[1], payload[2]);
      return;
    }
    if (event[8] === EVENT_ANIMATION_COMPLETE) {
      if (!Array.isArray(payload) || payload[0] !== 4 || node.lastAnimationGeneration === payload[1]) return;
      node.lastAnimationGeneration = payload[1];
      node.animationCompleteCallback?.(payload[1]);
      return;
    }
    if (node.kind !== "TextInput" || !Array.isArray(payload) || payload[0] !== 1) return;
    const callbacks = this.inputListeners.get(event[7]);
    if (callbacks === undefined) return;
    const textPayload = payload as TextInputEventPayload;
    const textEvent: TextInputEvent = {
      type: event[8] === EVENT_CHANGE ? "change" : event[8] === EVENT_SELECTION ? "selection" : event[8] === EVENT_FOCUS ? "focus" : "blur",
      text: textPayload[1],
      selection: { start: textPayload[2], end: textPayload[3], composing: textPayload[4] === null ? null : { start: textPayload[4], end: textPayload[5] as number } },
      editSeq: textPayload[6],
      target: node,
    };
    node.latestNativeText = textEvent.text;
    node.latestNativeEditSeq = textEvent.editSeq;
    node.latestNativeSelection = {
      start: textEvent.selection.start,
      end: textEvent.selection.end,
      markedStart: textEvent.selection.composing?.start ?? null,
      markedEnd: textEvent.selection.composing?.end ?? null,
    };
    if (event[8] === EVENT_CHANGE) callbacks.change?.(textEvent);
    else if (event[8] === EVENT_SELECTION) callbacks.selection?.(textEvent);
    else if (event[8] === EVENT_FOCUS) callbacks.focus?.(textEvent);
    else if (event[8] === EVENT_BLUR) callbacks.blur?.(textEvent);
  }

  dispose(): void {
    if (this.unmounted) return;
    this.unmounted = true;
    this.unsubscribe();
    for (const pending of this.pendingCommands.values()) pending.reject(new Error("root is unmounted"));
    this.pendingCommands.clear();
    for (const node of this.children) detachSubtree(node);
    this.children.length = 0;
    this.listeners.clear();
  }
}

const hostConfig = {
  rendererVersion: "0.1.0",
  rendererPackageName: "@react-gpui/core",
  isPrimaryRenderer: false,
  supportsMutation: true,
  supportsPersistence: false,
  shouldSetTextContent: () => false,
  resolveEventTimeStamp: () => Date.now(),
  resolveEventType: () => null,
  trackSchedulerEvent: () => undefined,
  supportsHydration: false,
  supportsMicrotasks: false,
  scheduleMicrotask: queueMicrotask,
  noTimeout: -1,
  now: Date.now,
  getCurrentUpdatePriority: () => currentUpdatePriority,
  setCurrentUpdatePriority: (priority: number) => {
    currentUpdatePriority = priority;
  },
  resolveUpdatePriority: () => currentUpdatePriority || DefaultEventPriority,
  getPublicInstance: (instance: HostNodeInternal) => instance,
  getRootHostContext: (container: RootContainer): HostContext => ({ root: container, parentKind: null }),
  getChildHostContext: (context: HostContext, type: string): HostContext => ({
    root: context.root,
    parentKind: type as HostKind,
  }),
  prepareForCommit: () => null,
  resetAfterCommit: (container: RootContainer) => container.commit(),
  createInstance: (type: string, props: HostProps, rootContainer: RootContainer, hostContext: HostContext): HostNodeInternal => {
    if (!VALID_HOST_TYPES[type]) throw new TypeError(`Unknown GPUI host type: ${type}`);
    assertChildForRoot(rootContainer, hostContext.parentKind, type as HostKind);
    const node = rootContainer.allocateNode(type as HostKind);
    setNodeProps(node, props);
    return node;
  },
  createTextInstance: (text: string, rootContainer: RootContainer, hostContext: HostContext): HostNodeInternal => {
    if (hostContext.parentKind !== "Text") {
      rootContainer.invalid = true;
      throw new TypeError("Raw text must be a direct child of Text");
    }
    const node = rootContainer.allocateNode("RawText");
    node.text = text;
    return node;
  },
  appendInitialChild: (parent: HostNodeInternal, child: HostNodeInternal) => {
    assertChildForRoot(parent.root, parent.kind, child.kind);
    parent.children.push(child);
    child.parent = parent;
    refreshChildIndexes(parent);
  },
  finalizeInitialChildren: () => false,
  prepareUpdate: (_instance: HostNodeInternal, _type: string, oldProps: HostProps, newProps: HostProps) => {
    validateProps(_instance.kind, newProps);
    const inputChanged = _instance.kind === "TextInput" && (oldProps.value !== newProps.value || oldProps.defaultValue !== newProps.defaultValue || oldProps.placeholder !== newProps.placeholder || oldProps.multiline !== newProps.multiline || oldProps.disabled !== newProps.disabled || oldProps.onChangeText !== newProps.onChangeText || oldProps.onSelectionChange !== newProps.onSelectionChange || oldProps.onFocus !== newProps.onFocus || oldProps.onBlur !== newProps.onBlur);
    const listChanged = _instance.kind === "VirtualList" && (oldProps.__itemCount !== newProps.__itemCount || oldProps.__rangeStart !== newProps.__rangeStart || oldProps.__rangeEnd !== newProps.__rangeEnd || oldProps.__estimatedItemSize !== newProps.__estimatedItemSize || oldProps.__overscan !== newProps.__overscan || oldProps.__onVisibleRange !== newProps.__onVisibleRange);
    const accessibilityChanged = JSON.stringify([oldProps.accessibilityRole, oldProps.accessibilityLabel, oldProps.accessibilityDescription, oldProps.accessibilityDisabled, oldProps.accessibilityChecked, oldProps.accessibilitySelected, oldProps.accessibilityValue]) !== JSON.stringify([newProps.accessibilityRole, newProps.accessibilityLabel, newProps.accessibilityDescription, newProps.accessibilityDisabled, newProps.accessibilityChecked, newProps.accessibilitySelected, newProps.accessibilityValue]);
    return oldProps.style !== newProps.style || oldProps.onPress !== newProps.onPress || inputChanged || listChanged || oldProps.__onAnimationComplete !== newProps.__onAnimationComplete || accessibilityChanged;
  },
  commitUpdate: (instance: HostNodeInternal, _type: string, _oldProps: HostProps, newProps: HostProps) => {
    const mask = updateNodeProps(instance, newProps);
    instance.root.markUpdated(instance, mask);
  },
  commitTextUpdate: (instance: HostNodeInternal, _oldText: string, newText: string) => {
    instance.text = newText;
    instance.root.markUpdated(instance, UPDATE_TEXT);
  },
  appendChild: (parent: HostNodeInternal, child: HostNodeInternal) => {
    assertChildForRoot(parent.root, parent.kind, child.kind);
    parent.root.markMoved(child);
    detachFromParent(child);
    parent.children.push(child);
    child.parent = parent;
    refreshChildIndexes(parent);
  },
  appendChildToContainer: (container: RootContainer, child: HostNodeInternal) => {
    assertChildForRoot(container, container.syntheticRoot.kind, child.kind);
    container.markMoved(child);
    detachFromParent(child);
    container.children.push(child);
    child.parent = null;
    refreshChildIndexes(container.syntheticRoot);
  },
  insertBefore: (parent: HostNodeInternal, child: HostNodeInternal, before: HostNodeInternal) => {
    assertChildForRoot(parent.root, parent.kind, child.kind);
    parent.root.markMoved(child);
    detachFromParent(child);
    const index = parent.children.indexOf(before);
    parent.children.splice(index < 0 ? parent.children.length : index, 0, child);
    child.parent = parent;
    refreshChildIndexes(parent);
  },
  insertInContainerBefore: (container: RootContainer, child: HostNodeInternal, before: HostNodeInternal) => {
    assertChildForRoot(container, container.syntheticRoot.kind, child.kind);
    container.markMoved(child);
    detachFromParent(child);
    const index = container.children.indexOf(before);
    container.children.splice(index < 0 ? container.children.length : index, 0, child);
    child.parent = null;
    refreshChildIndexes(container.syntheticRoot);
  },
  removeChild: (parent: HostNodeInternal, child: HostNodeInternal) => {
    parent.root.markDeleted(child);
    const index = parent.children.indexOf(child);
    if (index >= 0) parent.children.splice(index, 1);
    refreshChildIndexes(parent);
    child.parent = null;
    detachSubtree(child);
  },
  removeChildFromContainer: (container: RootContainer, child: HostNodeInternal) => {
    container.markDeleted(child);
    const index = container.children.indexOf(child);
    if (index >= 0) container.children.splice(index, 1);
    refreshChildIndexes(container.syntheticRoot);
    child.parent = null;
    detachSubtree(child);
  },
  clearContainer: (container: RootContainer) => {
    for (const child of container.children) {
      container.markDeleted(child);
      detachSubtree(child);
    }
    container.children.length = 0;
    refreshChildIndexes(container.syntheticRoot);
  },
  detachDeletedInstance: (instance: HostNodeInternal) => {
    instance.attached = false;
  },
  preparePortalMount: () => undefined,
  scheduleTimeout: setTimeout,
  cancelTimeout: clearTimeout,
  hideInstance: () => undefined,
  hideTextInstance: () => undefined,
  unhideInstance: () => undefined,
  unhideTextInstance: () => undefined,
};

const renderer = Reconciler(hostConfig);

export interface RootOptions {
  readonly surfaceId?: number;
  readonly epoch?: number;
  readonly maxFrameSize?: number;
}

export interface Root {
  render(element: ReactNode): void;
  unmount(): void;
}

export function createRoot(transport: Transport, options: RootOptions = {}): Root {
  const surfaceId = options.surfaceId ?? nextSurfaceId++;
  const epoch = options.epoch ?? 1;
  const container = new RootContainer(transport, surfaceId, epoch, options.maxFrameSize ?? 16 * 1024 * 1024);
  const reconcilerRoot = renderer.createContainer(container, LegacyRoot, null, false, null, "", console.error, console.error, console.error, null);
  let closed = false;
  return {
    render(element: ReactNode): void {
      if (closed) throw new Error("Cannot render into an unmounted root");
      container.beginRender();
      const flush = renderer.flushSyncFromReconciler;
      if (typeof flush === "function") {
        flush(() => renderer.updateContainerSync(element, reconcilerRoot, null, null));
      } else {
        renderer.updateContainerSync(element, reconcilerRoot, null, null);
      }
    },
    unmount(): void {
      if (closed) return;
      container.beginRender();
      const flush = renderer.flushSyncFromReconciler;
      if (typeof flush === "function") {
        flush(() => renderer.updateContainerSync(null, reconcilerRoot, null, null));
      } else {
        renderer.updateContainerSync(null, reconcilerRoot, null, null);
      }
      closed = true;
      container.dispose();
    },
  };
}
