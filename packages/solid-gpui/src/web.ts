import {
  TransportTerminatedError,
  type DisposableTransport,
  type TransportListener,
  type TransportTerminationListener,
} from "./transport";

/** The wasm-bindgen exports of solid-gpui-web, after initialization and start(). */
export interface WebHost {
  submit(frame: Uint8Array): void;
  drain_events(): Uint8Array;
  stop(): void;
}

/** Owns one GPUI browser application. Events are delivered outside Rust updates. */
export class WebTransport implements DisposableTransport {
  private listeners = new Set<TransportListener>();
  private termination = new Set<TransportTerminationListener>();
  private frame = 0;
  private error?: TransportTerminatedError;

  constructor(private host: WebHost) {
    this.frame = requestAnimationFrame(this.pump);
  }

  submit(frame: Uint8Array): boolean {
    if (this.error) throw this.error;
    try {
      this.host.submit(frame);
      return true;
    } catch (error) {
      this.fail(error);
      throw this.error;
    }
  }

  onDrain(_listener: () => void): () => void {
    return () => undefined;
  }

  onData(listener: TransportListener): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }
  onTermination(listener: TransportTerminationListener): () => void {
    if (this.error) listener(this.error);
    else this.termination.add(listener);
    return () => this.termination.delete(listener);
  }
  private pump = (): void => {
    if (this.error) return;
    try {
      const events = this.host.drain_events();
      if (events.length) for (const listener of this.listeners) listener(events);
      if (!this.error) this.frame = requestAnimationFrame(this.pump);
    } catch (error) {
      this.fail(error);
    }
  };
  private fail(error: unknown): void {
    if (this.error) return;
    this.error =
      error instanceof TransportTerminatedError
        ? error
        : new TransportTerminatedError(String(error), { kind: "protocol", detail: String(error) });
    cancelAnimationFrame(this.frame);
    this.host.stop();
    for (const listener of this.termination) listener(this.error);
    this.listeners.clear();
    this.termination.clear();
  }
  dispose(): void {
    this.fail(new TransportTerminatedError("Browser surface disposed", { kind: "shutdown" }));
  }
}
