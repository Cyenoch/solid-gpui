import { EVENT_APPLICATION_ACTIVATION } from "./constants";
import {
  COMMAND_INVOKE_NATIVE,
  MAX_NATIVE_CALL_BYTES,
  COMMAND_KINDS,
  DRAG_DROP,
  DRAG_EXTERNAL_FILE_DROP,
  DRAG_OVER,
  EVENT_ACTION,
  EVENT_ANIMATION_COMPLETE,
  EVENT_BLUR,
  EVENT_CHANGE,
  EVENT_CLOSE_REQUESTED,
  EVENT_COMMAND_RESULT,
  EVENT_DRAG,
  EVENT_EXTENSION,
  EVENT_FOCUS,
  EVENT_HOVER,
  EVENT_KEY,
  EVENT_LAYOUT,
  EVENT_NOTIFICATION_RESPONSE,
  EVENT_POINTER,
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
  EVENT_KEY_DOWN,
  EVENT_KEY_REPEAT,
  EVENT_KEY_UP,
  EVENT_POINTER_DOWN,
  EVENT_POINTER_UP,
  KEY_MODIFIER_NAMES,
  MAX_CLIPBOARD_IMAGE_BYTES,
  MAX_CLIPBOARD_TEXT_BYTES,
  MAX_FILE_READ_BYTES,
  PROTOCOL_VERSION,
  SCROLL_DELTA_LINES,
  SCROLL_DELTA_PIXELS,
} from "./constants";

const UTF8_ENCODER = new TextEncoder();
import type { ClipboardImage, CommandResult, CommandValue, Event, EventPayload, TextInputEventData } from "./types";
import { validateExtensionFields } from "./types";

export function utf8ByteLength(value: string): number {
  return UTF8_ENCODER.encode(value).byteLength;
}

export class ProtocolVersionMismatchError extends Error {
  readonly receivedVersion: number;
  readonly expectedVersion = PROTOCOL_VERSION;

  constructor(receivedVersion: number) {
    super(
      `protocol version mismatch: host binary speaks protocol v${receivedVersion}; this renderer package speaks protocol v${PROTOCOL_VERSION} — update @solid-gpui/core to a v${receivedVersion} release / pin the host binary to a v${PROTOCOL_VERSION} release`,
    );
    this.name = "ProtocolVersionMismatchError";
    this.receivedVersion = receivedVersion;
  }
}

