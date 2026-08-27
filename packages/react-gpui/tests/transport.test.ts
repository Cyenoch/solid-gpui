import { describe, expect, it } from "bun:test";
import { createRoot } from "../src/renderer";
import {
  DEFAULT_MAX_PENDING_BYTES,
  StdioTransport,
  TransportTerminatedError,
  createProcessTerminationHandler,
  type ByteInput,
  type ByteInputEventListener,
  type ByteInputListener,
  type ByteOutput,
  type ByteOutputEventListener,
  type TransportChunk,
} from "../src/transport";

class FakeInput implements ByteInput {
  private listener: ByteInputListener | undefined;
  private endListener: (() => void) | undefined;
  private closeListener: (() => void) | undefined;
  private errorListener: ((error: unknown) => void) | undefined;

  on(event: "data" | "end" | "close" | "error", listener: ByteInputEventListener): void {
    if (event === "data") this.listener = listener as ByteInputListener;
    else if (event === "end") this.endListener = listener as () => void;
    else if (event === "close") this.closeListener = listener as () => void;
    else this.errorListener = listener as (error: unknown) => void;
  }

  off(event: "data" | "end" | "close" | "error", listener: ByteInputEventListener): void {
    if (event === "data" && this.listener === listener) this.listener = undefined;
    if (event === "end" && this.endListener === listener) this.endListener = undefined;
    if (event === "close" && this.closeListener === listener) this.closeListener = undefined;
    if (event === "error" && this.errorListener === listener) this.errorListener = undefined;
  }

  emit(chunk: Uint8Array): void {
    this.listener?.(chunk);
  }

  emitEnd(): void {
    this.endListener?.();
  }

  emitClose(): void {
    this.closeListener?.();
  }

  emitError(error: unknown): void {
    this.errorListener?.(error);
  }

  listenerCount(): number {
    return this.listener === undefined ? 0 : 1;
  }
}

class FakeOutput implements ByteOutput {
  readonly writes: Uint8Array[] = [];
  private readonly results: boolean[];
  private drainListener: (() => void) | undefined;
  private closeListener: (() => void) | undefined;
  private errorListener: ((error: unknown) => void) | undefined;
  private writeError: unknown;

  constructor(results: boolean[]) {
    this.results = [...results];
    this.writeError = undefined;
  }

  write(frame: Uint8Array): boolean {
    if (this.writeError !== undefined) throw this.writeError;
    this.writes.push(frame.slice());
    return this.results.shift() ?? true;
  }

  on(event: "drain" | "error" | "close", listener: ByteOutputEventListener): void {
    if (event === "drain") this.drainListener = listener as () => void;
    else if (event === "close") this.closeListener = listener as () => void;
    else this.errorListener = listener as (error: unknown) => void;
  }

  off(event: "drain" | "error" | "close", listener: ByteOutputEventListener): void {
    if (event === "drain" && this.drainListener === listener) this.drainListener = undefined;
    if (event === "close" && this.closeListener === listener) this.closeListener = undefined;
    if (event === "error" && this.errorListener === listener) this.errorListener = undefined;
  }

  emitDrain(): void {
    this.drainListener?.();
  }

  emitClose(): void {
    this.closeListener?.();
  }

  emitError(error: unknown): void {
    this.errorListener?.(error);
  }

  failWrites(error: unknown): void {
    this.writeError = error;
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
    expect(() => transport.submit(frame(2))).toThrow(`pending output queue exceeds ${DEFAULT_MAX_PENDING_BYTES} bytes`);
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

  it("notifies once and exits through an injected handler when input closes", () => {
    const input = new FakeInput();
    const output = new FakeOutput([]);
    const transport = new StdioTransport(output, input);
    const exits: number[] = [];
    const errors: TransportTerminatedError[] = [];
    transport.onTermination((error) => errors.push(error));
    transport.onTermination(createProcessTerminationHandler((code) => exits.push(code)));

    input.emitClose();
    input.emitEnd();
    output.emitClose();

    expect(errors).toHaveLength(1);
    expect(errors[0]?.cause).toEqual({ kind: "eof" });
    expect(exits).toEqual([1]);
    expect(() => transport.submit(frame(1))).toThrow("StdioTransport input closed");
  });

  it("terminates on a synchronous EPIPE write failure and rejects later submits", () => {
    const input = new FakeInput();
    const output = new FakeOutput([]);
    const transport = new StdioTransport(output, input);
    const errors: TransportTerminatedError[] = [];
    transport.onTermination((error) => errors.push(error));
    output.failWrites(new Error("EPIPE"));

    expect(() => transport.submit(frame(1))).toThrow("StdioTransport output write failed: EPIPE");
    expect(errors).toHaveLength(1);
    expect(errors[0]?.cause).toEqual({ kind: "io", detail: "EPIPE" });
    expect(input.listenerCount()).toBe(0);
    expect(output.listenerCount()).toBe(0);
  });

  it("preserves host exit code and only the last 50 stderr lines", () => {
    const input = new FakeInput();
    const output = new FakeOutput([]);
    const transport = new StdioTransport(output, input);
    let termination: TransportTerminatedError | undefined;
    transport.onTermination((error) => {
      termination = error;
    });
    const stderrTail = `${Array.from({ length: 59 }, (_, index) => `line-${index}`).join("\n")}\nreact-gpui-host: crash report: /tmp/react-gpui-host-23-456.log`;
    input.emitError(Object.assign(new Error("host exited"), { exitCode: 23, stderrTail }));

    expect(termination?.message).toContain("host exit code: 23");
    expect(termination?.message).toContain("line-58");
    expect(termination?.message).toContain("line-10");
    expect(termination?.message).not.toContain("line-9");
    expect(termination?.exitCode).toBe(23);
    expect(termination?.crashReportPath).toBe("/tmp/react-gpui-host-23-456.log");
    expect(termination?.cause).toEqual({ kind: "exit", code: 23 });
  });

  it("delivers transport termination through createRoot without a global exit", () => {
    const input = new FakeInput();
    const output = new FakeOutput([]);
    const transport = new StdioTransport(output, input);
    const errors: string[] = [];
    const root = createRoot(transport, {
      onTransportTermination: (error) => errors.push(error.message),
    });

    root.render(null);
    input.emitEnd();
    input.emitClose();

    expect(errors).toHaveLength(1);
    root.unmount();
  });

  it("keeps a pending overflow as RangeError before a later termination", () => {
    const input = new FakeInput();
    const output = new FakeOutput([false]);
    const transport = new StdioTransport(output, input, { maxPendingBytes: 3 });
    const errors: string[] = [];
    transport.onTermination((error) => errors.push(error.message));

    transport.submit(frame(1));
    transport.submit(frame(2));
    transport.submit(frame(3));
    transport.submit(frame(4));
    expect(() => transport.submit(frame(5))).toThrow("pending output queue exceeds 3 bytes");
    expect(errors).toHaveLength(0);

    input.emitEnd();

    expect(errors).toHaveLength(1);
    expect(() => transport.submit(frame(6))).toThrow("StdioTransport input ended");
  });
});
