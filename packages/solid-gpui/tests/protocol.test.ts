import { expect, test } from "bun:test";
import {
  FrameDecoder,
  MAX_FRAME_SIZE,
  COMMAND_INVOKE_NATIVE,
  MAX_NATIVE_CALL_BYTES,
  UPDATE_FOCUSABLE,
  UPDATE_SELECTABLE,
  classifyPayload,
  decodeEvent,
  encodeFrame,
  encodePayload,
  validateExtensionProperties,
  type Command,
  type Event,
  type Patch,
  type Snapshot,
} from "../src/protocol";
import {
  Body as WireBody,
  Envelope,
  EventPayload as WireEventPayload,
  CommandValue as WireCommandValue,
} from "../src/protocol/generated/protocol";

test("InvokeNative preserves opaque arguments and accepts the byte budget boundary", () => {
  const command: Command = {
    type: "command",
    surfaceId: 1,
    epoch: 1,
    afterRevision: 0,
    requestId: 1,
    nodeId: 1,
    command: COMMAND_INVOKE_NATIVE,
    payload: {
      type: "invoke-native",
      moduleId: new Uint8Array(16),
      moduleDigest: new Uint8Array(32),
      functionId: 1,
      args: new Uint8Array(MAX_NATIVE_CALL_BYTES).fill(255),
    },
  };
  const envelope = Envelope.decode(encodePayload(command));
  expect(envelope.body?.tag).toBe(4);
  if (envelope.body?.tag !== 4) throw new Error("expected command");
  expect(envelope.body.value.payload).toEqual({
    tag: 12,
    value: {
      moduleId: new Uint8Array(16),
      moduleDigest: new Uint8Array(32),
      functionId: 1,
      args: new Uint8Array(MAX_NATIVE_CALL_BYTES).fill(255),
    },
  });
  if (command.payload?.type !== "invoke-native") throw new Error("expected invocation");
  for (const payload of [
    { ...command.payload, moduleId: new Uint8Array(17) },
    { ...command.payload, moduleDigest: new Uint8Array(31) },
    { ...command.payload, functionId: 0 },
    { ...command.payload, args: new Uint8Array(MAX_NATIVE_CALL_BYTES + 1) },
    null,
  ])
    expect(() => encodePayload({ ...command, payload })).toThrow();
  expect(() => encodePayload({ ...command, nodeId: 0 })).toThrow();
});

test("native bytes reject missing, oversized, and non-invocation wire results", () => {
  const wire = (command: number, value: Uint8Array | undefined, nodeId = 1, success = true, error?: string) =>
    Envelope.encode({
      protocolVersion: 5,
      body: WireBody.fromEvent({
        surfaceId: 1,
        epoch: 1,
        revision: 0,
        sequence: 1,
        nodeId: 1,
        listenerId: 0,
        eventType: 6,
        payload: WireEventPayload.fromCommandResult({
          requestId: 1,
          command,
          nodeId,
          success,
          error,
          value: WireCommandValue.fromBytesValue(value === undefined ? {} : { value }),
        }),
      }),
    });
  expect(decodeEvent(wire(COMMAND_INVOKE_NATIVE, undefined))).toBeNull();
  expect(decodeEvent(wire(6, new Uint8Array()))).toBeNull();
  expect(() => decodeEvent(wire(COMMAND_INVOKE_NATIVE, new Uint8Array(MAX_NATIVE_CALL_BYTES + 1)))).toThrow();
  expect(decodeEvent(wire(COMMAND_INVOKE_NATIVE, new Uint8Array(), 0))).toBeNull();
  expect(decodeEvent(wire(COMMAND_INVOKE_NATIVE, new Uint8Array(), 1, false))).toBeNull();
  expect(decodeEvent(wire(COMMAND_INVOKE_NATIVE, new Uint8Array(), 1, true, "contradictory error"))).toBeNull();
  for (const bytes of [new Uint8Array(), new Uint8Array(MAX_NATIVE_CALL_BYTES).fill(128)]) {
    const event = decodeEvent(wire(COMMAND_INVOKE_NATIVE, bytes));
    expect(event?.payload).toEqual({
      type: "command-result",
      result: {
        requestId: 1,
        command: COMMAND_INVOKE_NATIVE,
        nodeId: 1,
        success: true,
        error: null,
        value: { type: "bytes", value: bytes },
      },
    });
  }
});

