import { closeSync, openSync, writeSync } from "node:fs";
import { MAX_FRAME_SIZE, classifyPayload } from "./protocol";

export const PROTOCOL_TAP_CAPACITY_BYTES = 64 * 1024 * 1024;
const TAP_STOP_RESERVE_BYTES = 256;

interface Classification {
  readonly kind: "snapshot" | "patch" | "event" | "command" | "unknown";
  readonly event_type?: number;
  readonly command_kind?: number;
  readonly request_id?: number;
  readonly success?: boolean;
}

type Direction = "in" | "out";
type Peer = "host" | "renderer";

/** Process-local JSONL observer for framed protocol traffic. */
export class ProtocolTap {
  private readonly file: number;
  private readonly started = performance.now();
  private bytesWritten = 0;
  private sequence = 1;
  private stopped = false;
  private closed = false;
  private inboundBuffer = new Uint8Array(0);

  private constructor(
    private readonly capacityBytes: number,
    path: string,
  ) {
    try {
      this.file = openSync(path, "w", 0o600);
    } catch (error) {
      console.error(`solid-gpui: protocol tap disabled; cannot open ${path}: ${String(error)}`);
      throw error;
    }
  }

  static fromEnv(): ProtocolTap | undefined {
    const path = process.env.SOLID_GPUI_TAP;
    if (path === undefined || path === "") return undefined;
    try {
      return new ProtocolTap(PROTOCOL_TAP_CAPACITY_BYTES, path);
    } catch {
      return undefined;
    }
  }

  static openForTest(path: string, capacityBytes: number): ProtocolTap {
    if (!Number.isSafeInteger(capacityBytes) || capacityBytes < 1) {
      throw new RangeError("protocol tap capacity must be a positive integer");
    }
    return new ProtocolTap(capacityBytes, path);
  }

  get enabled(): boolean {
    return !this.stopped;
  }

  recordOutboundFrame(frame: Uint8Array): void {
    if (this.stopped) return;
    this.record("out", "host", frame.subarray(4), frame.byteLength);
  }

  observeInbound(chunk: Uint8Array): void {
    if (this.stopped || chunk.byteLength === 0) return;
    const combined = new Uint8Array(this.inboundBuffer.byteLength + chunk.byteLength);
    combined.set(this.inboundBuffer);
    combined.set(chunk, this.inboundBuffer.byteLength);
    this.inboundBuffer = combined;
    while (this.inboundBuffer.byteLength >= 4 && !this.stopped) {
      const size = new DataView(this.inboundBuffer.buffer, this.inboundBuffer.byteOffset, 4).getUint32(0, true);
      if (size > MAX_FRAME_SIZE) {
        this.record("in", "host", new Uint8Array(0), 4, { kind: "unknown" });
        this.inboundBuffer = new Uint8Array(0);
        return;
      }
      const frameSize = size + 4;
      if (this.inboundBuffer.byteLength < frameSize) return;
      const frame = this.inboundBuffer.subarray(0, frameSize);
      this.record("in", "host", frame.subarray(4), frame.byteLength);
      this.inboundBuffer =
        frameSize === this.inboundBuffer.byteLength ? new Uint8Array(0) : this.inboundBuffer.subarray(frameSize);
    }
  }

  dispose(): void {
    if (this.closed) return;
    closeSync(this.file);
    this.closed = true;
    this.stopped = true;
    this.inboundBuffer = new Uint8Array(0);
  }

  private record(
    direction: Direction,
    peer: Peer,
    payload: Uint8Array,
    frameBytes: number,
    override?: Classification,
  ): void {
    if (this.stopped) return;
    const classification = override ?? classify(payload);
    const timestamp = Math.floor(performance.now() - this.started);
    const line =
      JSON.stringify({
        t: timestamp,
        dir: direction,
        peer,
        kind: classification.kind,
        bytes: frameBytes,
        seq: this.sequence,
        ...(classification.event_type === undefined ? {} : { event_type: classification.event_type }),
        ...(classification.command_kind === undefined ? {} : { command_kind: classification.command_kind }),
        ...(classification.request_id === undefined ? {} : { request_id: classification.request_id }),
        ...(classification.success === undefined ? {} : { success: classification.success }),
      }) + "\n";
    if (this.bytesWritten + line.length > this.capacityBytes - TAP_STOP_RESERVE_BYTES) {
      this.writeStopped(direction, peer, frameBytes, timestamp);
      return;
    }
    try {
      writeSync(this.file, line, undefined, "utf8");
      this.bytesWritten += line.length;
      this.sequence += 1;
    } catch {
      this.stopped = true;
    }
  }

  private writeStopped(direction: Direction, peer: Peer, frameBytes: number, timestamp: number): void {
    const line =
      JSON.stringify({
        t: timestamp,
        dir: direction,
        peer,
        kind: "tap_stopped",
        bytes: frameBytes,
        seq: this.sequence,
        reason: "capacity",
      }) + "\n";
    if (this.bytesWritten + line.length <= this.capacityBytes) {
      try {
        writeSync(this.file, line, undefined, "utf8");
        this.bytesWritten += line.length;
        this.sequence += 1;
      } catch {
        // A tap failure is deliberately isolated from transport operation.
      }
    }
    this.stopped = true;
  }
}

function classify(payload: Uint8Array): Classification {
  return classifyPayload(payload);
}
