import { expect, test } from "bun:test";
import { readFile } from "node:fs/promises";

import { parseSchema, schemaDigest } from "./protocol-schema-meta";

test("strict parser preserves every canonical definition and enum member", async () => {
  const source = await readFile("packages/solid-gpui/src/protocol/protocol.bop", "utf8");
  const schema = parseSchema(source);
  expect(schema.root).toBe("Envelope");
  expect(schema.definitions.Body).toEqual({
    kind: "union",
    branches: [
      { id: 1, type: "Snapshot" },
      { id: 2, type: "Event" },
      { id: 3, type: "Patch" },
      { id: 4, type: "Command" },
    ],
  });
  expect(schema.definitions.EventKind).toMatchObject({
    kind: "enum",
    names: expect.arrayContaining(["Press", "CloseRequested"]),
    values: expect.arrayContaining([1, 23]),
  });
});

test("strict parser rejects malformed declarations instead of silently skipping them", () => {
  expect(() => parseSchema("message Envelope { 2 -> uint32 body; 1 -> uint32 protocolVersion; }")).toThrow(
    /strictly increasing/,
  );
  expect(() => parseSchema("message Envelope { 1 -> uint32 protocolVersion; 2 -> Missing body; }")).toThrow(
    /unknown definition Missing/,
  );
  expect(() => parseSchema("message Envelope { 1 -> uint32 protocolVersion; 2 -> uint32 body; @ }")).toThrow(
    /unexpected character/,
  );
});

test("schema digest is deterministic for the exact source bytes", () => {
  expect(schemaDigest("message Envelope { 1 -> uint32 protocolVersion; 2 -> uint32 body; }\n")).toBe(
    schemaDigest("message Envelope { 1 -> uint32 protocolVersion; 2 -> uint32 body; }\n"),
  );
  expect(schemaDigest("message Envelope { 1 -> uint32 protocolVersion; 2 -> uint32 body; }\n")).not.toBe(
    schemaDigest("message Envelope { 1 -> uint32 protocolVersion; 2 -> uint32 body; }"),
  );
});
