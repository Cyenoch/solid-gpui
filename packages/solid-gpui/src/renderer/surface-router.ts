import { FrameDecoder, MAX_FRAME_SIZE, decodeEvent, type Event } from "../protocol";
import { TransportTerminatedError, type Transport, type TransportTerminationListener } from "../transport";

export interface SurfaceRoute {
  deliver(events: readonly Event[]): void;
  terminate(error: TransportTerminatedError): void;
}

export interface SurfaceRouterOptions {
  readonly maxFrameSize?: number;
  readonly onTermination?: TransportTerminationListener;
}

export class SurfaceRouter {
  private readonly decoder: FrameDecoder;
  private readonly routes = new Map<number, SurfaceRoute>();
  private readonly onTermination: TransportTerminationListener | undefined;
  private unsubscribeData: (() => void) | undefined;
  private unsubscribeTermination: (() => void) | undefined;
  private started = false;
  private terminated = false;
  private disposed = false;
  private terminationError: TransportTerminatedError | undefined;

  constructor(
    private readonly transport: Transport,
    options: SurfaceRouterOptions = {},
  ) {
    this.decoder = new FrameDecoder(options.maxFrameSize ?? MAX_FRAME_SIZE);
    this.onTermination = options.onTermination;
  }

  start(): void {
    if (this.started || this.disposed) return;
    this.started = true;
    const unsubscribeData = this.transport.onData((chunk) => this.receive(chunk));
    this.unsubscribeData = unsubscribeData;
    if (this.terminated) {
      unsubscribeData();
      this.unsubscribeData = undefined;
      return;
    }
    const unsubscribeTermination = this.transport.onTermination((error) => this.terminate(error));
    this.unsubscribeTermination = unsubscribeTermination;
    if (this.terminated) {
      unsubscribeTermination();
      this.unsubscribeTermination = undefined;
    }
  }

  register(surfaceId: number, route: SurfaceRoute): () => void {
    if (this.terminated || this.disposed) {
      try {
        route.terminate(this.terminationError ?? new TransportTerminatedError("surface router is disposed"));
      } catch {
        // A route must not prevent registration callers from observing router state.
      }
      return () => undefined;
    }
    if (this.routes.has(surfaceId)) throw new Error(`surface ${surfaceId} is already registered`);
    this.routes.set(surfaceId, route);
    return () => {
      if (this.routes.get(surfaceId) === route) this.routes.delete(surfaceId);
    };
  }

  submit(frame: Uint8Array): void {
    if (this.terminated) throw this.terminationError;
    if (this.disposed) throw new Error("SurfaceRouter is disposed");
    try {
      this.transport.submit(frame);
    } catch (error) {
      const termination =
        error instanceof TransportTerminatedError
          ? error
          : new TransportTerminatedError("surface router output failed", error);
      this.terminate(termination);
      throw termination;
    }
  }

  terminate(error: TransportTerminatedError): void {
    if (this.terminated || this.disposed) return;
    this.terminated = true;
    this.terminationError = error;
    this.detachTransport();
    const routes = [...this.routes.values()];
    this.routes.clear();
    for (const route of routes) {
      try {
        route.terminate(error);
      } catch {
        // Isolate route teardown so all surfaces observe termination.
      }
    }
    try {
      this.onTermination?.(error);
    } catch {
      // User termination callbacks are isolated from transport teardown.
    }
  }

  dispose(): void {
    if (this.disposed) return;
    this.disposed = true;
    this.detachTransport();
    this.routes.clear();
  }

  private receive(chunk: Uint8Array | ArrayBuffer): void {
    if (this.terminated || this.disposed) return;
    let payloads: Uint8Array[];
    try {
      payloads = this.decoder.push(chunk);
    } catch (error) {
      this.terminate(
        new TransportTerminatedError(
          `SurfaceRouter protocol failure: ${error instanceof Error ? error.message : String(error)}`,
          {
            kind: "protocol",
            detail: error instanceof Error ? error.message : String(error),
          },
        ),
      );
      return;
    }
    if (payloads.length === 0) return;
    const bySurface = new Map<number, Event[]>();
    for (const payload of payloads) {
      let event: Event | null;
      try {
        event = decodeEvent(payload);
      } catch (error) {
        this.terminate(
          new TransportTerminatedError(
            `SurfaceRouter event decode failed: ${error instanceof Error ? error.message : String(error)}`,
            {
              kind: "protocol",
              detail: error instanceof Error ? error.message : String(error),
            },
          ),
        );
        return;
      }
      if (event === null) {
        this.terminate(
          new TransportTerminatedError("SurfaceRouter received malformed event frame", {
            kind: "protocol",
            detail: "malformed event frame",
          }),
        );
        return;
      }
      const events = bySurface.get(event.surfaceId);
      if (events === undefined) bySurface.set(event.surfaceId, [event]);
      else events.push(event);
    }
    for (const [surfaceId, events] of bySurface) this.routes.get(surfaceId)?.deliver(events);
  }

  private detachTransport(): void {
    const unsubscribeData = this.unsubscribeData;
    this.unsubscribeData = undefined;
    unsubscribeData?.();
    const unsubscribeTermination = this.unsubscribeTermination;
    this.unsubscribeTermination = undefined;
    unsubscribeTermination?.();
  }
}
