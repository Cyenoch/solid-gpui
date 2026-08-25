import { decode, encode } from "@msgpack/msgpack";

export const PROTOCOL_VERSION = 3 as const;
export const SNAPSHOT_KIND = 1 as const;
export const EVENT_KIND = 2 as const;
export const PATCH_KIND = 3 as const;
export const COMMAND_KIND = 4 as const;
export const MAX_FRAME_SIZE = 16 * 1024 * 1024;

export const EVENT_PRESS = 1 as const;
export const EVENT_CHANGE = 2 as const;
export const EVENT_SELECTION = 3 as const;
export const EVENT_FOCUS = 4 as const;
export const EVENT_BLUR = 5 as const;
export const EVENT_COMMAND_RESULT = 6 as const;
export const EVENT_VISIBLE_RANGE = 7 as const;
export const EVENT_ANIMATION_COMPLETE = 8 as const;
export const COMMAND_FOCUS = 1 as const;
export const COMMAND_BLUR = 2 as const;
export const COMMAND_SET_SELECTION = 3 as const;
export const COMMAND_SCROLL_TO_INDEX = 4 as const;
export const COMMAND_SCROLL_TO_END = 5 as const;

export const UPDATE_STYLE = 1 as const;
export const UPDATE_TEXT = 2 as const;
export const UPDATE_LISTENER = 4 as const;
export const UPDATE_PROPERTIES = 8 as const;
export const UPDATE_ACCESSIBILITY = 16 as const;

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
];
export type VirtualListPropertiesWire = readonly [2, number, number, number, number, number];
export type HostPropertiesWire = TextInputPropertiesWire | VirtualListPropertiesWire;

