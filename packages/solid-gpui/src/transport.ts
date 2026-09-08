import type { Event } from "./protocol";

export type TransportChunk = Uint8Array | ArrayBuffer;
export type TransportListener = (chunk: Uint8Array) => void;
export type SemanticEventListener = (event: Event) => void;
export type TransportTerminationCause =
  | { readonly kind: "shutdown" }
  | { readonly kind: "eof" }
  | { readonly kind: "exit"; readonly code: number }
  | { readonly kind: "protocol"; readonly detail: string }
  | { readonly kind: "io"; readonly detail: string };

function isTransportTerminationCause(value: unknown): value is TransportTerminationCause {
  if (value === null || typeof value !== "object") return false;
  const candidate = value as { kind?: unknown; code?: unknown; detail?: unknown };
  if (candidate.kind === "shutdown" || candidate.kind === "eof") return true;
  if (candidate.kind === "exit") return typeof candidate.code === "number" && Number.isInteger(candidate.code);
  return (
    (candidate.kind === "protocol" || candidate.kind === "io") &&
    typeof candidate.detail === "string" &&
    candidate.detail.length > 0
  );
}

export class TransportTerminatedError extends Error {
  readonly cause?: TransportTerminationCause;
  readonly exitCode?: number;
  readonly stderrTail?: string;
  readonly crashReportPath?: string;

  constructor(message: string, cause?: unknown, details: TransportTerminationDetails = {}) {
    super(message);
    this.name = "TransportTerminatedError";
    this.cause =
      cause === undefined
        ? undefined
        : isTransportTerminationCause(cause)
          ? cause
          : { kind: "io", detail: describeError(cause) };
    this.exitCode = details.exitCode;
    this.stderrTail = details.stderrTail;
    this.crashReportPath = details.crashReportPath;
  }
}

export interface TransportTerminationDetails {
  readonly exitCode?: number;
  readonly stderrTail?: string;
  readonly crashReportPath?: string;
}
export type TransportTerminationListener = (error: TransportTerminatedError) => void;

export interface Transport {
  /** False means accepted under pressure; resume only after onDrain. */
  submit(frame: Uint8Array): boolean;
  onDrain(listener: () => void): () => void;
  onData(listener: TransportListener): () => void;
  onTermination(listener: TransportTerminationListener): () => void;
}
export interface DisposableTransport extends Transport {
  dispose(): void;
}

export interface SemanticEventTransport extends Transport {
  onEvent(listener: SemanticEventListener): () => void;
}

function describeError(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function asBytes(chunk: TransportChunk): Uint8Array {
  return chunk instanceof Uint8Array ? chunk : new Uint8Array(chunk);
}

export class MemoryTransport implements DisposableTransport {
  readonly submitted: Uint8Array[] = [];
  private readonly listeners = new Set<TransportListener>();

  submit(frame: Uint8Array): boolean {
    this.submitted.push(frame.slice());
    return true;
  }

  onDrain(_listener: () => void): () => void {
    return () => undefined;
  }

  onData(listener: TransportListener): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  onTermination(_listener: TransportTerminationListener): () => void {
    return () => undefined;
  }

  dispose(): void {
    this.listeners.clear();
  }

  push(chunk: TransportChunk): void {
    const bytes = asBytes(chunk);
    for (const listener of this.listeners) listener(bytes);
  }
}
