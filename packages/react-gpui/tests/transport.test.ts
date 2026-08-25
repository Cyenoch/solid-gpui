import { describe, expect, it } from "bun:test";
import {
  DEFAULT_MAX_PENDING_BYTES,
  StdioTransport,
  type ByteInput,
  type ByteOutput,
  type TransportChunk,
} from "../src/transport";

class FakeInput implements ByteInput {
  private listener: ((chunk: TransportChunk) => void) | undefined;

  on(event: "data", listener: (chunk: TransportChunk) => void): void {
    if (event === "data") this.listener = listener;
  }

  off(event: "data", listener: (chunk: TransportChunk) => void): void {
    if (event === "data" && this.listener === listener) this.listener = undefined;
  }

  emit(chunk: Uint8Array): void {
    this.listener?.(chunk);
  }

  listenerCount(): number {
    return this.listener === undefined ? 0 : 1;
  }
}

class FakeOutput implements ByteOutput {
  readonly writes: Uint8Array[] = [];
  private readonly results: boolean[];
  private drainListener: (() => void) | undefined;

  constructor(results: boolean[]) {
    this.results = [...results];
  }

  write(frame: Uint8Array): boolean {
    this.writes.push(frame.slice());
    return this.results.shift() ?? true;
  }

  on(event: "drain", listener: () => void): void {
    if (event === "drain") this.drainListener = listener;
  }

  off(event: "drain", listener: () => void): void {
    if (event === "drain" && this.drainListener === listener) this.drainListener = undefined;
  }

  emitDrain(): void {
    this.drainListener?.();
  }

  listenerCount(): number {
    return this.drainListener === undefined ? 0 : 1;
  }
}

function frame(value: number): Uint8Array {
  return new Uint8Array([value]);
}

function values(output: FakeOutput): number[] {
  return output.writes.map((bytes) => bytes[0]);
}

describe("StdioTransport backpressure", () => {
  it("queues after a false write and flushes accepted frames in order", () => {
    const input = new FakeInput();
    const output = new FakeOutput([false, true, true]);
    const transport = new StdioTransport(output, input, { maxPendingBytes: 16 });

    transport.submit(frame(1));
    transport.submit(frame(2));
    transport.submit(frame(3));
    expect(values(output)).toEqual([1]);

    output.emitDrain();
    expect(values(output)).toEqual([1, 2, 3]);
    transport.dispose();
  });

  it("stays queued across repeated backpressure signals without resending", () => {
    const input = new FakeInput();
    const output = new FakeOutput([false, false, true]);
    const transport = new StdioTransport(output, input, { maxPendingBytes: 16 });

    transport.submit(frame(1));
    transport.submit(frame(2));
    transport.submit(frame(3));
    output.emitDrain();
    expect(values(output)).toEqual([1, 2]);
    output.emitDrain();
    expect(values(output)).toEqual([1, 2, 3]);
    transport.dispose();
  });

  it("rejects a frame before queue growth exceeds the byte cap", () => {
    const input = new FakeInput();
    const output = new FakeOutput([false]);
    const transport = new StdioTransport(output, input, { maxPendingBytes: 3 });

    transport.submit(frame(1));
    transport.submit(frame(2));
    transport.submit(frame(3));
    transport.submit(frame(4));
    expect(() => transport.submit(frame(5))).toThrow("pending output queue exceeds 3 bytes");
    expect(values(output)).toEqual([1]);
    transport.dispose();
  });
  it("allows one maximum-sized framed payload at the default cap", () => {
    const input = new FakeInput();
    const output = new FakeOutput([false]);
    const transport = new StdioTransport(output, input);
    transport.submit(frame(1));
    transport.submit(new Uint8Array(DEFAULT_MAX_PENDING_BYTES));
    expect(() => transport.submit(frame(2))).toThrow(
      `pending output queue exceeds ${DEFAULT_MAX_PENDING_BYTES} bytes`,
    );
    transport.dispose();
  });

  it("detaches input and drain listeners and drops pending frames on dispose", () => {
    const input = new FakeInput();
    const output = new FakeOutput([false, true]);
    const transport = new StdioTransport(output, input, { maxPendingBytes: 16 });
    let received = 0;
    transport.onData(() => {
      received += 1;
    });

    transport.submit(frame(1));
    transport.submit(frame(2));
    expect(input.listenerCount()).toBe(1);
    expect(output.listenerCount()).toBe(1);

    transport.dispose();
    input.emit(frame(9));
    output.emitDrain();
    expect(received).toBe(0);
    expect(values(output)).toEqual([1]);
    expect(input.listenerCount()).toBe(0);
    expect(output.listenerCount()).toBe(0);
    expect(() => transport.submit(frame(3))).toThrow("StdioTransport is disposed");
  });
});