function isU32(value: number): boolean {
  return Number.isInteger(value) && value >= 0 && value <= 0xffff_ffff;
}
function isFiniteF32(value: number): boolean {
  return Number.isFinite(value) && Number.isFinite(Math.fround(value));
}
function validModifiers(value: readonly string[]): boolean {
  for (let index = 0; index < value.length; index += 1) {
    const modifier = value[index];
    if (!Object.hasOwn(KEY_MODIFIER_NAMES, modifier)) return false;
    for (let previous = 0; previous < index; previous += 1) {
      if (value[previous] === modifier) return false;
    }
  }
  return true;
}
function validText(value: string, maxBytes: number): boolean {
  return utf8ByteLength(value) <= maxBytes;
}
function validResourcePaths(paths: readonly string[]): boolean {
  return paths.length > 0 && paths.every((path) => path.length > 0 && utf8ByteLength(path) <= MAX_FILE_READ_BYTES);
}
function validExternalPaths(paths: readonly string[]): boolean {
  return (
    paths.length > 0 &&
    paths.every((path) => path.length > 0 && utf8ByteLength(path) <= 4096 && !/[\u0000-\u001f\u007f]/.test(path))
  );
}
function validateTextInputData(value: TextInputEventData): boolean {
  return (
    validText(value.text, MAX_CLIPBOARD_TEXT_BYTES) &&
    isU32(value.selectionStart) &&
    isU32(value.selectionEnd) &&
    value.selectionStart <= value.selectionEnd &&
    isU32(value.editSeq) &&
    ((value.markedStart === null && value.markedEnd === null) ||
      (value.markedStart !== null &&
        value.markedEnd !== null &&
        isU32(value.markedStart) &&
        isU32(value.markedEnd) &&
        value.markedStart <= value.markedEnd))
  );
}
function validateImage(image: ClipboardImage): boolean {
  return (
    (image.format === "png" || image.format === "jpeg" || image.format === "gif" || image.format === "svg") &&
    image.bytes.byteLength > 0 &&
    image.bytes.byteLength <= MAX_CLIPBOARD_IMAGE_BYTES
  );
}
function validateCommandValue(value: CommandValue): boolean {
  switch (value.type) {
    case "bytes":
      return value.value instanceof Uint8Array && value.value.byteLength <= MAX_NATIVE_CALL_BYTES;
    case "number":
      return isU32(value.value);
    case "pair":
      return isFiniteF32(value.width) && isFiniteF32(value.height) && value.width >= 0 && value.height >= 0;
    case "boolean":
      return typeof value.value === "boolean";
    case "text":
      return validText(value.value, MAX_CLIPBOARD_TEXT_BYTES);
    case "paths":
      return validResourcePaths(value.paths);
    case "file-text":
      return validText(value.value, MAX_FILE_READ_BYTES);
    case "image":
      return validateImage(value.image);
    case "bounds":
      return [value.x, value.y, value.width, value.height].every(isFiniteF32) && value.width >= 0 && value.height >= 0;
    case "window-state":
      return typeof value.fullscreen === "boolean" && typeof value.maximized === "boolean";
    case "scroll-offset":
      return isFiniteF32(value.value) && value.value >= 0;
  }
}
function validateCommandResult(value: CommandResult): boolean {
  return (
    isU32(value.requestId) &&
    isU32(value.command) &&
    COMMAND_KINDS.includes(value.command as (typeof COMMAND_KINDS)[number]) &&
    isU32(value.nodeId) &&
    typeof value.success === "boolean" &&
    (value.value?.type !== "bytes" || (value.command === COMMAND_INVOKE_NATIVE && value.success)) &&
    (value.command !== COMMAND_INVOKE_NATIVE ||
      (value.nodeId !== 0 &&
        (value.success ? value.value?.type === "bytes" && value.error === null : value.value === null))) &&
    (value.error === null || (typeof value.error === "string" && validText(value.error, MAX_CLIPBOARD_TEXT_BYTES))) &&
    (value.value === null || validateCommandValue(value.value))
  );
}
function validateEventPayload(eventType: number, payload: EventPayload): boolean {
  switch (payload.type) {
    case "application-activation":
      return (
        eventType === EVENT_APPLICATION_ACTIVATION &&
        isU32(payload.targetSurfaceId) &&
        payload.targetSurfaceId > 0 &&
        ["launch", "reopen", "open-urls"].includes(payload.reason) &&
        payload.urls.length <= 64 &&
        payload.urls.every(
          (url) =>
            typeof url === "string" &&
            url.length > 0 &&
            utf8ByteLength(url) <= 4096 &&
            !/[\u0000-\u001f\u007f]/.test(url),
        ) &&
        (payload.reason === "open-urls" ? payload.urls.length > 0 : payload.urls.length === 0)
      );
    case "press":
      return eventType === EVENT_PRESS;
    case "change":
      return eventType === EVENT_CHANGE && validateTextInputData(payload.data);
    case "selection":
      return eventType === EVENT_SELECTION && validateTextInputData(payload.data);
    case "focus":
      return eventType === EVENT_FOCUS && (payload.data === undefined || validateTextInputData(payload.data));
    case "blur":
      return eventType === EVENT_BLUR && (payload.data === undefined || validateTextInputData(payload.data));
    case "command-result":
      return eventType === EVENT_COMMAND_RESULT && validateCommandResult(payload.result);
    case "visible-range":
      return (
        eventType === EVENT_VISIBLE_RANGE && isU32(payload.start) && isU32(payload.end) && payload.start <= payload.end
      );
    case "animation-complete":
      return eventType === 8 && isU32(payload.generation);
    case "key":
      return (
        eventType === EVENT_KEY &&
        payload.key.length > 0 &&
        validText(payload.key, MAX_CLIPBOARD_TEXT_BYTES) &&
        validModifiers(payload.modifiers) &&
        [EVENT_KEY_DOWN, EVENT_KEY_REPEAT, EVENT_KEY_UP].includes(payload.action)
      );
    case "pointer":
      return (
        eventType === EVENT_POINTER &&
        isU32(payload.button) &&
        payload.button >= 1 &&
        payload.button <= 5 &&
        validModifiers(payload.modifiers) &&
        [EVENT_POINTER_DOWN, EVENT_POINTER_UP].includes(payload.action) &&
        isU32(payload.clickCount) &&
        payload.clickCount > 0 &&
        isFiniteF32(payload.x) &&
        payload.x >= 0 &&
        isFiniteF32(payload.y) &&
        payload.y >= 0
      );
    case "pointer-move":
      return (
        eventType === EVENT_POINTER &&
        isFiniteF32(payload.x) &&
        payload.x >= 0 &&
        isFiniteF32(payload.y) &&
        payload.y >= 0 &&
        validModifiers(payload.modifiers)
      );
    case "hover":
      return eventType === EVENT_HOVER;
    case "scroll":
      return (
        eventType === EVENT_SCROLL &&
        [SCROLL_DELTA_PIXELS, SCROLL_DELTA_LINES].includes(payload.deltaKind) &&
        [payload.dx, payload.dy, payload.x, payload.y].every(isFiniteF32) &&
        validModifiers(payload.modifiers)
      );
    case "submit":
      return eventType === EVENT_SUBMIT && validText(payload.text, MAX_CLIPBOARD_TEXT_BYTES);
    case "window-resize":
      return (
        eventType === EVENT_WINDOW_RESIZE &&
        isFiniteF32(payload.width) &&
        payload.width >= 0 &&
        isFiniteF32(payload.height) &&
        payload.height >= 0 &&
        isFiniteF32(payload.scaleFactor) &&
        payload.scaleFactor > 0
      );
    case "window-activation":
      return eventType === EVENT_WINDOW_ACTIVATION && typeof payload.active === "boolean";
    case "surface-closed":
      return eventType === EVENT_SURFACE_CLOSED;
    case "action":
      return eventType === EVENT_ACTION && payload.action.length > 0 && validText(payload.action, 256);
    case "window-appearance":
      return eventType === EVENT_WINDOW_APPEARANCE && (payload.appearance === "light" || payload.appearance === "dark");
    case "layout":
      return eventType === EVENT_LAYOUT && [payload.x, payload.y, payload.width, payload.height].every(isFiniteF32);
    case "drag-over":
    case "drag-drop":
      return (
        eventType === EVENT_DRAG &&
        payload.dragType.length > 0 &&
        validText(payload.dragType, 512) &&
        !/[\u0000-\u001f\u007f]/.test(payload.dragType)
      );
    case "external-file-drop":
      return eventType === EVENT_DRAG && validExternalPaths(payload.paths);
    case "notification-response":
      return (
        eventType === EVENT_NOTIFICATION_RESPONSE &&
        payload.tag.length > 0 &&
        validText(payload.tag, 1024) &&
        (payload.actionId === null || (payload.actionId.length > 0 && validText(payload.actionId, 64)))
      );
    case "pointer-down-outside":
      return eventType === EVENT_POINTER_DOWN_OUTSIDE && isFiniteF32(payload.x) && isFiniteF32(payload.y);
    case "close-requested":
      return eventType === EVENT_CLOSE_REQUESTED && isU32(payload.requestId);
    case "extension":
      return (
        eventType === EVENT_EXTENSION &&
        isU32(payload.eventId) &&
        payload.eventId !== 0 &&
        validateExtensionFields(payload.fields)
      );
  }
  return false;
}

