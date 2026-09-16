import { ProtocolTap } from "./protocol-tap";
import {
  TransportTerminatedError,
  type DisposableTransport,
  type TransportChunk,
  type TransportListener,
  type TransportTerminationCause,
  type TransportTerminationDetails,
  type TransportTerminationListener,
} from "./transport";

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

/**
 * Explicit termination policy for applications that want a diagnostic line and
 * a failure status instead of the transport's own host-close handling. A
 * listener registered here runs before the transport ends the process, so it
 * replaces that default; a listener may also end the process itself.
 */
export function createProcessTerminationHandler(exit: ExitFunction = defaultProcessExit): TransportTerminationListener {
  return (error) => {
    console.error(`solid-gpui: transport terminated: ${error.message}`);
    exit(1);
  };
}

function defaultProcessExit(code: number): void {
  const { exit } = processGlobal();
  if (exit) {
    exit(code);
    return;
  }
  throw new Error(`solid-gpui: transport terminated with exit code ${code}`);
}

export interface StdioTransportOptions {
  /** Frame destination; defaults to `process.stdout`. */
  readonly output?: ByteOutput;
  /** Frame source; defaults to `process.stdin`. */
  readonly input?: ByteInput;
  /**
   * End this process when the host connection ends, even while application
   * timers or polling keep Bun's event loop alive. Defaults to true when the
   * connection reads the process's own stdin, which is the host pipe of a
   * renderer child. Pass false only when this process outlives the connection.
   */
  readonly exitOnHostClose?: boolean;
  /** Exit hook; defaults to `process.exit`. */
  readonly exit?: ExitFunction;
}

/**
 * Framed stdio connection to the host. The connection that reads the process's
 * own stdin also owns the renderer's lifetime: closing the host pipe ends this
 * process instead of leaving an orphan that application timers keep alive.
 * Intentionally disposed connections and connections over supplied streams
 * never end the process; that is the embedder's decision, not the host's.
 */
export class StdioTransport implements DisposableTransport {
  private readonly listeners = new Set<TransportListener>();
  private readonly terminationListeners = new Set<TransportTerminationListener>();
  private readonly inputListener: ByteInputListener;
  private readonly inputEndListener: TransportCloseListener;
  private readonly inputCloseListener: TransportCloseListener;
  private readonly inputErrorListener: TransportErrorListener;
  private readonly drainListener: DrainListener;
  private readonly outputCloseListener: TransportCloseListener;
  private readonly outputErrorListener: TransportErrorListener;
  private readonly drainListeners = new Set<() => void>();
  private disposed = false;
  private terminated = false;
  private terminationError: TransportTerminatedError | undefined;
  private readonly input: ByteInput;
  private readonly output: ByteOutput;
  private readonly exitOnHostClose: boolean;
  private readonly exit: ExitFunction;
  private readonly tap: ProtocolTap | undefined;

  constructor(options: StdioTransportOptions = {}) {
    const stdio = processGlobal();
    const input = options.input ?? stdio.stdin;
    const output = options.output ?? stdio.stdout;
    if (!input) throw new Error("StdioTransport requires a readable stdin");
    if (!output) throw new Error("StdioTransport requires a writable stdout");
    this.input = input;
    this.output = output;
    this.exit = options.exit ?? defaultProcessExit;
    this.exitOnHostClose = options.exitOnHostClose ?? input === stdio.stdin;
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
    this.drainListener = () => {
      for (const listener of this.drainListeners) listener();
    };
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

  submit(frame: Uint8Array): boolean {
    if (this.terminated) throw this.terminationError;
    if (this.disposed) throw new Error("StdioTransport is disposed");
    try {
      const writable = this.output.write(frame);
      this.tap?.recordOutboundFrame(frame);
      return writable;
    } catch (error) {
      const failure = terminatedError("StdioTransport output write failed", error);
      this.terminate(failure);
      throw failure;
    }
  }

  onDrain(listener: () => void): () => void {
    this.drainListeners.add(listener);
    return () => this.drainListeners.delete(listener);
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
    this.drainListeners.clear();
    this.tap?.dispose();
  }

  private terminate(error: TransportTerminatedError): void {
    if (this.disposed || this.terminated) return;
    this.terminated = true;
    this.terminationError = error;
    this.detachListeners();
    this.listeners.clear();
    this.drainListeners.clear();
    this.tap?.dispose();
    const listeners = [...this.terminationListeners];
    this.terminationListeners.clear();
    try {
      for (const listener of listeners) listener(error);
    } finally {
      this.endHostLifetime(error);
    }
  }

  /**
   * Terminating listeners have already observed the reason; end the process
   * afterwards so a closed host pipe cannot leave an orphan renderer whose
   * timers keep the event loop alive. Nothing here touches application-owned
   * children: detached services are separate processes and outlive the renderer.
   */
  private endHostLifetime(error: TransportTerminatedError): void {
    if (!this.exitOnHostClose) return;
    // A closed pipe is the host going away, not a renderer failure. Windows can
    // report the broken pipe as a read error instead of a clean end.
    if (error.cause?.kind === "eof") {
      this.exit(0);
      return;
    }
    console.error(`solid-gpui: transport terminated: ${error.message}`);
    this.exit(1);
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

interface ProcessGlobal {
  readonly stdin?: ByteInput;
  readonly stdout?: ByteOutput;
  readonly exit?: ExitFunction;
}

/** Bun and Node expose stdio and process exit here; other runtimes reach the host through a bridge. */
function processGlobal(): ProcessGlobal {
  const globals = globalThis as { process?: ProcessGlobal };
  return globals.process ?? {};
}
