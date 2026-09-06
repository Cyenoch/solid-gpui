import { MAX_FRAME_SIZE } from "./protocol";
import { createRootWithRouter, type Root, type RootOptions } from "./renderer";
import { SurfaceRouter } from "./renderer/surface-router";
import { TransportTerminatedError, type Transport, type TransportTerminationListener } from "./transport";

export class SurfaceIdReusedError extends Error {
  readonly surfaceId: number;

  constructor(surfaceId: number) {
    super(`surface ${surfaceId} was already closed and cannot be reused`);
    this.name = "SurfaceIdReusedError";
    this.surfaceId = surfaceId;
  }
}

export interface SurfaceHostOptions {
  readonly maxFrameSize?: number;
  readonly onTransportTermination?: TransportTerminationListener;
}

export interface SurfaceHost {
  createRoot(options?: RootOptions): Root;
  dispose(): void;
}

export class SurfaceHostImpl implements SurfaceHost {
  private readonly router: SurfaceRouter;
  private readonly roots = new Map<number, Root>();
  private readonly retiredSurfaceIds = new Set<number>();
  private nextSurfaceId = 1;
  private terminated = false;
  private disposed = false;
  private terminationError: TransportTerminatedError | undefined;

  constructor(
    private readonly transport: Transport,
    options: SurfaceHostOptions = {},
  ) {
    this.router = new SurfaceRouter(transport, {
      maxFrameSize: options.maxFrameSize ?? MAX_FRAME_SIZE,
      onTermination: (error) => this.handleTermination(error, options.onTransportTermination),
    });
    this.router.start();
  }

  createRoot(options: RootOptions = {}): Root {
    if (this.terminated) throw this.terminationError;
    if (this.disposed) throw new Error("SurfaceHost is disposed");
    const surfaceId = options.surfaceId ?? this.allocateSurfaceId();
    if (!Number.isInteger(surfaceId) || surfaceId < 1 || surfaceId > 0xffff_ffff)
      throw new RangeError("surfaceId must be a positive u32");
    if (this.retiredSurfaceIds.has(surfaceId)) throw new SurfaceIdReusedError(surfaceId);
    if (this.roots.has(surfaceId)) throw new Error(`surface ${surfaceId} is already registered`);
    let released = false;
    const release = (): void => {
      if (released) return;
      released = true;
      this.roots.delete(surfaceId);
      this.retiredSurfaceIds.add(surfaceId);
    };
    const root = createRootWithRouter(
      this.router,
      {
        ...options,
        surfaceId,
        onClose: () => {
          release();
          options.onClose?.();
        },
      },
      false,
    );
    this.roots.set(surfaceId, root);
    return {
      ...root,
      unmount: () => {
        try {
          root.unmount();
        } finally {
          release();
        }
      },
    };
  }

  dispose(): void {
    if (this.disposed) return;
    this.disposed = true;
    const error = new TransportTerminatedError("SurfaceHost is disposed", { kind: "shutdown" });
    this.router.terminate(error);
    this.roots.clear();
  }

  submit(frame: Uint8Array): void {
    this.router.submit(frame);
  }

  private allocateSurfaceId(): number {
    for (;;) {
      const surfaceId = this.nextSurfaceId;
      if (surfaceId < 1 || surfaceId > 0xffff_ffff) throw new RangeError("surface id exhausted u32 range");
      this.nextSurfaceId = surfaceId + 1;
      if (!this.roots.has(surfaceId) && !this.retiredSurfaceIds.has(surfaceId)) return surfaceId;
    }
  }

  private handleTermination(error: TransportTerminatedError, listener?: TransportTerminationListener): void {
    if (this.terminated) return;
    this.terminated = true;
    this.terminationError = error;
    this.roots.clear();
    try {
      listener?.(error);
    } catch {
      // Isolate the host callback from route teardown.
    }
  }
}

export function createSurfaceHost(transport: Transport, options: SurfaceHostOptions = {}): SurfaceHost {
  return new SurfaceHostImpl(transport, options);
}