function resizeRootLength(payload: Uint8Array, delta: number): Uint8Array {
  const result = new Uint8Array(payload.length + delta);
  result.set(payload.subarray(0, -1));
  result[result.length - 1] = 0;
  new DataView(result.buffer).setUint32(
    0,
    new DataView(payload.buffer, payload.byteOffset, 4).getUint32(0, true) + delta,
    true,
  );
  return result;
}

function littleEndian(value: number): number[] {
  return [value & 0xff, (value >>> 8) & 0xff, (value >>> 16) & 0xff, (value >>> 24) & 0xff];
}
function message(fields: number[]): number[] {
  return [...littleEndian(fields.length), ...fields];
}
function union(tag: number, value: number[]): number[] {
  return [...littleEndian(value.length), tag, ...value];
}
function eventPayload(eventType: number, payload: number[], nodeId = 1, listenerId = 0): Uint8Array {
  const fields = [
    1,
    ...littleEndian(7),
    2,
    ...littleEndian(3),
    3,
    ...littleEndian(1),
    4,
    ...littleEndian(1),
    5,
    ...littleEndian(nodeId),
    6,
    ...littleEndian(listenerId),
    7,
    eventType,
    ...(payload.length === 0 ? [] : [8, ...payload]),
    0,
  ];
  const body = union(2, message(fields));
  return Uint8Array.from(message([1, ...littleEndian(5), 2, ...body, 0]));
}

test("Bebop v5 encodes semantic commands and classifies the generated envelope", () => {
  const command: Command = {
    type: "command",
    surfaceId: 7,
    epoch: 3,
    afterRevision: 42,
    requestId: 106,
    nodeId: 1,
    command: 6,
    payload: { type: "text", value: "golden title" },
  };
  const payload = encodePayload(command);
  expect(payload.byteLength).toBeGreaterThan(0);
  expect(classifyPayload(payload)).toEqual({ kind: "command", command_kind: 6, request_id: 106 });
});

