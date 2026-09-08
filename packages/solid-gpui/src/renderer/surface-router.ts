import { FrameDecoder, MAX_FRAME_SIZE, decodeEvent, type Event } from "../protocol";
import { TransportTerminatedError, type Transport, type TransportTerminationListener } from "../transport";

export interface SurfaceRoute {
  deliver(events: readonly Event[]): void;
  terminate(error: TransportTerminatedError): void;
}

export interface SurfaceRouterOptions {
  readonly maxFrameSize?: number;
  readonly onTermination?: TransportTerminationListener;
  readonly onApplicationActivation?: (event: Event) => void;
}

export class SurfaceRouter {
  private readonly decoder: FrameDecoder;
  private readonly routes = new Map<number, SurfaceRoute>();
  private readonly onTermination: TransportTerminationListener | undefined;
  private unsubscribeData: (() => void) | undefined;
  private unsubscribeTermination: (() => void) | undefined;
  private unsubscribeDrain: (() => void) | undefined;
  private pressured = false;
  private output: Uint8Array[] = [];
  private outputBytes = 0;
  private input: Uint8Array[] = [];
  private inputBytes = 0;
  private started = false;
  private terminated = false;
  private disposed = false;
  private terminationError: TransportTerminatedError | undefined;

  constructor(
    private readonly transport: Transport,
    private readonly options: SurfaceRouterOptions = {},
  ) {
    this.decoder = new FrameDecoder(options.maxFrameSize ?? MAX_FRAME_SIZE);
    this.onTermination = options.onTermination;
  }

  start(): void {
    if (this.started || this.disposed) return;
    this.started = true;
    this.unsubscribeDrain = this.transport.onDrain(() => this.drain());
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
      if (this.pressured || this.output.length) {
        if (this.output.length >= 4096 || frame.length > MAX_FRAME_SIZE + 4 - this.outputBytes)
          throw new RangeError("renderer pending output capacity exceeded");
        this.output.push(frame.slice());
        this.outputBytes += frame.length;
      } else {
        this.pressured = !this.transport.submit(frame);
      }
    } catch (error) {
      const termination =
        error instanceof TransportTerminatedError
          ? error
          : new TransportTerminatedError(
              `surface router output failed: ${error instanceof Error ? error.message : String(error)}`,
              error,
            );
      this.terminate(termination);
      throw termination;
    }
  }

  private drain(): void {
    if (this.disposed || this.terminated) return;
    this.pressured = false;
    try {
      while (!this.pressured && this.output.length) {
        const frame = this.output.shift()!;
        this.outputBytes -= frame.length;
        this.pressured = !this.transport.submit(frame);
      }
      // Pause event-driven production at the shared application scheduler.
      // Timers may still submit, but share the same hard output budget.
      while (!this.pressured && this.input.length) {
        const frame = this.input.shift()!;
        this.inputBytes -= frame.length;
        this.receive(frame);
      }
    } catch (error) {
      this.terminate(new TransportTerminatedError("renderer drain failed", error));
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
    if (this.pressured) {
      const bytes = chunk instanceof Uint8Array ? chunk : new Uint8Array(chunk);
      if (this.input.length >= 4096 || bytes.length > MAX_FRAME_SIZE + 4 - this.inputBytes) {
        this.terminate(new TransportTerminatedError("renderer pending input capacity exceeded"));
        return;
      }
      this.input.push(bytes.slice());
      this.inputBytes += bytes.length;
      return;
    }
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
    const decoded: Event[] = [];
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
      decoded.push(event);
    }
    const bySurface = new Map<number, Event[]>();
    const flush = () => {
      for (const [surfaceId, events] of bySurface) this.routes.get(surfaceId)?.deliver(events);
      bySurface.clear();
    };
    for (const event of decoded) {
      if (event.payload.type === "application-activation") {
        flush();
        if (this.disposed || this.terminated) return;
        try {
          this.options.onApplicationActivation?.(event);
        } catch (error) {
          this.terminate(new TransportTerminatedError("application activation failed", error));
          return;
        }
      } else {
        const events = bySurface.get(event.surfaceId);
        if (events === undefined) bySurface.set(event.surfaceId, [event]);
        else events.push(event);
      }
    }
    flush();
  }

  private detachTransport(): void {
    this.unsubscribeDrain?.();
    this.unsubscribeDrain = undefined;
    this.output = [];
    this.input = [];
    this.outputBytes = this.inputBytes = 0;
    const unsubscribeData = this.unsubscribeData;
    this.unsubscribeData = undefined;
    unsubscribeData?.();
    const unsubscribeTermination = this.unsubscribeTermination;
    this.unsubscribeTermination = undefined;
    unsubscribeTermination?.();
  }
}
