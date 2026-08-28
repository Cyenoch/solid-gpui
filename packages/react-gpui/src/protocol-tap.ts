import { closeSync, openSync, writeSync } from "node:fs";
import { MAX_FRAME_SIZE } from "./protocol";

export const PROTOCOL_TAP_CAPACITY_BYTES = 64 * 1024 * 1024;
const TAP_STOP_RESERVE_BYTES = 256;

interface Classification {
  readonly kind: "snapshot" | "patch" | "event" | "command" | "unknown";
  readonly event_type?: number;
  readonly command_kind?: number;
  readonly request_id?: number;
  readonly success?: boolean;
}

type Direction = "in" | "out";
type Peer = "host" | "renderer";

/** Process-local JSONL observer for framed protocol traffic. */
export class ProtocolTap {
  private readonly file: number;
  private readonly started = performance.now();
  private bytesWritten = 0;
  private sequence = 1;
  private stopped = false;
  private closed = false;
  private inboundBuffer = new Uint8Array(0);

  private constructor(
    private readonly capacityBytes: number,
    path: string,
  ) {
    try {
      this.file = openSync(path, "w", 0o600);
    } catch (error) {
      console.error(`react-gpui: protocol tap disabled; cannot open ${path}: ${String(error)}`);
      throw error;
    }
  }

  static fromEnv(): ProtocolTap | undefined {
    const path = process.env.REACT_GPUI_TAP;
    if (path === undefined || path === "") return undefined;
    try {
      return new ProtocolTap(PROTOCOL_TAP_CAPACITY_BYTES, path);
    } catch {
      return undefined;
    }
  }

  static openForTest(path: string, capacityBytes: number): ProtocolTap {
    if (!Number.isSafeInteger(capacityBytes) || capacityBytes < 1) {
      throw new RangeError("protocol tap capacity must be a positive integer");
    }
    return new ProtocolTap(capacityBytes, path);
  }

  get enabled(): boolean {
    return !this.stopped;
  }

  recordOutboundFrame(frame: Uint8Array): void {
    if (this.stopped) return;
    this.record("out", "host", frame.subarray(4), frame.byteLength);
  }

  observeInbound(chunk: Uint8Array): void {
    if (this.stopped || chunk.byteLength === 0) return;
    const combined = new Uint8Array(this.inboundBuffer.byteLength + chunk.byteLength);
    combined.set(this.inboundBuffer);
    combined.set(chunk, this.inboundBuffer.byteLength);
    this.inboundBuffer = combined;
    while (this.inboundBuffer.byteLength >= 4 && !this.stopped) {
      const size = new DataView(this.inboundBuffer.buffer, this.inboundBuffer.byteOffset, 4).getUint32(0, true);
      if (size > MAX_FRAME_SIZE) {
        this.record("in", "host", new Uint8Array(0), 4, { kind: "unknown" });
        this.inboundBuffer = new Uint8Array(0);
        return;
      }
      const frameSize = size + 4;
      if (this.inboundBuffer.byteLength < frameSize) return;
      const frame = this.inboundBuffer.subarray(0, frameSize);
      this.record("in", "host", frame.subarray(4), frame.byteLength);
      this.inboundBuffer =
        frameSize === this.inboundBuffer.byteLength ? new Uint8Array(0) : this.inboundBuffer.subarray(frameSize);
    }
  }

  dispose(): void {
    if (this.closed) return;
    closeSync(this.file);
    this.closed = true;
    this.stopped = true;
    this.inboundBuffer = new Uint8Array(0);
  }

  private record(
    direction: Direction,
    peer: Peer,
    payload: Uint8Array,
    frameBytes: number,
    override?: Classification,
  ): void {
    if (this.stopped) return;
    const classification = override ?? classify(payload);
    const timestamp = Math.floor(performance.now() - this.started);
    const line =
      JSON.stringify({
        t: timestamp,
        dir: direction,
        peer,
        kind: classification.kind,
        bytes: frameBytes,
        seq: this.sequence,
        ...(classification.event_type === undefined ? {} : { event_type: classification.event_type }),
        ...(classification.command_kind === undefined ? {} : { command_kind: classification.command_kind }),
        ...(classification.request_id === undefined ? {} : { request_id: classification.request_id }),
        ...(classification.success === undefined ? {} : { success: classification.success }),
      }) + "\n";
    if (this.bytesWritten + line.length > this.capacityBytes - TAP_STOP_RESERVE_BYTES) {
      this.writeStopped(direction, peer, frameBytes, timestamp);
      return;
    }
    try {
      writeSync(this.file, line, undefined, "utf8");
      this.bytesWritten += line.length;
      this.sequence += 1;
    } catch {
      this.stopped = true;
    }
  }