test("all Event payload forms round-trip through the semantic seam", () => {
  const events: Event[] = [
    {
      type: "event",
      surfaceId: 7,
      epoch: 3,
      revision: 1,
      sequence: 1,
      nodeId: 2,
      listenerId: 4,
      payload: { type: "press" },
    },
    {
      type: "event",
      surfaceId: 7,
      epoch: 3,
      revision: 1,
      sequence: 2,
      nodeId: 2,
      listenerId: 4,
      payload: {
        type: "change",
        data: {
          text: "hé😀",
          selectionStart: 2,
          selectionEnd: 4,
          markedStart: 2,
          markedEnd: 3,
          editSeq: 8,
          reversed: true,
        },
      },
    },
    {
      type: "event",
      surfaceId: 7,
      epoch: 3,
      revision: 1,
      sequence: 3,
      nodeId: 2,
      listenerId: 4,
      payload: {
        type: "selection",
        data: {
          text: "text",
          selectionStart: 0,
          selectionEnd: 4,
          markedStart: null,
          markedEnd: null,
          editSeq: 9,
          reversed: false,
        },
      },
    },
    {
      type: "event",
      surfaceId: 7,
      epoch: 3,
      revision: 1,
      sequence: 4,
      nodeId: 2,
      listenerId: 4,
      payload: { type: "focus" },
    },
    {
      type: "event",
      surfaceId: 7,
      epoch: 3,
      revision: 1,
      sequence: 5,
      nodeId: 2,
      listenerId: 4,
      payload: {
        type: "blur",
        data: {
          text: "text",
          selectionStart: 0,
          selectionEnd: 4,
          markedStart: null,
          markedEnd: null,
          editSeq: 9,
          reversed: false,
        },
      },
    },
    {
      type: "event",
      surfaceId: 7,
      epoch: 3,
      revision: 1,
      sequence: 6,
      nodeId: 1,
      listenerId: 0,
      payload: {
        type: "command-result",
        result: {
          requestId: 1,
          command: 6,
          nodeId: 1,
          success: true,
          error: null,
          value: { type: "text", value: "ok" },
        },
      },
    },
    {
      type: "event",
      surfaceId: 7,
      epoch: 3,
      revision: 1,
      sequence: 7,
      nodeId: 2,
      listenerId: 4,
      payload: { type: "visible-range", start: 1, end: 9 },
    },
    {
      type: "event",
      surfaceId: 7,
      epoch: 3,
      revision: 1,
      sequence: 8,
      nodeId: 2,
      listenerId: 4,
      payload: { type: "animation-complete", generation: 4 },
    },
    {
      type: "event",
      surfaceId: 7,
      epoch: 3,
      revision: 1,
      sequence: 9,
      nodeId: 2,
      listenerId: 4,
      payload: { type: "key", key: "Enter", modifiers: ["ctrl"], action: 2 },
    },
    {
      type: "event",
      surfaceId: 7,
      epoch: 3,
      revision: 1,
      sequence: 10,
      nodeId: 2,
      listenerId: 4,
      payload: { type: "pointer", button: 1, modifiers: ["cmd"], action: 1, clickCount: 1, x: 1.25, y: 2.5 },
    },
    {
      type: "event",
      surfaceId: 7,
      epoch: 3,
      revision: 1,
      sequence: 11,
      nodeId: 2,
      listenerId: 4,
      payload: { type: "pointer-move", x: 1, y: 2, modifiers: [] },
    },
    {
      type: "event",
      surfaceId: 7,
      epoch: 3,
      revision: 1,
      sequence: 12,
      nodeId: 2,
      listenerId: 4,
      payload: { type: "hover" },
    },
    {
      type: "event",
      surfaceId: 7,
      epoch: 3,
      revision: 1,
      sequence: 13,
      nodeId: 2,
      listenerId: 4,
      payload: { type: "scroll", deltaKind: 1, dx: 1, dy: -2, x: 3, y: 4, modifiers: ["alt"] },
    },
    {
      type: "event",
      surfaceId: 7,
      epoch: 3,
      revision: 1,
      sequence: 14,
      nodeId: 2,
      listenerId: 4,
      payload: { type: "submit", text: "submitted" },
    },
    {
      type: "event",
      surfaceId: 7,
      epoch: 3,
      revision: 1,
      sequence: 15,
      nodeId: 1,
      listenerId: 0,
      payload: { type: "window-resize", width: 800, height: 600, scaleFactor: 2 },
    },
    {
      type: "event",
      surfaceId: 7,
      epoch: 3,
      revision: 1,
      sequence: 16,
      nodeId: 1,
      listenerId: 0,
      payload: { type: "window-activation", active: true },
    },
    {
      type: "event",
      surfaceId: 7,
      epoch: 3,
      revision: 1,
      sequence: 17,
      nodeId: 0,
      listenerId: 0,
      payload: { type: "surface-closed" },
    },
    {
      type: "event",
      surfaceId: 7,
      epoch: 3,
      revision: 1,
      sequence: 18,
      nodeId: 1,
      listenerId: 0,
      payload: { type: "action", action: "open" },
    },
    {
      type: "event",
      surfaceId: 7,
      epoch: 3,
      revision: 1,
      sequence: 19,
      nodeId: 1,
      listenerId: 0,
      payload: { type: "window-appearance", appearance: "dark" },
    },
    {
      type: "event",
      surfaceId: 7,
      epoch: 3,
      revision: 1,
      sequence: 20,
      nodeId: 2,
      listenerId: 4,
      payload: { type: "layout", x: 1, y: 2, width: 100, height: 50 },
    },
    {
      type: "event",
      surfaceId: 7,
      epoch: 3,
      revision: 1,
      sequence: 21,
      nodeId: 2,
      listenerId: 4,
      payload: { type: "drag-over", dragType: "card" },
    },
    {
      type: "event",
      surfaceId: 7,
      epoch: 3,
      revision: 1,
      sequence: 22,
      nodeId: 2,
      listenerId: 4,
      payload: { type: "drag-drop", dragType: "card" },
    },
    {
      type: "event",
      surfaceId: 7,
      epoch: 3,
      revision: 1,
      sequence: 23,
      nodeId: 2,
      listenerId: 4,
      payload: { type: "external-file-drop", paths: ["/tmp/a.txt"] },
    },
    {
      type: "event",
      surfaceId: 7,
      epoch: 3,
      revision: 1,
      sequence: 24,
      nodeId: 1,
      listenerId: 0,
      payload: { type: "notification-response", tag: "tag", actionId: null },
    },
    {
      type: "event",
      surfaceId: 7,
      epoch: 3,
      revision: 1,
      sequence: 25,
      nodeId: 2,
      listenerId: 4,
      payload: { type: "pointer-down-outside", x: 2, y: 3 },
    },
    {
      type: "event",
      surfaceId: 7,
      epoch: 3,
      revision: 1,
      sequence: 26,
      nodeId: 1,
      listenerId: 0,
      payload: { type: "close-requested", requestId: 4 },
    },
  ];
  for (const event of events) expect(decodeEvent(encodePayload(event))).toEqual(event);
});

