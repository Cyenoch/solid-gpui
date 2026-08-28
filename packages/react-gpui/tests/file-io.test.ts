import { describe, expect, it } from "bun:test";
import {
  COMMAND_KIND,
  COMMAND_READ_TEXT_FILE,
  COMMAND_WRITE_TEXT_FILE,
  MAX_FILE_WRITE_BYTES,
  PROTOCOL_VERSION,
  decodeEvent,
  encodePayload,
} from "../src/protocol";

const header = [PROTOCOL_VERSION, COMMAND_KIND, 7, 3, 42] as const;

describe("text file protocol", () => {
  it("encodes root-only read and write command shapes", () => {
    expect(Array.from(encodePayload([...header, 1, 1, COMMAND_READ_TEXT_FILE, "/tmp/notes.txt"]))).not.toHaveLength(0);
    expect(
      Array.from(encodePayload([...header, 2, 1, COMMAND_WRITE_TEXT_FILE, ["/tmp/notes.txt", "hello π"]])),
    ).not.toHaveLength(0);
  });

  it("decodes file text values and numeric write acknowledgements", () => {
    const read = decodeEvent(
      encodePayload([3, 2, 7, 3, 42, 1, 1, 0, 6, [2, 1, COMMAND_READ_TEXT_FILE, 1, true, null, [6, "hello π"]]]),
    );
    expect(read).not.toBeNull();
    const write = decodeEvent(
      encodePayload([3, 2, 7, 3, 42, 2, 1, 0, 6, [2, 2, COMMAND_WRITE_TEXT_FILE, 1, true, null, [1, 8]]]),
    );
    expect(write).not.toBeNull();
  });

  it("keeps file content below the frame envelope cap", () => {
    expect(MAX_FILE_WRITE_BYTES).toBe(16 * 1024 * 1024 - 1024);
  });
});
