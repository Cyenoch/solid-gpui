import type { Subprocess } from "bun";
import {
  TransportTerminatedError,
  type Transport,
  type TransportListener,
  type TransportTerminationListener,
} from "../transport";

const PACKET_LIMIT = 16 * 1024 * 1024;
const COMMIT_LIMIT = 4 * 1024 * 1024;
const encoder = new TextEncoder();

export interface NativeBounds {
  readonly x: number;
  readonly y: number;
  readonly width: number;
  readonly height: number;
}
export interface NativeLocator {
  readonly id?: number;
  /** Exact accessibilityLabel; duplicate matches fail. */
  readonly label?: string;
  /** Exact native text for a painted Text/RawText/TextInput element. */
  readonly text?: string;
}
export interface NativeTarget {
  readonly surfaceId: number;
  readonly epoch: number;
  readonly revision: number;
  readonly id: number;
  readonly parentId: number;
  readonly kind: number;
  readonly listenerId: number;
  readonly label: string | null;
  readonly text: string | null;
  readonly input: Readonly<{
    value: string;
    selectionStart: number;
    selectionEnd: number;
    reversed: boolean;
    editSeq: number;
    focused: boolean;
    markedStart: number | null;
    markedEnd: number | null;
  }> | null;
  /** Positive native VirtualList offset in logical pixels; other elements expose null. */
  readonly scrollOffset: number | null;
  /** Native core selectable Text selection; renderer-wide selection is owned by its public native module. */
  readonly selectedText: string | null;
  /** Last painted bounds clipped to the native content mask, in logical pixels. */
  readonly bounds: NativeBounds;
}
export interface NativeSnapshot {
  readonly surfaceId: number;
  readonly epoch: number;
  readonly revision: number;
  readonly nodes: readonly NativeTarget[];
}
export interface NativeAcceptanceCapabilities {
  readonly version: 1;
  readonly mode: "deterministic" | "gpu";
  readonly platform: string;
  readonly screenshots: boolean;
  readonly clock: boolean;
}
export interface NativeScreenshot {
  readonly width: number;
  readonly height: number;
  readonly png: Uint8Array;
}
export interface NativeCleanup {
  readonly surfaces: number;
  readonly windows: number;
  readonly popups: number;
}
export interface NativeAcceptanceOptions {
  /** Built solid-gpui-acceptance executable, or a custom production HostProfile runner. */
  readonly command: readonly [string, ...string[]];
  readonly mode: "deterministic" | "gpu";
  readonly cwd?: string;
  readonly timeoutMs?: number;
}
interface Reply {
  id: number;
  error: string | null;
  frames: number[][];
  result: unknown;
}

/** Owns one explicitly launched native process and its framed protocol transport.
 * Deterministic mode runs native layout/scene/input with GPUI's test platform.
 * GPU mode uses macOS Metal; neither mode injects physical OS input.
 */
export class NativeAcceptance {
  readonly transport: Transport;
  readonly capabilities: NativeAcceptanceCapabilities;
  private readonly listeners = new Set<TransportListener>();
  private readonly termination = new Set<TransportTerminationListener>();
  private commits: Uint8Array[] = [];
  private commitBytes = 0;
  private sequence = 0;
  private serial: Promise<unknown> = Promise.resolve();
  private closed = false;
  private closing = false;
  private cleanup?: NativeCleanup;
  private closePromise?: Promise<NativeCleanup>;
  private readonly targets = new WeakSet<NativeTarget>();
  private pending?: { resolve: (value: unknown) => void; reject: (error: Error) => void };
  private readonly exited: Promise<number>;

  private constructor(
    private readonly process: Subprocess<"pipe", "pipe", "pipe">,
    capabilities: NativeAcceptanceCapabilities,
    private readonly timeoutMs: number,
  ) {
    this.capabilities = Object.freeze(capabilities);
    this.transport = {
      submit: (frame) => {
        this.assertOpen();
        if (this.commits.length >= 256 || this.commitBytes + frame.length > COMMIT_LIMIT)
          throw new Error("Native acceptance commit budget exceeded; flush between application updates");
        this.commits.push(frame.slice());
        this.commitBytes += frame.length;
        return true;
      },
      onDrain: () => () => undefined,
      onData: (listener) => {
        this.listeners.add(listener);
        return () => this.listeners.delete(listener);
      },
      onTermination: (listener) => {
        this.termination.add(listener);
        return () => this.termination.delete(listener);
      },
    };
    this.exited = process.exited;
  }

