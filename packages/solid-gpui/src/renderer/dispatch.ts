import type { Appearance } from "../hooks";
import type { CommandResult, Event } from "../protocol";
import { truncateUtf16 } from "./props";

import type { HostNodeInternal, ListenerBinding, TextInputEvent } from "./types";
const POINTER_BUTTON_NAMES: Record<number, "left" | "right" | "middle" | "back" | "forward"> = {
  1: "left",
  2: "right",
  3: "middle",
  4: "back",
  5: "forward",
};
const INTERACTIVE_NODE_KINDS: readonly HostNodeInternal["kind"][] = ["View", "Pressable"];
const FOCUSABLE_NODE_KINDS: readonly HostNodeInternal["kind"][] = ["View", "Pressable", "Text"];

export interface DispatchContext {
  acceptEvent(event: Event): boolean;
  findListener(listenerId: number, revision: number): ListenerBinding | undefined;
  resolveCommandResult(result: CommandResult): void;
  onNotificationResponse?: (response: { readonly tag: string; readonly actionId: string | null }) => void;
  onAction?: (action: string) => void;
  onCloseRequested?: (requestId: number) => void;
  onSurfaceClosed?: () => void;
  onWindowResize?: (width: number, height: number, scaleFactor?: number) => void;
  onWindowActivation?: (active: boolean) => void;
  onAppearance?: (appearance: Appearance) => void;
}

function nodeMatchesListener(
  context: DispatchContext,
  event: Event,
  allowedKinds: readonly HostNodeInternal["kind"][],
): ListenerBinding | undefined {
  const binding = context.findListener(event.listenerId, event.revision);
  const node = binding?.node;
  if (node === undefined || !node.attached || !allowedKinds.includes(node.kind) || node.id !== event.nodeId) {
    return undefined;
  }
  return binding;
}