export function validateEvent(value: Event): Event | null {
  if (
    value.payload.type === "application-activation" &&
    (value.surfaceId !== 0 ||
      value.nodeId !== 0 ||
      value.listenerId !== 0 ||
      value.revision !== 0 ||
      value.epoch === 0 ||
      value.sequence === 0)
  )
    return null;
  if (
    value.surfaceId < 0 ||
    !isU32(value.surfaceId) ||
    !isU32(value.epoch) ||
    !isU32(value.revision) ||
    !isU32(value.sequence) ||
    !isU32(value.nodeId) ||
    !isU32(value.listenerId)
  )
    return null;
  const eventType = (() => {
    switch (value.payload.type) {
      case "application-activation":
        return EVENT_APPLICATION_ACTIVATION;
      case "press":
        return EVENT_PRESS;
      case "change":
        return EVENT_CHANGE;
      case "selection":
        return EVENT_SELECTION;
      case "focus":
        return EVENT_FOCUS;
      case "blur":
        return EVENT_BLUR;
      case "command-result":
        return EVENT_COMMAND_RESULT;
      case "visible-range":
        return EVENT_VISIBLE_RANGE;
      case "animation-complete":
        return 8;
      case "key":
        return EVENT_KEY;
      case "pointer":
      case "pointer-move":
        return EVENT_POINTER;
      case "hover":
        return EVENT_HOVER;
      case "scroll":
        return EVENT_SCROLL;
      case "submit":
        return EVENT_SUBMIT;
      case "window-resize":
        return EVENT_WINDOW_RESIZE;
      case "window-activation":
        return EVENT_WINDOW_ACTIVATION;
      case "surface-closed":
        return EVENT_SURFACE_CLOSED;
      case "action":
        return EVENT_ACTION;
      case "window-appearance":
        return EVENT_WINDOW_APPEARANCE;
      case "layout":
        return EVENT_LAYOUT;
      case "drag-over":
      case "drag-drop":
      case "external-file-drop":
        return EVENT_DRAG;
      case "notification-response":
        return EVENT_NOTIFICATION_RESPONSE;
      case "pointer-down-outside":
        return EVENT_POINTER_DOWN_OUTSIDE;
      case "close-requested":
        return EVENT_CLOSE_REQUESTED;
      case "extension":
        return EVENT_EXTENSION;
    }
  })();
  if (!validateEventPayload(eventType, value.payload)) return null;
  if (value.payload.type === "surface-closed" && (value.nodeId !== 0 || value.listenerId !== 0)) return null;
  if (
    value.payload.type === "command-result" &&
    value.payload.result.command === COMMAND_INVOKE_NATIVE &&
    (value.nodeId === 0 || value.nodeId !== value.payload.result.nodeId || value.listenerId !== 0)
  )
    return null;
  if (
    (value.payload.type === "focus" || value.payload.type === "blur") &&
    value.payload.data === undefined &&
    (value.nodeId === 0 || value.listenerId === 0)
  )
    return null;
  if (
    (value.payload.type === "action" ||
      value.payload.type === "window-appearance" ||
      value.payload.type === "notification-response") &&
    (value.nodeId !== 1 || value.listenerId !== 0)
  )
    return null;
  if (
    (value.payload.type === "drag-over" ||
      value.payload.type === "drag-drop" ||
      value.payload.type === "external-file-drop" ||
      value.payload.type === "pointer-down-outside") &&
    (value.nodeId === 0 || value.listenerId === 0)
  )
    return null;
  return value;
}

export { DRAG_DROP, DRAG_EXTERNAL_FILE_DROP, DRAG_OVER, PROTOCOL_VERSION };
