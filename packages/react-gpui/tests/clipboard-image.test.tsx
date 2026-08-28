import { describe, expect, it } from "bun:test";
import { decode } from "@msgpack/msgpack";
import { MemoryTransport, View, createRoot } from "../src/index";
import {
  COMMAND_CLIPBOARD_READ_IMAGE,
  COMMAND_CLIPBOARD_WRITE_IMAGE,
  EVENT_COMMAND_RESULT,
  EVENT_KIND,
  MAX_CLIPBOARD_IMAGE_BYTES,
  PROTOCOL_VERSION,
  decodeEvent,
  encodeFrame,
} from "../src/protocol";

function lastCommand(transport: MemoryTransport): readonly unknown[] {
  const frame = transport.submitted.at(-1);
  if (frame === undefined) throw new Error("expected a submitted command");
  return decode(frame.slice(4)) as readonly unknown[];
}

function commandResult(
  transport: MemoryTransport,
  command: readonly unknown[],
  value: readonly unknown[] | null,
  sequence: number,
): void {
  transport.push(
    encodeFrame([
      PROTOCOL_VERSION,
      EVENT_KIND,
      command[2],
      command[3],
      command[4],
      sequence,
      command[6],
      0,
      EVENT_COMMAND_RESULT,
      [2, command[5], command[7], command[6], true, null, value],
    ] as never),
  );
}

describe("clipboard image protocol", () => {
  it("round-trips bounded binary bytes and preserves format", async () => {
    const transport = new MemoryTransport();
    const root = createRoot(transport, { surfaceId: 17, epoch: 2 });
    root.render(<View />);
    const bytes = new Uint8Array([0x89, 0x50, 0x4e, 0x47, 0x00, 0xff]);
    const writePromise = root.setClipboardImage({ format: "png", bytes });
    const write = lastCommand(transport);
    expect(write[7]).toBe(COMMAND_CLIPBOARD_WRITE_IMAGE);
    expect(write[8]).toEqual([1, bytes]);
    commandResult(transport, write, null, 1);
    await expect(writePromise).resolves.toBeUndefined();

    const readPromise = root.getClipboardImage();
    const read = lastCommand(transport);
    expect(read[7]).toBe(COMMAND_CLIPBOARD_READ_IMAGE);
    commandResult(transport, read, [7, [3, bytes]], 2);
    await expect(readPromise).resolves.toEqual({ format: "gif", bytes });
    root.unmount();
  });

  it("returns null for an empty/non-image clipboard result", async () => {
    const transport = new MemoryTransport();
    const root = createRoot(transport, { surfaceId: 18, epoch: 2 });
    root.render(<View />);
    const promise = root.getClipboardImage();
    const command = lastCommand(transport);
    commandResult(transport, command, null, 1);
    await expect(promise).resolves.toBeNull();
    root.unmount();
  });

  it("rejects empty and oversized writes before submitting", async () => {
    const transport = new MemoryTransport();
    const root = createRoot(transport, { surfaceId: 19, epoch: 2 });
    root.render(<View />);
    const before = transport.submitted.length;
    await expect(root.setClipboardImage({ format: "png", bytes: new Uint8Array() })).rejects.toThrow("non-empty");
    await expect(
      root.setClipboardImage({ format: "png", bytes: new Uint8Array(MAX_CLIPBOARD_IMAGE_BYTES + 1) }),
    ).rejects.toThrow("non-empty");
    expect(transport.submitted).toHaveLength(before);
    root.unmount();
  });

  it("rejects malformed image command results", async () => {
    const transport = new MemoryTransport();
    const root = createRoot(transport, { surfaceId: 20, epoch: 2 });
    root.render(<View />);
    const promise = root.getClipboardImage();
    const command = lastCommand(transport);
    transport.push(
      encodeFrame([
        PROTOCOL_VERSION,
        EVENT_KIND,
        command[2],
        command[3],
        command[4],
        1,
        command[6],
        0,
        EVENT_COMMAND_RESULT,
        [2, command[5], command[7], command[6], true, null, [7, [9, new Uint8Array([1])]]],
      ] as never),
    );
    await expect(promise).rejects.toThrow("malformed event frame");
    expect(decodeEvent(new Uint8Array([0xc1]))).toBeNull();
    root.unmount();
  });
});