test("Extension values, properties, and events round-trip through Bebop v5", () => {
  const extension: Snapshot = {
    type: "snapshot",
    surfaceId: 7,
    epoch: 3,
    baseRevision: 0,
    revision: 1,
    nodes: [
      {
        id: 1,
        parentId: 0,
        index: 0,
        kind: "Extension",
        style: null,
        text: null,
        listenerId: 1,
        hostProperties: {
          type: "extension",
          value: {
            providerId: new Uint8Array(16),
            catalogDigest: new Uint8Array(32),
            entryId: 1001,
            entryVersion: 1,
            fields: [
              { id: 1, value: { type: "bool", value: true } },
              { id: 2, value: { type: "i32", value: -4 } },
              { id: 3, value: { type: "u32", value: 8 } },
              { id: 4, value: { type: "f32", value: 1.5 } },
              { id: 5, value: { type: "text", value: "hello" } },
              { id: 6, value: { type: "bytes", value: Uint8Array.of(1, 2) } },
            ],
            eventIds: [1, 3],
          },
        },
        accessibility: null,
        focusable: false,
        selectable: false,
        tooltip: null,
        acceptsPointerMove: false,
      },
    ],
  };
  expect(
    decodeEvent(
      encodePayload({
        type: "event",
        surfaceId: 7,
        epoch: 3,
        revision: 1,
        sequence: 1,
        nodeId: 1,
        listenerId: 2,
        payload: { type: "extension", eventId: 1, fields: [{ id: 2, value: { type: "text", value: "ok" } }] },
      }),
    ),
  ).toEqual({
    type: "event",
    surfaceId: 7,
    epoch: 3,
    revision: 1,
    sequence: 1,
    nodeId: 1,
    listenerId: 2,
    payload: { type: "extension", eventId: 1, fields: [{ id: 2, value: { type: "text", value: "ok" } }] },
  });
  const body = Envelope.decode(encodePayload(extension)).body;
  expect(body?.tag).toBe(1);
  if (body?.tag === 1) expect(body.value.nodes?.[0]?.kind).toBe(8);
});

