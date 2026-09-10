import { TraceMap, originalPositionFor } from "@jridgewell/trace-mapping";
import { build, DevEnvironment, type ResolvedConfig, type ViteDevServer } from "vite";
import { createServer, type Server, type Socket } from "node:net";
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import type { NativeHostOptions, StdioConfig } from "./environment.ts";
import { DevSession, type DevSessionOptions } from "./dev-session.ts";

/** Vite watch builds feed whole applications to the existing QuickJS reload seam. */
export class QuickJsDevEnvironment extends DevEnvironment {
  private directory?: string;
  private control?: Server;
  private socket?: Socket;
  private child?: ReturnType<typeof Bun.spawn>;
  private watcher?: Awaited<ReturnType<typeof build>> & { close(): Promise<void> };
  private closeTask?: Promise<void>;
  private stopTask?: Promise<void>;
  private bundleTask: Promise<void> = Promise.resolve();
  private rejectStartup?: (error: unknown) => void;
  private startupTimeout?: ReturnType<typeof setTimeout>;
  private stopped = false;
  private publishTask?: Promise<void>;
  private pending?: Uint8Array;
  private connected?: () => void;
  private connection?: Promise<void>;
  private readonly maps = new Map<number, TraceMap>();
  private sent = 0;
  session?: DevSession;

  constructor(
    name: string,
    config: ResolvedConfig,
    private readonly options: DevSessionOptions,
  ) {
    super(name, config, { hot: false });
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
    try {
      signal.throwIfAborted();
      this.stopped = false;
      this.connection = new Promise<void>((resolve) => {
        this.connected = resolve;
      });
      signal.addEventListener(
        "abort",
        () => {
          this.stopped = true;
          this.rejectStartup?.(new Error("QuickJS session stopped during startup"));
          this.connected?.();
          this.child?.kill();
          this.socket?.destroy();
        },
        { once: true },
      );
      this.directory = await mkdtemp(join(tmpdir(), "solid-gpui-quickjs-"));
      signal.throwIfAborted();
      this.control = createServer((peer) => {
        if (this.socket) {
          peer.destroy();
          return;
        }
        this.socket = peer;
        this.connected?.();
        let tail = "";
        peer.on("data", (data) => {
          tail += data.toString();
          if (tail.length > 8192) {
            peer.destroy(new Error("Reload diagnostic exceeds budget"));
            return;
          }
          for (;;) {
            const end = tail.indexOf("\n");
            if (end < 0) break;
            const message = tail
              .slice(0, end)
              .replace(/([^ ()]+)\?generation=(\d+):(\d+):(\d+)/g, (location, _file, generation, line, column) => {
                const map = this.maps.get(Number(generation));
                if (!map) return location;
                const original = originalPositionFor(map, {
                  line: Number(line),
                  column: Math.max(0, Number(column) - 1),
                });
                return original.source && original.line !== null
                  ? `${original.source}:${original.line}:${(original.column ?? 0) + 1}`
                  : location;
              });
            server.config.logger.info(`solid-gpui: ${message}`);
            tail = tail.slice(end + 1);
          }
        });
        peer.on("error", (error) => {
          onExit(new Error(`Reload connection failed: ${error.message}`));
        });
        peer.on("close", () => {
          if (!signal.aborted) this.rejectStartup?.(new Error("QuickJS reload connection closed during startup"));
        });
      });
      await new Promise<void>((resolve, reject) => {
        this.control!.once("error", reject);
        this.control!.listen(0, "127.0.0.1", resolve);
      });
      signal.throwIfAborted();
      const address = this.control.address();
      if (!address || typeof address === "string") throw new Error("Missing QuickJS reload endpoint");
      const buildConfig: StdioConfig = {
        ...server.config.inlineConfig,
        __solidGpuiPreparedHost: host,
        root: server.config.root,
        configFile: server.config.configFile || false,
        mode: server.config.mode,
        build: {
          outDir: this.directory,
          emptyOutDir: false,
          sourcemap: "inline",
          watch: {},
          rolldownOptions: { output: { entryFileNames: "app.js" } },
        },
      };
      const result = await build(buildConfig);
      if (!("on" in result)) throw new Error("QuickJS development requires a Vite build watcher");
      this.watcher = result;
      signal.throwIfAborted();
      let ready!: () => void;
      let failed!: (error: unknown) => void;
      const started = new Promise<void>((resolve, reject) => {
        ready = resolve;
        failed = reject;
      });
      this.rejectStartup = failed;
      result.on("event", (event) => {
        if (event.code === "ERROR") {
          server.config.logger.error(`solid-gpui: build rejected: ${String(event.error)}`);
          if (!this.child) failed(event.error);
        }
        if (event.code !== "BUNDLE_END") return;
        this.bundleTask = this.bundleTask
          .then(async () => {
            try {
              if (this.stopped) return;
              const bytes = await readFile(join(this.directory!, "app.js"));
              if (this.stopped) return;
              if (bytes.length > 32 * 1024 * 1024) throw new Error("Reload bundle exceeds 32 MiB");
              if (!this.child) {
                const initial = join(this.directory!, "initial.js");
                await writeFile(initial, bytes);
                if (this.stopped) return;
                this.child = Bun.spawn(
                  [
                    host.command,
                    ...(host.args ?? []),
                    "--runtime",
                    "quickjs-dev",
                    initial,
                    `127.0.0.1:${address.port}`,
                  ],
                  {
                    cwd: server.config.root,
                    stdin: "ignore",
                    stdout: "inherit",
                    stderr: "inherit",
                  },
                );
                void this.child.exited.then((code) => {
                  if (signal.aborted) return;
                  const error = new Error(`QuickJS host exited with status ${code}`);
                  failed(error);
                  onExit(code === 0 ? undefined : error);
                });
                this.startupTimeout = setTimeout(
                  () => failed(new Error("QuickJS host did not connect within 30 seconds")),
                  30_000,
                );
                void this.connection!.then(ready);
              } else {
                this.pending = bytes;
                this.publishTask ??= this.publish()
                  .catch((error) => {
                    if (!signal.aborted) onExit(error instanceof Error ? error : new Error(String(error)));
                  })
                  .finally(() => {
                    this.publishTask = undefined;
                  });
              }
            } catch (error) {
              if (!this.child) failed(error);
              else server.config.logger.error(`solid-gpui: build rejected: ${String(error)}`);
            } finally {
              await event.result.close();
            }
          })
          .catch((error) => {
            failed(error);
            if (!signal.aborted) onExit(error instanceof Error ? error : new Error(String(error)));
          });
      });
      await started;
    } finally {
      clearTimeout(this.startupTimeout);
      this.rejectStartup = undefined;
    }
  }