  /** Bun-only controller; the application uses the ordinary createRoot transport. */
  static async launch(options: NativeAcceptanceOptions): Promise<NativeAcceptance> {
    if (typeof Bun === "undefined")
      throw new Error("NativeAcceptance.launch requires Bun; browser/QuickJS controllers are unsupported");
    const timeoutMs = options.timeoutMs ?? 30_000;
    if (!Number.isFinite(timeoutMs) || timeoutMs <= 0) throw new TypeError("Acceptance timeout must be positive");
    const process = Bun.spawn([...options.command, "--native-acceptance", options.mode], {
      cwd: options.cwd,
      stdin: "pipe",
      stdout: "pipe",
      stderr: "pipe",
    });
    let stderr = "";
    const readErrors = (async () => {
      for await (const bytes of process.stderr) stderr = (stderr + new TextDecoder().decode(bytes)).slice(-8192);
    })();
    const reader = process.stdout.getReader();
    let buffer = new Uint8Array(0);
    const readPacket = async (): Promise<unknown> => {
      for (;;) {
        if (buffer.length >= 4) {
          const length = new DataView(buffer.buffer, buffer.byteOffset, 4).getUint32(0, true);
          if (length === 0 || length > PACKET_LIMIT) throw new Error("Native acceptance packet exceeds budget");
          if (buffer.length >= length + 4) {
            const packet = JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(buffer.subarray(4, length + 4)));
            buffer = buffer.slice(length + 4);
            return packet;
          }
        }
        const next = await reader.read();
        if (next.done) {
          await process.exited;
          await readErrors;
          throw new Error(`Native acceptance exited: ${stderr || "unexpected EOF"}`);
        }
        if (buffer.length + next.value.length > PACKET_LIMIT + 4)
          throw new Error("Native acceptance receive budget exceeded");
        const joined = new Uint8Array(buffer.length + next.value.length);
        joined.set(buffer);
        joined.set(next.value, buffer.length);
        buffer = joined;
      }
    };
    let host: NativeAcceptance;
    try {
      const hello = (await deadline(readPacket(), timeoutMs)) as NativeAcceptanceCapabilities;
      if (
        hello.version !== 1 ||
        hello.mode !== options.mode ||
        typeof hello.screenshots !== "boolean" ||
        typeof hello.clock !== "boolean"
      )
        throw new Error("Native acceptance contract mismatch");
      host = new NativeAcceptance(process, hello, timeoutMs);
    } catch (error) {
      process.kill();
      await process.exited;
      await readErrors;
      throw error;
    }
    void (async () => {
      try {
        while (!host.closed) {
          const packet = await readPacket();
          const pending = host.pending;
          if (!pending) throw new Error("Unexpected native acceptance response");
          host.pending = undefined;
          pending.resolve(packet);
          if (host.closing) break;
        }
      } catch (error) {
        if (!host.closed) host.fail(error instanceof Error ? error : new Error(String(error)));
      } finally {
        reader.releaseLock();
      }
    })();
    void readErrors.catch((error) => host.fail(new Error(String(error))));
    return host;
  }

  async flush(): Promise<void> {
    await this.operation("flush");
  }
  async snapshot(surfaceId = 1): Promise<NativeSnapshot> {
    const snapshot = (await this.operation("snapshot", { surfaceId })) as Omit<NativeSnapshot, "nodes"> & {
      nodes: Omit<NativeTarget, "surfaceId" | "epoch" | "revision">[];
    };
    return Object.freeze({
      ...snapshot,
      nodes: Object.freeze(
        snapshot.nodes.map((node) => {
          const target = Object.freeze({
            ...node,
            surfaceId,
            epoch: snapshot.epoch,
            revision: snapshot.revision,
            bounds: Object.freeze(node.bounds),
            input: node.input === null ? null : Object.freeze(node.input),
          });
          this.targets.add(target);
          return target;
        }),
      ),
    });
  }
  async locate(locator: NativeLocator, surfaceId = 1): Promise<NativeTarget> {
    if (locator.id === undefined && locator.label === undefined && locator.text === undefined)
      throw new TypeError("A native locator requires id, label, or text");
    const matches = (await this.snapshot(surfaceId)).nodes.filter(
      (node) =>
        (locator.id === undefined || node.id === locator.id) &&
        (locator.label === undefined || node.label === locator.label) &&
        (locator.text === undefined || node.text === locator.text),
    );
    if (matches.length !== 1)
      throw new Error(`Native locator matched ${matches.length} painted nodes; require exactly one`);
    return matches[0]!;
  }
  async click(target: NativeTarget): Promise<void> {
    await this.action("click", target);
  }
  async drag(
    target: NativeTarget,
    destination: { x: number; y: number },
    options: { from?: { x: number; y: number } } = {},
  ): Promise<void> {
    await this.action("drag", target, { point: destination, from: options.from });
  }
  async wheel(target: NativeTarget, delta: { x: number; y: number }): Promise<void> {
    await this.action("wheel", target, { delta });
  }
  /** Types into the native focused editor after a hit-tested click. */
  async type(text: string, surfaceId = 1): Promise<void> {
    await this.operation("type", { text, surfaceId });
  }
  async key(keystroke: string, surfaceId = 1): Promise<void> {
    await this.operation("key", { text: keystroke, surfaceId });
  }
  async advanceClock(milliseconds: number): Promise<void> {
    await this.operation("advance-clock", { milliseconds });
  }
  async clipboardText(): Promise<string | null> {
    return (await this.operation("clipboard-text")) as string | null;
  }
  async screenshot(surfaceId = 1): Promise<NativeScreenshot> {
    const image = (await this.operation("screenshot", { surfaceId })) as {
      width: number;
      height: number;
      png: number[];
    };
    return Object.freeze({ width: image.width, height: image.height, png: Uint8Array.from(image.png) });
  }
  /** Unmount application roots first; closes every owned native window and process. */
  close(): Promise<NativeCleanup> {
    return (this.closePromise ??= this.closeOwned());
  }
  private async closeOwned(): Promise<NativeCleanup> {
    if (this.closed) {
      await this.exited;
      if (this.cleanup) return this.cleanup;
      throw new Error("Native acceptance closed without successful cleanup");
    }
    try {
      this.cleanup = Object.freeze((await this.operation("close")) as NativeCleanup);
    } finally {
      this.closed = true;
      this.listeners.clear();
      this.termination.clear();
      this.commits = [];
      this.commitBytes = 0;
      this.process.stdin.end();
      let code: number;
      try {
        code = await deadline(this.exited, this.timeoutMs);
      } catch {
        this.process.kill();
        await this.exited;
        throw new Error("Native acceptance cleanup timed out");
      }
      if (code !== 0) throw new Error(`Native acceptance cleanup exited with code ${code}`);
    }
    return this.cleanup!;
  }
  private async action(action: string, target: NativeTarget, data: Record<string, unknown> = {}): Promise<void> {
    if (!this.targets.has(target)) throw new TypeError("Use a target obtained from this NativeAcceptance instance");
    await this.operation(action, {
      ...data,
      surfaceId: target.surfaceId,
      target: {
        surfaceId: target.surfaceId,
        epoch: target.epoch,
        revision: target.revision,
        id: target.id,
        listenerId: target.listenerId,
      },
    });
  }
  private operation(action: string, data: Record<string, unknown> = {}): Promise<unknown> {
    const operation = this.serial.then(async () => {
      this.assertOpen();
      // Process feedback commits until Solid and native state are coherent. A
      // bounded loop reports non-settling application work rather than guessing.
      let result: unknown;
      let refreshObservation = false;
      for (let turn = 0; turn < 32; turn++) {
        await Promise.resolve();
        const frames = this.commits.map((frame) => Array.from(frame));
        this.commits = [];
        this.commitBytes = 0;
        const observing = turn === 0 || refreshObservation;
        const reply = await this.request(
          observing ? action : "flush",
          observing ? data : { surfaceId: data.surfaceId },
          frames,
        );
        for (const frame of reply.frames) for (const listener of this.listeners) listener(Uint8Array.from(frame));
        if (reply.error) throw new Error(reply.error);
        if (observing) result = reply.result;
        await Promise.resolve();
        if (this.commits.length === 0 || action === "close") {
          // Snapshot/screenshot must observe the acknowledged feedback paint.
          if (!observing && (action === "snapshot" || action === "screenshot")) {
            refreshObservation = true;
            continue;
          }
          return result;
        }
      }
      throw new Error("Native acceptance did not settle within 32 feedback turns");
    });
    this.serial = operation.catch(() => undefined);
    return operation;
  }
  private async request(action: string, data: Record<string, unknown>, frames: number[][]): Promise<Reply> {
    if (action === "close") this.closing = true;
    const id = ++this.sequence;
    const packet = encoder.encode(JSON.stringify({ id, action, ...data, frames }));
    if (packet.length > PACKET_LIMIT) throw new Error("Native acceptance request exceeds budget");
    const bytes = new Uint8Array(packet.length + 4);
    new DataView(bytes.buffer).setUint32(0, packet.length, true);
    bytes.set(packet, 4);
    const received = new Promise<unknown>((resolve, reject) => {
      this.pending = { resolve, reject };
    });
    try {
      this.process.stdin.write(bytes);
      await this.process.stdin.flush();
      const reply = (await deadline(received, this.timeoutMs)) as Reply;
      if (reply.id !== id || !Array.isArray(reply.frames))
        throw new Error("Native acceptance response identity mismatch");
      return reply;
    } catch (error) {
      this.fail(error instanceof Error ? error : new Error(String(error)));
      throw error;
    }
  }
  private assertOpen(): void {
    if (this.closed) throw new Error("Native acceptance is closed");
  }
  private fail(error: Error): void {
    if (this.closed) return;
    this.closed = true;
    this.pending?.reject(error);
    this.pending = undefined;
    const terminated = new TransportTerminatedError(error.message, { kind: "io", detail: error.message });
    for (const listener of this.termination) listener(terminated);
    this.listeners.clear();
    this.termination.clear();
    this.commits = [];
    this.commitBytes = 0;
    this.process.kill();
  }
}
async function deadline<T>(promise: Promise<T>, milliseconds: number): Promise<T> {
  let timer: ReturnType<typeof setTimeout>;
  try {
    return await Promise.race([
      promise,
      new Promise<never>((_, reject) => {
        timer = setTimeout(() => reject(new Error("Native acceptance timed out")), milliseconds);
      }),
    ]);
  } finally {
    clearTimeout(timer!);
  }
}