  private writeStopped(direction: Direction, peer: Peer, frameBytes: number, timestamp: number): void {
    const line =
      JSON.stringify({
        t: timestamp,
        dir: direction,
        peer,
        kind: "tap_stopped",
        bytes: frameBytes,
        seq: this.sequence,
        reason: "capacity",
      }) + "\n";
    if (this.bytesWritten + line.length <= this.capacityBytes) {
      try {
        writeSync(this.file, line, undefined, "utf8");
        this.bytesWritten += line.length;
        this.sequence += 1;
      } catch {
        // A tap failure is deliberately isolated from transport operation.
      }
    }
    this.stopped = true;
  }
}

function classify(payload: Uint8Array): Classification {
  const reader = new PrefixReader(payload);
  const length = reader.readArrayLength();
  if (length === undefined || length < 2 || reader.readUnsigned() === undefined) return { kind: "unknown" };
  const message = reader.readUnsigned();
  if (message === undefined) return { kind: "unknown" };
  if (message === 1) return { kind: "snapshot" };
  if (message === 3) return { kind: "patch" };
  if (message === 2) return classifyEvent(reader, length);
  if (message === 4) return classifyCommand(reader, length);
  return { kind: "unknown" };
}

function classifyEvent(reader: PrefixReader, length: number): Classification {
  if (length < 9) return { kind: "unknown" };
  for (let index = 2; index <= 8; index += 1) {
    if (index === 8) {
      const eventType = reader.readUnsigned();
      if (eventType === undefined) return { kind: "unknown" };
      const result: Classification = { kind: "event", event_type: eventType };
      if (eventType !== 6 || length < 10) return result;
      return parseCommandResult(reader, result);
    }
    if (!reader.skipValue()) return { kind: "unknown" };
  }
  return { kind: "unknown" };
}

function parseCommandResult(reader: PrefixReader, result: Classification): Classification {
  const length = reader.readArrayLength();
  if (length === undefined || length < 5 || reader.readUnsigned() !== 2) return result;
  const requestId = reader.readUnsigned();
  if (requestId === undefined || !reader.skipValue() || !reader.skipValue()) return result;
  const success = reader.readBoolean();
  return success === undefined ? { ...result, request_id: requestId } : { ...result, request_id: requestId, success };
}

function classifyCommand(reader: PrefixReader, length: number): Classification {
  if (length < 8) return { kind: "unknown" };
  let requestId: number | undefined;
  let commandKind: number | undefined;
  for (let index = 2; index <= 7; index += 1) {
    if (index === 5) requestId = reader.readUnsigned();
    else if (index === 7) commandKind = reader.readUnsigned();
    else if (!reader.skipValue()) return { kind: "unknown" };
    if (index === 5 && requestId === undefined) return { kind: "unknown" };
    if (index === 7 && commandKind === undefined) return { kind: "unknown" };
  }
  return { kind: "command", command_kind: commandKind, request_id: requestId };
}

class PrefixReader {
  private offset = 0;

  constructor(private readonly bytes: Uint8Array) {}

  readArrayLength(): number | undefined {
    const tag = this.readByte();
    if (tag === undefined) return undefined;
    if (tag >= 0x90 && tag <= 0x9f) return tag & 0x0f;
    if (tag === 0xdc) return this.readUint16();
    if (tag === 0xdd) return this.readUint32();
    return undefined;
  }

  readUnsigned(): number | undefined {
    const tag = this.readByte();
    if (tag === undefined) return undefined;
    if (tag <= 0x7f) return tag;
    if (tag === 0xcc) return this.readByte();
    if (tag === 0xcd) return this.readUint16();
    if (tag === 0xce) return this.readUint32();
    if (tag === 0xcf) return this.readUint64();
    if (tag === 0xd0) return this.readSigned(1);
    if (tag === 0xd1) return this.readSigned(2);
    if (tag === 0xd2) return this.readSigned(4);
    if (tag === 0xd3) return this.readSigned(8);
    return undefined;
  }

  readBoolean(): boolean | undefined {
    const tag = this.readByte();
    return tag === 0xc2 ? false : tag === 0xc3 ? true : undefined;
  }

  skipValue(): boolean {
    return this.skipValueDepth(0);
  }