export function dispatchEvent(context: DispatchContext, event: Event | null): void {
  if (event === null || !context.acceptEvent(event)) return;
  const payload = event.payload;

  if (payload.type === "surface-closed") {
    context.onSurfaceClosed?.();
    return;
  }
  if (payload.type === "close-requested") {
    context.onCloseRequested?.(payload.requestId);
    return;
  }
  if (payload.type === "action") {
    context.onAction?.(payload.action);
    return;
  }
  if (payload.type === "notification-response") {
    context.onNotificationResponse?.({ tag: payload.tag, actionId: payload.actionId });
    return;
  }
  if (payload.type === "command-result") {
    context.resolveCommandResult(payload.result);
    return;
  }
  if (payload.type === "window-resize") {
    context.onWindowResize?.(payload.width, payload.height, payload.scaleFactor);
    return;
  }
  if (payload.type === "window-activation") {
    context.onWindowActivation?.(payload.active);
    return;
  }
  if (payload.type === "window-appearance") {
    context.onAppearance?.(payload.appearance);
    return;
  }
  if (payload.type === "pointer-down-outside") {
    const binding = nodeMatchesListener(context, event, ["View"]);
    binding?.pointerDownOutsideCallback?.({ x: payload.x, y: payload.y });
    return;
  }
  if ((payload.type === "focus" || payload.type === "blur") && payload.data === undefined) {
    const binding = context.findListener(event.listenerId, event.revision);
    const node = binding?.node;
    if (binding === undefined || node === undefined || node.id !== event.nodeId) return;
    if (!node.attached) {
      if (!node.detachedFocusPending) return;
    } else if (!FOCUSABLE_NODE_KINDS.includes(node.kind) || !node.focusable) {
      return;
    }
    const callback = payload.type === "focus" ? binding.focusCallback : binding.blurCallback;
    node.nativeFocused = payload.type === "focus";
    if (!node.attached) node.root.releaseDetachedFocus(node);
    callback?.({
      type: payload.type,
      target: node,
    });
    return;
  }
  if (payload.type === "layout") {
    const binding = nodeMatchesListener(context, event, ["View", "Pressable", "Text", "Image"]);
    binding?.layoutCallback?.({ x: payload.x, y: payload.y, width: payload.width, height: payload.height });
    return;
  }
  if (payload.type === "drag-over" || payload.type === "drag-drop" || payload.type === "external-file-drop") {
    const binding = nodeMatchesListener(context, event, INTERACTIVE_NODE_KINDS);
    if (binding === undefined) return;
    switch (payload.type) {
      case "drag-over":
        binding.dragCallbacks.over?.(payload.dragType);
        break;
      case "drag-drop":
        binding.dragCallbacks.drop?.(payload.dragType);
        break;
      case "external-file-drop":
        binding.dragCallbacks.externalFileDrop?.([...payload.paths]);
        break;
    }
    return;
  }
  if (payload.type === "press") {
    const binding = nodeMatchesListener(context, event, ["Pressable", "Text"]);
    if (binding === undefined || binding.listener === undefined) return;
    const node = binding.node;
    binding.listener({
      type: "press",
      surfaceId: event.surfaceId,
      epoch: event.epoch,
      revision: event.revision,
      sequence: event.sequence,
      target: node,
    });
    return;
  }
  if (payload.type === "scroll") {
    const binding = nodeMatchesListener(context, event, ["View"]);
    if (binding === undefined || binding.scrollCallback === undefined) return;
    binding.scrollCallback({
      deltaKind: payload.deltaKind === 1 ? "pixels" : "lines",
      dx: payload.dx,
      dy: payload.dy,
      x: payload.x,
      y: payload.y,
      modifiers: [...payload.modifiers],
    });
    return;
  }
  if (payload.type === "key") {
    const binding = nodeMatchesListener(context, event, ["View", "Pressable", "TextInput", "Text"]);
    if (binding === undefined) return;
    const node = binding.node;
    if ((FOCUSABLE_NODE_KINDS.includes(node.kind) && !node.focusable) || binding.keyListener === undefined) return;
    binding.keyListener({
      key: payload.key,
      modifiers: [...payload.modifiers],
      action: payload.action === 1 ? "down" : payload.action === 2 ? "repeat" : "up",
    });
    return;
  }
  if (payload.type === "pointer" || payload.type === "pointer-move") {
    const binding = nodeMatchesListener(context, event, INTERACTIVE_NODE_KINDS);
    if (binding === undefined) return;
    const node = binding.node;
    if (payload.type === "pointer-move") {
      if (binding.pointerMoveCallback === undefined) return;
      binding.pointerMoveCallback({
        type: "pointermove",
        x: payload.x,
        y: payload.y,
        modifiers: [...payload.modifiers],
        target: node,
      });
      return;
    }
    const callback = payload.action === 1 ? binding.pointerCallbacks?.down : binding.pointerCallbacks?.up;
    callback?.({
      type: payload.action === 1 ? "pointerdown" : "pointerup",
      button: POINTER_BUTTON_NAMES[payload.button],
      modifiers: [...payload.modifiers],
      clickCount: payload.clickCount,
      x: payload.x,
      y: payload.y,
      target: node,
    });
    return;
  }
  if (payload.type === "hover") {
    const binding = nodeMatchesListener(context, event, INTERACTIVE_NODE_KINDS);
    if (binding === undefined) return;
    const node = binding.node;
    node.hovered = !node.hovered;
    binding.hoverCallback?.(node.hovered);
    return;
  }
  if (payload.type === "extension") {
    const binding = context.findListener(event.listenerId, event.revision);
    const node = binding?.node;
    if (
      binding === undefined ||
      node === undefined ||
      !node.attached ||
      node.kind !== "Extension" ||
      node.id !== event.nodeId ||
      payload.eventId === 0 ||
      !node.extensionEventIds.includes(payload.eventId)
    )
      return;
    binding.extensionEventCallback?.({
      eventId: payload.eventId,
      fields: payload.fields,
      target: node,
    });
    return;
  }

  const binding = context.findListener(event.listenerId, event.revision);
  const node = binding?.node;
  if (binding === undefined || node === undefined || !node.attached || node.id !== event.nodeId) return;
  if (payload.type === "visible-range") {
    if (node.kind !== "VirtualList") return;
    binding.visibleRangeCallback?.(payload.start, payload.end);
    return;
  }
  if (payload.type === "animation-complete") {
    if (node.lastAnimationGeneration === payload.generation) return;
    node.lastAnimationGeneration = payload.generation;
    binding.animationCompleteCallback?.(payload.generation);
    return;
  }
  if (payload.type === "submit") {
    if (node.kind !== "TextInput") return;
    binding.inputCallbacks?.submit?.(payload.text);
    return;
  }
  if (
    (payload.type === "change" ||
      payload.type === "selection" ||
      payload.type === "focus" ||
      payload.type === "blur") &&
    payload.data !== undefined
  ) {
    if (node.kind !== "TextInput") return;
    const callbacks = binding.inputCallbacks;
    if (callbacks === null || callbacks === undefined) return;
    const hostProperties = node.hostProperties;
    const maxLength = hostProperties?.type === "text-input" ? hostProperties.value.maxLength : null;
    const text = truncateUtf16(payload.data.text, maxLength);
    const textEvent: TextInputEvent = {
      type: payload.type,
      text,
      selection: {
        start: payload.data.selectionStart,
        end: payload.data.selectionEnd,
        reversed: payload.data.reversed,
        composing:
          payload.data.markedStart === null
            ? null
            : { start: payload.data.markedStart, end: payload.data.markedEnd as number },
      },
      editSeq: payload.data.editSeq,
      target: node,
    };
    node.latestNativeText = textEvent.text;
    node.latestNativeEditSeq = textEvent.editSeq;
    node.latestNativeSelection = {
      start: textEvent.selection.start,
      end: textEvent.selection.end,
      reversed: textEvent.selection.reversed,
      markedStart: textEvent.selection.composing?.start ?? null,
      markedEnd: textEvent.selection.composing?.end ?? null,
    };
    switch (payload.type) {
      case "change":
        callbacks.change?.(textEvent);
        break;
      case "selection":
        callbacks.selection?.(textEvent);
        break;
      case "focus":
        callbacks.focus?.(textEvent);
        break;
      case "blur":
        callbacks.blur?.(textEvent);
        break;
    }
  }
}
