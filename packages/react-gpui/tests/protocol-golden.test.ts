import { describe, expect, it } from "bun:test";
import {
  FrameDecoder,
  MAX_FRAME_SIZE,
  PROTOCOL_VERSION,
  decodeEvent,
  decodeWireForGolden,
  encodePayload,
} from "../src/protocol";

const fixtureDir = `${import.meta.dir}/../../../fixtures/protocol`;

type Vector = { id: string; kind: string; payload: Uint8Array };

function decodeHex(value: string): Uint8Array {
  if (value.length % 2 !== 0) throw new Error(`odd fixture hex length: ${value}`);
  return Uint8Array.from({ length: value.length / 2 }, (_, index) =>
    Number.parseInt(value.slice(index * 2, index * 2 + 2), 16),
  );
}
function hex(value: Uint8Array): string {
  return Buffer.from(value).toString("hex");
}
async function readLines(file: string): Promise<string[]> {
  return (await Bun.file(`${fixtureDir}/${file}`).text())
    .split(/\r?\n/)
    .filter((line) => line !== "" && !line.startsWith("#"));
}
async function validVectors(file: string): Promise<Vector[]> {
  return (await readLines(file)).map((line) => {
    const fields = line.split("\t");
    if (fields.length !== 3) throw new Error(`invalid vector row: ${line}`);
    return { id: fields[0], kind: fields[1], payload: decodeHex(fields[2]) };
  });
}

function assertRepresentativeFields(vector: Vector, decoded: unknown): void {
  if (vector.id === "rust-snapshot-all-kinds" || vector.id === "ts-snapshot-all-kinds") {
    const snapshot = decoded as readonly unknown[];
    const nodes = snapshot[6] as readonly (readonly unknown[])[];
    expect(nodes).toHaveLength(8);
    expect(nodes.map((node) => node[3])).toEqual([1, 2, 4, 3, 5, 6, 7, 3]);
    expect(nodes.slice(4).map((node) => (node[7] as readonly unknown[] | null)?.[0])).toEqual([1, 2, 3, 4]);
    expect((nodes[0][4] as readonly unknown[]).length).toBe(42);
    expect((nodes[0][4] as readonly unknown[])[40]).toEqual([1, [-2, 3, 4, 1, 0x01020380, 1]]);
    expect(nodes[6][7]).toEqual([3, "assets/😀.png", 3, "assets/avatar-fallback.png"]);
    expect(nodes[7][7]).toEqual([4, "card", ["assets/logo.png"], true, true]);
    expect(nodes[0][10]).toBeUndefined();
    expect(nodes[1][10]).toBe(true);
  }
  if (vector.id.endsWith("snapshot-box-shadow-double")) {
    const snapshot = decoded as readonly unknown[];
    const nodes = snapshot[6] as readonly (readonly unknown[])[];
    expect((nodes[0][4] as readonly unknown[])[40]).toEqual([
      2,
      [
        [-2, 3, 4, 0, 0x11223344, 0],
        [0, -1, 8, 2, 0xaabbccdd, 1],
      ],
    ]);
  }
  if (vector.id === "rust-patch-all-operations" || vector.id === "ts-patch-all-operations") {
    const patch = decoded as readonly unknown[];
    expect((patch[6] as readonly (readonly unknown[])[]).map((operation) => operation[0])).toEqual([1, 2, 3, 4]);
  }
  if (vector.id.endsWith("command-open-surface-options")) {
    expect((decoded as readonly unknown[])[8]).toEqual(["Inspector", [640, 480], [1, false, 320, 240]]);
  }
  if (vector.id.endsWith("command-set-keybindings")) {
    const command = decoded as readonly unknown[];
    expect(command[7]).toBe(22);
    expect(command[8]).toEqual([
      ["cmd-shift-p", "palette.open"],
      ["ctrl-k ctrl-1", "menu.other"],
    ]);
  }
  if (vector.id.endsWith("event-window-resize")) {
    const event = decoded as readonly unknown[];
    expect(event[9]).toEqual([800.5, 600.5, 2]);
  }
  if (vector.id.endsWith("event-window-activation")) {
    expect((decoded as readonly unknown[])[9]).toBe(true);
  }
  if (vector.id.endsWith("event-command-result-size")) {
    expect((decoded as readonly unknown[])[9]).toEqual([2, 110, 13, 1, true, null, [2, [800.5, 600.5]]]);
  }
  if (vector.id.endsWith("event-command-result-focus")) {
    expect((decoded as readonly unknown[])[9]).toEqual([2, 111, 14, 4, true, null, [3, true]]);
  }
  if (vector.id.endsWith("event-command-result-clipboard")) {
    expect((decoded as readonly unknown[])[9]).toEqual([2, 112, 16, 1, true, null, [4, "pasted text"]]);
  }
  if (vector.id.endsWith("event-submit-text")) {
    expect((decoded as readonly unknown[])[9]).toBe("submitted text");
  }
  if (vector.id.endsWith("event-surface-closed")) {
    const event = decoded as readonly unknown[];
    expect(event[6]).toBe(0);
    expect(event[7]).toBe(0);
    expect(event[9]).toBeNull();
  }
  if (vector.id.endsWith("event-command-result-open-surface")) {
    expect((decoded as readonly unknown[])[9]).toEqual([2, 117, 17, 1, true, null, [1, 41]]);
  }
  if (vector.id.endsWith("event-command-result-file-open")) {
    expect((decoded as readonly unknown[])[9]).toEqual([2, 118, 18, 1, true, null, [5, ["/tmp/a.txt", "/tmp/b.txt"]]]);
  }
  if (vector.id.endsWith("event-command-result-file-save")) {
    expect((decoded as readonly unknown[])[9]).toEqual([2, 119, 19, 1, true, null, [4, "/tmp/report.json"]]);
  }
}

