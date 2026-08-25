import { decode, encode } from "@msgpack/msgpack";

const MESSAGEPACK_V3_ENCODE_OPTIONS = { sortKeys: false, forceFloat32: true } as const;
const UTF8_ENCODER = new TextEncoder();

/** V3 resource limits count UTF-8 bytes; TextInput maxLength remains UTF-16 units. */
export function utf8ByteLength(value: string): number {
  return UTF8_ENCODER.encode(value).byteLength;
}

export const PROTOCOL_VERSION = 3 as const;
export const SNAPSHOT_KIND = 1 as const;
export const EVENT_KIND = 2 as const;
export const PATCH_KIND = 3 as const;
export const COMMAND_KIND = 4 as const;
export const MAX_FRAME_SIZE = 16 * 1024 * 1024;
export const MAX_CLIPBOARD_TEXT_BYTES = 1 << 20;

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

export const KIND_IMAGE = 7 as const;
export const IMAGE_OBJECT_FIT_FILL = 1 as const;
export const IMAGE_OBJECT_FIT_CONTAIN = 2 as const;
export const IMAGE_OBJECT_FIT_COVER = 3 as const;
export const IMAGE_OBJECT_FIT_SCALE_DOWN = 4 as const;
export const IMAGE_OBJECT_FIT_NONE = 5 as const;
export const UPDATE_STYLE = 1 as const;
export const UPDATE_TEXT = 2 as const;
export const UPDATE_LISTENER = 4 as const;
export const UPDATE_PROPERTIES = 8 as const;
export const UPDATE_ACCESSIBILITY = 16 as const;
export const UPDATE_FOCUSABLE = 32 as const;
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
];
export type VirtualListPropertiesWire = readonly [2, number, number, number, number, number];
export type ImagePropertiesWire = readonly [3, string, 1 | 2 | 3 | 4 | 5];
export type HostPropertiesWire = TextInputPropertiesWire | VirtualListPropertiesWire | ImagePropertiesWire;

export type SnapshotNode = readonly [
  number,
  number,
  number,
  1 | 2 | 3 | 4 | 5 | 6 | 7,
  readonly unknown[] | null,
  string | null,
  number,
  HostPropertiesWire | null,
  readonly unknown[] | null,
  boolean,
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
  readonly unknown[] | null,
  boolean,
];
export type PatchUpdate = readonly [
  2,
  number,
  number,
  readonly unknown[] | null,
  string | null,
  number,
  HostPropertiesWire | null,
  readonly unknown[] | null,
  boolean,
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
  ),
  readonly [number, number] | readonly [string, readonly [number, number]] | string | null,
];
export type CommandValuePayload =
  | readonly [1, number]
  | readonly [2, readonly [number, number]]
  | readonly [3, boolean]
  | readonly [4, string];
export type CommandResultPayload =
  | readonly [2, number, number, number, boolean, string | null]
  | readonly [2, number, number, number, boolean, string | null, CommandValuePayload | null];
export type TextInputEventPayload = readonly [1, string, number, number, number | null, number | null, number];
export type VisibleRangePayload = readonly [3, number, number];
export type AnimationCompletePayload = readonly [4, number];
export type KeyEventPayload = readonly [5, string, readonly string[], 1 | 2 | 3];
export type PointerEventPayload = readonly [6, 1 | 2 | 3 | 4 | 5, readonly string[], 1 | 2, number];
export type ScrollEventPayload = readonly [7, 1 | 2, number, number, number, number, readonly string[]];
export type SubmitEventPayload = string;
export type WindowResizeEventPayload = readonly [number, number];
export type WindowActivationEventPayload = boolean;
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
  | WindowActivationEventPayload;
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
  ),
  EventPayload | null,
];

function assertU32(name: string, value: unknown): asserts value is number {
  if (typeof value !== "number" || !Number.isInteger(value) || value < 0 || value > 0xffff_ffff)
    throw new TypeError(`${name} must be a u32`);
}
function assertBool(name: string, value: unknown): asserts value is boolean {
  if (typeof value !== "boolean") throw new TypeError(`${name} must be a boolean`);
}
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
function validateHostProperties(value: unknown): value is HostPropertiesWire {
  if (!Array.isArray(value)) return false;
  if (value[0] === 1) {
    if (value.length !== 12 || typeof value[1] !== "string" || (value[2] !== null && typeof value[2] !== "string"))
      return false;
    if (typeof value[3] !== "boolean" || typeof value[4] !== "boolean" || typeof value[5] !== "boolean") return false;
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
      value.length === 3 &&
      typeof value[1] === "string" &&
      value[1].length > 0 &&
      value[1].length <= 1024 &&
      !/[\u0000-\u001f\u007f]/.test(value[1]) &&
      typeof value[2] === "number" &&
      Number.isInteger(value[2]) &&
      value[2] >= 1 &&
      value[2] <= 5
    );
  }
  return false;
}

function validateEventPayload(eventType: number, payload: unknown): payload is EventPayload | null {
  if (eventType === EVENT_PRESS || eventType === EVENT_HOVER || eventType === EVENT_SURFACE_CLOSED)
    return payload === null;
  if (eventType === EVENT_SUBMIT) return payload === null || typeof payload === "string";
  if (eventType === EVENT_WINDOW_ACTIVATION) return typeof payload === "boolean";
  if (eventType === EVENT_WINDOW_RESIZE) {
    return (
      Array.isArray(payload) &&
      payload.length === 2 &&
      typeof payload[0] === "number" &&
      Number.isFinite(payload[0]) &&
      payload[0] >= 0 &&
      typeof payload[1] === "number" &&
      Number.isFinite(payload[1]) &&
      payload[1] >= 0
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
    if ((payload.length !== 6 && payload.length !== 7) || payload[0] !== 2) return false;
    try {
      assertU32("requestId", payload[1]);
      assertU32("command", payload[2]);
      assertU32("nodeId", payload[3]);
    } catch {
      return false;
    }
    if (
      !(
        [
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
        ] as readonly number[]
      ).includes(payload[2] as number)
    )
      return false;
    if (typeof payload[4] !== "boolean" || (payload[5] !== null && typeof payload[5] !== "string")) return false;
    return payload.length === 6 || payload[6] === null || validateCommandValue(payload[6]);
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
  if (payload.length !== 7 || payload[0] !== 1 || typeof payload[1] !== "string") return false;
  try {
    assertU32("selectionStart", payload[2]);
    assertU32("selectionEnd", payload[3]);
    assertU32("editSeq", payload[6]);
  } catch {
    return false;
  }
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
  if (!Array.isArray(value) || value.length !== 10 || value[0] !== PROTOCOL_VERSION || value[1] !== EVENT_KIND)
    return null;
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
    value[8] > EVENT_SURFACE_CLOSED
  )
    return null;
  if (value[8] === EVENT_SURFACE_CLOSED && (value[6] !== 0 || value[7] !== 0)) return null;
  return validateEventPayload(value[8], value[9]) ? (value as unknown as PressEventFrame) : null;
}
