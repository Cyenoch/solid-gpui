import { decode, encode } from "@msgpack/msgpack";

const MESSAGEPACK_V3_ENCODE_OPTIONS = { sortKeys: false, forceFloat32: true } as const;
const UTF8_ENCODER = new TextEncoder();

/** V3 resource limits count UTF-8 bytes; TextInput maxLength remains UTF-16 units. */
export function utf8ByteLength(value: string): number {
  return UTF8_ENCODER.encode(value).byteLength;
}

export const PROTOCOL_VERSION = 3 as const;
export class ProtocolVersionMismatchError extends Error {
  readonly receivedVersion: number;
  readonly expectedVersion = PROTOCOL_VERSION;

  constructor(receivedVersion: number) {
    super(
      `protocol version mismatch: host binary speaks protocol v${receivedVersion}; this renderer package speaks protocol v${PROTOCOL_VERSION} — update @react-gpui/core to a v${receivedVersion} release / pin the host binary to a v${PROTOCOL_VERSION} release`,
    );
    this.receivedVersion = receivedVersion;
    this.name = "ProtocolVersionMismatchError";
  }
}

export const SNAPSHOT_KIND = 1 as const;
export const EVENT_KIND = 2 as const;
export const PATCH_KIND = 3 as const;
export const COMMAND_KIND = 4 as const;
export const MAX_FRAME_SIZE = 16 * 1024 * 1024;
export const MAX_CLIPBOARD_TEXT_BYTES = 1 << 20;
/** File and clipboard-image payloads leave 1 KiB for the complete MessagePack envelope. */
export const MAX_CLIPBOARD_IMAGE_BYTES = MAX_FRAME_SIZE - 1024;
/** File payloads leave 1 KiB for the complete MessagePack command/frame envelope. */
export const MAX_FILE_WRITE_BYTES = MAX_FRAME_SIZE - 1024;
export const MAX_FILE_READ_BYTES = MAX_FRAME_SIZE - 1024;
export const EVENT_PRESS = 1 as const;
export const EVENT_CHANGE = 2 as const;
export const EVENT_SELECTION = 3 as const;
export const EVENT_FOCUS = 4 as const;
export const EVENT_BLUR = 5 as const;
export const EVENT_COMMAND_RESULT = 6 as const;
export const EVENT_VISIBLE_RANGE = 7 as const;
export const EVENT_ANIMATION_COMPLETE = 8 as const;
export const EVENT_KEY = 9 as const;
export const EVENT_POINTER = 10 as const;
export const EVENT_HOVER = 11 as const;
export const EVENT_SCROLL = 12 as const;
export const EVENT_SUBMIT = 13 as const;
export const EVENT_WINDOW_RESIZE = 14 as const;
export const EVENT_WINDOW_ACTIVATION = 15 as const;
export const EVENT_SURFACE_CLOSED = 16 as const;
export const EVENT_ACTION = 17 as const;
export const EVENT_WINDOW_APPEARANCE = 18 as const;
export const EVENT_LAYOUT = 19 as const;
export const EVENT_DRAG = 20 as const;
export const EVENT_NOTIFICATION_RESPONSE = 21 as const;
export const EVENT_POINTER_DOWN_OUTSIDE = 22 as const;
export const EVENT_CLOSE_REQUESTED = 23 as const;
export const DRAG_OVER = 1 as const;
export const DRAG_DROP = 2 as const;
export const DRAG_EXTERNAL_FILE_DROP = 3 as const;
export const EVENT_POINTER_DOWN = 1 as const;
export const EVENT_POINTER_UP = 2 as const;
export const POINTER_BUTTON_LEFT = 1 as const;
export const POINTER_BUTTON_RIGHT = 2 as const;
export const POINTER_BUTTON_MIDDLE = 3 as const;
export const POINTER_BUTTON_BACK = 4 as const;
export const POINTER_BUTTON_FORWARD = 5 as const;
export const SCROLL_DELTA_PIXELS = 1 as const;
export const SCROLL_DELTA_LINES = 2 as const;
export const EVENT_KEY_DOWN = 1 as const;
export const EVENT_KEY_REPEAT = 2 as const;
export const EVENT_KEY_UP = 3 as const;
export const COMMAND_FOCUS = 1 as const;
export const COMMAND_BLUR = 2 as const;
export const COMMAND_SET_SELECTION = 3 as const;
export const COMMAND_SCROLL_TO_INDEX = 4 as const;
export const COMMAND_SCROLL_TO_END = 5 as const;
export const COMMAND_SET_TITLE = 6 as const;
export const COMMAND_RESIZE_WINDOW = 7 as const;
export const COMMAND_ZOOM_WINDOW = 8 as const;
export const COMMAND_TOGGLE_FULLSCREEN = 9 as const;
export const COMMAND_OPEN_URL = 10 as const;
export const COMMAND_FOCUS_NEXT = 11 as const;
export const COMMAND_FOCUS_PREV = 12 as const;
export const COMMAND_GET_WINDOW_SIZE = 13 as const;
export const COMMAND_GET_FOCUS = 14 as const;
export const COMMAND_CLIPBOARD_WRITE = 15 as const;
export const COMMAND_CLIPBOARD_READ = 16 as const;
export const COMMAND_OPEN_SURFACE = 17 as const;
export const COMMAND_FILE_DIALOG_OPEN = 18 as const;
export const COMMAND_FILE_DIALOG_SAVE = 19 as const;
export const COMMAND_SHOW_NOTIFICATION = 20 as const;
export const COMMAND_SET_MENUS = 21 as const;
export const COMMAND_SET_KEYBINDINGS = 22 as const;
export const COMMAND_SET_CLOSE_POLICY = 23 as const;
export const COMMAND_RESOLVE_CLOSE_REQUEST = 24 as const;
export const COMMAND_READ_TEXT_FILE = 25 as const;
export const COMMAND_WRITE_TEXT_FILE = 26 as const;
export const COMMAND_CLIPBOARD_WRITE_IMAGE = 27 as const;
export const COMMAND_CLIPBOARD_READ_IMAGE = 28 as const;
export const CLIPBOARD_IMAGE_FORMAT_PNG = 1 as const;
export const CLIPBOARD_IMAGE_FORMAT_JPEG = 2 as const;
export const CLIPBOARD_IMAGE_FORMAT_GIF = 3 as const;
export const CLIPBOARD_IMAGE_FORMAT_SVG = 4 as const;
export type ClipboardImageFormat = "png" | "jpeg" | "gif" | "svg";
export type ClipboardImage = { readonly format: ClipboardImageFormat; readonly bytes: Uint8Array };
export const IMAGE_OBJECT_FIT_SCALE_DOWN = 4 as const;
export const IMAGE_OBJECT_FIT_NONE = 5 as const;
export const UPDATE_STYLE = 1 as const;
export const UPDATE_TEXT = 2 as const;
export const UPDATE_LISTENER = 4 as const;
export const UPDATE_PROPERTIES = 8 as const;
export const UPDATE_ACCESSIBILITY = 16 as const;
export const UPDATE_FOCUSABLE = 32 as const;
export const UPDATE_SELECTABLE = 64 as const;
export const UPDATE_TOOLTIP = 128 as const;
function assertU32(name: string, value: unknown): asserts value is number {
  if (typeof value !== "number" || !Number.isInteger(value) || value < 0 || value > 0xffff_ffff) {
    throw new TypeError(`${name} must be a uint32`);
  }
}

