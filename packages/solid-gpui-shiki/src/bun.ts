import type { BunHighlighterOptions, WorkerRequest, WorkerResponse } from "./messages";
import { validateRequest, type CodeHighlighter, type HighlightRequest, type HighlightResult } from "./types";

export type { BunHighlighterOptions } from "./messages";

interface Job {
  readonly id: number;
  readonly request: HighlightRequest;
  readonly resolve: (result: HighlightResult) => void;
  readonly reject: (error: Error) => void;
  cleanup: () => void;
}

/** Creates one persistent Bun worker using Shiki's Oniguruma engine. */
export async function createBunHighlighter(options: BunHighlighterOptions): Promise<CodeHighlighter> {
  if (typeof Bun === "undefined") throw new Error("createBunHighlighter requires Bun");
  if (
    !options.languages.length ||
    options.languages.length > 32 ||
    !options.themes.length ||
    options.themes.length > 8
  ) {
    throw new Error("Load 1..32 languages and 1..8 themes per highlighter");
  }
  const timeoutMs = options.timeoutMs ?? 30_000;
  if (!Number.isSafeInteger(timeoutMs) || timeoutMs < 1 || timeoutMs > 300_000) {
    throw new Error("timeoutMs must be an integer between 1 and 300000");
  }
  const worker = new Worker(options.workerURL ?? import.meta.resolve("@solid-gpui/shiki/worker"));
  const queue: Job[] = [];
  let active: Job | undefined;
  let closed: Error | undefined;
  let sequence = 0;
  let ready = false;
  let timer: ReturnType<typeof setTimeout>;
  let resolveReady: () => void;
  let rejectReady: (error: Error) => void;
  const initialized = new Promise<void>((resolve, reject) => {
    resolveReady = resolve;
    rejectReady = reject;
  });

  const stop = (error: Error): void => {
    if (closed) return;
    closed = error;
    clearTimeout(timer);
    worker.terminate();
    rejectReady(error);
    for (const job of [...(active ? [active] : []), ...queue]) {
      job.cleanup();
      job.reject(error);
    }
    active = undefined;
    queue.length = 0;
  };
  const armDeadline = (): void => {
    clearTimeout(timer);
    timer = setTimeout(() => stop(new Error("Shiki worker deadline exceeded; create a new highlighter")), timeoutMs);
  };
  const send = (message: WorkerRequest): void => {
    try {
      worker.postMessage(message);
    } catch (error) {
      stop(error instanceof Error ? error : new Error(String(error)));
    }
  };
  const pump = (): void => {
    if (!ready || active || closed) return;
    active = queue.shift();
    if (!active) return;
    armDeadline();
    send({ type: "highlight", id: active.id, request: active.request });
  };
  worker.onmessage = (event: MessageEvent<WorkerResponse>) => {
    const message = event.data;
    if (message.type === "fatal") {
      stop(new Error(message.error));
      return;
    }
    if (message.type === "ready") {
      clearTimeout(timer);
      ready = true;
      resolveReady();
      return;
    }
    if (!active || active.id !== message.id) return;
    clearTimeout(timer);
    const job = active;
    active = undefined;
    job.cleanup();
    if (message.type === "result") job.resolve(message.result);
    else job.reject(new Error(message.error));
    pump();
  };
  worker.onerror = (event) => stop(new Error(`Shiki worker failed: ${event.message}`));
  worker.addEventListener("close", () => stop(new Error("Shiki worker closed")));
  armDeadline();
  send({ type: "init", options: { languages: options.languages, themes: options.themes } });
  await initialized;

  return {
    highlight(request, { signal } = {}) {
      return new Promise<HighlightResult>((resolve, reject) => {
        if (closed) {
          reject(closed);
          return;
        }
        if (signal?.aborted) {
          reject(new DOMException("Highlighting aborted", "AbortError"));
          return;
        }
        try {
          validateRequest(request);
        } catch (error) {
          reject(error);
          return;
        }
        if (queue.length >= 32) {
          reject(new Error("Shiki request queue is full"));
          return;
        }
        const job: Job = { id: ++sequence, request: { ...request }, resolve, reject, cleanup: () => {} };
        const abort = (): void => {
          const index = queue.indexOf(job);
          if (index >= 0) queue.splice(index, 1);
          job.cleanup();
          reject(new DOMException("Highlighting aborted", "AbortError"));
          // An active match keeps its deadline and worker slot until completion.
          // Aborting one consumer must not destroy another consumer's service.
        };
        if (signal) {
          signal.addEventListener("abort", abort, { once: true });
          job.cleanup = () => signal.removeEventListener("abort", abort);
        }
        queue.push(job);
        pump();
      });
    },
    dispose() {
      stop(new Error("Shiki highlighter disposed"));
    },
  };
}
