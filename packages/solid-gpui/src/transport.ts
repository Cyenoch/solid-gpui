import { MAX_FRAME_SIZE, type Event } from "./protocol";
import { ProtocolTap } from "./protocol-tap";

export type TransportChunk = Uint8Array | ArrayBuffer;
export type TransportListener = (chunk: Uint8Array) => void;
export type SemanticEventListener = (event: Event) => void;
export type TransportTerminationCause =
  | { readonly kind: "shutdown" }
  | { readonly kind: "eof" }
  | { readonly kind: "exit"; readonly code: number }
  | { readonly kind: "protocol"; readonly detail: string }
  | { readonly kind: "io"; readonly detail: string };

function isTransportTerminationCause(value: unknown): value is TransportTerminationCause {
  if (value === null || typeof value !== "object") return false;
  const candidate = value as { kind?: unknown; code?: unknown; detail?: unknown };
  if (candidate.kind === "shutdown" || candidate.kind === "eof") return true;
  if (candidate.kind === "exit") return typeof candidate.code === "number" && Number.isInteger(candidate.code);
  return (
    (candidate.kind === "protocol" || candidate.kind === "io") &&
    typeof candidate.detail === "string" &&
    candidate.detail.length > 0
  );
}

export class TransportTerminatedError extends Error {
  readonly cause?: TransportTerminationCause;
  readonly exitCode?: number;
  readonly stderrTail?: string;
  readonly crashReportPath?: string;

  constructor(message: string, cause?: unknown, details: TransportTerminationDetails = {}) {
    super(message);
    this.name = "TransportTerminatedError";
    this.cause =
      cause === undefined
        ? undefined
        : isTransportTerminationCause(cause)
          ? cause
          : { kind: "io", detail: describeError(cause) };
    this.exitCode = details.exitCode;
    this.stderrTail = details.stderrTail;
    this.crashReportPath = details.crashReportPath;
  }
}

export interface TransportTerminationDetails {
  readonly exitCode?: number;
  readonly stderrTail?: string;
  readonly crashReportPath?: string;
}
export type TransportTerminationListener = (error: TransportTerminatedError) => void;

export interface Transport {
  submit(frame: Uint8Array): void;
  onData(listener: TransportListener): () => void;
  onTermination(listener: TransportTerminationListener): () => void;
}
export interface SemanticEventTransport extends Transport {
  onEvent(listener: SemanticEventListener): () => void;
}

export type ByteInputListener = (chunk: TransportChunk) => void;
export type TransportErrorListener = (error: unknown) => void;
export type TransportCloseListener = () => void;
export type ByteInputEventListener = ByteInputListener | TransportErrorListener | TransportCloseListener;

export interface ByteInput {
  on(event: "data" | "end" | "close" | "error", listener: ByteInputEventListener): unknown;
  off?: (event: "data" | "end" | "close" | "error", listener: ByteInputEventListener) => unknown;
  removeListener?: (event: "data" | "end" | "close" | "error", listener: ByteInputEventListener) => unknown;
}

export type DrainListener = () => void;
export type ByteOutputEventListener = DrainListener | TransportErrorListener | TransportCloseListener;

export interface ByteOutput {
  write(frame: Uint8Array): boolean;
  on(event: "drain" | "error" | "close", listener: ByteOutputEventListener): unknown;
  off?: (event: "drain" | "error" | "close", listener: ByteOutputEventListener) => unknown;
  removeListener?: (event: "drain" | "error" | "close", listener: ByteOutputEventListener) => unknown;
}

export interface StdioTransportOptions {
  readonly maxPendingBytes?: number;
}

export const DEFAULT_MAX_PENDING_BYTES = MAX_FRAME_SIZE + 4;

function asBytes(chunk: TransportChunk): Uint8Array {
  return chunk instanceof Uint8Array ? chunk : new Uint8Array(chunk);
}

