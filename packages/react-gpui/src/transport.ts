import { MAX_FRAME_SIZE } from "./protocol";
export type TransportChunk = Uint8Array | ArrayBuffer;
export type TransportListener = (chunk: Uint8Array) => void;

export interface Transport {
  submit(frame: Uint8Array): void;
  onData(listener: TransportListener): () => void;
}

export type ByteInputListener = (chunk: TransportChunk) => void;

export interface ByteInput {
  on(event: "data", listener: ByteInputListener): unknown;
  off?: (event: "data", listener: ByteInputListener) => unknown;
  removeListener?: (event: "data", listener: ByteInputListener) => unknown;
}

export type DrainListener = () => void;

export interface ByteOutput {
  write(frame: Uint8Array): boolean;
  on(event: "drain", listener: DrainListener): unknown;
  off?: (event: "drain", listener: DrainListener) => unknown;
  removeListener?: (event: "drain", listener: DrainListener) => unknown;
}

export interface StdioTransportOptions {
  readonly maxPendingBytes?: number;
}

export const DEFAULT_MAX_PENDING_BYTES = MAX_FRAME_SIZE + 4;

function asBytes(chunk: TransportChunk): Uint8Array {
  return chunk instanceof Uint8Array ? chunk : new Uint8Array(chunk);
}

export class StdioTransport implements Transport {
  private readonly listeners = new Set<TransportListener>();
  private readonly inputListener: ByteInputListener;
  private readonly drainListener: DrainListener;
  private readonly maxPendingBytes: number;
  private pendingFrames: Array<Uint8Array | undefined> = [];
  private pendingHead = 0;
  private pendingBytes = 0;
  private backpressured = false;
  private disposed = false;

  constructor(
    private readonly output: ByteOutput = getDefaultOutput(),
    private readonly input: ByteInput = getDefaultInput(),
    options: StdioTransportOptions = {},
  ) {
    const maxPendingBytes = options.maxPendingBytes ?? DEFAULT_MAX_PENDING_BYTES;
    if (!Number.isInteger(maxPendingBytes) || maxPendingBytes < 1) {
      throw new RangeError("StdioTransport maxPendingBytes must be a positive integer");
    }
    this.maxPendingBytes = maxPendingBytes;
    this.inputListener = (chunk) => {
      const bytes = asBytes(chunk);
      for (const listener of this.listeners) listener(bytes);
    };
    this.drainListener = () => this.flushPending();
    input.on("data", this.inputListener);
    output.on("drain", this.drainListener);
  }

  submit(frame: Uint8Array): void {
    if (this.disposed) throw new Error("StdioTransport is disposed");
    if (this.backpressured || this.pendingHead < this.pendingFrames.length) {
      this.enqueue(frame);
      return;
    }
    if (!this.output.write(frame)) this.backpressured = true;
  }

  onData(listener: TransportListener): () => void {
    if (this.disposed) throw new Error("StdioTransport is disposed");
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  dispose(): void {
    if (this.disposed) return;
    this.disposed = true;
    detachInputListener(this.input, this.inputListener);
    detachDrainListener(this.output, this.drainListener);
    this.listeners.clear();
    this.pendingFrames = [];
    this.pendingHead = 0;
    this.pendingBytes = 0;
    this.backpressured = false;
  }

  private enqueue(frame: Uint8Array): void {
    const frameBytes = frame.byteLength;
    if (this.pendingBytes + frameBytes > this.maxPendingBytes) {
      throw new RangeError(
        `StdioTransport pending output queue exceeds ${this.maxPendingBytes} bytes`,
      );
    }
    this.pendingFrames.push(frame.slice());
    this.pendingBytes += frameBytes;
  }

  private flushPending(): void {
    if (this.disposed) return;
    this.backpressured = false;
    while (this.pendingHead < this.pendingFrames.length) {
      const frame = this.pendingFrames[this.pendingHead];
      this.pendingFrames[this.pendingHead] = undefined;
      this.pendingHead += 1;
      if (frame === undefined) continue;
      this.pendingBytes -= frame.byteLength;
      if (!this.output.write(frame)) {
        this.backpressured = true;
        this.compactPending();
        return;
      }
    }
    this.pendingFrames = [];
    this.pendingHead = 0;
    this.pendingBytes = 0;
  }

  private compactPending(): void {
    if (this.pendingHead > 64 && this.pendingHead * 2 >= this.pendingFrames.length) {
      this.pendingFrames = this.pendingFrames.slice(this.pendingHead);
      this.pendingHead = 0;
    }
  }
}

export class MemoryTransport implements Transport {
  readonly submitted: Uint8Array[] = [];
  private readonly listeners = new Set<TransportListener>();

  submit(frame: Uint8Array): void {
    this.submitted.push(frame.slice());
  }

  onData(listener: TransportListener): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  push(chunk: TransportChunk): void {
    const bytes = asBytes(chunk);
    for (const listener of this.listeners) listener(bytes);
  }
}
function detachInputListener(input: ByteInput, listener: ByteInputListener): void {
  if (input.off) input.off("data", listener);
  else input.removeListener?.("data", listener);
}

function detachDrainListener(output: ByteOutput, listener: DrainListener): void {
  if (output.off) output.off("drain", listener);
  else output.removeListener?.("drain", listener);
}

function getDefaultInput(): ByteInput {
  const input = (globalThis as { process?: { stdin?: ByteInput } }).process?.stdin;
  if (!input) throw new Error("StdioTransport requires a readable stdin");
  return input;
}

function getDefaultOutput(): ByteOutput {
  const output = (globalThis as { process?: { stdout?: ByteOutput } }).process?.stdout;
  if (!output) throw new Error("StdioTransport requires a writable stdout");
  return output;
}
