import React from "react";
import { describe, expect, it } from "bun:test";
import { decode } from "@msgpack/msgpack";
import { MemoryTransport, View, createRoot } from "../src/index";
import {
  COMMAND_CLIPBOARD_READ,
  COMMAND_CLIPBOARD_WRITE,
  EVENT_COMMAND_RESULT,
  EVENT_KIND,
  MAX_CLIPBOARD_TEXT_BYTES,
  PROTOCOL_VERSION,
  decodeEvent,
  encodeFrame,
  utf8ByteLength,
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

describe("interchange clipboard byte limits", () => {
  it("keeps CJK and emoji writes and reads symmetric at the UTF-8 byte boundary", async () => {
    const cases = [
      {
        label: "CJK",
        atLimit: `${"界".repeat(Math.floor(MAX_CLIPBOARD_TEXT_BYTES / 3))}a`,
      },
      {
        label: "emoji",
        atLimit: "😀".repeat(MAX_CLIPBOARD_TEXT_BYTES / 4),
      },
    ];

    for (const [index, { label, atLimit }] of cases.entries()) {
      expect(utf8ByteLength(atLimit), `${label} boundary`).toBe(MAX_CLIPBOARD_TEXT_BYTES);
      const overLimit = `${atLimit}x`;
      expect(utf8ByteLength(overLimit), `${label} over boundary`).toBe(MAX_CLIPBOARD_TEXT_BYTES + 1);

      const oversizedRead = encodeFrame([
        PROTOCOL_VERSION,
        EVENT_KIND,
        1,
        1,
        1,
        1,
        1,
        0,
        EVENT_COMMAND_RESULT,
        [2, 1, COMMAND_CLIPBOARD_READ, 1, true, null, [4, overLimit]],
      ] as never);
      expect(decodeEvent(oversizedRead.slice(4)), `${label} oversized read`).toBeNull();

      const transport = new MemoryTransport();
      const root = createRoot(transport, { surfaceId: 131 + index, epoch: 132 + index });
      root.render(<View />);

      const writePromise = root.setClipboardText(atLimit);
      const writeCommand = lastCommand(transport);
      expect(writeCommand[7]).toBe(COMMAND_CLIPBOARD_WRITE);
      expect(writeCommand[8]).toBe(atLimit);
      commandResult(transport, writeCommand, null, 1);
      await expect(writePromise).resolves.toBeUndefined();

      const beforeOversizedWrite = transport.submitted.length;
      await expect(root.setClipboardText(overLimit)).rejects.toThrow(
        "clipboard text exceeds the supported size",
      );
      expect(transport.submitted).toHaveLength(beforeOversizedWrite);

      const readPromise = root.getClipboardText();
      const readCommand = lastCommand(transport);
      expect(readCommand[7]).toBe(COMMAND_CLIPBOARD_READ);
      commandResult(transport, readCommand, [4, atLimit], 2);
      await expect(readPromise).resolves.toBe(atLimit);

      root.unmount();
    }
  });
});
