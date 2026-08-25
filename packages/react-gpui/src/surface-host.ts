import { FrameDecoder, MAX_FRAME_SIZE, decodeEvent, framePayload } from "./protocol";
import { createRoot, type Root, type RootOptions } from "./renderer";
import {
  TransportTerminatedError,
  type Transport,
  type TransportListener,
  type TransportTerminationListener,
} from "./transport";

export interface SurfaceHostOptions {
  readonly maxFrameSize?: number;
  readonly onTransportTermination?: TransportTerminationListener;
}

export interface SurfaceHost {
  createRoot(options?: RootOptions): Root;
  dispose(): void;
}

class RoutedTransport implements Transport {
  private readonly listeners = new Set<TransportListener>();
  private readonly terminationListeners = new Set<TransportTerminationListener>();
  private disposed = false;
  private terminated = false;
  private terminationError: TransportTerminatedError | undefined;

  constructor(private readonly host: SurfaceHostImpl) {}

  submit(frame: Uint8Array): void {
    if (this.terminated) throw this.terminationError;
    if (this.disposed) throw new Error("SurfaceHost root transport is disposed");
    this.host.submit(frame);
  }

  onData(listener: TransportListener): () => void {
    if (this.terminated) throw this.terminationError;
    if (this.disposed) throw new Error("SurfaceHost root transport is disposed");
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

  deliver(frame: Uint8Array): void {
    if (this.disposed || this.terminated) return;
    for (const listener of this.listeners) listener(frame);
  }

  terminate(error: TransportTerminatedError): void {
    if (this.disposed || this.terminated) return;
    this.terminated = true;
    this.terminationError = error;
    this.listeners.clear();
    const listeners = [...this.terminationListeners];
    this.terminationListeners.clear();
    let firstError: unknown;
    for (const listener of listeners) {
      try {
        listener(error);
      } catch (listenerError) {
        firstError ??= listenerError;
      }
    }
    if (firstError !== undefined) throw firstError;
  }

  dispose(): void {
    if (this.disposed) return;
    this.disposed = true;
    this.listeners.clear();
    this.terminationListeners.clear();
  }
}

export class SurfaceHostImpl implements SurfaceHost {
  private readonly decoder: FrameDecoder;
  private readonly roots = new Map<number, RoutedTransport>();
  private readonly unsubscribe: () => void;
  private readonly unsubscribeTermination: () => void;
  private readonly onTransportTermination: TransportTerminationListener | undefined;
  private nextSurfaceId = 1;
  private terminated = false;
  private disposed = false;
  private terminationError: TransportTerminatedError | undefined;

  constructor(
    private readonly transport: Transport,
    options: SurfaceHostOptions = {},
  ) {
    this.decoder = new FrameDecoder(options.maxFrameSize ?? MAX_FRAME_SIZE);
    this.onTransportTermination = options.onTransportTermination;
    this.unsubscribe = transport.onData((chunk) => this.receive(chunk));
    this.unsubscribeTermination = transport.onTermination((error) => this.handleTermination(error));
  }

  createRoot(options: RootOptions = {}): Root {
    if (this.terminated) throw this.terminationError;
    if (this.disposed) throw new Error("SurfaceHost is disposed");
    const surfaceId = options.surfaceId ?? this.allocateSurfaceId();
    if (!Number.isInteger(surfaceId) || surfaceId < 1 || surfaceId > 0xffff_ffff)
      throw new RangeError("surfaceId must be a positive u32");
    if (this.roots.has(surfaceId)) throw new Error(`surface ${surfaceId} is already registered`);

    const routed = new RoutedTransport(this);
    const root = createRoot(routed, {
      ...options,
      surfaceId,
      onClose: () => {
        this.roots.delete(surfaceId);
        routed.dispose();
        options.onClose?.();
      },
    });
    this.roots.set(surfaceId, routed);
    return root;
  }

  dispose(): void {
    if (this.disposed) return;
    this.disposed = true;
    this.unsubscribe();
    this.unsubscribeTermination();
    const error = new TransportTerminatedError("SurfaceHost is disposed");
    this.terminateRoots(error);
    this.roots.clear();
  }

  submit(frame: Uint8Array): void {
    if (this.terminated) throw this.terminationError;
    if (this.disposed) throw new Error("SurfaceHost is disposed");
    try {
      this.transport.submit(frame);
    } catch (error) {
      if (error instanceof TransportTerminatedError) this.handleTermination(error);
      throw error;
    }
  }

  private allocateSurfaceId(): number {
    while (this.roots.has(this.nextSurfaceId)) {
      if (this.nextSurfaceId >= 0xffff_ffff) throw new RangeError("surface id exhausted u32 range");
      this.nextSurfaceId += 1;
    }
    const surfaceId = this.nextSurfaceId;
    if (this.nextSurfaceId >= 0xffff_ffff) this.nextSurfaceId = 1;
    else this.nextSurfaceId += 1;
    return surfaceId;
  }

  private receive(chunk: Uint8Array | ArrayBuffer): void {
    let payloads: Uint8Array[];
    try {
      payloads = this.decoder.push(chunk);
    } catch {
      return;
    }
    for (const payload of payloads) {
      const event = decodeEvent(payload);
      if (event === null) continue;
      const routed = this.roots.get(event[2]);
      if (routed !== undefined) routed.deliver(framePayload(payload));
    }
  }

  private terminateRoots(error: TransportTerminatedError): void {
    for (const routed of this.roots.values()) {
      try {
        routed.terminate(error);
      } catch {
        // A user termination callback must not prevent other roots from rejecting.
      }
    }
  }

  private handleTermination(error: TransportTerminatedError): void {
    if (this.terminated || this.disposed) return;
    this.terminated = true;
    this.terminationError = error;
    this.unsubscribe();
    this.unsubscribeTermination();
    this.terminateRoots(error);
    this.roots.clear();
    this.onTransportTermination?.(error);
  }
}

export function createSurfaceHost(transport: Transport, options: SurfaceHostOptions = {}): SurfaceHost {
  return new SurfaceHostImpl(transport, options);
}