test("Extension fields and event IDs must be sorted, unique, and bounded", () => {
  const badFields: Event = {
    type: "event",
    surfaceId: 7,
    epoch: 3,
    revision: 1,
    sequence: 1,
    nodeId: 1,
    listenerId: 2,
    payload: {
      type: "extension",
      eventId: 1,
      fields: [
        { id: 2, value: { type: "bool", value: true } },
        { id: 1, value: { type: "bool", value: false } },
      ],
    },
  };
  expect(() => encodePayload(badFields)).toThrow(/event is invalid/);
  const badValue: Event = {
    ...badFields,
    payload: { type: "extension", eventId: 1, fields: [{ id: 1, value: { type: "f32", value: Number.NaN } }] },
  };
  expect(() => encodePayload(badValue)).toThrow(/event is invalid/);
  expect(() =>
    encodePayload({
      ...badFields,
      payload: { type: "extension", eventId: 1, fields: [{ id: 0, value: { type: "bool", value: true } }] },
    }),
  ).toThrow(/event is invalid/);
  expect(() =>
    encodePayload({
      ...badFields,
      payload: { type: "extension", eventId: 0, fields: [] },
    }),
  ).toThrow(/event is invalid/);
  expect(
    validateExtensionProperties({
      providerId: new Uint8Array(16),
      catalogDigest: new Uint8Array(32),
      entryId: 0,
      entryVersion: 1,
      fields: [],
      eventIds: [],
    }),
  ).toBe(false);
  expect(
    validateExtensionProperties({
      providerId: new Uint8Array(16),
      catalogDigest: new Uint8Array(32),
      entryId: 1,
      entryVersion: 1,
      fields: [],
      eventIds: [0],
    }),
  ).toBe(false);
});

test("Bebop frames remain fragmented/coalesced at the existing transport seam", () => {
  const first: Event = {
    type: "event",
    surfaceId: 7,
    epoch: 3,
    revision: 1,
    sequence: 1,
    nodeId: 2,
    listenerId: 4,
    payload: { type: "press" },
  };
  const second: Event = {
    type: "event",
    surfaceId: 7,
    epoch: 3,
    revision: 1,
    sequence: 2,
    nodeId: 2,
    listenerId: 4,
    payload: { type: "hover" },
  };
  const firstFrame = encodeFrame(first);
  const secondFrame = encodeFrame(second);
  const decoder = new FrameDecoder();
  expect(decoder.push(firstFrame.subarray(0, 2))).toEqual([]);
  expect(decoder.push(firstFrame.subarray(2))).toEqual([expect.any(Uint8Array)]);
  expect(decoder.push(new Uint8Array([...secondFrame]))).toHaveLength(1);
  expect(decoder.push(new Uint8Array(0))).toEqual([]);
  expect(() => decoder.push(new Uint8Array([1, 0, 0, 1]))).toThrow(/exceeds maximum/);
  expect(MAX_FRAME_SIZE).toBe(16 * 1024 * 1024);
});

test("schema guard rejects an unknown field before generated decoding", () => {
  const event: Event = {
    type: "event",
    surfaceId: 7,
    epoch: 3,
    revision: 1,
    sequence: 1,
    nodeId: 2,
    listenerId: 4,
    payload: { type: "press" },
  };
  const payload = encodePayload(event);
  const malformed = resizeRootLength(payload, 1);
  malformed[malformed.length - 2] = 99;
  expect(() => decodeEvent(malformed)).toThrow(/unknown message field/);
});

