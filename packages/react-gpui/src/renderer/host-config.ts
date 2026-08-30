import { DefaultEventPriority } from "react-reconciler/constants";

import { type HostContext, type HostKind, type HostNodeInternal, type HostProps } from "./types";
import { VALID_HOST_TYPES, validateProps } from "./props";
import { assertChildForRoot } from "./nodes";
import type { RootContainer } from "./root-container";
import { UPDATE_TEXT } from "../protocol";

let currentUpdatePriority = DefaultEventPriority;

export const hostConfig = {
  rendererVersion: "0.2.0",
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
  getRootHostContext: (container: RootContainer): HostContext => ({ root: container, parentKind: null, textDepth: 0 }),
  getChildHostContext: (context: HostContext, type: string): HostContext => ({
    root: context.root,
    parentKind: type as HostKind,
    textDepth: context.textDepth + (type === "Text" ? 1 : 0),
  }),
  prepareForCommit: () => null,
  resetAfterCommit: (container: RootContainer) => container.commit(),
  createInstance: (
    type: string,
    props: HostProps,
    rootContainer: RootContainer,
    hostContext: HostContext,
  ): HostNodeInternal => {
    if (!VALID_HOST_TYPES[type]) throw new TypeError(`Unknown GPUI host type: ${type}`);
    assertChildForRoot(rootContainer, hostContext.parentKind, type as HostKind, hostContext.textDepth);
    const node = rootContainer.allocateNode(type as HostKind);
    rootContainer.setNodeProps(node, props);
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
    assertChildForRoot(
      parent.root,
      parent.kind,
      child.kind,
      parent.kind === "Text" && parent.parent?.kind === "Text" ? 2 : 1,
    );
    parent.children.push(child);
    child.parent = parent;
    parent.root.refreshChildIndexes(parent);
  },
  finalizeInitialChildren: () => false,
  prepareUpdate: (_instance: HostNodeInternal, _type: string, oldProps: HostProps, newProps: HostProps) => {
    try {
      validateProps(_instance.kind, newProps);
    } catch (error) {
      _instance.root.invalid = true;
      throw error;
    }
    const inputChanged =
      _instance.kind === "TextInput" &&
      (oldProps.value !== newProps.value ||
        oldProps.defaultValue !== newProps.defaultValue ||
        oldProps.placeholder !== newProps.placeholder ||
        oldProps.multiline !== newProps.multiline ||
        oldProps.disabled !== newProps.disabled ||
        oldProps.onChangeText !== newProps.onChangeText ||
        oldProps.onSelectionChange !== newProps.onSelectionChange ||
        oldProps.onFocus !== newProps.onFocus ||
        oldProps.onBlur !== newProps.onBlur ||
        oldProps.onSubmitEditing !== newProps.onSubmitEditing ||
        oldProps.onKeyDown !== newProps.onKeyDown ||
        oldProps.maxLength !== newProps.maxLength);
    const listChanged =
      _instance.kind === "VirtualList" &&
      (oldProps.__itemCount !== newProps.__itemCount ||
        oldProps.__rangeStart !== newProps.__rangeStart ||
        oldProps.__rangeEnd !== newProps.__rangeEnd ||
        oldProps.__estimatedItemSize !== newProps.__estimatedItemSize ||
        oldProps.__overscan !== newProps.__overscan ||
        oldProps.__onVisibleRange !== newProps.__onVisibleRange);
    const accessibilityChanged =
      JSON.stringify([
        oldProps.accessibilityRole,
        oldProps.accessibilityLabel,
        oldProps.accessibilityDescription,
        oldProps.accessibilityDisabled,
        oldProps.disabled,
        oldProps.accessibilityChecked,
        oldProps.accessibilitySelected,
        oldProps.accessibilityValue,
        oldProps.accessibilityExpanded,
        oldProps.accessibilityLevel,
      ]) !==
      JSON.stringify([
        newProps.accessibilityRole,
        newProps.accessibilityLabel,
        newProps.accessibilityDescription,
        newProps.accessibilityDisabled,
        newProps.disabled,
        newProps.accessibilityChecked,
        newProps.accessibilitySelected,
        newProps.accessibilityValue,
        newProps.accessibilityExpanded,
        newProps.accessibilityLevel,
      ]);
    const interactionChanged =
      oldProps.disabled !== newProps.disabled ||
      oldProps.focusable !== newProps.focusable ||
      oldProps.onKeyDown !== newProps.onKeyDown ||
      oldProps.onPointerDown !== newProps.onPointerDown ||
      oldProps.onPointerUp !== newProps.onPointerUp ||
      oldProps.onHoverChange !== newProps.onHoverChange ||
      oldProps.onFocus !== newProps.onFocus ||
      oldProps.onBlur !== newProps.onBlur ||
      oldProps.onPointerDownOutside !== newProps.onPointerDownOutside ||
      oldProps.onScroll !== newProps.onScroll ||
      oldProps.draggable !== newProps.draggable ||
      oldProps.onDragOver !== newProps.onDragOver ||
      oldProps.onDrop !== newProps.onDrop ||
      oldProps.onExternalFileDrop !== newProps.onExternalFileDrop;
    return (
      oldProps.style !== newProps.style ||
      oldProps.onPress !== newProps.onPress ||
      inputChanged ||
      listChanged ||
      oldProps.__onAnimationComplete !== newProps.__onAnimationComplete ||
      accessibilityChanged ||
      interactionChanged
    );
  },
  commitUpdate: (instance: HostNodeInternal, _type: string, _oldProps: HostProps, newProps: HostProps) => {
    const mask = instance.root.updateNodeProps(instance, newProps);
    instance.root.markUpdated(instance, mask);
  },
  commitTextUpdate: (instance: HostNodeInternal, _oldText: string, newText: string) => {
    instance.text = newText;
    instance.root.markUpdated(instance, UPDATE_TEXT);
  },
  appendChild: (parent: HostNodeInternal, child: HostNodeInternal) => {
    assertChildForRoot(
      parent.root,
      parent.kind,
      child.kind,
      parent.kind === "Text" && parent.parent?.kind === "Text" ? 2 : 1,
    );
    parent.root.markMoved(child);
    child.root.detachFromParent(child);
    child.detachedFocusPending = false;
    parent.children.push(child);
    child.parent = parent;
    parent.root.refreshChildIndexes(parent);
  },
  appendChildToContainer: (container: RootContainer, child: HostNodeInternal) => {
    assertChildForRoot(container, container.syntheticRoot.kind, child.kind);
    container.markMoved(child);
    child.root.detachFromParent(child);
    child.detachedFocusPending = false;
    container.children.push(child);
    child.parent = null;
    container.refreshChildIndexes(container.syntheticRoot);
  },
  insertBefore: (parent: HostNodeInternal, child: HostNodeInternal, before: HostNodeInternal) => {
    assertChildForRoot(
      parent.root,
      parent.kind,
      child.kind,
      parent.kind === "Text" && parent.parent?.kind === "Text" ? 2 : 1,
    );
    parent.root.markMoved(child);
    child.root.detachFromParent(child);
    child.detachedFocusPending = false;
    const index = parent.children.indexOf(before);
    parent.children.splice(index < 0 ? parent.children.length : index, 0, child);
    child.parent = parent;
    parent.root.refreshChildIndexes(parent);
  },
  insertInContainerBefore: (container: RootContainer, child: HostNodeInternal, before: HostNodeInternal) => {
    assertChildForRoot(container, container.syntheticRoot.kind, child.kind);
    container.markMoved(child);
    child.root.detachFromParent(child);
    child.detachedFocusPending = false;
    const index = container.children.indexOf(before);
    container.children.splice(index < 0 ? container.children.length : index, 0, child);
    child.parent = null;
    container.refreshChildIndexes(container.syntheticRoot);
  },
  removeChild: (parent: HostNodeInternal, child: HostNodeInternal) => {
    parent.root.markDeleted(child);
    const index = parent.children.indexOf(child);
    if (index >= 0) parent.children.splice(index, 1);
    parent.root.refreshChildIndexes(parent);
    child.parent = null;
    child.root.detachSubtree(child);
  },
  removeChildFromContainer: (container: RootContainer, child: HostNodeInternal) => {
    container.markDeleted(child);
    const index = container.children.indexOf(child);
    if (index >= 0) container.children.splice(index, 1);
    container.refreshChildIndexes(container.syntheticRoot);
    child.parent = null;
    child.root.detachSubtree(child);
  },
  clearContainer: (container: RootContainer) => {
    for (const child of container.children) {
      container.markDeleted(child);
      container.detachSubtree(child);
    }
    container.children.length = 0;
    container.refreshChildIndexes(container.syntheticRoot);
  },
  detachDeletedInstance: (instance: HostNodeInternal) => {
    instance.attached = false;
    instance.detachedFocusPending = true;
  },
  preparePortalMount: () => undefined,
  scheduleTimeout: setTimeout,
  cancelTimeout: clearTimeout,
  hideInstance: () => undefined,
  hideTextInstance: () => undefined,
  unhideInstance: () => undefined,
  unhideTextInstance: () => undefined,
};
