import {
  DRAG_DROP,
  DRAG_EXTERNAL_FILE_DROP,
  DRAG_OVER,
  EVENT_ACTION,
  EVENT_CLOSE_REQUESTED,
  EVENT_ANIMATION_COMPLETE,
  EVENT_BLUR,
  EVENT_CHANGE,
  EVENT_COMMAND_RESULT,
  EVENT_FOCUS,
  EVENT_HOVER,
  EVENT_KEY,
  EVENT_KEY_DOWN,
  EVENT_KEY_REPEAT,
  EVENT_NOTIFICATION_RESPONSE,
  EVENT_POINTER,
  EVENT_POINTER_DOWN,
  EVENT_POINTER_DOWN_OUTSIDE,
  EVENT_PRESS,
  EVENT_SCROLL,
  EVENT_SELECTION,
  EVENT_SUBMIT,
  EVENT_SURFACE_CLOSED,
  EVENT_VISIBLE_RANGE,
  EVENT_WINDOW_ACTIVATION,
  EVENT_WINDOW_APPEARANCE,
  EVENT_WINDOW_RESIZE,
  EVENT_LAYOUT,
  EVENT_DRAG,
  POINTER_BUTTON_BACK,
  POINTER_BUTTON_FORWARD,
  POINTER_BUTTON_LEFT,
  POINTER_BUTTON_MIDDLE,
  POINTER_BUTTON_RIGHT,
  SCROLL_DELTA_LINES,
  SCROLL_DELTA_PIXELS,
  type PressEventFrame,
  type PointerEventPayload,
  type KeyEventPayload,
  type ScrollEventPayload,
  type TextInputEventPayload,
} from "../protocol";
import { truncateUtf16 } from "./props";
import type { Appearance } from "../hooks";
import type { HostNodeInternal, TextInputCallbacks, TextInputEvent } from "./types";

const POINTER_BUTTON_NAMES: Record<number, "left" | "right" | "middle" | "back" | "forward"> = {
  1: "left",
  2: "right",
  3: "middle",
  4: "back",
  5: "forward",
};

export interface DispatchContext {
  acceptEvent(event: PressEventFrame): boolean;
  findListener(listenerId: number): HostNodeInternal | undefined;
  findNode(nodeId: number): HostNodeInternal | undefined;
  findInputCallbacks(listenerId: number): TextInputCallbacks | undefined;
  releaseDetachedFocus(node: HostNodeInternal): void;
  resolveCommandResult(requestId: number, success: boolean, errorPayload: unknown, value?: unknown): void;
  onNotificationResponse?: (response: { readonly tag: string; readonly actionId: string | null }) => void;
  onAction?: (action: string) => void;
  onCloseRequested?: (requestId: number) => void;
  onSurfaceClosed?: () => void;
  onWindowResize?: (width: number, height: number, scaleFactor?: number) => void;
  onWindowActivation?: (active: boolean) => void;
  onAppearance?: (appearance: Appearance) => void;
}

