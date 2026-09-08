import { generationHost, encodeGenerationState } from "./generation";
import { SurfaceRouter } from "./renderer/surface-router";
import { createRoot as createOwner } from "solid-js";
import { createRootWithRouter, type Root, type RootOptions, type SolidElement } from "./renderer";
import { MAX_FRAME_SIZE, COMMAND_CONFIGURE_APPLICATION, encodeFrame } from "./protocol";
import type { DisposableTransport, Transport, TransportListener, TransportTerminationListener } from "./transport";

export interface ApplicationActivation {
  readonly reason: "launch" | "reopen" | "open-urls";
  readonly urls: readonly string[];
  readonly root: Root;
}

export interface ApplicationDefinition<State> {
  readonly onActivate?: (activation: ApplicationActivation) => void;
  readonly render: () => SolidElement;
  readonly rootOptions?: Omit<RootOptions, "surfaceId" | "epoch">;
  readonly onMount?: (root: Root) => void;
  /** Return structured-cloneable application state; component-local signals remount. */
  readonly captureState?: () => State;
}

export interface ApplicationOptions<State> {
  /** Stable entry URL during development HMR. Omit in production. */
  readonly hotKey?: string;
  readonly lastWindowClose?: "quit" | "keep-alive";
  readonly surfaceId?: number;
  /** Creates the connection owned by this application; hot reload retains it. */
  readonly transport: () => DisposableTransport;
  /** Runs inside a Solid owner: register resource cleanup with onCleanup. */
  readonly setup: (previousState: State | undefined) => ApplicationDefinition<State>;
}

export interface MountedApplication {
  readonly root: Root | undefined;
  quit(): void;
  dispose(): void;
}

// Kept across development module replacement. Only transport bytes and explicit
// application state cross generations; old renderer/Solid instances are not reused.
interface Session extends MountedApplication {
  readonly transport: DisposableTransport;
  readonly closeTransport: () => void;
  retire(): void;
  readonly epoch: number;
  readonly surfaceId: number;
  readonly captureState?: () => unknown;
  readonly activationSequence: number;
}
const sessionKey = Symbol.for("solid-gpui.application.sessions");
function hotSessions(): Map<string, Session> {
  const globals = globalThis as typeof globalThis & { [sessionKey]?: Map<string, Session> };
  return (globals[sessionKey] ??= new Map());
}

/** Candidate output stays private until setup/render succeed. */
class CandidateTransport implements Transport {
  private active = false;
  private disposed = false;
  private bytes = 0;
  private frames: Uint8Array[] = [];
  private pressured = false;

  constructor(private readonly transport: Transport) {}

  submit(frame: Uint8Array): boolean {
    if (this.disposed) throw new Error("application generation is disposed");
    if (this.active && !this.pressured) {
      this.pressured = !this.transport.submit(frame);
      return !this.pressured;
    }
    if (this.frames.length >= 4096 || this.bytes + frame.length > MAX_FRAME_SIZE + 4) {
      throw new Error("application initial commit exceeds the candidate byte budget");
    }
    this.frames.push(frame.slice());
    this.bytes += frame.length;
    return !this.pressured;
  }

  onDrain(listener: () => void): () => void {
    return this.transport.onDrain(() => {
      if (this.disposed) return;
      this.pressured = false;
      if (this.active) this.flush();
      if (!this.pressured) listener();
    });
  }

  onData(listener: TransportListener): () => void {
    return this.transport.onData((chunk) => {
      if (this.active && !this.disposed) listener(chunk);
    });
  }

  onTermination(listener: TransportTerminationListener): () => void {
    return this.transport.onTermination(listener);
  }

  assertReady(): void {
    if (this.frames.length === 0) throw new Error("application must render a nonempty initial tree");
  }

  activate(): void {
    this.active = true;
    this.flush();
  }

  private flush(): void {
    while (!this.pressured && this.frames.length > 0) {
      const frame = this.frames.shift()!;
      this.bytes -= frame.length;
      this.pressured = !this.transport.submit(frame);
    }
  }

  dispose(): void {
    this.disposed = true;
    this.active = false;
    this.frames = [];
    this.bytes = 0;
  }
}

