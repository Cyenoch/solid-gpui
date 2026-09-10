import { EventEmitter } from "node:events";
import { fileURLToPath } from "node:url";
import { extname } from "node:path";
import {
  createRunnableDevEnvironment,
  DevEnvironment,
  type InlineConfig,
  type HotChannel,
  type HotPayload,
  type ResolvedConfig,
  type ViteDevServer,
} from "vite";
import { DevSession, type DevSessionOptions } from "./dev-session.ts";

export interface StdioConfig extends InlineConfig {
  readonly __solidGpuiStdio?: { readonly nativeHost?: string };
  /** The enclosing QuickJS dev session already built and exported this host. */
  readonly __solidGpuiPreparedHost?: NativeHostOptions;
}

export interface NativeHostOptions {
  readonly command: string;
  readonly args?: readonly string[];
}

/** Vite owns the module channel; Rust owns the framed-protocol JS child. */
export class NativeDevEnvironment extends DevEnvironment {
  private readonly messages: EventEmitter;
  private endpoint?: ReturnType<typeof Bun.serve>;
  private peer?: Bun.ServerWebSocket<unknown>;
  private child?: ReturnType<typeof Bun.spawn>;
  private closeTask?: Promise<void>;
  private stopTask?: Promise<void>;
  private rejectStartup?: (error: Error) => void;
  session?: DevSession;

  constructor(
    name: string,
    config: ResolvedConfig,
    private readonly entry: string,
    private readonly options: DevSessionOptions,
  ) {
    const messages = new EventEmitter();
    let send: HotChannel["send"];
    super(name, config, {
      hot: true,
      transport: {
        // Only our authenticated local child can use this channel.
        skipFsCheck: true,
        send: (payload) => send?.(payload),
        on: (event: string, listener: (...args: any[]) => void) => {
          messages.on(event, listener);
        },
        off: (event: string, listener: (...args: any[]) => void) => {
          messages.off(event, listener);
        },
      },
    });
    this.messages = messages;
    send = (payload) => this.peer?.send(JSON.stringify(payload));
  }

  override async listen(server: ViteDevServer): Promise<void> {
    if (typeof Bun === "undefined") throw new Error("Native Vite development requires Bun: bun --bun vite");
    await super.listen(server);
    this.session = new DevSession(server, this, this.options, {
      start: (host, signal, ended) => this.start(server, host, signal, ended),
      stop: () => this.stopSession(),
    });
    await this.session.listen();
  }

  private async start(
    server: ViteDevServer,
    host: NativeHostOptions,
    signal: AbortSignal,
    onExit: (error?: Error) => void,
  ): Promise<void> {
    signal.throwIfAborted();
    const token = crypto.randomUUID();
    let timeout: ReturnType<typeof setTimeout> | undefined;
    try {
      await new Promise<void>((ready, failed) => {
        this.rejectStartup = failed;
        const fail = (error: Error) => {
          failed(error);
          onExit(error);
        };
        signal.addEventListener(
          "abort",
          () => {
            failed(new Error("Native session stopped during startup"));
            this.child?.kill();
            this.peer?.close();
          },
          { once: true },
        );
        this.endpoint = Bun.serve({
          hostname: "127.0.0.1",
          port: 0,
          fetch: (request, endpoint) => {
            if (new URL(request.url).pathname !== `/${token}` || this.peer) return new Response(null, { status: 403 });
            if (endpoint.upgrade(request, { data: undefined })) return;
            return new Response(null, { status: 400 });
          },
          websocket: {
            maxPayloadLength: 32 * 1024 * 1024,
            backpressureLimit: 32 * 1024 * 1024,
            closeOnBackpressureLimit: true,
            open: (peer) => {
              if (this.peer) {
                peer.close();
                return;
              }
              this.peer = peer;
              this.messages.emit("connection");
            },
            message: (peer, bytes) => {
              try {
                const payload = JSON.parse(String(bytes)) as HotPayload;
                if (payload.type === "ping") return;
                if (payload.type !== "custom") throw new Error("Expected a Vite custom message");
                if (payload.event === "solid-gpui:ready") ready();
                else if (payload.event === "solid-gpui:error") fail(new Error(String(payload.data)));
                else
                  this.messages.emit(payload.event, payload.data, {
                    send: (reply: HotPayload) => peer.send(JSON.stringify(reply)),
                  });
              } catch (error) {
                fail(error instanceof Error ? error : new Error(String(error)));
                peer.close(1008, "Invalid module channel message");
              }
            },
            close: () => {
              if (!signal.aborted) {
                failed(new Error("Native module runner disconnected during startup"));
              }
            },
          },
        });
        timeout = setTimeout(
          () => failed(new Error("Native host did not start its Vite module runner within 30 seconds")),
          30_000,
        );
        const runner = [
          process.execPath,
          "--conditions=browser",
          fileURLToPath(new URL("./runner" + extname(fileURLToPath(import.meta.url)), import.meta.url)),
          `ws://127.0.0.1:${this.endpoint.port}/${token}`,
          this.entry,
        ];
        this.child = Bun.spawn([host.command, ...(host.args ?? [])], {
          cwd: server.config.root,
          env: { ...process.env, SOLID_GPUI_VITE_RUNNER: JSON.stringify(runner) },
          stdin: "ignore",
          stdout: "inherit",
          stderr: "inherit",
        });
        void this.child.exited.then((code) => {
          if (signal.aborted) return;
          if (code === 0) {
            failed(new Error("Native host closed during startup"));
            onExit();
          } else fail(new Error(`Native host exited with status ${code}`));
        });
      });
    } finally {
      clearTimeout(timeout);
      this.rejectStartup = undefined;
    }
  }

  private stopSession(): Promise<void> {
    return (this.stopTask ??= (async () => {
      this.rejectStartup?.(new Error("Native environment closed during startup"));
      this.child?.kill();
      this.peer?.close();
      await this.endpoint?.stop(true);
      // The runner also exits on module-channel closure and native stdin EOF.
      await this.child?.exited;
      this.peer = undefined;
      this.endpoint = undefined;
      this.child = undefined;
    })().finally(() => {
      this.stopTask = undefined;
    }));
  }

  override async close(): Promise<void> {
    return (this.closeTask ??= (async () => {
      await this.session?.close();
      this.messages.removeAllListeners();
      await super.close();
    })());
  }
}

export function createStdioEnvironment(
  name: string,
  config: ResolvedConfig,
  entry: string,
  prepare: () => Promise<unknown>,
) {
  const environment = createRunnableDevEnvironment(name, config);
  const listen = environment.listen.bind(environment);
  environment.listen = async (server) => {
    try {
      await prepare();
      await listen(server);
      await environment.runner.import(entry);
    } catch (error) {
      await server.close();
      throw error;
    }
  };
  return environment;
}
