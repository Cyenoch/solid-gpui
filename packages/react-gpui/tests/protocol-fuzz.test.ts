import { describe, expect, it } from "bun:test";
import { encode } from "@msgpack/msgpack";
import {
  EVENT_KEY,
  EVENT_KEY_DOWN,
  EVENT_KEY_REPEAT,
  EVENT_KEY_UP,
  EVENT_KIND,
  FrameDecoder,
  MAX_FRAME_SIZE,
  PROTOCOL_VERSION,
  decodeEvent,
  encodePayload,
  encodeFrame,
  utf8ByteLength,
} from "../src/protocol";
const RANDOM_CASES_PER_SEED = 256;

type Bytes = Uint8Array;

class XorShift32 {
  private state: number;

  constructor(seed: number) {
    if (seed === 0) throw new RangeError("seed must be non-zero");
    this.state = seed >>> 0;
  }

  next(): number {
    let value = this.state;
    value ^= value << 13;
    value ^= value >>> 17;
    value ^= value << 5;
    this.state = value >>> 0;
    return this.state;
  }

  index(upper: number): number {
    return this.next() % upper;
  }
}

const validSnapshot = [PROTOCOL_VERSION, 1, 7, 3, 0, 1, [[1, 0, 0, 1, null, null, 0, null, null, false]]] as const;
const validPatch = [PROTOCOL_VERSION, 3, 7, 3, 1, 2, []] as const;
const validEvent = [
  PROTOCOL_VERSION,
  EVENT_KIND,
  7,
  3,
  1,
  1,
  1,
  9,
  EVENT_KEY,
  [5, "ArrowLeft", ["shift", "cmd"], EVENT_KEY_DOWN],
] as const;
const validCommand = [PROTOCOL_VERSION, 4, 7, 3, 1, 1, 1, 1, null] as const;
const seeds = [validSnapshot, validPatch, validEvent, validCommand];

function bytes(value: readonly unknown[]): Bytes {
  return encode(value);
}

function frameWithLength(payload: Bytes, declaredLength: number): Bytes {
  const frame = new Uint8Array(4 + payload.byteLength);
  new DataView(frame.buffer).setUint32(0, declaredLength >>> 0, true);
  frame.set(payload, 4);
  return frame;
}

function mutate(payload: Bytes, rng: XorShift32): Bytes {
  const mutation = rng.next() % 5;
  const output = payload.slice();
  if (mutation === 0) {
    const length = rng.index(output.length + 1);
    return output.subarray(0, length);
  }
  if (mutation === 1 && output.length > 0) {
    const index = rng.index(output.length);
    output[index] ^= 1 << rng.next() % 8;
    return output;
  }
  if (mutation === 2 && output.length > 0) {
    output[0] = 0x90;
    return output;
  }
  if (mutation === 3) {
    const extra = new Uint8Array(4);
    new DataView(extra.buffer).setUint32(0, rng.next(), true);
    const appended = new Uint8Array(output.length + extra.length);
    appended.set(output);
    appended.set(extra, output.length);
    return appended;
  }
  if (output.length > 0) {
    output[rng.index(output.length)] = rng.next() & 0xff;
  }
  return output;
}

function assertSafeEventDecode(payload: Bytes): void {
  let result: ReturnType<typeof decodeEvent> = null;
  expect(() => {
    result = decodeEvent(payload);
  }).not.toThrow();
  expect(result === null || Array.isArray(result)).toBe(true);
}

function assertSafeFramePush(frame: Bytes): void {
  try {
    const result = new FrameDecoder().push(frame);
    expect(Array.isArray(result)).toBe(true);
  } catch (error) {
    expect(error).toBeInstanceOf(RangeError);
  }
}

