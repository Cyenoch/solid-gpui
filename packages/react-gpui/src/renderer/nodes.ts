import {
  COMMAND_BLUR,
  COMMAND_FOCUS,
  COMMAND_GET_FOCUS,
  COMMAND_SCROLL_TO_END,
  COMMAND_SCROLL_TO_INDEX,
  COMMAND_SET_SELECTION,
  UPDATE_ACCESSIBILITY,
  UPDATE_FOCUSABLE,
  UPDATE_SELECTABLE,
  UPDATE_LISTENER,
  UPDATE_STYLE,
  UPDATE_PROPERTIES,
  UPDATE_TOOLTIP,
  type SnapshotNode,
} from "../protocol";
import { encodeStyle } from "../style";
import {
  accessibilityFor,
  accessibilityWire,
  dragFor,
  hostPropertiesWire,
  imageFor,
  inputFor,
  KIND_CODES,
  nextU32,
  validateProps,
  virtualListFor,
} from "./props";
import type { HostKind, HostNodeInternal, HostProps, RootOwner, TextInputCallbacks, TextInputProps } from "./types";

export function assertChildKind(parentKind: HostKind | null, childKind: HostKind): void {
  if (parentKind === null) return;
  if (parentKind === "Text" && childKind !== "RawText") {
    throw new TypeError("Text children must be raw text; use a separate Text node for nested content");
  }
  if (parentKind === "Image") throw new TypeError("Image nodes cannot contain children");
  if (childKind === "RawText" && parentKind !== "Text") {
    throw new TypeError("Raw text is only valid directly under Text");
  }
}

export function assertChildForRoot(root: RootOwner, parentKind: HostKind | null, childKind: HostKind): void {
  try {
    assertChildKind(parentKind, childKind);
  } catch (error) {
    root.invalid = true;
    throw error;
  }
}