function assertBool(name: string, value: unknown): asserts value is boolean {
  if (typeof value !== "boolean") throw new TypeError(`${name} must be a boolean`);
}

const KEY_MODIFIER_NAMES: Record<string, true> = {
  cmd: true,
  ctrl: true,
  alt: true,
  shift: true,
  function: true,
};

export type TextInputPropertiesWire = readonly [
  1,
  string,
  string | null,
  boolean,
  boolean,
  boolean,
  number,
  number,
  number,
  number | null,
  number | null,
  number | null,
  boolean,
];
export type VirtualListPropertiesWire = readonly [2, number, number, number, number, number];
export type ImagePropertiesWire = readonly [3, string, 1 | 2 | 3 | 4 | 5, string | null];
export type DragPropertiesWire = readonly [4, string | null, readonly string[] | null, boolean, boolean];
export type HostPropertiesWire =
  | TextInputPropertiesWire
  | VirtualListPropertiesWire
  | ImagePropertiesWire
  | DragPropertiesWire;

export type AccessibilityPropertiesWire = readonly [
  number,
  string | null,
  string | null,
  boolean,
  boolean | null,
  boolean | null,
  string | null,
  (boolean | null)?,
  (number | null)?,
];

export type SnapshotNode = readonly [
  number,
  number,
  number,
  1 | 2 | 3 | 4 | 5 | 6 | 7,
  readonly unknown[] | null,
  string | null,
  number,
  HostPropertiesWire | null,
  AccessibilityPropertiesWire | null,
  boolean,
  boolean?,
  (string | null)?,
];
export type Snapshot = readonly [
  typeof PROTOCOL_VERSION,
  typeof SNAPSHOT_KIND,
  number,
  number,
  number,
  number,
  readonly SnapshotNode[],
];

