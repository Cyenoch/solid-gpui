import { mkdir, readFile, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import { EVENT_ACTION } from "../packages/solid-gpui/src/protocol/constants";
import { FrameDecoder, decodeEvent, encodePayload } from "../packages/solid-gpui/src/protocol/index";
import { Envelope } from "../packages/solid-gpui/src/protocol/generated/protocol";
import type {
  Command,
  CommandPayload,
  Event,
  EventPayload,
  HostProperties,
  MenuDefinition,
  Patch,
  Snapshot,
  SnapshotNode,
} from "../packages/solid-gpui/src/protocol/types";

const outputDir = resolve(process.argv[2] ?? "fixtures/protocol");
const verify = process.argv.includes("--verify");
const hex = (bytes: Uint8Array): string => Buffer.from(bytes).toString("hex");
const row = (id: string, kind: string, value: Snapshot | Patch | Command | Event): string =>
  `${id}\t${kind}\t${hex(encodePayload(value))}`;

function normalizeGenerated(value: unknown): unknown {
  if (value instanceof Uint8Array) return [...value];
  if (Array.isArray(value)) return value.map(normalizeGenerated);
  if (value !== null && typeof value === "object") {
    return Object.fromEntries(
      Object.keys(value as Record<string, unknown>)
        .filter((key) => key !== "encode")
        .sort()
        .map((key) => [key, normalizeGenerated((value as Record<string, unknown>)[key])]),
    );
  }
  return value;
}

const fullStyle = {
  width: 120,
  height: 48,
  flexDirection: "column" as const,
  flexGrow: 1,
  padding: 4,
  gap: 2,
  justifyContent: "space-between" as const,
  alignItems: "center" as const,
  borderRadius: 3,
  borderWidth: 1,
  borderColor: "#11223344",
  fontSize: 14,
  fontWeight: "bold" as const,
  overflow: "hidden" as const,
  lineClamp: 2,
  textOverflow: "ellipsis" as const,
  marginTop: 1,
  marginRight: 2,
  marginBottom: 3,
  marginLeft: 4,
  fontStyle: "italic" as const,
  textDecoration: "underline" as const,
  lineHeight: 18,
  minWidth: 10,
  maxWidth: 200,
  minHeight: 10,
  maxHeight: 80,
  flexShrink: 1,
  alignSelf: "center" as const,
  position: "relative" as const,
  left: -1,
  top: 2,
  right: 3,
  bottom: 4,
  cursor: "pointer" as const,
  textAlign: "center" as const,
  backgroundColor: "#223344",
  color: "#aabbccdd",
  opacity: 0.8,
  transition: { durationMs: 100, delayMs: 20, easing: "easeInOut" as const, properties: ["opacity", "width"] as const },
  boxShadow: {
    offsetX: 1,
    offsetY: -2,
    blurRadius: 3,
    spreadRadius: 0,
    color: "#00000080",
    inset: true,
  },
  fontFamily: "Inter",
};
const accessibility = {
  role: 5,
  label: "label",
  description: "description",
  disabled: false,
  checked: true,
  selected: false,
  value: "42",
  expanded: true,
  level: 2,
};
const input: HostProperties = {
  type: "text-input",
  value: {
    value: "text",
    placeholder: "placeholder",
    multiline: false,
    disabled: false,
    controlled: true,
    ackEditSeq: 4,
    selectionStart: 1,
    selectionEnd: 3,
    markedStart: 1,
    markedEnd: 2,
    maxLength: 8,
    selectionReversed: false,
  },
};
const virtualList: HostProperties = {
  type: "virtual-list",
  value: { itemCount: 20, rangeStart: 2, rangeEnd: 9, estimatedItemSize: 24.5, overscan: 3 },
};
const image: HostProperties = {
  type: "image",
  value: { source: "assets/😀.png", objectFit: 3, fallbackSource: "assets/fallback.png" },
};
const drag: HostProperties = {
  type: "drag",
  value: { dragType: "card", exportFiles: ["assets/logo.png"], acceptsDragOver: true, acceptsDrop: true },
};
const node = (
  id: number,
  parentId: number,
  index: number,
  kind: SnapshotNode["kind"],
  extras: Partial<SnapshotNode> = {},
): SnapshotNode => ({
  id,
  parentId,
  index,
  kind,
  style: null,
  text: null,
  listenerId: 0,
  hostProperties: null,
  accessibility: null,
  focusable: false,
  selectable: false,
  tooltip: null,
  acceptsPointerMove: false,
  ...extras,
});
const snapshot: Snapshot = {
  type: "snapshot",
  surfaceId: 7,
  epoch: 3,
  baseRevision: 0,
  revision: 42,
  nodes: [
    node(1, 0, 0, "View", { style: fullStyle, accessibility }),
    node(2, 1, 0, "Text", { style: fullStyle, selectable: true }),
    node(3, 2, 0, "RawText", { text: "Hello 😀" }),
    node(4, 1, 1, "Pressable", { listenerId: 7, tooltip: "Press to open", acceptsPointerMove: true, accessibility }),
    node(5, 1, 2, "TextInput", { listenerId: 8, hostProperties: input }),
    node(6, 1, 3, "VirtualList", { hostProperties: virtualList }),
    node(7, 1, 4, "Image", { hostProperties: image }),
    node(8, 1, 5, "View", { listenerId: 11, hostProperties: drag }),
  ],
};
const patch: Patch = {
  type: "patch",
  surfaceId: 7,
  epoch: 3,
  baseRevision: 42,
  revision: 43,
  operations: [
    { type: "create", node: node(9, 1, 6, "Text", { text: null }) },
    {
      type: "update",
      id: 4,
      mask: 1 | 2 | 4 | 8 | 16 | 32 | 64 | 128 | 256,
      style: fullStyle,
      text: "new",
      listenerId: 12,
      hostProperties: drag,
      accessibility,
      focusable: true,
      selectable: true,
      tooltip: "updated",
      acceptsPointerMove: true,
    },
    {
      type: "update",
      id: 4,
      mask: 1,
      style: null,
      text: null,
      listenerId: 0,
      hostProperties: null,
      accessibility: null,
      focusable: false,
      selectable: false,
      tooltip: null,
      acceptsPointerMove: false,
    },
    { type: "move", id: 3, parentId: 1, index: 0 },
    { type: "delete", id: 8 },
  ],
};
const command = (requestId: number, nodeId: number, kind: number, payload: CommandPayload): Command => ({
  type: "command",
  surfaceId: 7,
  epoch: 3,
  afterRevision: 43,
  requestId,
  nodeId,
  command: kind as Command["command"],
  payload,
});
const commands: readonly Command[] = [
  command(101, 4, 1, null),
  command(102, 4, 2, null),
  command(103, 5, 3, { type: "selection", start: 2, end: 4 }),
  command(104, 6, 4, { type: "scroll-index", index: 9, alignment: 0 }),
  command(105, 6, 5, null),
  command(106, 1, 6, { type: "text", value: "title" }),
  command(107, 1, 7, { type: "window-size", width: 800, height: 600 }),
  command(108, 1, 8, null),
  command(109, 1, 9, null),
  command(110, 1, 10, { type: "text", value: "https://example.com/😀" }),
  command(111, 1, 11, null),
  command(112, 1, 12, null),
  command(113, 1, 13, null),
  command(114, 4, 14, null),
  command(115, 1, 15, { type: "text", value: "clipboard" }),
  command(116, 1, 16, null),
  command(117, 1, 17, {
    type: "open-surface",
    title: "Child",
    width: 640,
    height: 480,
    options: { kind: 1, resizable: false, minWidth: 320, minHeight: 240 },
  }),
  command(118, 1, 18, { type: "file-dialog-open", title: "Choose", directories: true, multiple: true }),
  command(119, 1, 19, { type: "text", value: "report.json" }),
  command(120, 1, 20, {
    type: "notification",
    title: "Done",
    body: "Finished",
    actions: [{ id: "open", label: "Open" }],
  }),
  command(121, 1, 21, {
    type: "menus",
    menus: [
      {
        title: "File",
        items: [
          { type: "action", name: "open", disabled: true, checked: true },
          { type: "separator" },
          { type: "submenu", title: "More", items: [{ type: "action", name: "other" }] },
        ],
      },
    ],
  }),
  command(122, 1, 22, { type: "keybindings", bindings: [{ keystrokes: "cmd-shift-p", actionName: "palette.open" }] }),
  command(123, 1, 23, { type: "text", value: "require-confirmation" }),
  command(124, 1, 24, { type: "close-resolution", requestId: 123, allow: true }),
  command(125, 1, 25, { type: "text", value: "/tmp/notes.txt" }),
  command(126, 1, 26, { type: "file-write", path: "/tmp/notes.txt", content: "hello π" }),
  command(127, 1, 27, {
    type: "clipboard-image",
    image: { format: "png", bytes: new Uint8Array([0x89, 0x50, 0x4e, 0x47]) },
  }),
  command(128, 1, 28, null),
  command(129, 1, 29, { type: "text", value: "/tmp/Tuffy.ttf" }),
  command(130, 1, 30, null),
  command(131, 1, 31, null),
  command(132, 1, 32, null),
  command(133, 1, 33, null),
  command(134, 6, 34, null),
  command(135, 6, 35, { type: "number", value: 37.5 }),
  command(136, 1, 36, {
    type: "invoke-native",
    moduleId: new Uint8Array(16).fill(1),
    moduleDigest: new Uint8Array(32).fill(2),
    functionId: 1,
    args: new Uint8Array([3, 4]),
  }),
];
const event = (sequence: number, nodeId: number, listenerId: number, payload: EventPayload): Event => ({
  type: "event",
  surfaceId: 7,
  epoch: 3,
  revision: 42,
  sequence,
  nodeId,
  listenerId,
  payload,
});
const textData = {
  text: "hé😀",
  selectionStart: 2,
  selectionEnd: 4,
  markedStart: 2,
  markedEnd: 3,
  editSeq: 8,
  reversed: true,
};
const commandValueEvents: readonly Event[] = [
  event(29, 1, 0, {
    type: "command-result",
    result: {
      requestId: 201,
      command: 13,
      nodeId: 1,
      success: true,
      error: null,
      value: { type: "pair", width: 800, height: 600 },
    },
  }),
  event(30, 1, 0, {
    type: "command-result",
    result: {
      requestId: 202,
      command: 14,
      nodeId: 2,
      success: true,
      error: null,
      value: { type: "boolean", value: false },
    },
  }),
  event(31, 1, 0, {
    type: "command-result",
    result: {
      requestId: 203,
      command: 16,
      nodeId: 1,
      success: true,
      error: null,
      value: { type: "text", value: "clipboard" },
    },
  }),
  event(32, 1, 0, {
    type: "command-result",
    result: {
      requestId: 204,
      command: 18,
      nodeId: 1,
      success: true,
      error: null,
      value: { type: "paths", paths: ["/tmp/a", "/tmp/b"] },
    },
  }),
  event(33, 1, 0, {
    type: "command-result",
    result: {
      requestId: 205,
      command: 25,
      nodeId: 1,
      success: true,
      error: null,
      value: { type: "file-text", value: "contents" },
    },
  }),
  event(34, 1, 0, {
    type: "command-result",
    result: {
      requestId: 206,
      command: 28,
      nodeId: 1,
      success: true,
      error: null,
      value: { type: "image", image: { format: "png", bytes: new Uint8Array([1, 2, 3]) } },
    },
  }),
  event(35, 1, 0, {
    type: "command-result",
    result: {
      requestId: 207,
      command: 31,
      nodeId: 1,
      success: true,
      error: null,
      value: { type: "bounds", x: 1, y: 2, width: 3, height: 4 },
    },
  }),
  event(36, 1, 0, {
    type: "command-result",
    result: {
      requestId: 208,
      command: 32,
      nodeId: 1,
      success: true,
      error: null,
      value: { type: "window-state", fullscreen: true, maximized: false },
    },
  }),
  event(37, 6, 0, {
    type: "command-result",
    result: {
      requestId: 209,
      command: 34,
      nodeId: 6,
      success: true,
      error: null,
      value: { type: "scroll-offset", value: 37.5 },
    },
  }),
  event(38, 1, 0, {
    type: "command-result",
    result: {
      requestId: 210,
      command: 36,
      nodeId: 1,
      success: true,
      error: null,
      value: { type: "bytes", value: new Uint8Array([5, 6]) },
    },
  }),
];
const events: readonly Event[] = [
  event(1, 4, 7, { type: "press" }),
  event(2, 5, 8, { type: "change", data: textData }),
  event(3, 5, 8, { type: "selection", data: textData }),
  event(4, 5, 8, { type: "focus" }),
  event(5, 5, 8, { type: "focus", data: textData }),
  event(6, 5, 8, { type: "blur" }),
  event(7, 5, 8, { type: "blur", data: textData }),
  event(8, 1, 0, {
    type: "command-result",
    result: { requestId: 117, command: 17, nodeId: 1, success: true, error: null, value: { type: "number", value: 9 } },
  }),
  event(9, 6, 10, { type: "visible-range", start: 2, end: 9 }),
  event(10, 4, 7, { type: "animation-complete", generation: 4 }),
  event(11, 4, 7, { type: "key", key: "Enter", modifiers: ["ctrl", "shift"], action: 2 }),
  event(12, 4, 7, { type: "pointer", button: 1, modifiers: ["cmd"], action: 1, clickCount: 2, x: 310.5, y: 220.25 }),
  event(13, 4, 7, { type: "pointer-move", x: 310.5, y: 220.25, modifiers: ["cmd", "shift"] }),
  event(14, 4, 7, { type: "hover" }),
  event(15, 4, 7, { type: "scroll", deltaKind: 1, dx: 3.5, dy: -2.25, x: 10, y: 20.5, modifiers: ["alt"] }),
  event(16, 4, 7, { type: "submit", text: "submitted text" }),
  event(17, 1, 0, { type: "window-resize", width: 800.5, height: 600.5, scaleFactor: 2 }),
  event(18, 1, 0, { type: "window-activation", active: true }),
  event(19, 0, 0, { type: "surface-closed" }),
  event(20, 1, 0, { type: "action", action: "open" }),
  event(21, 1, 0, { type: "window-appearance", appearance: "dark" }),
  event(22, 4, 7, { type: "layout", x: 12.5, y: -3.25, width: 100, height: 48.75 }),
  event(23, 4, 7, { type: "drag-over", dragType: "card" }),
  event(24, 4, 7, { type: "drag-drop", dragType: "card" }),
  event(25, 4, 7, { type: "external-file-drop", paths: ["/tmp/a.txt"] }),
  event(26, 1, 0, { type: "notification-response", tag: "solid-gpui:7:120", actionId: "open" }),
  event(27, 4, 7, { type: "pointer-down-outside", x: 12.5, y: -3.25 }),
  event(28, 1, 0, { type: "close-requested", requestId: 123 }),
  ...commandValueEvents,
];
const rows = [
  row("ts-snapshot-full", "snapshot", snapshot),
  row("ts-patch-all-operations", "patch", patch),
  ...commands.map((value) => row(`ts-command-${value.command}`, "command", value)),
  ...events.map((value) => row(`ts-event-${value.sequence}`, "event", value)),
].sort();
await mkdir(outputDir, { recursive: true });
await writeFile(
  resolve(outputDir, "ts_to_rust.hex"),
  `# protocol-golden-v5\n# id\tmessage\tpayload_hex\n${rows.join("\n")}\n`,
);

const invalidEvent = encodePayload(
  event(99, 4, 7, { type: "pointer", button: 1, modifiers: [], action: 1, clickCount: 1, x: 0, y: 0 }),
);
invalidEvent[5] = 3;
const unknownField = encodePayload(events[0]!);
const unknownFieldResult = new Uint8Array(unknownField.length + 1);
unknownFieldResult.set(unknownField.subarray(0, -1));
unknownFieldResult[unknownFieldResult.length - 2] = 99;
unknownFieldResult[unknownFieldResult.length - 1] = 0;
new DataView(unknownFieldResult.buffer).setUint32(0, new DataView(unknownField.buffer).getUint32(0, true) + 1, true);
const invalidEventType = encodePayload(event(100, 1, 0, { type: "action", action: "wrong" }));
for (let index = 0; index + 1 < invalidEventType.length; index += 1) {
  if (invalidEventType[index] === 7 && invalidEventType[index + 1] === EVENT_ACTION) {
    invalidEventType[index + 1] = 99;
    break;
  }
}
const invalidRows = [
  `ts-invalid-version\tevent\t${hex(invalidEvent)}\terror\terror`,
  `ts-invalid-event-type\tevent\t${hex(invalidEventType)}\terror\terror`,
  `ts-invalid-unknown-field\tevent\t${hex(unknownFieldResult)}\terror\terror`,
];
await writeFile(
  resolve(outputDir, "invalid.hex"),
  `# protocol-golden-v5\n# id\tmessage\tpayload_hex\trust_expected\tts_expected\n${invalidRows.join("\n")}\n`,
);
await writeFile(
  resolve(outputDir, "frames.hex"),
  `# protocol-golden-v5\n# id\theader_hex\tpayload_hex\trust_expected\tts_expected\nempty\t00000000\t\tok\tok\ntruncated-header\t00\t\ttruncated\tpending\ntruncated-payload\t04000000\t01\ttruncated\tpending\nmaximum-plus-one\t01000001\t\toversize\toversize\nmaximum-exact\t00000001\t\ttruncated\tpending\n`,
);

if (verify) {
  const rustText = await readFile(resolve(outputDir, "rust_to_ts.hex"), "utf8");
  const rustRows = rustText.split("\n").filter((line) => line.length > 0 && !line.startsWith("#"));
  if (rustRows.length !== rows.length)
    throw new Error(`Rust golden row count ${rustRows.length} does not match TS ${rows.length}`);
  const expectedRows = new Map(rows.map((line) => [line.split("\t")[0], line]));
  const bodyTags = { snapshot: 1, event: 2, patch: 3, command: 4 } as const;
  for (const line of rustRows) {
    const [id, kind, encoded] = line.split("\t");
    const expected = expectedRows.get(id!);
    if (expected === undefined || expected.split("\t")[1] !== kind)
      throw new Error(`Rust golden row ${id} is missing or has the wrong message kind`);
    const payload = Uint8Array.from(Buffer.from(encoded!, "hex"));
    const decoded = Envelope.decode(payload);
    if (decoded.body?.tag !== bodyTags[kind as keyof typeof bodyTags])
      throw new Error(`Rust golden row ${id} has the wrong envelope body`);
    if (kind === "event" && decodeEvent(payload) === null)
      throw new Error(`Rust golden event ${id} failed TypeScript semantic decoding`);
    const expectedPayload = Uint8Array.from(Buffer.from(expected.split("\t")[2]!, "hex"));
    const expectedDecoded = Envelope.decode(expectedPayload);
    if (JSON.stringify(normalizeGenerated(decoded)) !== JSON.stringify(normalizeGenerated(expectedDecoded)))
      throw new Error(`Rust golden row ${id} failed TypeScript semantic verification`);
    if (expected.split("\t")[2] !== encoded)
      throw new Error(`Rust golden row ${id} does not match the canonical TypeScript bytes`);
  }
  const invalidText = await readFile(resolve(outputDir, "invalid.hex"), "utf8");
  const invalidRows = invalidText.split("\n").filter((line) => line.length > 0 && !line.startsWith("#"));
  for (const line of invalidRows) {
    const [id, kind, encoded, _rustExpected, tsExpected] = line.split("\t");
    const bytes = Uint8Array.from(Buffer.from(encoded!, "hex"));
    let actual = "ok";
    try {
      const decoded = Envelope.decode(bytes);
      if (kind === "event" && decodeEvent(bytes) === null) actual = "error";
      if (decoded.body === undefined) actual = "error";
    } catch {
      actual = "error";
    }
    if (actual !== tsExpected) throw new Error(`TypeScript invalid vector ${id} expected ${tsExpected}, got ${actual}`);
  }
  const frameText = await readFile(resolve(outputDir, "frames.hex"), "utf8");
  const frameRows = frameText.split("\n").filter((line) => line.length > 0 && !line.startsWith("#"));
  for (const line of frameRows) {
    const [id, headerHex, payloadHex, _rustExpected, tsExpected] = line.split("\t");
    const frame = Uint8Array.from(Buffer.from(`${headerHex}${payloadHex}`, "hex"));
    let actual: string;
    try {
      actual = new FrameDecoder().push(frame).length > 0 ? "ok" : "pending";
    } catch {
      actual = "oversize";
    }
    if (actual !== tsExpected) throw new Error(`TypeScript frame vector ${id} expected ${tsExpected}, got ${actual}`);
  }
}
console.log(`wrote ${rows.length} Bebop v5 golden rows to ${outputDir}`);