  private readByte(): number | undefined {
    const value = this.bytes[this.offset];
    if (value === undefined) return undefined;
    this.offset += 1;
    return value;
  }

  private take(length: number): boolean {
    if (!Number.isSafeInteger(length) || length < 0 || this.offset + length > this.bytes.byteLength) return false;
    this.offset += length;
    return true;
  }

  private readUint16(): number | undefined {
    const a = this.readByte();
    const b = this.readByte();
    return a === undefined || b === undefined ? undefined : (a << 8) | b;
  }

  private readUint32(): number | undefined {
    const a = this.readByte();
    const b = this.readByte();
    const c = this.readByte();
    const d = this.readByte();
    return a === undefined || b === undefined || c === undefined || d === undefined
      ? undefined
      : a * 0x1000000 + (b << 16) + (c << 8) + d;
  }

  private readUint64(): number | undefined {
    const high = this.readUint32();
    const low = this.readUint32();
    if (high === undefined || low === undefined) return undefined;
    const value = high * 0x1_0000_0000 + low;
    return Number.isSafeInteger(value) ? value : undefined;
  }

  private readSigned(width: number): number | undefined {
    const bytes =
      this.offset + width <= this.bytes.byteLength ? this.bytes.subarray(this.offset, this.offset + width) : undefined;
    if (bytes === undefined) return undefined;
    this.offset += width;
    if (width === 1) return new DataView(bytes.buffer, bytes.byteOffset, 1).getInt8(0);
    if (width === 2) return new DataView(bytes.buffer, bytes.byteOffset, 2).getInt16(0, false);
    if (width === 4) return new DataView(bytes.buffer, bytes.byteOffset, 4).getInt32(0, false);
    const value = new DataView(bytes.buffer, bytes.byteOffset, 8).getBigInt64(0, false);
    const number = Number(value);
    return Number.isSafeInteger(number) ? number : undefined;
  }

  private skipValueDepth(depth: number): boolean {
    if (depth > 64) return false;
    const tag = this.readByte();
    if (tag === undefined) return false;
    if (tag <= 0x7f || tag >= 0xe0 || tag === 0xc0 || tag === 0xc2 || tag === 0xc3) return true;
    if (tag >= 0xa0 && tag <= 0xbf) return this.take(tag & 0x1f);
    if (tag >= 0x90 && tag <= 0x9f) return this.skipMany(tag & 0x0f, depth);
    if (tag >= 0x80 && tag <= 0x8f) return this.skipMany((tag & 0x0f) * 2, depth);
    if (tag === 0xc4 || tag === 0xd9) return this.skipSized(1);
    if (tag === 0xc5 || tag === 0xda) return this.skipSized(2);
    if (tag === 0xc6 || tag === 0xdb) return this.skipSized(4);
    if (tag === 0xca || tag === 0xd2) return this.take(4);
    if (tag === 0xcb || tag === 0xd3) return this.take(8);
    if (tag === 0xcc || tag === 0xd0) return this.take(1);
    if (tag === 0xcd || tag === 0xd1) return this.take(2);
    if (tag === 0xce || tag === 0xd2) return this.take(4);
    if (tag === 0xcf) return this.take(8);
    if (tag === 0xdc) return this.skipMany(this.readUint16() ?? -1, depth);
    if (tag === 0xdd) return this.skipMany(this.readUint32() ?? -1, depth);
    if (tag === 0xde) return this.skipMany((this.readUint16() ?? -1) * 2, depth);
    if (tag === 0xdf) return this.skipMany((this.readUint32() ?? -1) * 2, depth);
    if (tag === 0xc7) return this.skipExt(1);
    if (tag === 0xc8) return this.skipExt(2);
    if (tag === 0xc9) return this.skipExt(4);
    if (tag >= 0xd4 && tag <= 0xd8) return this.take([2, 3, 5, 9, 17][tag - 0xd4]);
    return false;
  }

  private skipMany(count: number, depth: number): boolean {
    if (!Number.isSafeInteger(count) || count < 0) return false;
    for (let index = 0; index < count; index += 1) if (!this.skipValueDepth(depth + 1)) return false;
    return true;
  }

  private skipSized(width: number): boolean {
    const length = width === 1 ? this.readByte() : width === 2 ? this.readUint16() : this.readUint32();
    return length !== undefined && this.take(length);
  }

  private skipExt(width: number): boolean {
    const length = width === 1 ? this.readByte() : width === 2 ? this.readUint16() : this.readUint32();
    return length !== undefined && this.take(length + 1);
  }
}