/** Mount once, or replace the hotKey's application without closing its native window. */
export function mountApplication<State = never>(options: ApplicationOptions<State>): MountedApplication {
  const generation = generationHost();
  const handoff = generation
    ? (JSON.parse(generation.state) as { state: State[]; surfaceId: number; open: boolean; activationSequence: number })
    : undefined;
  const sessions = options.hotKey === undefined ? undefined : hotSessions();
  const previous = options.hotKey === undefined ? undefined : sessions?.get(options.hotKey);
  const surfaceId = options.surfaceId ?? handoff?.surfaceId ?? previous?.surfaceId ?? 1;
  if (previous && previous.surfaceId !== surfaceId) throw new Error("hot reload cannot change surfaceId");
  const epoch = generation?.epoch ?? (previous?.epoch ?? 0) + 1;
  if (epoch > 0xffff_ffff) throw new Error("application epoch exhausted; restart the host");
  const saved = previous?.captureState?.();
  const state = handoff ? handoff.state[0] : saved === undefined ? undefined : (structuredClone(saved) as State);
  const transport = previous?.transport ?? options.transport();
  const closeTransport = previous?.closeTransport ?? (() => transport.dispose());
  const candidate = new CandidateTransport(transport);
  let root: Root | undefined;
  let disposeOwner: (() => void) | undefined;
  let definition: ApplicationDefinition<State> | undefined;
  let disposed = false;
  let session: Session | undefined;
  let lastSurfaceId = surfaceId;
  let activationSequence = handoff?.activationSequence ?? previous?.activationSequence ?? 0;
  let nextControlRequest = 1;
  const keepAlive = options.lastWindowClose === "keep-alive";
  const router = new SurfaceRouter(candidate, {
    onTermination: (error) => {
      try {
        definition?.rootOptions?.onTransportTermination?.(error);
      } finally {
        dispose();
      }
    },
    onApplicationActivation: (event) => {
      if (
        disposed ||
        event.epoch !== epoch ||
        event.sequence <= activationSequence ||
        event.payload.type !== "application-activation"
      )
        return;

      const activation = event.payload;
      if (!root) {
        if (activation.targetSurfaceId <= lastSurfaceId) throw new Error("activation reused a retired Surface");
        openRoot(activation.targetSurfaceId);
        root!.render(definition!.render);
        definition?.onMount?.(root!);
      }
      if (disposed) return;
      if (lastSurfaceId !== activation.targetSurfaceId) throw new Error("activation targets another Surface");
      definition?.onActivate?.({ reason: activation.reason, urls: activation.urls, root: root! });
      if (disposed) return;
      activationSequence = event.sequence;
      control(false);
    },
  });
  const control = (quit: boolean) =>
    router.submit(
      encodeFrame({
        type: "command",
        surfaceId: 0,
        epoch,
        afterRevision: 0,
        requestId: nextControlRequest++,
        nodeId: 0,
        command: COMMAND_CONFIGURE_APPLICATION,
        payload: {
          type: "configure-application",
          keepAlive: !quit && keepAlive,
          quit,
          acknowledgedSequence: activationSequence,
        },
      }),
    );
  const openRoot = (id: number) => {
    lastSurfaceId = id;
    const callbacks = definition?.rootOptions;
    root = createRootWithRouter(router, {
      ...callbacks,
      surfaceId: id,
      epoch,
      onClose: () => {
        root = undefined;
        // The host owns the last-window policy; auxiliary Surfaces may still be open.
        callbacks?.onClose?.();
      },
      // The application connection owns termination, including zero-window time.
      onTransportTermination: undefined,
    });
  };
  const retire = (): void => {
    if (disposed) return;
    disposed = true;
    candidate.dispose();
    router.dispose();
    if (options.hotKey !== undefined && sessions?.get(options.hotKey) === session) {
      sessions?.delete(options.hotKey);
    }
    try {
      const mounted = root;
      root = undefined;
      mounted?.unmount();
    } finally {
      disposeOwner?.();
    }
  };
  const dispose = (): void => {
    if (disposed) return;
    try {
      retire();
    } finally {
      closeTransport();
    }
  };
  try {
    createOwner((cleanup) => {
      disposeOwner = cleanup;
      definition = options.setup(state);
      if (handoff ? handoff.open : !previous || previous.root) openRoot(surfaceId);
      router.start();
    });
    if (!definition) throw new Error("application setup did not finish");
    if (root) {
      root.render(definition.render);
      candidate.assertReady();
    }
    control(false);
  } catch (error) {
    try {
      retire();
    } finally {
      if (!previous) closeTransport();
    }
    throw error; // The previous generation is still live and interactive.
  }
  if (!definition) throw new Error("application setup did not finish");
  session = {
    get root() {
      return root;
    },
    get activationSequence() {
      return activationSequence;
    },
    quit: () => {
      if (!disposed) {
        control(true);
        dispose();
      }
    },
    transport,
    closeTransport,
    retire,
    epoch,
    get surfaceId() {
      return lastSurfaceId;
    },
    captureState: definition.captureState,
    dispose,
  };
  // Cleanup errors must not leave two event consumers active. Root unmount and
  // owner disposal have already been attempted before publishing the new tree.
  try {
    previous?.retire();
  } catch (error) {
    console.error("solid-gpui: application cleanup failed", error);
  }
  if (options.hotKey !== undefined) sessions?.set(options.hotKey, session);
  try {
    candidate.activate();
    if (generation) {
      const activeDefinition = definition;
      generation.register({
        capture: () => {
          const state = activeDefinition.captureState?.();
          return encodeGenerationState({
            state: state === undefined ? [] : [state],
            surfaceId: lastSurfaceId,
            open: Boolean(root),
            activationSequence,
          });
        },
        activate: () => {
          if (root) activeDefinition.onMount?.(root);
        },
        retire,
      });
    } else if (root) definition.onMount?.(root);
  } catch (error) {
    dispose();
    throw error;
  }
  if (options.hotKey !== undefined) console.error(`solid-gpui: hot reload applied (epoch ${epoch})`);
  return session;
}
