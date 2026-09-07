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
 * Incremental decoder for the little-endian u32 length-prefixed frame stream.
 * Complete frames borrow the caller's chunk when possible; consume them before
 * mutating a retained input chunk. Only an incomplete frame is buffered.
 */
export class FrameDecoder {
  private buffer = new Uint8Array(4);
  private bufferedBytes = 0;

  constructor(private readonly maxFrameSize = MAX_FRAME_SIZE) {
    if (!Number.isInteger(maxFrameSize) || maxFrameSize < 1) throw new RangeError("maxFrameSize must be positive");
  }

  push(chunk: FrameChunk): Uint8Array[] {
    const incoming = bytesFrom(chunk);
    const payloads: Uint8Array[] = [];
    const view = new DataView(incoming.buffer, incoming.byteOffset, incoming.byteLength);
    let offset = 0;
    while (offset < incoming.byteLength) {
      if (this.bufferedBytes > 0) {
        // Finish the header before accepting body bytes: its length bounds all
        // retained allocation, even when one chunk contains many more frames.
        if (this.bufferedBytes < 4) {
          const count = Math.min(4 - this.bufferedBytes, incoming.byteLength - offset);
          this.buffer.set(incoming.subarray(offset, offset + count), this.bufferedBytes);
          this.bufferedBytes += count;
          offset += count;
          if (this.bufferedBytes < 4) break;
        }
        const size = new DataView(this.buffer.buffer).getUint32(0, true);
        this.checkSize(size);
        const count = Math.min(4 + size - this.bufferedBytes, incoming.byteLength - offset);
        this.ensureCapacity(this.bufferedBytes + count);
        this.buffer.set(incoming.subarray(offset, offset + count), this.bufferedBytes);
        this.bufferedBytes += count;
        offset += count;
        if (this.bufferedBytes < 4 + size) break;
        // Buffered payloads must own their bytes before the next partial frame
        // reuses this buffer; complete input frames stay on the zero-copy path.
        payloads.push(this.buffer.slice(4, 4 + size));
        this.bufferedBytes = 0;
        continue;
      }
      if (incoming.byteLength - offset >= 4) {
        const size = view.getUint32(offset, true);
        this.checkSize(size);
        if (incoming.byteLength - offset - 4 >= size) {
          payloads.push(incoming.subarray(offset + 4, offset + 4 + size));
          offset += 4 + size;
          continue;
        }
      }
      const remainder = incoming.subarray(offset);
      this.ensureCapacity(remainder.byteLength);
      this.buffer.set(remainder);
      this.bufferedBytes = remainder.byteLength;
      break;
    }
    return payloads;
  }

  private checkSize(size: number): void {
    if (size <= this.maxFrameSize) return;
    this.bufferedBytes = 0;
    throw new RangeError(`frame length ${size} exceeds maximum ${this.maxFrameSize}`);
  }

  private ensureCapacity(required: number): void {
    if (required <= this.buffer.byteLength) return;
    const capacity = Math.min(Math.max(this.buffer.byteLength * 2, required), this.maxFrameSize + 4);
    const grown = new Uint8Array(capacity);
    grown.set(this.buffer.subarray(0, this.bufferedBytes));
    this.buffer = grown;
  }
}
