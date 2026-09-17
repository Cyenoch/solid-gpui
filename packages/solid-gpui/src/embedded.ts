import {
  TransportTerminatedError,
  type DisposableTransport,
  type TransportListener,
  type TransportTerminationListener,
} from "./transport";

interface EmbeddedBridge {
  submit(frame: Uint8Array): boolean;
  subscribe(onData: TransportListener, onTermination: (message: string) => void, onDrain: () => void): () => void;
  /**
   * Present on hosts that accept a typed completion result. Absent elsewhere,
   * which is why the bridge shape is never part of the public contract.
   */
  complete?(code: number): void;
}

/**
 * The result an embedded session reports to its host.
 *
 * `code` is a full 32-bit value, deliberately not the process exit status the
 * operating system limits to one byte: an application that reports a real
 * process result — a Windows UAC cancellation is 1223 — keeps it intact here.
 */
export interface EmbeddedResult {
  readonly code: number;
}

/** The running host does not accept a typed completion result. */
export class EmbeddedCompletionUnsupportedError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "EmbeddedCompletionUnsupportedError";
  }
}

/**
 * The embedded bridge global, or `undefined` outside a packaged session.
 *
 * The name and the methods on it are owned by the embedded runtime's bootstrap,
 * which installs them before the application entry runs; no TypeScript type
 * space describes them, so this is the single place that reads them.
 */
function embeddedBridgeGlobal(): EmbeddedBridge | undefined {
  const host = globalThis as { __solidGpuiHost?: EmbeddedBridge };
  return host.__solidGpuiHost;
}

function hostBridge(): EmbeddedBridge {
  const bridge = embeddedBridgeGlobal();
  if (!bridge || typeof bridge.submit !== "function" || typeof bridge.subscribe !== "function") {
    throw new Error("EmbeddedTransport requires an embedded Solid GPUI host");
  }
  return bridge;
}

/**
 * Whether the running host accepts a typed completion result.
 *
 * False in a session that runs under the QuickJS renderer, and in a separate
 * worker VM: the completion API belongs to the VM the embedded runtime started.
 */
export function supportsEmbeddedCompletion(): boolean {
  return typeof embeddedBridgeGlobal()?.complete === "function";
}

/**
 * Declares this session's completion result.
 *
 * The host reads it as a typed value (`EmbeddedBunAdapter::result` on the Rust
 * side), which is independent of the VM's own exit status. Applications call
 * this instead of reaching into the host bridge global or inventing a result
 * frame, and it is the only supported way to report a result wider than one
 * byte. It may be called once per session; a second call throws.
 *
 * Unsupported hosts (the QuickJS renderer, a separate worker VM) fail here with
 * {@link EmbeddedCompletionUnsupportedError} rather than silently dropping the
 * result.
 */
export function completeEmbedded(result: EmbeddedResult | number): void {
  const code = typeof result === "number" ? result : result.code;
  if (!Number.isInteger(code) || code < 0 || code > 0xffff_ffff) {
    throw new TypeError(`embedded completion code must be an integer in [0, 2**32 - 1], received ${String(code)}`);
  }
  const bridge = hostBridge();
  if (typeof bridge.complete !== "function") {
    throw new EmbeddedCompletionUnsupportedError(
      "this host does not accept a typed completion result; a packaged embedded application reports one through " +
        "@solid-gpui/core/embedded",
    );
  }
  bridge.complete(code);
}

/** One VM connection, shared by every Surface owned by the application. */
export class EmbeddedTransport implements DisposableTransport {
  private readonly bridge = hostBridge();
  private readonly listeners = new Set<TransportListener>();
  private readonly terminationListeners = new Set<TransportTerminationListener>();
  private readonly unsubscribe: () => void;
  private readonly drainListeners = new Set<() => void>();
  private disposed = false;
  private failure: TransportTerminatedError | undefined;

  constructor() {
    this.unsubscribe = this.bridge.subscribe(
      (frame) => {
        for (const listener of this.listeners) listener(frame);
      },
      (message) => this.terminate(new TransportTerminatedError(message)),
      () => {
        for (const listener of this.drainListeners) listener();
      },
    );
  }

  submit(frame: Uint8Array): boolean {
    this.assertActive();
    try {
      return this.bridge.submit(frame);
    } catch (error) {
      const failure = new TransportTerminatedError("EmbeddedTransport output failed", error);
      this.terminate(failure);
      throw failure;
    }
  }

  onDrain(listener: () => void): () => void {
    this.drainListeners.add(listener);
    return () => this.drainListeners.delete(listener);
  }

  onData(listener: TransportListener): () => void {
    this.assertActive();
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  onTermination(listener: TransportTerminationListener): () => void {
    if (this.failure) {
      listener(this.failure);
      return () => undefined;
    }
    if (this.disposed) return () => undefined;
    this.terminationListeners.add(listener);
    return () => this.terminationListeners.delete(listener);
  }

  dispose(): void {
    if (this.disposed) return;
    this.disposed = true;
    this.unsubscribe();
    this.listeners.clear();
    this.drainListeners.clear();
    this.terminationListeners.clear();
  }

  private assertActive(): void {
    if (this.failure) throw this.failure;
    if (this.disposed) throw new Error("EmbeddedTransport is disposed");
  }

  private terminate(error: TransportTerminatedError): void {
    if (this.disposed || this.failure) return;
    this.failure = error;
    const listeners = [...this.terminationListeners];
    this.dispose();
    for (const listener of listeners) listener(error);
  }
}