describe("protocol fuzz safety", () => {
  it("round-trips legal seed frames and handles deterministic mutations", () => {
    for (const seed of seeds) {
      const frame = encodeFrame(seed as never);
      const decoder = new FrameDecoder();
      expect(decoder.push(frame)).toEqual([frame.slice(4)]);
    }
    expect(decodeEvent(new Uint8Array([0xd9, 1, 0xff]))).toBeNull();
    expect(decodeEvent(new Uint8Array([0x90]))).toBeNull();
    expect(decodeEvent(encodeFrame(validEvent).slice(4))).toEqual(validEvent);

    const structured: Bytes[] = [
      bytes([PROTOCOL_VERSION, EVENT_KIND, 7, 3, 1, 1, 1, 9, EVENT_KEY, [99, "A", [], EVENT_KEY_DOWN]]),
      bytes([PROTOCOL_VERSION, EVENT_KIND, 7, 3, 1, 2, 1, 9, EVENT_KEY, [5, "A", ["bogus"], EVENT_KEY_DOWN]]),
      bytes([PROTOCOL_VERSION, EVENT_KIND, 7, 3, 1, 3, 1, 9, EVENT_KEY, [5, "A", [], 99]]),
      bytes([PROTOCOL_VERSION, EVENT_KIND, 7, 3, 1, 4, 1, 9, EVENT_KEY, [5, "A", ["shift", "shift"], EVENT_KEY_UP]]),
      bytes([PROTOCOL_VERSION, EVENT_KIND, 7, 3, 1, -1, 1, 9, EVENT_KEY, [5, "A", [], EVENT_KEY_DOWN]]),
      bytes([PROTOCOL_VERSION, EVENT_KIND, 7, 3, 1, 0x1_0000_0000, 1, 9, EVENT_KEY, [5, "A", [], EVENT_KEY_DOWN]]),
      bytes([PROTOCOL_VERSION, EVENT_KIND, 7, 3, 1, Number.NaN, 1, 9, EVENT_KEY, [5, "A", [], EVENT_KEY_DOWN]]),
      bytes([
        PROTOCOL_VERSION,
        EVENT_KIND,
        7,
        3,
        1,
        Number.POSITIVE_INFINITY,
        1,
        9,
        EVENT_KEY,
        [5, "A", [], EVENT_KEY_DOWN],
      ]),
      new Uint8Array([0xd9, 1, 0xff]),
      new Uint8Array([0x90]),
    ];

    let cases = 0;
    const rng = new XorShift32(0x4d595df4);
    for (const seed of seeds) {
      const payload = encode(seed as never);
      for (let iteration = 0; iteration < RANDOM_CASES_PER_SEED; iteration += 1) {
        const mutated = mutate(payload, rng);
        assertSafeEventDecode(mutated);
        const declaredLength =
          iteration % 5 === 0
            ? mutated.byteLength
            : iteration % 5 === 1
              ? Math.max(0, mutated.byteLength - 1)
              : iteration % 5 === 2
                ? mutated.byteLength + 1
                : iteration % 5 === 3
                  ? 0xffff
                  : MAX_FRAME_SIZE + 1;
        assertSafeFramePush(frameWithLength(mutated, declaredLength));
        if (iteration % 3 === 0) {
          const frame = frameWithLength(mutated, declaredLength);
          assertSafeFramePush(frame.slice(0, rng.index(frame.length + 1)));
        }
        cases += 1;
      }
    }

    for (const payload of structured) {
      assertSafeEventDecode(payload);
      assertSafeFramePush(frameWithLength(payload, payload.byteLength));
      cases += 1;
    }

    expect(cases).toBe(seeds.length * RANDOM_CASES_PER_SEED + structured.length);
  });

  it("handles fragmented, coalesced, truncated, and hostile frame lengths", () => {
    const first = encodeFrame(validEvent);
    const second = encodeFrame(validEvent);
    const decoder = new FrameDecoder();
    expect(decoder.push(first.slice(0, 2))).toEqual([]);
    expect(decoder.push(new Uint8Array([...first.slice(2), ...second]))).toEqual([first.slice(4), second.slice(4)]);
    expect(new FrameDecoder().push(new Uint8Array([1, 2, 3]))).toEqual([]);
    expect(() => new FrameDecoder().push(frameWithLength(new Uint8Array(), MAX_FRAME_SIZE + 1))).toThrow(RangeError);
    expect(() => new FrameDecoder(0)).toThrow(RangeError);
  });

  it("uses float32 numeric wire encoding and UTF-8 resource lengths", () => {
    const event = [3, 2, 7, 3, 1, 1, 1, 1, 12, [7, 1, 3.5, -2.25, 0, 1, []]] as const;
    expect(Buffer.from(encodePayload(event as never)).toString("hex")).toBe(
      "9a03020703010101010c970701ca40600000cac0100000000190",
    );
    expect(utf8ByteLength("😀".repeat(512))).toBe(2048);
    expect(utf8ByteLength("😀".repeat(513))).toBe(2052);
  });
});