export type PatchCreate = readonly [
  1,
  number,
  number,
  number,
  1 | 2 | 3 | 4 | 5 | 6 | 7,
  readonly unknown[] | null,
  string | null,
  number,
  HostPropertiesWire | null,
  AccessibilityPropertiesWire | null,
  boolean,
  boolean?,
  (string | null)?,
];
export type PatchUpdate = readonly [
  2,
  number,
  number,
  readonly unknown[] | null,
  string | null,
  number,
  HostPropertiesWire | null,
  AccessibilityPropertiesWire | null,
  boolean,
  boolean?,
  (string | null)?,
];
export type PatchMove = readonly [3, number, number, number];
export type PatchDelete = readonly [4, number];
export type PatchOperation = PatchCreate | PatchUpdate | PatchMove | PatchDelete;
export type Patch = readonly [
  typeof PROTOCOL_VERSION,
  typeof PATCH_KIND,
  number,
  number,
  number,
  number,
  readonly PatchOperation[],
];
export type MenuItemPayload =
  | readonly [0]
  | readonly [1, string]
  | readonly [1, string, readonly [boolean, boolean]]
  | readonly [2, readonly [string, readonly MenuItemPayload[]]];
export type MenuPayload = readonly (readonly [string, readonly MenuItemPayload[]])[];
export type KeybindingsPayload = readonly (readonly [string, string])[];
export type WindowOpenOptionsPayload = readonly [0 | 1 | 2 | null, boolean | null, number | null, number | null];

export type ClipboardImageFormatCode =
  | typeof CLIPBOARD_IMAGE_FORMAT_PNG
  | typeof CLIPBOARD_IMAGE_FORMAT_JPEG
  | typeof CLIPBOARD_IMAGE_FORMAT_GIF
  | typeof CLIPBOARD_IMAGE_FORMAT_SVG;
export type ClipboardImagePayload = readonly [ClipboardImageFormatCode, Uint8Array];
export type Command = readonly [
  typeof PROTOCOL_VERSION,
  typeof COMMAND_KIND,
  number,
  number,
  number,
  number,
  number,
  (
    | typeof COMMAND_FOCUS
    | typeof COMMAND_BLUR
    | typeof COMMAND_SET_SELECTION
    | typeof COMMAND_SCROLL_TO_INDEX
    | typeof COMMAND_SCROLL_TO_END
    | typeof COMMAND_SET_TITLE
    | typeof COMMAND_RESIZE_WINDOW
    | typeof COMMAND_ZOOM_WINDOW
    | typeof COMMAND_TOGGLE_FULLSCREEN
    | typeof COMMAND_OPEN_URL
    | typeof COMMAND_FOCUS_NEXT
    | typeof COMMAND_FOCUS_PREV
    | typeof COMMAND_GET_WINDOW_SIZE
    | typeof COMMAND_GET_FOCUS
    | typeof COMMAND_CLIPBOARD_WRITE
    | typeof COMMAND_CLIPBOARD_READ
    | typeof COMMAND_OPEN_SURFACE
    | typeof COMMAND_FILE_DIALOG_OPEN
    | typeof COMMAND_FILE_DIALOG_SAVE
    | typeof COMMAND_SHOW_NOTIFICATION
    | typeof COMMAND_SET_MENUS
    | typeof COMMAND_SET_KEYBINDINGS
    | typeof COMMAND_SET_CLOSE_POLICY
    | typeof COMMAND_RESOLVE_CLOSE_REQUEST
    | typeof COMMAND_READ_TEXT_FILE
    | typeof COMMAND_WRITE_TEXT_FILE
    | typeof COMMAND_CLIPBOARD_WRITE_IMAGE
    | typeof COMMAND_CLIPBOARD_READ_IMAGE
  ),
  (
    | readonly [number, number]
    | readonly [string, readonly [number, number]]
    | readonly [string, readonly [number, number], WindowOpenOptionsPayload]
    | readonly [string, string]
    | readonly [string, string, readonly (readonly [string, string])[]]
    | ClipboardImagePayload
    | KeybindingsPayload
    | string
    | MenuPayload
    | null
  ),
];
export type CommandValuePayload =
  | readonly [1, number]
  | readonly [2, readonly [number, number]]
  | readonly [3, boolean]
  | readonly [4, string]
  | readonly [5, readonly string[]]
  | readonly [6, string]
  | readonly [7, ClipboardImagePayload];
