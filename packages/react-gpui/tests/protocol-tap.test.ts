import { describe, expect, it } from "bun:test";
import { readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import {
  StdioTransport,
  type ByteInput,
  type ByteInputEventListener,
  type ByteInputListener,
  type ByteOutput,
  type ByteOutputEventListener,
} from "../src/transport";
import { ProtocolTap } from "../src/protocol-tap";

class FakeInput implements ByteInput {
  private listener: ByteInputListener | undefined;
  private endListener: (() => void) | undefined;
  private closeListener: (() => void) | undefined;
  private errorListener: ((error: unknown) => void) | undefined;

  on(event: "data" | "end" | "close" | "error", listener: ByteInputEventListener): void {
    if (event === "data") this.listener = listener as ByteInputListener;
    else if (event === "end") this.endListener = listener as () => void;
    else if (event === "close") this.closeListener = listener as () => void;
    else this.errorListener = listener as (error: unknown) => void;
  }

  off(event: "data" | "end" | "close" | "error", listener: ByteInputEventListener): void {
    if (event === "data" && this.listener === listener) this.listener = undefined;
    if (event === "end" && this.endListener === listener) this.endListener = undefined;
    if (event === "close" && this.closeListener === listener) this.closeListener = undefined;
    if (event === "error" && this.errorListener === listener) this.errorListener = undefined;
  }

  emit(chunk: Uint8Array): void {
    this.listener?.(chunk);
  }
}

class FakeOutput implements ByteOutput {
  readonly writes: Uint8Array[] = [];

  write(frame: Uint8Array): boolean {
    this.writes.push(frame.slice());
    return true;
  }

  on(_event: "drain" | "error" | "close", _listener: ByteOutputEventListener): void {}

  off(_event: "drain" | "error" | "close", _listener: ByteOutputEventListener): void {}
}

function protocolFrame(payload: readonly number[]): Uint8Array {
  const frame = new Uint8Array(4 + payload.length);
  new DataView(frame.buffer).setUint32(0, payload.length, true);
  frame.set(payload, 4);
  return frame;
}

function readRecords(path: string): Array<Record<string, unknown>> {
  return readFileSync(path, "utf8")
    .trim()
    .split("\n")
    .filter((line) => line.length > 0)
    .map((line) => JSON.parse(line) as Record<string, unknown>);
}

describe("protocol tap", () => {
  it("records complete frames across fragmented/coalesced input without payload bytes", () => {
    const path = join(tmpdir(), `react-gpui-tap-${Date.now()}-${Math.random()}.jsonl`);
    const previous = process.env.REACT_GPUI_TAP;
    process.env.REACT_GPUI_TAP = path;
    const input = new FakeInput();
    const output = new FakeOutput();
    const transport = new StdioTransport(output, input);
    const snapshot = protocolFrame([0x92, 0x03, 0x01]);
    const command = protocolFrame([0x98, 0x03, 0x04, 0, 0, 0, 0x2a, 0, 0x0e]);
    const event = protocolFrame([0x99, 0x03, 0x02, 0, 0, 0, 0, 0, 0, 0x0e]);
    const commandResult = protocolFrame([
      0x9a, 0x03, 0x02, 0, 0, 0, 0, 0, 0, 0x06, 0x96, 0x02, 0x2a, 0x0e, 0, 0xc3, 0xc0,
    ]);
    const malformed = new Uint8Array([0xff, 0xff, 0xff, 0xff]);

    try {
      transport.submit(snapshot);
      transport.submit(command);
      input.emit(event.subarray(0, 2));
      input.emit(new Uint8Array([...event.subarray(2), ...commandResult, ...malformed]));
      transport.dispose();
      transport.dispose();
      const records = readRecords(path);
      expect(records.map((record) => record.seq)).toEqual([1, 2, 3, 4, 5]);
      expect(records.map((record) => record.kind)).toEqual(["snapshot", "command", "event", "event", "unknown"]);
      expect(records[0]).toMatchObject({ dir: "out", peer: "host", bytes: snapshot.byteLength });
      expect(records[1]).toMatchObject({ command_kind: 14, request_id: 42 });
      expect(records[2]).toMatchObject({ event_type: 14 });
      expect(records[3]).toMatchObject({ event_type: 6, request_id: 42, success: true });
      expect(records[0]).not.toHaveProperty("payload");
      expect(records[4]).toMatchObject({ dir: "in", bytes: 4 });
    } finally {
      if (previous === undefined) delete process.env.REACT_GPUI_TAP;
      else process.env.REACT_GPUI_TAP = previous;
      rmSync(path, { force: true });
    }
  });

  it("keeps transport usable when the tap path cannot be opened", () => {
    const previous = process.env.REACT_GPUI_TAP;
    process.env.REACT_GPUI_TAP = join(tmpdir(), `react-gpui-tap-missing-${Date.now()}-${Math.random()}`, "tap.jsonl");
    const input = new FakeInput();
    const output = new FakeOutput();
    const transport = new StdioTransport(output, input);
    try {
      expect(() => transport.submit(protocolFrame([0x92, 0x03, 0x01]))).not.toThrow();
      expect(output.writes).toHaveLength(1);
    } finally {
      transport.dispose();
      if (previous === undefined) delete process.env.REACT_GPUI_TAP;
      else process.env.REACT_GPUI_TAP = previous;
    }
  });

  it("writes a final capacity record and then disables tap writes", () => {
    const path = join(tmpdir(), `react-gpui-tap-capacity-${Date.now()}-${Math.random()}.jsonl`);
    const tap = ProtocolTap.openForTest(path, 600);
    const frame = protocolFrame([0x92, 0x03, 0x01]);
    try {
      for (let index = 0; index < 20; index += 1) tap.recordOutboundFrame(frame);
      tap.dispose();
      const records = readRecords(path);
      expect(records.at(-1)).toMatchObject({ kind: "tap_stopped", reason: "capacity" });
      expect(records.map((record) => record.seq)).toEqual(
        records.map((record) => record.seq).sort((a, b) => Number(a) - Number(b)),
      );
      expect(readFileSync(path).byteLength).toBeLessThanOrEqual(600);
    } finally {
      rmSync(path, { force: true });
    }
  });

  it("does not allocate a tap when the environment is unset", () => {
    const previous = process.env.REACT_GPUI_TAP;
    delete process.env.REACT_GPUI_TAP;
    try {
      expect(ProtocolTap.fromEnv()).toBeUndefined();
    } finally {
      if (previous === undefined) delete process.env.REACT_GPUI_TAP;
      else process.env.REACT_GPUI_TAP = previous;
    }
  });
});