export class NodeGraph {
  readonly children: HostNodeInternal[];
  readonly syntheticRoot: HostNodeInternal;
  readonly listeners = new Map<number, HostNodeInternal>();
  readonly nodesById = new Map<number, HostNodeInternal>();
  readonly inputListeners = new Map<number, TextInputCallbacks>();
  readonly createdIds = new Set<number>();
  readonly deletedRoots = new Set<number>();
  readonly movedIds = new Set<number>();
  readonly updatedMasks = new Map<number, number>();
  private nextNodeId = 2;
  private nextListenerId = 1;

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
      node.focus = () => this.owner.submitCommand(node, COMMAND_FOCUS, null);
      node.blur = () => this.owner.submitCommand(node, COMMAND_BLUR, null);
      node.setSelection = (start, end) => this.owner.submitCommand(node, COMMAND_SET_SELECTION, [start, end]);
    }
    if (kind === "View") {
      node.focus = () => this.owner.submitCommand(node, COMMAND_FOCUS, null);
      node.isFocused = () =>
        this.owner.submitCommandValue(node, COMMAND_GET_FOCUS, null).then((value) => {
          if (!Array.isArray(value) || value.length !== 2 || value[0] !== 3 || typeof value[1] !== "boolean")
            throw new Error("native getFocus returned an invalid value");
          return value[1];
        });
      node.blur = () => this.owner.submitCommand(node, COMMAND_BLUR, null);
    }
    if (kind === "VirtualList") {
      node.scrollToIndex = (index) => this.owner.submitCommand(node, COMMAND_SCROLL_TO_INDEX, [index, 0]);
      node.scrollToEnd = () => this.owner.submitCommand(node, COMMAND_SCROLL_TO_END, null);
    }
    return node;
  }

  allocateListener(node: HostNodeInternal): number {
    const listenerId = this.nextListenerId;
    this.nextListenerId = nextU32(this.nextListenerId, "listener id");
    this.listeners.set(listenerId, node);
    return listenerId;
  }

  setNodeProps(node: HostNodeInternal, props: HostProps): void {
    try {
      validateProps(node.kind, props);
    } catch (error) {
      node.root.invalid = true;
      node.root.validationError = error instanceof Error ? error : new Error(String(error));
      throw error;
    }
    node.style = node.kind === "RawText" ? null : props.style;
    node.tooltip =
      node.kind === "View" || node.kind === "Pressable" ? ((props.tooltip as string | undefined) ?? null) : null;
    node.hostProperties =
      node.kind === "TextInput"
        ? inputFor(node, props)
        : node.kind === "VirtualList"
          ? virtualListFor(node, props)
          : node.kind === "Image"
            ? imageFor(node, props)
            : dragFor(node, props);
    node.accessibility = accessibilityFor(node.kind, props);
    node.disabled = (node.kind === "Pressable" || node.kind === "TextInput") && props.disabled === true;
    node.focusable =
      (node.kind === "View" || node.kind === "Pressable") && !node.disabled ? (props.focusable ?? false) : false;
    node.selectable = node.kind === "Text" && props.selectable === true;
    node.keyListener =
      !node.disabled && (node.kind === "View" || node.kind === "Pressable" || node.kind === "TextInput")
        ? props.onKeyDown
        : undefined;
    node.listener = node.kind === "Pressable" && !node.disabled ? props.onPress : undefined;
    node.pointerCallbacks =
      !node.disabled && (node.kind === "View" || node.kind === "Pressable")
        ? { down: props.onPointerDown, up: props.onPointerUp }
        : null;
    node.focusCallback =
      !node.disabled && (node.kind === "View" || node.kind === "Pressable") ? props.onFocus : undefined;
    node.blurCallback =
      !node.disabled && (node.kind === "View" || node.kind === "Pressable") ? props.onBlur : undefined;
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
    node.layoutCallback =
      node.kind === "View" || node.kind === "Pressable" || node.kind === "Text" || node.kind === "Image"
        ? props.onLayout
        : undefined;
    if (node.hoverCallback === undefined) node.hovered = false;
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
        submit: input.onSubmitEditing ? (value) => input.onSubmitEditing?.(value) : undefined,
      };
    }
    const hasListener =
      node.listener !== undefined ||
      node.keyListener !== undefined ||
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
      node.animationCompleteCallback !== undefined;
    if (hasListener && node.listenerId === 0) node.listenerId = this.allocateListener(node);
    if (!hasListener && node.listenerId !== 0) {
      this.listeners.delete(node.listenerId);
      this.inputListeners.delete(node.listenerId);
      node.listenerId = 0;
    }
    if (node.listenerId !== 0) {
      this.listeners.set(node.listenerId, node);
      if (node.inputCallbacks !== null) this.inputListeners.set(node.listenerId, node.inputCallbacks);
    }
  }
  updateNodeProps(node: HostNodeInternal, props: HostProps): number {
    const previousStyle = node.style;
    const previousListenerId = node.listenerId;
    const previousFocusable = node.focusable;
    const previousSelectable = node.selectable;
    const previousProperties = node.hostProperties;
    const previousAccessibility = node.accessibility;
    const previousTooltip = node.tooltip;
    this.setNodeProps(node, props);
    let mask = 0;
    if (previousStyle !== node.style) mask |= UPDATE_STYLE;
    if (previousListenerId !== node.listenerId) mask |= UPDATE_LISTENER;
    if (previousFocusable !== node.focusable) mask |= UPDATE_FOCUSABLE;
    if (previousSelectable !== node.selectable) mask |= UPDATE_SELECTABLE;
    if (previousTooltip !== node.tooltip) mask |= UPDATE_TOOLTIP;
    if (JSON.stringify(previousProperties) !== JSON.stringify(node.hostProperties)) mask |= UPDATE_PROPERTIES;
    if (JSON.stringify(previousAccessibility) !== JSON.stringify(node.accessibility)) mask |= UPDATE_ACCESSIBILITY;
    return mask;
  }

  detachFromParent(node: HostNodeInternal): void {
    const parent = node.parent;
    const siblings = parent ? parent.children : this.children;
    const index = siblings.indexOf(node);
    if (index >= 0) siblings.splice(index, 1);
    node.parent = null;
    this.refreshChildIndexes(parent ?? this.syntheticRoot);
  }

  detachSubtree(node: HostNodeInternal): void {
    const retainFocusCallbacks = node.nativeFocused;
    node.attached = false;
    node.detachedFocusPending = retainFocusCallbacks;
    this.nodesById.delete(node.id);
    if (!retainFocusCallbacks && node.listenerId !== 0) {
      this.listeners.delete(node.listenerId);
      this.inputListeners.delete(node.listenerId);
    }
    node.listener = undefined;
    node.keyListener = undefined;
    node.pointerCallbacks = null;
    if (!retainFocusCallbacks) {
      node.focusCallback = undefined;
      node.blurCallback = undefined;
    }
    node.pointerDownOutsideCallback = undefined;
    node.hoverCallback = undefined;
    node.scrollCallback = undefined;
    node.inputCallbacks = null;
    for (const child of node.children) this.detachSubtree(child);
  }

  releaseDetachedFocus(node: HostNodeInternal): void {
    if (node.attached || !node.detachedFocusPending) return;
    node.detachedFocusPending = false;
    if (node.listenerId !== 0) {
      this.listeners.delete(node.listenerId);
      this.inputListeners.delete(node.listenerId);
    }
    node.focusCallback = undefined;
    node.blurCallback = undefined;
    node.nativeFocused = false;
  }

  refreshChildIndexes(parent: HostNodeInternal): void {
    parent.children.forEach((child, index) => {
      child.index = index;
    });
  }

  markMoved(node: HostNodeInternal, bootstrapped: boolean): void {
    if (bootstrapped && !this.createdIds.has(node.id)) this.movedIds.add(node.id);
  }

  markUpdated(node: HostNodeInternal, mask: number, bootstrapped: boolean): void {
    if (bootstrapped && mask !== 0 && !this.createdIds.has(node.id)) {
      this.updatedMasks.set(node.id, (this.updatedMasks.get(node.id) ?? 0) | mask);
    }
  }

  markDeleted(node: HostNodeInternal, bootstrapped: boolean): void {
    if (bootstrapped) this.deletedRoots.add(node.id);
  }

  clearMutations(): void {
    this.createdIds.clear();
    this.deletedRoots.clear();
    this.movedIds.clear();
    this.updatedMasks.clear();
  }

  snapshotNodes(): SnapshotNode[] {
    const nodes: SnapshotNode[] = [];
    const visit = (node: HostNodeInternal, parentId: number, index: number): void => {
      const tooltip =
        (node.kind === "View" || node.kind === "Pressable") && typeof node.tooltip === "string" ? node.tooltip : null;
      const base = [
        node.id,
        parentId,
        index,
        KIND_CODES[node.kind],
        encodeStyle(node.style),
        node.kind === "RawText" ? node.text : null,
        node.listenerId,
        hostPropertiesWire(node.hostProperties),
        accessibilityWire(node.accessibility),
        node.focusable,
      ] as const;
      if (tooltip !== null) nodes.push([...base, node.selectable, tooltip]);
      else if (node.selectable) nodes.push([...base, true]);
      else nodes.push(base);
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