export type CommandResultPayload = readonly [
  2,
  number,
  number,
  number,
  boolean,
  string | null,
  CommandValuePayload | null,
];
export type TextInputEventPayload = readonly [1, string, number, number, number | null, number | null, number, boolean];
export type VisibleRangePayload = readonly [3, number, number];
export type AnimationCompletePayload = readonly [4, number];
export type KeyEventPayload = readonly [5, string, readonly string[], 1 | 2 | 3];
export type PointerEventPayload = readonly [6, 1 | 2 | 3 | 4 | 5, readonly string[], 1 | 2, number];
export type ScrollEventPayload = readonly [7, 1 | 2, number, number, number, number, readonly string[]];
export type SubmitEventPayload = string;
export type WindowResizeEventPayload = readonly [number, number, number];
export type WindowActivationEventPayload = boolean;
export type ActionEventPayload = string;
export type WindowAppearanceEventPayload = "light" | "dark";
export type CloseRequestedEventPayload = readonly [9, number];
export type LayoutEventPayload = readonly [number, number, number, number];
export type DragEventPayload =
  | readonly [typeof DRAG_OVER, string]
  | readonly [typeof DRAG_DROP, string]
  | readonly [typeof DRAG_EXTERNAL_FILE_DROP, readonly string[]];
export type NotificationResponseEventPayload = readonly [string, string | null];
export type PointerDownOutsideEventPayload = readonly [8, number, number];
export type EventPayload =
  | TextInputEventPayload
  | CommandResultPayload
  | VisibleRangePayload
  | AnimationCompletePayload
  | KeyEventPayload
  | PointerEventPayload
  | ScrollEventPayload
  | SubmitEventPayload
  | WindowResizeEventPayload
  | WindowActivationEventPayload
  | ActionEventPayload
  | LayoutEventPayload
  | DragEventPayload
  | NotificationResponseEventPayload
  | PointerDownOutsideEventPayload
  | CloseRequestedEventPayload;
export type PressEventFrame = readonly [
  typeof PROTOCOL_VERSION,
  typeof EVENT_KIND,
  number,
  number,
  number,
  number,
  number,
  number,
  (
    | typeof EVENT_PRESS
    | typeof EVENT_CHANGE
    | typeof EVENT_SELECTION
    | typeof EVENT_FOCUS
    | typeof EVENT_BLUR
    | typeof EVENT_COMMAND_RESULT
    | typeof EVENT_VISIBLE_RANGE
    | typeof EVENT_ANIMATION_COMPLETE
    | typeof EVENT_KEY
    | typeof EVENT_POINTER
    | typeof EVENT_HOVER
    | typeof EVENT_SCROLL
    | typeof EVENT_SUBMIT
    | typeof EVENT_WINDOW_RESIZE
    | typeof EVENT_WINDOW_ACTIVATION
    | typeof EVENT_SURFACE_CLOSED
    | typeof EVENT_ACTION
    | typeof EVENT_WINDOW_APPEARANCE
    | typeof EVENT_LAYOUT
    | typeof EVENT_DRAG
    | typeof EVENT_NOTIFICATION_RESPONSE
    | typeof EVENT_POINTER_DOWN_OUTSIDE
    | typeof EVENT_CLOSE_REQUESTED
  ),
  EventPayload | null,
];

function validateCommandValue(value: unknown): value is CommandValuePayload {
  if (!Array.isArray(value)) return false;
  if (value[0] === 1) return value.length === 2 && typeof value[1] === "number" && Number.isFinite(value[1]);
  if (value[0] === 2) {
    return (
      value.length === 2 &&
      Array.isArray(value[1]) &&
      value[1].length === 2 &&
      typeof value[1][0] === "number" &&
      Number.isFinite(value[1][0]) &&
      value[1][0] >= 0 &&
      typeof value[1][1] === "number" &&
      Number.isFinite(value[1][1]) &&
      value[1][1] >= 0
    );
  }
  if (value[0] === 3) return value.length === 2 && typeof value[1] === "boolean";
  if (value[0] === 5) {
    return (
      value.length === 2 &&
      Array.isArray(value[1]) &&
      value[1].length > 0 &&
      value[1].every((path) => typeof path === "string" && path.length > 0)
    );
  }
  if (value[0] === 6) {
    return value.length === 2 && typeof value[1] === "string" && utf8ByteLength(value[1]) <= MAX_FILE_READ_BYTES;
  }
  if (value[0] === 7) {
    return (
      value.length === 2 &&
      Array.isArray(value[1]) &&
      value[1].length === 2 &&
      typeof value[1][0] === "number" &&
      Number.isInteger(value[1][0]) &&
      value[1][0] >= CLIPBOARD_IMAGE_FORMAT_PNG &&
      value[1][0] <= CLIPBOARD_IMAGE_FORMAT_SVG &&
      value[1][1] instanceof Uint8Array &&
      value[1][1].byteLength > 0 &&
      value[1][1].byteLength <= MAX_CLIPBOARD_IMAGE_BYTES
    );
  }
  return (
    value[0] === 4 &&
    value.length === 2 &&
    typeof value[1] === "string" &&
    utf8ByteLength(value[1]) <= MAX_CLIPBOARD_TEXT_BYTES
  );
}
function bytesFrom(value: Uint8Array | ArrayBuffer): Uint8Array {
  return value instanceof Uint8Array ? value : new Uint8Array(value);
}