  private async publish(): Promise<void> {
    await this.connection;
    while (this.pending && !this.stopped) {
      const bytes = this.pending;
      this.pending = undefined;
      const source = new TextDecoder().decode(bytes);
      const encoded = source.match(/sourceMappingURL=data:application\/json(?:;charset=utf-8)?;base64,([^\s]+)/)?.[1];
      this.sent++;
      if (encoded) this.maps.set(this.sent, new TraceMap(JSON.parse(Buffer.from(encoded, "base64").toString())));
      for (const generation of this.maps.keys()) if (generation < this.sent - 2) this.maps.delete(generation);
      const frame = Buffer.allocUnsafe(4 + bytes.length);
      frame.writeUInt32LE(bytes.length);
      frame.set(bytes, 4);
      await new Promise<void>((resolve, reject) =>
        this.socket!.write(frame, (error) => (error ? reject(error) : resolve())),
      );
    }
  }

  private stopSession(): Promise<void> {
    return (this.stopTask ??= (async () => {
      this.stopped = true;
      this.rejectStartup?.(new Error("QuickJS environment closed during startup"));
      clearTimeout(this.startupTimeout);
      this.connected?.();
      this.child?.kill();
      this.socket?.destroy();
      await this.child?.exited;
      await this.watcher?.close();
      await this.bundleTask;
      await this.publishTask;
      await new Promise<void>((resolve) => (this.control ? this.control.close(() => resolve()) : resolve()));
      if (this.directory) await rm(this.directory, { recursive: true, force: true });
      this.child = undefined;
      this.socket = undefined;
      this.control = undefined;
      this.watcher = undefined;
      this.directory = undefined;
      this.pending = undefined;
      this.connection = undefined;
      this.bundleTask = Promise.resolve();
      this.maps.clear();
      this.sent = 0;
    })().finally(() => {
      this.stopTask = undefined;
    }));
  }

  override async close(): Promise<void> {
    return (this.closeTask ??= (async () => {
      await this.session?.close();
      await super.close();
    })());
  }
}