test("schema guard rejects strict bool/enum/UTF-8 and repeated-count violations", () => {
  expect(() => decodeEvent(eventPayload(15, union(11, message([1, 2, 0]))))).toThrow(/boolean/);
  expect(() => decodeEvent(eventPayload(18, union(13, message([1, 99, 0]))))).toThrow(/enum/);
  const key = union(5, message([1, 1, 0, 0, 0, 255, 2, ...littleEndian(0), 3, ...littleEndian(1), 0]));
  expect(() => decodeEvent(eventPayload(9, key, 1, 1))).toThrow(/UTF-8/);
  const snapshot = union(1, message([5, ...littleEndian(500_000), 0]));
  const oversized = Uint8Array.from(message([1, ...littleEndian(5), 2, ...snapshot, 0]));
  expect(() => decodeEvent(oversized)).toThrow(/array item budget/);
});
test("schema guard rejects overlong, surrogate, out-of-range, and truncated UTF-8", () => {
  for (const bytes of [
    [0xc0, 0x80],
    [0xe0, 0x80, 0x80],
    [0xed, 0xa0, 0x80],
    [0xf4, 0x90, 0x80, 0x80],
    [0xe2, 0x82],
  ]) {
    const key = union(
      5,
      message([1, bytes.length, 0, 0, 0, ...bytes, 2, ...littleEndian(0), 3, ...littleEndian(1), 0]),
    );
    expect(() => decodeEvent(eventPayload(9, key, 1, 1))).toThrow(/UTF-8/);
  }
});

test("FrameDecoder returns borrowed views for complete input chunks", () => {
  const event: Event = {
    type: "event",
    surfaceId: 7,
    epoch: 3,
    revision: 1,
    sequence: 1,
    nodeId: 2,
    listenerId: 4,
    payload: { type: "press" },
  };
  const frame = encodeFrame(event);
  const stream = new Uint8Array(frame);
  const payloads = new FrameDecoder().push(stream);
  expect(payloads).toHaveLength(1);
  expect(payloads[0]?.buffer).toBe(stream.buffer);
  expect(payloads[0]?.byteOffset).toBe(4);
});
test("Bebop preserves border width and defaults transition delay", () => {
  const snapshot: Snapshot = {
    type: "snapshot",
    surfaceId: 7,
    epoch: 3,
    baseRevision: 0,
    revision: 1,
    nodes: [
      {
        id: 1,
        parentId: 0,
        index: 0,
        kind: "View",
        style: { borderWidth: 3, transition: { durationMs: 20 } },
        text: null,
        listenerId: 0,
        hostProperties: null,
        accessibility: null,
        focusable: false,
        selectable: false,
        tooltip: null,
        acceptsPointerMove: false,
      },
    ],
  };
  const body = Envelope.decode(encodePayload(snapshot)).body;
  expect(body?.tag).toBe(1);
  if (body?.tag === 1) {
    expect(body.value.nodes?.[0]?.style?.borderWidth).toBe(3);
    expect(body.value.nodes?.[0]?.style?.transition?.delayMs).toBe(0);
  }
});

test("Bebop encodes an omitted patch style as an explicit clear", () => {
  const patch: Patch = {
    type: "patch",
    surfaceId: 7,
    epoch: 3,
    baseRevision: 1,
    revision: 2,
    operations: [
      {
        type: "update",
        id: 1,
        mask: 1,
        style: undefined,
        text: null,
        listenerId: 0,
        hostProperties: null,
        accessibility: null,
        focusable: false,
        selectable: false,
        tooltip: null,
        acceptsPointerMove: false,
      },
    ],
  };
  const body = Envelope.decode(encodePayload(patch)).body;
  expect(body?.tag).toBe(3);
  if (body?.tag === 3) {
    const operation = body.value.operations?.[0]?.operation;
    expect(operation?.tag).toBe(2);
    if (operation?.tag === 2) {
      expect(operation.value.style).toBeUndefined();
      expect(operation.value.clearStyle).toBeDefined();
    }
  }
});

