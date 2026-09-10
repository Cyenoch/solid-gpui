import { dirname, resolve, sep } from "node:path";
import type { DevEnvironment, ViteDevServer } from "vite";
import type { NativeHostOptions } from "./environment.ts";
import type { NativeExportOptions } from "./native-export.ts";
import { isNativeInput, nativeWatchInputs, type NativeWatchInputs } from "./native-watch.ts";

export interface DevSessionOptions {
  readonly native?: NativeExportOptions;
  readonly prepare: (signal: AbortSignal) => Promise<NativeHostOptions>;
}

interface SessionRuntime {
  start(host: NativeHostOptions, signal: AbortSignal, ended: (error?: Error) => void): Promise<void>;
  stop(): Promise<void>;
}

/** One watcher outlives failed applications; only a new edit requests another attempt. */
export class DevSession {
  private inputs: NativeWatchInputs;
  private revision = 0;
  private active = false;
  private closed = false;
  private controller?: AbortController;
  private task?: Promise<void>;
  private timer?: ReturnType<typeof setTimeout>;
  private host?: NativeHostOptions;

  constructor(
    private readonly server: ViteDevServer,
    private readonly environment: DevEnvironment,
    private readonly options: DevSessionOptions,
    private readonly runtime: SessionRuntime,
  ) {
    this.inputs = {
      roots: options.native ? [dirname(resolve(server.config.root, options.native.manifestPath))] : [],
      configuration: [],
    };
  }

  async listen(): Promise<void> {
    this.server.watcher.add(this.inputs.roots);
    this.server.watcher.on("all", this.changed);
    this.revision++;
    await this.run();
  }

  /** Generated bindings must never reach the previous native session through HMR. */
  suppressUpdate(file: string): boolean {
    return !this.active || isNativeInput(resolve(file), this.inputs) || this.isGenerated(resolve(file));
  }

  private isGenerated(file: string): boolean {
    const output = this.options.native && resolve(this.server.config.root, this.options.native.output);
    return !!output && (file === output || file.startsWith(output + "."));
  }

  private changed = (event: string, path: string): void => {
    if (this.closed || !["add", "change", "unlink"].includes(event)) return;
    const file = resolve(path);
    const target = this.inputs.targetDirectory;
    if (target && (file === target || file.startsWith(target + sep))) return;
    if (this.isGenerated(file) || /[/\\](?:target|node_modules|\.git)[/\\]/.test(file)) return;
    const native = isNativeInput(file, this.inputs);
    const source = /\.[cm]?[jt]sx?$/.test(file) || !!this.environment.moduleGraph.getModulesByFile(file)?.size;
    if (!native && (this.active || !source)) return;
    this.active = false;
    this.revision++;
    // Revoke the old connection immediately, before Vite can publish another update.
    this.controller?.abort();
    clearTimeout(this.timer);
    this.timer = setTimeout(() => {
      void this.run();
    }, 100);
  };

  private run(): Promise<void> {
    if (this.task) return this.task;
    this.task = this.replace().finally(() => {
      this.task = undefined;
    });
    return this.task;
  }

  private async replace(): Promise<void> {
    let attempted = -1;
    while (!this.closed && attempted !== this.revision) {
      clearTimeout(this.timer);
      attempted = this.revision;
      this.active = false;
      this.host = undefined;
      this.controller?.abort();
      await this.runtime.stop();
      if (this.closed) return;
      const controller = new AbortController();
      this.controller = controller;
      const { signal } = controller;
      try {
        if (this.options.native) {
          this.server.config.logger.info("solid-gpui: rebuilding native host and bindings...");
          this.inputs = await nativeWatchInputs(this.options.native, this.server.config.root, signal);
          this.server.watcher.add([...this.inputs.roots, ...this.inputs.configuration]);
        }
        this.host = await this.options.prepare(signal);
        signal.throwIfAborted();
        this.environment.moduleGraph.invalidateAll();
        await this.runtime.start(this.host, signal, (error) => {
          if (signal.aborted) return;
          this.active = false;
          controller.abort();
          if (error) this.report(error);
          else
            this.server.config.logger.info(
              "solid-gpui: application closed. Watching for changes; save source to reopen.",
            );
          void this.runtime.stop();
        });
        signal.throwIfAborted();
        this.active = true;
        this.server.config.logger.info(`solid-gpui: native session ready (${this.host.command})`);
      } catch (error) {
        if (!signal.aborted) this.report(error);
        controller.abort();
        await this.runtime.stop();
      }
    }
  }

  private report(error: unknown): void {
    const host = this.host ? `\nHost: ${this.host.command}` : "";
    this.server.config.logger.error(
      `solid-gpui: ${error instanceof Error ? error.message : String(error)}${host}\nWatching for changes. Fix the error and save to start a new session.`,
    );
  }

  async close(): Promise<void> {
    this.closed = true;
    clearTimeout(this.timer);
    this.server.watcher.off("all", this.changed);
    this.controller?.abort();
    await this.task;
    await this.runtime.stop();
  }
}
