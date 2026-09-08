import { TraceMap, originalPositionFor } from "@jridgewell/trace-mapping";
import { build, DevEnvironment, type ResolvedConfig, type ViteDevServer } from "vite";
import { createServer, type Server, type Socket } from "node:net";
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import type { NativeHostOptions } from "./environment.ts";

/** Vite watch builds feed whole applications to the existing QuickJS reload seam. */
export class QuickJsDevEnvironment extends DevEnvironment {
  private directory?: string;
  private control?: Server;
  private socket?: Socket;
  private child?: ReturnType<typeof Bun.spawn>;
  private watcher?: Awaited<ReturnType<typeof build>> & { close(): Promise<void> };
  private closeTask?: Promise<void>;
  private startTask?: Promise<void>;
  private bundleTask: Promise<void> = Promise.resolve();
  private rejectStartup?: (error: unknown) => void;
  private startupTimeout?: ReturnType<typeof setTimeout>;
  private stopped = false;
  private publishTask?: Promise<void>;
  private pending?: Uint8Array;
  private connected?: () => void;
  private readonly connection = new Promise<void>((resolve) => {
    this.connected = resolve;
  });
  private readonly maps = new Map<number, TraceMap>();
  private sent = 0;

  constructor(
    name: string,
    config: ResolvedConfig,
    private readonly prepare: () => Promise<NativeHostOptions>,
  ) {
    super(name, config, { hot: false });
  }

  override async listen(server: ViteDevServer): Promise<void> {
    this.startTask = this.start(server);
    try {
      await this.startTask;
    } catch (error) {
      await server.close();
      throw error;
    }
  }

  private async start(server: ViteDevServer): Promise<void> {
    try {
      if (typeof Bun === "undefined") throw new Error("Native Vite development requires Bun: bun --bun vite");
      const host = await this.prepare();
      if (this.stopped) return;
      this.directory = await mkdtemp(join(tmpdir(), "solid-gpui-quickjs-"));
      if (this.stopped) return;
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
          server.config.logger.error(`Reload connection failed: ${error.message}`);
          void server.close();
        });
      });
      await new Promise<void>((resolve, reject) => {
        this.control!.once("error", reject);
        this.control!.listen(0, "127.0.0.1", resolve);
      });
      if (this.stopped) return;
      const address = this.control.address();
      if (!address || typeof address === "string") throw new Error("Missing QuickJS reload endpoint");
      const result = await build({
        ...server.config.inlineConfig,
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
      });
      if (!("on" in result)) throw new Error("QuickJS development requires a Vite build watcher");
      this.watcher = result;
      if (this.stopped) return;
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
                  if (this.stopped) return;
                  failed(new Error(`QuickJS host exited with status ${code}`));
                  process.exitCode = code;
                  void server.close();
                });
                this.startupTimeout = setTimeout(
                  () => failed(new Error("QuickJS host did not connect within 30 seconds")),
                  30_000,
                );
                void this.connection.then(ready);
              } else {
                this.pending = bytes;
                this.publishTask ??= this.publish()
                  .catch((error) => {
                    server.config.logger.error(String(error));
                    void server.close();
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
            server.config.logger.error(String(error));
            void server.close();
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

  override async close(): Promise<void> {
    return (this.closeTask ??= (async () => {
      this.stopped = true;
      this.rejectStartup?.(new Error("QuickJS environment closed during startup"));
      clearTimeout(this.startupTimeout);
      this.connected?.();
      await this.startTask?.catch(() => {});
      this.socket?.destroy();
      this.child?.kill();
      await this.child?.exited;
      await this.watcher?.close();
      await this.bundleTask;
      await this.publishTask;
      await new Promise<void>((resolve) => (this.control ? this.control.close(() => resolve()) : resolve()));
      if (this.directory) await rm(this.directory, { recursive: true, force: true });
      await super.close();
    })());
  }
}