test("Bebop patch booleans are present only for their mask bits", () => {
  const patch: Patch = {
    type: "patch",
    surfaceId: 7,
    epoch: 3,
    baseRevision: 1,
    revision: 2,
    operations: [
      {
        type: "update",
        id: 1,
        mask: 0,
        style: undefined,
        text: null,
        listenerId: 0,
        hostProperties: null,
        accessibility: null,
        focusable: true,
        selectable: true,
        tooltip: null,
        acceptsPointerMove: false,
      },
      {
        type: "update",
        id: 1,
        mask: UPDATE_FOCUSABLE | UPDATE_SELECTABLE,
        style: undefined,
        text: null,
        listenerId: 0,
        hostProperties: null,
        accessibility: null,
        focusable: false,
        selectable: false,
        tooltip: null,
        acceptsPointerMove: false,
      },
    ],
  };
  const body = Envelope.decode(encodePayload(patch)).body;
  expect(body?.tag).toBe(3);
  if (body?.tag !== 3) return;

  const omitted = body.value.operations?.[0]?.operation;
  expect(omitted?.tag).toBe(2);
  if (omitted?.tag === 2) {
    expect(omitted.value.focusable).toBeUndefined();
    expect(omitted.value.selectable).toBeUndefined();
  }

  const present = body.value.operations?.[1]?.operation;
  expect(present?.tag).toBe(2);
  if (present?.tag === 2) {
    expect(present.value.focusable).toBe(false);
    expect(present.value.selectable).toBe(false);
  }
});

test("Bebop rejects malformed present command values", () => {
  const payload = Envelope.encode({
    protocolVersion: 5,
    body: WireBody.fromEvent({
      surfaceId: 7,
      epoch: 3,
      revision: 1,
      sequence: 1,
      nodeId: 1,
      listenerId: 0,
      eventType: 6,
      payload: WireEventPayload.fromCommandResult({
        requestId: 1,
        command: 6,
        nodeId: 1,
        success: true,
        value: WireCommandValue.fromNumberValue({}),
      }),
    }),
  });
  expect(decodeEvent(payload)).toBeNull();
});

test("Bebop validates command kind and payload together", () => {
  const invalid: Command = {
    type: "command",
    surfaceId: 7,
    epoch: 3,
    afterRevision: 1,
    requestId: 1,
    nodeId: 1,
    command: 6,
    payload: null,
  };
  expect(() => encodePayload(invalid)).toThrow(/payload/);
});

test("Bebop rejects values that overflow float32", () => {
  const snapshot: Snapshot = {
    type: "snapshot",
    surfaceId: 7,
    epoch: 3,
    baseRevision: 0,
    revision: 1,
    nodes: [
      {
        id: 1,
        parentId: 0,
        index: 0,
        kind: "View",
        style: { width: 1e40 },
        text: null,
        listenerId: 0,
        hostProperties: null,
        accessibility: null,
        focusable: false,
        selectable: false,
        tooltip: null,
        acceptsPointerMove: false,
      },
    ],
  };
  expect(() => encodePayload(snapshot)).toThrow(/float32/);
});

test("protocol classification keeps command result metadata", () => {
  const event: Event = {
    type: "event",
    surfaceId: 7,
    epoch: 3,
    revision: 1,
    sequence: 1,
    nodeId: 1,
    listenerId: 0,
    payload: {
      type: "command-result",
      result: { requestId: 42, command: 6, nodeId: 1, success: false, error: "failed", value: null },
    },
  };
  expect(classifyPayload(encodePayload(event))).toEqual({
    kind: "event",
    event_type: 6,
    request_id: 42,
    success: false,
  });
});
test("Bebop command numbers retain the full u32 range", () => {
  const event: Event = {
    type: "event",
    surfaceId: 7,
    epoch: 3,
    revision: 1,
    sequence: 1,
    nodeId: 1,
    listenerId: 0,
    payload: {
      type: "command-result",
      result: {
        requestId: 1,
        command: 17,
        nodeId: 1,
        success: true,
        error: null,
        value: { type: "number", value: 0xffff_ffff },
      },
    },
  };
  expect(decodeEvent(encodePayload(event))).toEqual(event);
});

test("FrameDecoder retains at most one bounded partial frame buffer", () => {
  const decoder = new FrameDecoder(8);
  decoder.push(Uint8Array.from([8, 0, 0, 0, 1, 2, 3]));
  const retained = decoder as unknown as { buffer: Uint8Array };
  expect(retained.buffer.byteLength).toBeLessThanOrEqual(12);
  decoder.push(new Uint8Array(64));
  expect(retained.buffer.byteLength).toBeLessThanOrEqual(12);
});