export function encodePayload(value: Snapshot | Patch | Command | PressEventFrame): Uint8Array {
  return encode(value, MESSAGEPACK_V3_ENCODE_OPTIONS);
}
export function framePayload(payload: Uint8Array): Uint8Array {
  if (payload.byteLength > MAX_FRAME_SIZE) throw new RangeError("frame exceeds maximum size");
  const frame = new Uint8Array(4 + payload.byteLength);
  new DataView(frame.buffer).setUint32(0, payload.byteLength, true);
  frame.set(payload, 4);
  return frame;
}
export function encodeFrame(value: Snapshot | Patch | Command | PressEventFrame): Uint8Array {
  return framePayload(encodePayload(value));
}

export class FrameDecoder {
  private buffer = new Uint8Array(4);
  private readOffset = 0;
  private writeOffset = 0;
  constructor(private readonly maxFrameSize = MAX_FRAME_SIZE) {
    if (!Number.isInteger(maxFrameSize) || maxFrameSize < 1) throw new RangeError("maxFrameSize must be positive");
  }
  push(chunk: Uint8Array | ArrayBuffer): Uint8Array[] {
    const incoming = bytesFrom(chunk);
    if (incoming.byteLength === 0) return [];
    this.ensureCapacity(incoming.byteLength);
    this.buffer.set(incoming, this.writeOffset);
    this.writeOffset += incoming.byteLength;
    const payloads: Uint8Array[] = [];
    while (this.writeOffset - this.readOffset >= 4) {
      const size = new DataView(this.buffer.buffer, this.buffer.byteOffset + this.readOffset, 4).getUint32(0, true);
      if (size > this.maxFrameSize) {
        this.readOffset = 0;
        this.writeOffset = 0;
        throw new RangeError(`frame length ${size} exceeds maximum ${this.maxFrameSize}`);
      }
      if (this.writeOffset - this.readOffset - 4 < size) break;
      payloads.push(this.buffer.slice(this.readOffset + 4, this.readOffset + 4 + size));
      this.readOffset += 4 + size;
    }
    if (this.readOffset === this.writeOffset) {
      this.readOffset = 0;
      this.writeOffset = 0;
    }
    return payloads;
  }
  private ensureCapacity(incomingLength: number): void {
    let required = this.writeOffset + incomingLength;
    if (required <= this.buffer.byteLength) return;
    const unreadLength = this.writeOffset - this.readOffset;
    if (this.readOffset > 0) {
      this.buffer.copyWithin(0, this.readOffset, this.writeOffset);
      this.readOffset = 0;
      this.writeOffset = unreadLength;
      required = unreadLength + incomingLength;
      if (required <= this.buffer.byteLength) return;
    }
    let capacity = this.buffer.byteLength;
    while (capacity < required) capacity *= 2;
    const grown = new Uint8Array(capacity);
    grown.set(this.buffer.subarray(this.readOffset, this.writeOffset));
    this.buffer = grown;
    this.readOffset = 0;
    this.writeOffset = unreadLength;
  }
}

function decodeWire(payload: Uint8Array): unknown {
  try {
    return decode(payload, { useBigInt64: false });
  } catch {
    return null;
  }
}

