import {
  TransportTerminatedError,
  type DisposableTransport,
  type TransportListener,
  type TransportTerminationListener,
} from "./transport";

interface EmbeddedBridge {
  submit(frame: Uint8Array): void;
  subscribe(onData: TransportListener, onTermination: (message: string) => void): () => void;
}

function hostBridge(): EmbeddedBridge {
  const bridge = (globalThis as { __solidGpuiHost?: EmbeddedBridge }).__solidGpuiHost;
  if (!bridge || typeof bridge.submit !== "function" || typeof bridge.subscribe !== "function") {
    throw new Error("EmbeddedTransport requires an embedded Solid GPUI host");
  }
  return bridge;
}

/** One VM connection, shared by every Surface owned by the application. */
export class EmbeddedTransport implements DisposableTransport {
  private readonly bridge = hostBridge();
  private readonly listeners = new Set<TransportListener>();
  private readonly terminationListeners = new Set<TransportTerminationListener>();
  private readonly unsubscribe: () => void;
  private disposed = false;
  private failure: TransportTerminatedError | undefined;

  constructor() {
    this.unsubscribe = this.bridge.subscribe(
      (frame) => {
        for (const listener of this.listeners) listener(frame);
      },
      (message) => this.terminate(new TransportTerminatedError(message)),
    );
  }

  submit(frame: Uint8Array): void {
    this.assertActive();
    try {
      this.bridge.submit(frame);
    } catch (error) {
      const failure = new TransportTerminatedError("EmbeddedTransport output failed", error);
      this.terminate(failure);
      throw failure;
    }
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
