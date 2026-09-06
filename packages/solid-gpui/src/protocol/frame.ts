import { MAX_FRAME_SIZE } from "./constants";

export type FrameChunk = Uint8Array | ArrayBuffer;

export function bytesFrom(value: FrameChunk): Uint8Array {
  return value instanceof Uint8Array ? value : new Uint8Array(value);
}

export function framePayload(payload: Uint8Array): Uint8Array {
  if (payload.byteLength > MAX_FRAME_SIZE) throw new RangeError("frame exceeds maximum size");
  const frame = new Uint8Array(4 + payload.byteLength);
  new DataView(frame.buffer).setUint32(0, payload.byteLength, true);
  frame.set(payload, 4);
  return frame;
}

/**
 * Incremental decoder for the little-endian u32 length-prefixed v4 frame stream.
 * Complete frames borrow the caller's chunk when possible; consume them before
 * mutating a retained input chunk.
 */
export class FrameDecoder {
  private buffer = new Uint8Array(4);
  private readOffset = 0;
  private writeOffset = 0;

  constructor(private readonly maxFrameSize = MAX_FRAME_SIZE) {
    if (!Number.isInteger(maxFrameSize) || maxFrameSize < 1) throw new RangeError("maxFrameSize must be positive");
  }

  push(chunk: FrameChunk): Uint8Array[] {
    const incoming = bytesFrom(chunk);
    if (incoming.byteLength === 0) return [];
    if (this.readOffset !== this.writeOffset) {
      const unreadLength = this.writeOffset - this.readOffset;
      let needed = 4 - unreadLength;
      if (unreadLength >= 4) {
        const view = new DataView(this.buffer.buffer, this.buffer.byteOffset, this.buffer.byteLength);
        const size = view.getUint32(this.readOffset, true);
        if (size > this.maxFrameSize) {
          this.readOffset = 0;
          this.writeOffset = 0;
          throw new RangeError(`frame length ${size} exceeds maximum ${this.maxFrameSize}`);
        }
        needed = 4 + size - unreadLength;
      }
      if (incoming.byteLength > needed) {
        const payloads = this.push(incoming.subarray(0, needed));
        payloads.push(...this.push(incoming.subarray(needed)));
        return payloads;
      }
    }
    if (this.readOffset === this.writeOffset) {
      const payloads: Uint8Array[] = [];
      const view = new DataView(incoming.buffer, incoming.byteOffset, incoming.byteLength);
      let offset = 0;
      while (incoming.byteLength - offset >= 4) {
        const size = view.getUint32(offset, true);
        if (size > this.maxFrameSize) throw new RangeError(`frame length ${size} exceeds maximum ${this.maxFrameSize}`);
        if (incoming.byteLength - offset - 4 < size) break;
        payloads.push(incoming.subarray(offset + 4, offset + 4 + size));
        offset += 4 + size;
      }
      if (offset === incoming.byteLength) return payloads;
      const remainder = incoming.subarray(offset);
      this.ensureCapacity(remainder.byteLength);
      this.buffer.set(remainder, this.writeOffset);
      this.writeOffset += remainder.byteLength;
      return payloads;
    }
    this.ensureCapacity(incoming.byteLength);
    this.buffer.set(incoming, this.writeOffset);
    this.writeOffset += incoming.byteLength;
    const payloads: Uint8Array[] = [];
    const view = new DataView(this.buffer.buffer, this.buffer.byteOffset, this.buffer.byteLength);
    while (this.writeOffset - this.readOffset >= 4) {
      const size = view.getUint32(this.readOffset, true);
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
    const maxCapacity = this.maxFrameSize + 4;
    let capacity = this.buffer.byteLength;
    while (capacity < required) {
      if (capacity === maxCapacity) {
        throw new RangeError(`partial frame exceeds maximum retained size ${maxCapacity}`);
      }
      capacity = Math.min(capacity * 2, maxCapacity);
    }
    const grown = new Uint8Array(capacity);
    grown.set(this.buffer.subarray(this.readOffset, this.writeOffset));
    this.buffer = grown;
    this.readOffset = 0;
    this.writeOffset = unreadLength;
  }
}