function describeError(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

const CRASH_REPORT_PREFIX = "solid-gpui-host: crash report: ";

function crashReportPathFromStderr(stderrTail: string | undefined): string | undefined {
  if (stderrTail === undefined) return undefined;
  let path: string | undefined;
  for (const line of stderrTail.split(/\r?\n/)) {
    if (line.startsWith(CRASH_REPORT_PREFIX) && line.length > CRASH_REPORT_PREFIX.length)
      path = line.slice(CRASH_REPORT_PREFIX.length);
  }
  return path;
}

function detailsFromCause(cause: unknown): TransportTerminationDetails {
  if (cause === null || typeof cause !== "object") return {};
  const value = cause as { exitCode?: unknown; stderrTail?: unknown };
  const stderrTail =
    typeof value.stderrTail === "string" ? value.stderrTail.split(/\r?\n/).slice(-50).join("\n") : undefined;
  return {
    exitCode: typeof value.exitCode === "number" && Number.isInteger(value.exitCode) ? value.exitCode : undefined,
    stderrTail,
    crashReportPath: crashReportPathFromStderr(stderrTail),
  };
}

function terminatedError(
  context: string,
  cause?: unknown,
  terminationCause?: TransportTerminationCause,
): TransportTerminatedError {
  if (cause instanceof TransportTerminatedError) return cause;
  const details = detailsFromCause(cause);
  const message = cause === undefined ? context : `${context}: ${describeError(cause)}`;
  const diagnostics = [
    details.exitCode === undefined ? undefined : `host exit code: ${details.exitCode}`,
    details.stderrTail === undefined ? undefined : `host stderr tail:\n${details.stderrTail}`,
    details.crashReportPath === undefined ? undefined : `host crash report: ${details.crashReportPath}`,
  ].filter((value): value is string => value !== undefined);
  const typedCause: TransportTerminationCause =
    terminationCause ??
    (details.exitCode === undefined
      ? { kind: "io", detail: describeError(cause ?? context) }
      : { kind: "exit", code: details.exitCode });
  return new TransportTerminatedError(
    diagnostics.length === 0 ? message : `${message}\n${diagnostics.join("\n")}`,
    typedCause,
    details,
  );
}

export type ExitFunction = (code: number) => void;

export function createProcessTerminationHandler(exit: ExitFunction = defaultProcessExit): TransportTerminationListener {
  return (error) => {
    console.error(`solid-gpui: transport terminated: ${error.message}`);
    exit(1);
  };
}

function defaultProcessExit(code: number): void {
  const processObject = (globalThis as { process?: { exit?: ExitFunction } }).process;
  if (processObject?.exit) {
    processObject.exit(code);
    return;
  }
  throw new Error(`solid-gpui: transport terminated with exit code ${code}`);
}

export class StdioTransport implements Transport {
  private readonly listeners = new Set<TransportListener>();
  private readonly terminationListeners = new Set<TransportTerminationListener>();
  private readonly inputListener: ByteInputListener;
  private readonly inputEndListener: TransportCloseListener;
  private readonly inputCloseListener: TransportCloseListener;
  private readonly inputErrorListener: TransportErrorListener;
  private readonly drainListener: DrainListener;
  private readonly outputCloseListener: TransportCloseListener;
  private readonly outputErrorListener: TransportErrorListener;
  private readonly maxPendingBytes: number;
  private pendingFrames: Array<Uint8Array | undefined> = [];
  private pendingHead = 0;
  private pendingBytes = 0;
  private backpressured = false;
  private disposed = false;
  private terminated = false;
  private terminationError: TransportTerminatedError | undefined;
  private readonly tap: ProtocolTap | undefined;

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
    this.tap = ProtocolTap.fromEnv();
    this.inputListener = (chunk) => {
      if (this.disposed || this.terminated) return;
      const bytes = asBytes(chunk);
      this.tap?.observeInbound(bytes);
      for (const listener of this.listeners) listener(bytes);
    };
    this.inputEndListener = () =>
      this.terminate(terminatedError("StdioTransport input ended", undefined, { kind: "eof" }));
    this.inputCloseListener = () =>
      this.terminate(terminatedError("StdioTransport input closed", undefined, { kind: "eof" }));
    this.inputErrorListener = (error) => this.terminate(terminatedError("StdioTransport input failed", error));
    this.drainListener = () => this.flushPending();
    this.outputCloseListener = () =>
      this.terminate(terminatedError("StdioTransport output closed", undefined, { kind: "eof" }));
    this.outputErrorListener = (error) => this.terminate(terminatedError("StdioTransport output failed", error));
    input.on("data", this.inputListener);
    input.on("end", this.inputEndListener);
    input.on("close", this.inputCloseListener);
    input.on("error", this.inputErrorListener);
    output.on("drain", this.drainListener);
    output.on("close", this.outputCloseListener);
    output.on("error", this.outputErrorListener);
  }

  submit(frame: Uint8Array): void {
    if (this.terminated) throw this.terminationError;
    if (this.disposed) throw new Error("StdioTransport is disposed");
    if (this.backpressured || this.pendingHead < this.pendingFrames.length) {
      this.enqueue(frame);
      this.tap?.recordOutboundFrame(frame);
      return;
    }
    try {
      if (!this.output.write(frame)) this.backpressured = true;
      this.tap?.recordOutboundFrame(frame);
    } catch (error) {
      const failure = terminatedError("StdioTransport output write failed", error);
      this.terminate(failure);
      throw failure;
    }
  }

  onData(listener: TransportListener): () => void {
    if (this.terminated) throw this.terminationError;
    if (this.disposed) throw new Error("StdioTransport is disposed");
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  onTermination(listener: TransportTerminationListener): () => void {
    if (this.terminated) {
      listener(this.terminationError as TransportTerminatedError);
      return () => undefined;
    }
    if (this.disposed) return () => undefined;
    this.terminationListeners.add(listener);
    return () => this.terminationListeners.delete(listener);
  }

  dispose(): void {
    if (this.disposed) return;
    this.disposed = true;
    this.detachListeners();
    this.listeners.clear();
    this.terminationListeners.clear();
    this.pendingFrames = [];
    this.pendingHead = 0;
    this.pendingBytes = 0;
    this.backpressured = false;
    this.tap?.dispose();
  }

  private terminate(error: TransportTerminatedError): void {
    if (this.disposed || this.terminated) return;
    this.terminated = true;
    this.terminationError = error;
    this.detachListeners();
    this.listeners.clear();
    this.pendingFrames = [];
    this.pendingHead = 0;
    this.pendingBytes = 0;
    this.tap?.dispose();
    this.backpressured = false;
    const listeners = [...this.terminationListeners];
    this.terminationListeners.clear();
    for (const listener of listeners) listener(error);
  }

  private detachListeners(): void {
    detachInputListener(this.input, "data", this.inputListener);
    detachInputListener(this.input, "end", this.inputEndListener);
    detachInputListener(this.input, "close", this.inputCloseListener);
    detachInputListener(this.input, "error", this.inputErrorListener);
    detachOutputListener(this.output, "drain", this.drainListener);
    detachOutputListener(this.output, "close", this.outputCloseListener);
    detachOutputListener(this.output, "error", this.outputErrorListener);
  }

  private enqueue(frame: Uint8Array): void {
    const frameBytes = frame.byteLength;
    if (frameBytes > this.maxPendingBytes - this.pendingBytes) {
      throw new RangeError(`StdioTransport pending output queue exceeds ${this.maxPendingBytes} bytes`);
    }
    this.pendingFrames.push(frame.slice());
    this.pendingBytes += frameBytes;
  }

  private flushPending(): void {
    if (this.disposed || this.terminated) return;
    this.backpressured = false;
    try {
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
    } catch (error) {
      this.terminate(terminatedError("StdioTransport output write failed", error));
      return;
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

  onTermination(_listener: TransportTerminationListener): () => void {
    return () => undefined;
  }

  push(chunk: TransportChunk): void {
    const bytes = asBytes(chunk);
    for (const listener of this.listeners) listener(bytes);
  }
}

function detachInputListener(
  input: ByteInput,
  event: "data" | "end" | "close" | "error",
  listener: ByteInputEventListener,
): void {
  if (input.off) input.off(event, listener);
  else input.removeListener?.(event, listener);
}

function detachOutputListener(
  output: ByteOutput,
  event: "drain" | "close" | "error",
  listener: ByteOutputEventListener,
): void {
  if (output.off) output.off(event, listener);
  else output.removeListener?.(event, listener);
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