/** @internal Golden-vector seam; not re-exported from the package entry point. */
export function decodeWireForGolden(payload: Uint8Array): unknown {
  return decodeWire(payload);
}
function validateTooltip(value: unknown): value is string {
  return (
    typeof value === "string" &&
    value.length > 0 &&
    utf8ByteLength(value) <= 256 &&
    !/[\u0000-\u001f\u007f]/.test(value)
  );
}
function validateHostProperties(value: unknown): value is HostPropertiesWire {
  if (!Array.isArray(value)) return false;
  if (value[0] === 1) {
    if (
      value.length !== 13 ||
      typeof value[1] !== "string" ||
      (value[2] !== null && typeof value[2] !== "string") ||
      typeof value[3] !== "boolean" ||
      typeof value[4] !== "boolean" ||
      typeof value[5] !== "boolean" ||
      typeof value[12] !== "boolean"
    )
      return false;
    for (const [index, name] of [
      [6, "ackEditSeq"],
      [7, "selectionStart"],
      [8, "selectionEnd"],
    ] as const) {
      try {
        assertU32(name, value[index]);
      } catch {
        return false;
      }
    }
    if ((value[9] !== null && typeof value[9] !== "number") || (value[10] !== null && typeof value[10] !== "number"))
      return false;
    if (
      value[9] !== null &&
      (value[10] === null ||
        !Number.isInteger(value[9]) ||
        !Number.isInteger(value[10]) ||
        value[9] < 0 ||
        value[10] < value[9] ||
        value[10] > 0xffff_ffff)
    )
      return false;
    if (value[11] !== null) {
      try {
        assertU32("maxLength", value[11]);
      } catch {
        return false;
      }
    }
    return true;
  }
  if (value[0] === 2) {
    if (value.length !== 6) return false;
    try {
      assertU32("itemCount", value[1]);
      assertU32("rangeStart", value[2]);
      assertU32("rangeEnd", value[3]);
      assertU32("overscan", value[5]);
    } catch {
      return false;
    }
    return (
      typeof value[4] === "number" &&
      Number.isFinite(value[4]) &&
      value[4] > 0 &&
      value[2] <= value[3] &&
      value[3] <= value[1]
    );
  }
  if (value[0] === 3) {
    return (
      value.length === 4 &&
      typeof value[1] === "string" &&
      value[1].length > 0 &&
      utf8ByteLength(value[1]) <= 1024 &&
      !/[\u0000-\u001f\u007f]/.test(value[1]) &&
      typeof value[2] === "number" &&
      Number.isInteger(value[2]) &&
      value[2] >= 1 &&
      value[2] <= 5 &&
      (value[3] === null ||
        (typeof value[3] === "string" &&
          value[3].length > 0 &&
          utf8ByteLength(value[3]) <= 1024 &&
          !/[\u0000-\u001f\u007f]/.test(value[3])))
    );
  }
  if (value[0] === 4) {
    return (
      value.length === 5 &&
      (value[2] === null ||
        (Array.isArray(value[2]) &&
          value[2].length > 0 &&
          value[2].length <= 8 &&
          value[2].every(
            (path) =>
              typeof path === "string" &&
              path.length > 0 &&
              utf8ByteLength(path) <= 1024 &&
              !/[\u0000-\u001f\u007f]/.test(path),
          ))) &&
      typeof value[3] === "boolean" &&
      typeof value[4] === "boolean" &&
      (value[1] === null ||
        (typeof value[1] === "string" &&
          value[1].length > 0 &&
          [...value[1]].length <= 128 &&
          !/[\u0000-\u001f\u007f]/.test(value[1])))
    );
  }
  return false;
}
function validateEventPayload(eventType: number, payload: unknown): payload is EventPayload | null {
  if (eventType === EVENT_PRESS || eventType === EVENT_HOVER || eventType === EVENT_SURFACE_CLOSED)
    return payload === null;
  if (eventType === EVENT_CLOSE_REQUESTED) {
    return (
      Array.isArray(payload) &&
      payload.length === 2 &&
      payload[0] === 9 &&
      typeof payload[1] === "number" &&
      Number.isInteger(payload[1]) &&
      payload[1] >= 0 &&
      payload[1] <= 0xffff_ffff
    );
  }
  if ((eventType === EVENT_FOCUS || eventType === EVENT_BLUR) && payload === null) return true;
  if (eventType === EVENT_SUBMIT) return typeof payload === "string";
  if (eventType === EVENT_LAYOUT) {
    return (
      Array.isArray(payload) &&
      payload.length === 4 &&
      payload.every((value) => typeof value === "number" && Number.isFinite(value))
    );
  }
  if (eventType === EVENT_DRAG) {
    if (!Array.isArray(payload) || payload.length !== 2) return false;
    if (payload[0] === DRAG_OVER || payload[0] === DRAG_DROP) {
      return (
        typeof payload[1] === "string" &&
        payload[1].length > 0 &&
        [...payload[1]].length <= 128 &&
        !/[\u0000-\u001f\u007f]/.test(payload[1])
      );
    }
    if (payload[0] === DRAG_EXTERNAL_FILE_DROP && Array.isArray(payload[1])) {
      return (
        payload[1].length > 0 &&
        payload[1].every(
          (path) =>
            typeof path === "string" && path.length > 0 && path.length <= 4096 && !/[\u0000-\u001f\u007f]/.test(path),
        )
      );
    }
    return false;
  }
  if (eventType === EVENT_NOTIFICATION_RESPONSE) {
    return (
      Array.isArray(payload) &&
      payload.length === 2 &&
      typeof payload[0] === "string" &&
      payload[0].length > 0 &&
      [...payload[0]].length <= 256 &&
      (payload[1] === null ||
        (typeof payload[1] === "string" && payload[1].length > 0 && utf8ByteLength(payload[1]) <= 64))
    );
  }
  if (eventType === EVENT_ACTION)
    return typeof payload === "string" && payload.length > 0 && [...payload].length <= 256;
  if (eventType === EVENT_POINTER_DOWN_OUTSIDE) {
    return (
      Array.isArray(payload) &&
      payload.length === 3 &&
      payload[0] === 8 &&
      typeof payload[1] === "number" &&
      Number.isFinite(payload[1]) &&
      typeof payload[2] === "number" &&
      Number.isFinite(payload[2])
    );
  }
  if (eventType === EVENT_WINDOW_APPEARANCE) return payload === "light" || payload === "dark";
  if (eventType === EVENT_WINDOW_ACTIVATION) return typeof payload === "boolean";
  if (eventType === EVENT_WINDOW_RESIZE) {
    return (
      Array.isArray(payload) &&
      payload.length === 3 &&
      typeof payload[0] === "number" &&
      Number.isFinite(payload[0]) &&
      payload[0] >= 0 &&
      typeof payload[1] === "number" &&
      Number.isFinite(payload[1]) &&
      payload[1] >= 0 &&
      typeof payload[2] === "number" &&
      Number.isFinite(payload[2]) &&
      payload[2] > 0
    );
  }
  if (!Array.isArray(payload)) return false;
  if (eventType === EVENT_POINTER) {
    if (payload.length !== 5 || payload[0] !== 6) return false;
    const pointerButtons: readonly number[] = [
      POINTER_BUTTON_LEFT,
      POINTER_BUTTON_RIGHT,
      POINTER_BUTTON_MIDDLE,
      POINTER_BUTTON_BACK,
      POINTER_BUTTON_FORWARD,
    ];
    if (typeof payload[1] !== "number" || !pointerButtons.includes(payload[1])) return false;
    if (!Array.isArray(payload[2])) return false;
    const seen: Record<string, true> = {};
    for (const modifier of payload[2]) {
      if (typeof modifier !== "string" || !Object.hasOwn(KEY_MODIFIER_NAMES, modifier) || seen[modifier]) return false;
      seen[modifier] = true;
    }
    if (payload[3] !== EVENT_POINTER_DOWN && payload[3] !== EVENT_POINTER_UP) return false;
    try {
      assertU32("clickCount", payload[4]);
    } catch {
      return false;
    }
    return payload[4] > 0;
  }
  if (eventType === EVENT_SCROLL) {
    if (
      payload.length !== 7 ||
      payload[0] !== 7 ||
      (payload[1] !== SCROLL_DELTA_PIXELS && payload[1] !== SCROLL_DELTA_LINES) ||
      ![2, 3, 4, 5].every((index) => typeof payload[index] === "number" && Number.isFinite(payload[index]))
    )
      return false;
    if (!Array.isArray(payload[6])) return false;
    const seen: Record<string, true> = {};
    for (const modifier of payload[6]) {
      if (typeof modifier !== "string" || !Object.hasOwn(KEY_MODIFIER_NAMES, modifier) || seen[modifier]) return false;
      seen[modifier] = true;
    }
    return true;
  }
  if (eventType === EVENT_KEY) {
    if (payload.length !== 4 || payload[0] !== 5 || typeof payload[1] !== "string" || payload[1].length === 0)
      return false;
    if (!Array.isArray(payload[2])) return false;
    const seen: Record<string, true> = {};
    for (const modifier of payload[2]) {
      if (typeof modifier !== "string" || !Object.hasOwn(KEY_MODIFIER_NAMES, modifier) || seen[modifier]) return false;
      seen[modifier] = true;
    }
    return payload[3] === EVENT_KEY_DOWN || payload[3] === EVENT_KEY_REPEAT || payload[3] === EVENT_KEY_UP;
  }
  if (eventType === EVENT_COMMAND_RESULT) {
    if (payload.length !== 7 || payload[0] !== 2) return false;
    try {
      assertU32("requestId", payload[1]);
      assertU32("command", payload[2]);
      assertU32("nodeId", payload[3]);
    } catch {
      return false;
    }
    const validCommands: readonly number[] = [
      COMMAND_CLIPBOARD_WRITE_IMAGE,
      COMMAND_CLIPBOARD_READ_IMAGE,
      COMMAND_FOCUS,
      COMMAND_BLUR,
      COMMAND_SET_SELECTION,
      COMMAND_SCROLL_TO_INDEX,
      COMMAND_SCROLL_TO_END,
      COMMAND_SET_TITLE,
      COMMAND_RESIZE_WINDOW,
      COMMAND_ZOOM_WINDOW,
      COMMAND_TOGGLE_FULLSCREEN,
      COMMAND_OPEN_URL,
      COMMAND_FOCUS_NEXT,
      COMMAND_FOCUS_PREV,
      COMMAND_GET_WINDOW_SIZE,
      COMMAND_GET_FOCUS,
      COMMAND_CLIPBOARD_WRITE,
      COMMAND_CLIPBOARD_READ,
      COMMAND_OPEN_SURFACE,
      COMMAND_FILE_DIALOG_OPEN,
      COMMAND_FILE_DIALOG_SAVE,
      COMMAND_SHOW_NOTIFICATION,
      COMMAND_SET_MENUS,
      COMMAND_SET_KEYBINDINGS,
      COMMAND_SET_CLOSE_POLICY,
      COMMAND_RESOLVE_CLOSE_REQUEST,
      COMMAND_READ_TEXT_FILE,
      COMMAND_WRITE_TEXT_FILE,
    ];
    if (!validCommands.includes(payload[2] as number)) return false;
    if (typeof payload[4] !== "boolean" || (payload[5] !== null && typeof payload[5] !== "string")) return false;
    return payload[6] === null || validateCommandValue(payload[6]);
  }
  if (eventType === EVENT_VISIBLE_RANGE) {
    if (payload.length !== 3 || payload[0] !== 3) return false;
    try {
      assertU32("rangeStart", payload[1]);
      assertU32("rangeEnd", payload[2]);
    } catch {
      return false;
    }
    return payload[1] <= payload[2];
  }
  if (eventType === EVENT_ANIMATION_COMPLETE) {
    if (payload.length !== 2 || payload[0] !== 4) return false;
    try {
      assertU32("generation", payload[1]);
    } catch {
      return false;
    }
    return true;
  }
  if (eventType < EVENT_CHANGE || eventType > EVENT_BLUR) return false;
  if (payload.length !== 8 || payload[0] !== 1 || typeof payload[1] !== "string") return false;
  try {
    assertU32("selectionStart", payload[2]);
    assertU32("selectionEnd", payload[3]);
    assertU32("editSeq", payload[6]);
  } catch {
    return false;
  }
  if (typeof payload[7] !== "boolean") return false;
  if (payload[2] > payload[3] || (payload[4] === null) !== (payload[5] === null)) return false;
  if (
    payload[4] !== null &&
    (!Number.isInteger(payload[4]) ||
      !Number.isInteger(payload[5]) ||
      payload[4] < 0 ||
      payload[5] < payload[4] ||
      payload[5] > 0xffff_ffff)
  )
    return false;
  return true;
}
export function decodeEvent(payload: Uint8Array): PressEventFrame | null {
  const value = decodeWire(payload);
  if (!Array.isArray(value)) return null;
  if (
    value.length > 0 &&
    value[0] !== PROTOCOL_VERSION &&
    typeof value[0] === "number" &&
    Number.isInteger(value[0]) &&
    value[0] >= 0 &&
    value[0] <= 0xffff_ffff
  )
    throw new ProtocolVersionMismatchError(value[0]);
  if (value.length !== 10 || value[0] !== PROTOCOL_VERSION || value[1] !== EVENT_KIND) return null;
  for (const [index, name] of [
    [2, "surfaceId"],
    [3, "epoch"],
    [4, "revision"],
    [5, "sequence"],
    [6, "nodeId"],
    [7, "listenerId"],
  ] as const) {
    try {
      assertU32(name, value[index]);
    } catch {
      return null;
    }
  }
  if (
    typeof value[8] !== "number" ||
    !Number.isInteger(value[8]) ||
    value[8] < EVENT_PRESS ||
    value[8] > EVENT_CLOSE_REQUESTED
  )
    return null;
  if (value[8] === EVENT_SURFACE_CLOSED && (value[6] !== 0 || value[7] !== 0)) return null;
  if ((value[8] === EVENT_FOCUS || value[8] === EVENT_BLUR) && value[9] === null && (value[6] === 0 || value[7] === 0))
    return null;
  if (
    (value[8] === EVENT_ACTION || value[8] === EVENT_WINDOW_APPEARANCE || value[8] === EVENT_NOTIFICATION_RESPONSE) &&
    (value[6] !== 1 || value[7] !== 0)
  )
    return null;
  if ((value[8] === EVENT_DRAG || value[8] === EVENT_POINTER_DOWN_OUTSIDE) && (value[6] === 0 || value[7] === 0))
    return null;
  return validateEventPayload(value[8], value[9]) ? (value as unknown as PressEventFrame) : null;
}