describe("protocol golden vectors", () => {
  it("locks own bytes and cross-direction semantic values", async () => {
    const rustVectors = await validVectors("rust_to_ts.hex");
    const tsVectors = await validVectors("ts_to_rust.hex");
    const tsByKey = new Map(tsVectors.map((vector) => [vector.id.replace(/^ts-/, ""), vector]));
    let count = 0;
    for (const vector of rustVectors) {
      const rustDecoded = decodeWireForGolden(vector.payload);
      const tsVector = tsByKey.get(vector.id.replace(/^rust-/, ""));
      expect(tsVector, vector.id).toBeDefined();
      const tsDecoded = decodeWireForGolden(tsVector!.payload);
      expect(tsDecoded, vector.id).toEqual(rustDecoded);
      expect(hex(encodePayload(tsDecoded as never)), tsVector!.id).toBe(hex(tsVector!.payload));
      if (vector.kind === "event") expect(decodeEvent(vector.payload), vector.id).not.toBeNull();
      assertRepresentativeFields(vector, rustDecoded);
      count += 1;
    }
    expect(count).toBeGreaterThanOrEqual(20);
  });

  it("rejects adjacent protocol versions with actionable diagnostics", () => {
    expect(PROTOCOL_VERSION).toBe(3);
    const validEvent = [PROTOCOL_VERSION, 2, 7, 3, 1, 1, 0, 0, 1, null] as const;
    expect((decodeWireForGolden(encodePayload(validEvent)) as readonly unknown[])[0]).toBe(PROTOCOL_VERSION);
    for (const version of [PROTOCOL_VERSION - 1, PROTOCOL_VERSION + 1]) {
      const payload = encodePayload([version, 2, 7, 3, 1, 1, 0, 0, 1, null] as never);
      expect(() => decodeEvent(payload)).toThrow(
        `host binary speaks protocol v${version}; this renderer package speaks protocol v${PROTOCOL_VERSION}`,
      );
    }
  });

  it("locks unknown and malformed wire behavior", async () => {
    let count = 0;
    for (const line of await readLines("invalid.hex")) {
      const fields = line.split("\t");
      expect(fields).toHaveLength(5);
      const payload = decodeHex(fields[2]);
      const decoded = decodeWireForGolden(payload);
      if (fields[4] === "event-null") {
        expect(decodeEvent(payload), fields[0]).toBeNull();
        expect(decoded, fields[0]).not.toBeNull();
      } else if (fields[4] === "decode-null") {
        expect(decoded, fields[0]).toBeNull();
      } else if (fields[4] === "raw") {
        expect(decoded, fields[0]).not.toBeNull();
      } else {
        throw new Error(`unknown TypeScript invalid expectation: ${fields[4]}`);
      }
      count += 1;
    }
    expect(count).toBeGreaterThanOrEqual(4);
  });

  it("locks frame length boundary behavior", async () => {
    let count = 0;
    for (const line of await readLines("frames.hex")) {
      const fields = line.split("\t");
      expect(fields).toHaveLength(5);
      const frame = new Uint8Array([...decodeHex(fields[1]), ...decodeHex(fields[2])]);
      const decoder = new FrameDecoder();
      if (fields[4] === "ok") {
        expect(decoder.push(frame)).toEqual([new Uint8Array()]);
      } else if (fields[4] === "pending") {
        expect(decoder.push(frame)).toEqual([]);
      } else if (fields[4] === "oversize") {
        expect(() => decoder.push(frame)).toThrow(RangeError);
      } else {
        throw new Error(`unknown TypeScript frame expectation: ${fields[4]}`);
      }
      count += 1;
    }
    expect(count).toBeGreaterThanOrEqual(5);
    expect(MAX_FRAME_SIZE).toBe(16 * 1024 * 1024);
  });
});
