import { createRoot as createOwner } from "solid-js";
import { createRoot, type Root, type RootOptions, type SolidElement } from "./renderer";
import { MAX_FRAME_SIZE } from "./protocol";
import type { DisposableTransport, Transport, TransportListener, TransportTerminationListener } from "./transport";

export interface ApplicationDefinition<State> {
  readonly render: () => SolidElement;
  readonly rootOptions?: Omit<RootOptions, "surfaceId" | "epoch">;
  readonly onMount?: (root: Root) => void;
  /** Return structured-cloneable application state; component-local signals remount. */
  readonly captureState?: () => State;
}

export interface ApplicationOptions<State> {
  /** Stable entry URL during development HMR. Omit in production. */
  readonly hotKey?: string;
  readonly surfaceId?: number;
  /** Creates the connection owned by this application; hot reload retains it. */
  readonly transport: () => DisposableTransport;
  /** Runs inside a Solid owner: register resource cleanup with onCleanup. */
  readonly setup: (previousState: State | undefined) => ApplicationDefinition<State>;
}

export interface MountedApplication {
  readonly root: Root;
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
}
const sessionKey = Symbol.for("solid-gpui.application.sessions");
function hotSessions(): Map<string, Session> {
  const globals = globalThis as typeof globalThis & { [sessionKey]?: Map<string, Session> };
  return (globals[sessionKey] ??= new Map());
}

/** Candidate output stays private until setup/render succeed. No new wire protocol. */
class CandidateTransport implements Transport {
  private active = false;
  private disposed = false;
  private bytes = 0;
  private frames: Uint8Array[] = [];

  constructor(private readonly transport: Transport) {}

  submit(frame: Uint8Array): void {
    if (this.disposed) throw new Error("application generation is disposed");
    if (this.active) {
      this.transport.submit(frame);
      return;
    }
    if (this.bytes + frame.length > MAX_FRAME_SIZE + 4) {
      throw new Error("application initial commit exceeds the candidate byte budget");
    }
    this.frames.push(frame.slice());
    this.bytes += frame.length;
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
    const frames = this.frames;
    this.frames = [];
    this.bytes = 0;
    for (const frame of frames) this.transport.submit(frame);
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
  const sessions = options.hotKey === undefined ? undefined : hotSessions();
  const previous = options.hotKey === undefined ? undefined : sessions?.get(options.hotKey);
  const surfaceId = options.surfaceId ?? 1;
  if (previous && previous.surfaceId !== surfaceId) throw new Error("hot reload cannot change surfaceId");
  const epoch = (previous?.epoch ?? 0) + 1;
  if (epoch > 0xffff_ffff) throw new Error("application epoch exhausted; restart the host");
  const saved = previous?.captureState?.();
  const state = saved === undefined ? undefined : (structuredClone(saved) as State);
  const transport = previous?.transport ?? options.transport();
  const closeTransport = previous?.closeTransport ?? (() => transport.dispose());
  const candidate = new CandidateTransport(transport);
  let root: Root | undefined;
  let disposeOwner: (() => void) | undefined;
  let definition: ApplicationDefinition<State> | undefined;
  let disposed = false;
  let session: Session | undefined;
  const retire = (): void => {
    if (disposed) return;
    disposed = true;
    candidate.dispose();
    if (options.hotKey !== undefined && sessions?.get(options.hotKey) === session) {
      sessions?.delete(options.hotKey);
    }
    try {
      root?.unmount();
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
      const callbacks = definition.rootOptions;
      root = createRoot(candidate, {
        ...callbacks,
        surfaceId,
        epoch,
        onClose: () => {
          try {
            callbacks?.onClose?.();
          } finally {
            dispose();
          }
        },
        onTransportTermination: (error) => {
          try {
            callbacks?.onTransportTermination?.(error);
          } finally {
            dispose();
          }
        },
      });
    });
    if (!root || !definition) throw new Error("application setup did not finish");
    root.render(definition.render);
    candidate.assertReady();
  } catch (error) {
    try {
      retire();
    } finally {
      if (!previous) closeTransport();
    }
    throw error; // The previous generation is still live and interactive.
  }
  if (!root || !definition) throw new Error("application setup did not finish");
  session = {
    root,
    transport,
    closeTransport,
    retire,
    epoch,
    surfaceId,
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
  candidate.activate();
  definition.onMount?.(root);
  if (options.hotKey !== undefined) console.error(`solid-gpui: hot reload applied (epoch ${epoch})`);
  return session;
}