export type SnapshotNode = readonly [
  number,
  number,
  number,
  1 | 2 | 3 | 4 | 5 | 6,
  readonly unknown[] | null,
  string | null,
  number,
  HostPropertiesWire | null,
  readonly unknown[] | null,
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

export type PatchCreate = readonly [1, number, number, number, 1 | 2 | 3 | 4 | 5 | 6, readonly unknown[] | null, string | null, number, HostPropertiesWire | null, readonly unknown[] | null];
export type PatchUpdate = readonly [2, number, number, readonly unknown[] | null, string | null, number, HostPropertiesWire | null, readonly unknown[] | null];
export type PatchMove = readonly [3, number, number, number];
export type PatchDelete = readonly [4, number];
export type PatchOperation = PatchCreate | PatchUpdate | PatchMove | PatchDelete;
export type Patch = readonly [typeof PROTOCOL_VERSION, typeof PATCH_KIND, number, number, number, number, readonly PatchOperation[]];

export type Command = readonly [
  typeof PROTOCOL_VERSION,
  typeof COMMAND_KIND,
  number,
  number,
  number,
  number,
  number,
  typeof COMMAND_FOCUS | typeof COMMAND_BLUR | typeof COMMAND_SET_SELECTION | typeof COMMAND_SCROLL_TO_INDEX | typeof COMMAND_SCROLL_TO_END,
  readonly [number, number] | null,
];
export type CommandResultPayload = readonly [2, number, number, number, boolean, string | null];
export type TextInputEventPayload = readonly [1, string, number, number, number | null, number | null, number];
export type VisibleRangePayload = readonly [3, number, number];
export type AnimationCompletePayload = readonly [4, number];
export type EventPayload = TextInputEventPayload | CommandResultPayload | VisibleRangePayload | AnimationCompletePayload;
export type PressEventFrame = readonly [
  typeof PROTOCOL_VERSION,
  typeof EVENT_KIND,
  number,
  number,
  number,
  number,
  number,
  number,
  typeof EVENT_PRESS | typeof EVENT_CHANGE | typeof EVENT_SELECTION | typeof EVENT_FOCUS | typeof EVENT_BLUR | typeof EVENT_COMMAND_RESULT | typeof EVENT_VISIBLE_RANGE | typeof EVENT_ANIMATION_COMPLETE,
  EventPayload | null,
];

function assertU32(name: string, value: unknown): asserts value is number {
  if (typeof value !== "number" || !Number.isInteger(value) || value < 0 || value > 0xffff_ffff) throw new TypeError(`${name} must be a u32`);
}
function assertBool(name: string, value: unknown): asserts value is boolean {
  if (typeof value !== "boolean") throw new TypeError(`${name} must be a boolean`);
}
function bytesFrom(value: Uint8Array | ArrayBuffer): Uint8Array {
  return value instanceof Uint8Array ? value : new Uint8Array(value);
}

export function encodePayload(value: Snapshot | Patch | Command | PressEventFrame): Uint8Array {
  return encode(value, { sortKeys: false });
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
function validateHostProperties(value: unknown): value is HostPropertiesWire {
  if (!Array.isArray(value)) return false;
  if (value[0] === 1) {
    if (value.length !== 11 || typeof value[1] !== "string" || (value[2] !== null && typeof value[2] !== "string")) return false;
    if (typeof value[3] !== "boolean" || typeof value[4] !== "boolean" || typeof value[5] !== "boolean") return false;
    for (const [index, name] of [[6, "ackEditSeq"], [7, "selectionStart"], [8, "selectionEnd"]] as const) {
      try { assertU32(name, value[index]); } catch { return false; }
    }
    if ((value[9] !== null && typeof value[9] !== "number") || (value[10] !== null && typeof value[10] !== "number")) return false;
    if (value[9] !== null && (value[10] === null || !Number.isInteger(value[9]) || !Number.isInteger(value[10]) || value[9] < 0 || value[10] < value[9] || value[10] > 0xffff_ffff)) return false;
    return true;
  }
  if (value[0] === 2) {
    if (value.length !== 6) return false;
    try {
      assertU32("itemCount", value[1]);
      assertU32("rangeStart", value[2]);
      assertU32("rangeEnd", value[3]);
      assertU32("overscan", value[5]);
    } catch { return false; }
    return typeof value[4] === "number" && Number.isFinite(value[4]) && value[4] > 0 && value[2] <= value[3] && value[3] <= value[1];
  }
  return false;
}
function validateEventPayload(eventType: number, payload: unknown): payload is EventPayload | null {
  if (eventType === EVENT_PRESS) return payload === null;
  if (!Array.isArray(payload)) return false;
  if (eventType === EVENT_COMMAND_RESULT) {
    if (payload.length !== 6 || payload[0] !== 2) return false;
    try { assertU32("requestId", payload[1]); assertU32("command", payload[2]); assertU32("nodeId", payload[3]); } catch { return false; }
    if (!([COMMAND_FOCUS, COMMAND_BLUR, COMMAND_SET_SELECTION, COMMAND_SCROLL_TO_INDEX, COMMAND_SCROLL_TO_END] as readonly number[]).includes(payload[2] as number)) return false;
    return typeof payload[4] === "boolean" && (payload[5] === null || typeof payload[5] === "string");
  }
  if (eventType === EVENT_VISIBLE_RANGE) {
    if (payload.length !== 3 || payload[0] !== 3) return false;
    try { assertU32("rangeStart", payload[1]); assertU32("rangeEnd", payload[2]); } catch { return false; }
    return payload[1] <= payload[2];
  }
  if (eventType === EVENT_ANIMATION_COMPLETE) {
    if (payload.length !== 2 || payload[0] !== 4) return false;
    try { assertU32("generation", payload[1]); } catch { return false; }
    return true;
  }
  if (eventType < EVENT_CHANGE || eventType > EVENT_BLUR) return false;
  if (payload.length !== 7 || payload[0] !== 1 || typeof payload[1] !== "string") return false;
  try { assertU32("selectionStart", payload[2]); assertU32("selectionEnd", payload[3]); assertU32("editSeq", payload[6]); } catch { return false; }
  if (payload[2] > payload[3] || (payload[4] === null) !== (payload[5] === null)) return false;
  if (payload[4] !== null && (!Number.isInteger(payload[4]) || !Number.isInteger(payload[5]) || payload[4] < 0 || payload[5] < payload[4] || payload[5] > 0xffff_ffff)) return false;
  return true;
}
export function decodeEvent(payload: Uint8Array): PressEventFrame | null {
  const value = decodeWire(payload);
  if (!Array.isArray(value) || value.length !== 10 || value[0] !== PROTOCOL_VERSION || value[1] !== EVENT_KIND) return null;
  for (const [index, name] of [[2, "surfaceId"], [3, "epoch"], [4, "revision"], [5, "sequence"], [6, "nodeId"], [7, "listenerId"]] as const) {
    try { assertU32(name, value[index]); } catch { return null; }
  }
  if (typeof value[8] !== "number" || !Number.isInteger(value[8]) || value[8] < EVENT_PRESS || value[8] > EVENT_ANIMATION_COMPLETE) return null;
  return validateEventPayload(value[8], value[9]) ? value as unknown as PressEventFrame : null;
}