export function dispatchEvent(context: DispatchContext, event: PressEventFrame | null): void {
  if (event === null || !context.acceptEvent(event)) return;
  const payload = event[9];
  if (event[8] === EVENT_SURFACE_CLOSED) {
    context.onSurfaceClosed?.();
    return;
  }
  if (event[8] === EVENT_CLOSE_REQUESTED) {
    if (
      !Array.isArray(payload) ||
      payload.length !== 2 ||
      payload[0] !== 9 ||
      typeof payload[1] !== "number" ||
      !Number.isInteger(payload[1])
    )
      return;
    context.onCloseRequested?.(payload[1]);
    return;
  }
  if (event[8] === EVENT_ACTION) {
    if (typeof payload !== "string") return;
    context.onAction?.(payload);
    return;
  }
  if (event[8] === EVENT_NOTIFICATION_RESPONSE) {
    if (
      !Array.isArray(payload) ||
      payload.length !== 2 ||
      typeof payload[0] !== "string" ||
      (payload[1] !== null && typeof payload[1] !== "string")
    )
      return;
    context.onNotificationResponse?.({ tag: payload[0], actionId: payload[1] });
    return;
  }
  if (event[8] === EVENT_COMMAND_RESULT) {
    if (!Array.isArray(payload) || payload[0] !== 2) return;
    context.resolveCommandResult(
      payload[1],
      payload[4] === true,
      payload[5],
      payload.length === 7 ? payload[6] : undefined,
    );
    return;
  }
  if (event[8] === EVENT_WINDOW_RESIZE) {
    if (
      !Array.isArray(payload) ||
      (payload.length !== 2 && payload.length !== 3) ||
      typeof payload[0] !== "number" ||
      typeof payload[1] !== "number" ||
      (payload.length === 3 && typeof payload[2] !== "number")
    )
      return;
    context.onWindowResize?.(payload[0], payload[1], payload.length === 3 ? payload[2] : 1);
    return;
  }
  if (event[8] === EVENT_WINDOW_ACTIVATION) {
    if (typeof payload !== "boolean") return;
    context.onWindowActivation?.(payload);
    return;
  }
  if (event[8] === EVENT_WINDOW_APPEARANCE) {
    if (payload !== "light" && payload !== "dark") return;
    context.onAppearance?.(payload);
    return;
  }
  if (event[8] === EVENT_POINTER_DOWN_OUTSIDE) {
    const node = context.findListener(event[7]);
    if (
      node === undefined ||
      !node.attached ||
      node.kind !== "View" ||
      node.id !== event[6] ||
      node.listenerId !== event[7] ||
      !Array.isArray(payload) ||
      payload.length !== 3 ||
      payload[0] !== 8 ||
      typeof payload[1] !== "number" ||
      !Number.isFinite(payload[1]) ||
      typeof payload[2] !== "number" ||
      !Number.isFinite(payload[2])
    )
      return;
    node.pointerDownOutsideCallback?.({ x: payload[1], y: payload[2] });
    return;
  }
  if ((event[8] === EVENT_FOCUS || event[8] === EVENT_BLUR) && payload === null) {
    const node = context.findListener(event[7]);
    if (
      node === undefined ||
      (node.attached ? node.kind !== "View" && node.kind !== "Pressable" : !node.detachedFocusPending) ||
      (node.attached && !node.focusable) ||
      node.id !== event[6] ||
      node.listenerId !== event[7]
    )
      return;
    const callback = event[8] === EVENT_FOCUS ? node.focusCallback : node.blurCallback;
    node.nativeFocused = event[8] === EVENT_FOCUS;
    if (!node.attached) context.releaseDetachedFocus(node);
    callback?.({
      type: event[8] === EVENT_FOCUS ? "focus" : "blur",
      target: node,
    });
    return;
  }
  if (event[8] === EVENT_LAYOUT) {
    const node = context.findListener(event[7]);
    if (
      node === undefined ||
      !node.attached ||
      node.id !== event[6] ||
      node.listenerId !== event[7] ||
      node.layoutCallback === undefined ||
      !Array.isArray(payload) ||
      !payload.every((value) => typeof value === "number" && Number.isFinite(value))
    )
      return;
    node.layoutCallback({ x: payload[0], y: payload[1], width: payload[2], height: payload[3] });
    return;
  }
  if (event[8] === EVENT_DRAG) {
    const node = context.findListener(event[7]);
    if (
      node === undefined ||
      !node.attached ||
      node.id !== event[6] ||
      node.listenerId !== event[7] ||
      !Array.isArray(payload) ||
      payload.length !== 2
    )
      return;
    if ((payload[0] === DRAG_OVER || payload[0] === DRAG_DROP) && typeof payload[1] === "string") {
      if (payload[0] === DRAG_OVER) node.dragCallbacks.over?.(payload[1]);
      else node.dragCallbacks.drop?.(payload[1]);
    } else if (
      payload[0] === DRAG_EXTERNAL_FILE_DROP &&
      Array.isArray(payload[1]) &&
      payload[1].every((path) => typeof path === "string")
    ) {
      node.dragCallbacks.externalFileDrop?.([...payload[1]]);
    }
    return;
  }
  if (event[8] === EVENT_PRESS) {
    const node = context.findListener(event[7]);
    if (
      node === undefined ||
      !node.attached ||
      node.kind !== "Pressable" ||
      node.id !== event[6] ||
      node.listenerId !== event[7] ||
      node.listener === undefined
    )
      return;
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
  if (event[8] === EVENT_SCROLL) {
    const node = context.findListener(event[7]);
    if (
      node === undefined ||
      !node.attached ||
      node.kind !== "View" ||
      node.id !== event[6] ||
      node.listenerId !== event[7] ||
      node.scrollCallback === undefined ||
      !Array.isArray(payload) ||
      payload[0] !== 7
    )
      return;
    const scrollPayload = payload as ScrollEventPayload;
    node.scrollCallback({
      deltaKind: scrollPayload[1] === SCROLL_DELTA_PIXELS ? "pixels" : "lines",
      dx: scrollPayload[2],
      dy: scrollPayload[3],
      x: scrollPayload[4],
      y: scrollPayload[5],
      modifiers: [...scrollPayload[6]],
    });
    return;
  }
  if (event[8] === EVENT_KEY) {
    const node = context.findListener(event[7]);
    if (
      node === undefined ||
      !node.attached ||
      ((node.kind === "View" || node.kind === "Pressable") && !node.focusable) ||
      (node.kind !== "View" && node.kind !== "Pressable" && node.kind !== "TextInput") ||
      node.id !== event[6] ||
      node.listenerId !== event[7] ||
      node.keyListener === undefined
    )
      return;
    const keyPayload = payload as KeyEventPayload;
    node.keyListener({
      key: keyPayload[1],
      modifiers: [...keyPayload[2]],
      action: keyPayload[3] === EVENT_KEY_DOWN ? "down" : keyPayload[3] === EVENT_KEY_REPEAT ? "repeat" : "up",
    });
    return;
  }
  if (event[8] === EVENT_POINTER) {
    const node = context.findListener(event[7]);
    if (
      node === undefined ||
      !node.attached ||
      (node.kind !== "View" && node.kind !== "Pressable") ||
      node.id !== event[6] ||
      node.listenerId !== event[7] ||
      !Array.isArray(payload)
    )
      return;
    if (payload[0] === 10) {
      if (!node.acceptsPointerMove || payload.length !== 4) return;
      node.pointerMoveCallback?.({
        type: "pointermove",
        x: payload[1] as number,
        y: payload[2] as number,
        modifiers: [...(payload[3] as readonly string[])],
        target: node,
      });
      return;
    }
    if (payload[0] !== 6) return;
    const pointerPayload = payload as Exclude<PointerEventPayload, readonly [10, number, number, readonly string[]]>;
    const callback = pointerPayload[3] === EVENT_POINTER_DOWN ? node.pointerCallbacks?.down : node.pointerCallbacks?.up;
    callback?.({
      type: pointerPayload[3] === EVENT_POINTER_DOWN ? "pointerdown" : "pointerup",
      button: POINTER_BUTTON_NAMES[pointerPayload[1]],
      modifiers: [...pointerPayload[2]],
      clickCount: pointerPayload[4],
      x: pointerPayload[5],
      y: pointerPayload[6],
      target: node,
    });
    return;
  }
  if (event[8] === EVENT_HOVER) {
    const node = context.findNode(event[6]);
    if (
      node === undefined ||
      !node.attached ||
      (node.kind !== "View" && node.kind !== "Pressable") ||
      node.listenerId !== event[7] ||
      payload !== null
    )
      return;
    node.hovered = !node.hovered;
    node.hoverCallback?.(node.hovered);
    return;
  }
  const node = context.findNode(event[6]);
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
  if (event[8] === EVENT_SUBMIT) {
    if (node.kind !== "TextInput" || typeof payload !== "string") return;
    context.findInputCallbacks(event[7])?.submit?.(payload);
    return;
  }
  if (node.kind !== "TextInput" || !Array.isArray(payload) || payload[0] !== 1) return;
  const callbacks = context.findInputCallbacks(event[7]);
  if (callbacks === undefined) return;
  const textPayload = payload as TextInputEventPayload;
  const maxLength =
    node.hostProperties !== null && "maxLength" in node.hostProperties ? node.hostProperties.maxLength : null;
  const text = truncateUtf16(textPayload[1], maxLength);
  const textEvent: TextInputEvent = {
    type:
      event[8] === EVENT_CHANGE
        ? "change"
        : event[8] === EVENT_SELECTION
          ? "selection"
          : event[8] === EVENT_FOCUS
            ? "focus"
            : "blur",
    text,
    selection: {
      start: textPayload[2],
      end: textPayload[3],
      reversed: textPayload.length === 8 ? textPayload[7] : false,
      composing: textPayload[4] === null ? null : { start: textPayload[4], end: textPayload[5] as number },
    },
    editSeq: textPayload[6],
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
  if (event[8] === EVENT_CHANGE) callbacks.change?.(textEvent);
  else if (event[8] === EVENT_SELECTION) callbacks.selection?.(textEvent);
  else if (event[8] === EVENT_FOCUS) callbacks.focus?.(textEvent);
  else if (event[8] === EVENT_BLUR) callbacks.blur?.(textEvent);
}
