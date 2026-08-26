import { encode } from "@msgpack/msgpack";
import { encodePayload } from "../packages/react-gpui/src/protocol";

const outputDir = process.argv[2] ?? "fixtures/protocol";
const fullStyle = [
  1.5,
  2.25,
  2,
  3.5,
  4.5,
  5.25,
  0x11223344,
  0xaabbccdd,
  0.75,
  [250, 15, 3, 15],
  6,
  5,
  7.5,
  1.25,
  0x12345678,
  14.5,
  700,
  3,
  4,
  2,
  1.5,
  2.5,
  3.5,
  4.5,
  1,
  2,
  6.5,
  10.5,
  20.5,
  30.5,
  40.5,
  0.75,
  6,
  1,
  -8,
  4,
  12,
  6,
  2,
  3,
  [1, [-2, 3, 4, 1, 0x01020380, 1]],
  "Avenir Next",
];
const doubleStyle = fullStyle.map((value, index) =>
  index === 40
    ? [
        2,
        [
          [-2, 3, 4, 0, 0x11223344, 0],
          [0, -1, 8, 2, 0xaabbccdd, 1],
        ],
      ]
    : value,
);
const accessibility = [5, "golden label", "golden description", false, true, false, "42"];
const textInput = [1, "text", "placeholder", false, false, true, 4, 1, 3, 1, 2, 8, false];
const virtualList = [2, 20, 2, 9, 24.5, 3];
const image = [3, "assets/😀.png", 3];
const drag = [4, "card"];

function hex(bytes: Uint8Array): string {
  return Buffer.from(bytes).toString("hex");
}
function row(id: string, kind: string, value: unknown): string {
  return `${id}\t${kind}\t${hex(encodePayload(value as never))}`;
}

const root = [1, 0, 0, 1, fullStyle, null, 0, null, accessibility, false];
const doubleRoot = [1, 0, 0, 1, doubleStyle, null, 0, null, null, false];
const doubleSnapshot = [3, 1, 7, 3, 0, 44, [doubleRoot]];
const invalidShadowStyle = fullStyle.map((value, index) => (index === 40 ? [1, [0, 0, -1, 0, 0, 0]] : value));
const invalidShadowSnapshot = [3, 1, 7, 3, 0, 45, [[1, 0, 0, 1, invalidShadowStyle, null, 0, null, null, false]]];
const text = [2, 1, 0, 2, null, "hello", 0, null, null, false];
const input = [5, 1, 2, 5, null, null, 0, textInput, null, false];
const rawText = [3, 2, 0, 4, null, "raw 😀", 0, null, null, false];
const pressable = [4, 1, 1, 3, null, null, 7, null, accessibility, false];
const list = [6, 1, 3, 6, null, null, 0, virtualList, null, false];
const dragNode = [8, 1, 5, 3, null, null, 11, drag, null, false];
const imageNode = [7, 1, 4, 7, null, null, 0, image, null, false];
const snapshot = [3, 1, 7, 3, 0, 42, [root, text, rawText, pressable, input, list, imageNode, dragNode]];
const patch = [
  3,
  3,
  7,
  3,
  42,
  43,
  [
    [1, 8, 1, 5, 3, null, null, 11, drag, null, false],
    [2, 4, 63, fullStyle, "updated", 12, [3, "assets/logo.png", 2], accessibility, true],
    [3, 4, 1, 0],
    [4, 6],
  ],
];

const rows = [
  row("ts-snapshot-box-shadow-double", "snapshot", doubleSnapshot),
  row("ts-snapshot-all-kinds", "snapshot", snapshot),
  row("ts-patch-all-operations", "patch", patch),
  row("ts-event-press", "event", [3, 2, 7, 3, 42, 1, 4, 7, 1, null]),
  row("ts-event-hover", "event", [3, 2, 7, 3, 42, 10, 4, 7, 11, null]),
  row("ts-event-submit", "event", [3, 2, 7, 3, 42, 9, 5, 9, 13, null]),
  row("ts-event-submit-text", "event", [3, 2, 7, 3, 42, 16, 5, 9, 13, "submitted text"]),
  row("ts-event-surface-closed", "event", [3, 2, 7, 3, 42, 17, 0, 0, 16, null]),
  row("ts-event-action", "event", [3, 2, 7, 3, 42, 21, 1, 0, 17, "open"]),
  row("ts-event-notification-response", "event", [3, 2, 7, 3, 42, 22, 1, 0, 21, ["react-gpui:7:120", "open"]]),
  row("ts-event-command-result-open-surface", "event", [
    3,
    2,
    7,
    3,
    42,
    18,
    1,
    0,
    6,
    [2, 117, 17, 1, true, null, [1, 41]],
  ]),
  row("ts-event-command-result-file-open", "event", [
    3,
    2,
    7,
    3,
    42,
    19,
    1,
    0,
    6,
    [2, 118, 18, 1, true, null, [5, ["/tmp/a.txt", "/tmp/b.txt"]]],
  ]),
  row("ts-event-command-result-file-save", "event", [
    3,
    2,
    7,
    3,
    42,
    20,
    1,
    0,
    6,
    [2, 119, 19, 1, true, null, [4, "/tmp/report.json"]],
  ]),
  row("ts-event-command-result-notification", "event", [
    3,
    2,
    7,
    3,
    42,
    22,
    1,
    0,
    6,
    [2, 120, 20, 1, true, null, null],
  ]),
  row("ts-event-command-result-menus", "event", [3, 2, 7, 3, 42, 23, 1, 0, 6, [2, 121, 21, 1, true, null, null]]),
  row("ts-event-text-unicode", "event", [3, 2, 7, 3, 42, 2, 5, 9, 2, [1, "hé😀", 2, 4, 2, 3, 8, true]]),
  row("ts-event-key", "event", [3, 2, 7, 3, 42, 3, 4, 7, 9, [5, "Enter", ["ctrl", "shift"], 2]]),
  row("ts-event-pointer", "event", [3, 2, 7, 3, 42, 4, 4, 7, 10, [6, 5, ["cmd"], 2, 2]]),
  row("ts-event-scroll", "event", [3, 2, 7, 3, 42, 5, 1, 0, 12, [7, 1, 3.5, -2.25, 10, 20.5, ["alt"]]]),
  row("ts-event-visible", "event", [3, 2, 7, 3, 42, 6, 6, 13, 7, [3, 2, 9]]),
  row("ts-event-animation", "event", [3, 2, 7, 3, 42, 7, 1, 0, 8, [4, 4]]),
  row("ts-event-command-result", "event", [3, 2, 7, 3, 42, 8, 1, 0, 6, [2, 109, 10, 1, false, "rejected", null]]),
  row("ts-event-window-resize", "event", [3, 2, 7, 3, 42, 11, 1, 0, 14, [800.5, 600.5]]),
  row("ts-event-window-activation", "event", [3, 2, 7, 3, 42, 12, 1, 0, 15, true]),
  row("ts-event-window-appearance", "event", [3, 2, 7, 3, 42, 16, 1, 0, 18, "dark"]),
  row("ts-event-layout", "event", [3, 2, 7, 3, 42, 17, 4, 7, 19, [12.5, -3.25, 100, 48.75]]),
  row("ts-event-drag-over", "event", [3, 2, 7, 3, 42, 18, 8, 11, 20, [1, "card"]]),
  row("ts-event-drag-drop", "event", [3, 2, 7, 3, 42, 19, 8, 11, 20, [2, "card"]]),
  row("ts-event-drag-external", "event", [3, 2, 7, 3, 42, 20, 8, 11, 20, [3, ["/tmp/a.txt", "/tmp/b"]]]),
  row("ts-event-command-result-size", "event", [
    3,
    2,
    7,
    3,
    42,
    13,
    1,
    0,
    6,
    [2, 110, 13, 1, true, null, [2, [800.5, 600.5]]],
  ]),
  row("ts-event-command-result-focus", "event", [3, 2, 7, 3, 42, 14, 4, 0, 6, [2, 111, 14, 4, true, null, [3, true]]]),
  row("ts-event-command-result-clipboard", "event", [
    3,
    2,
    7,
    3,
    42,
    15,
    1,
    0,
    6,
    [2, 112, 16, 1, true, null, [4, "pasted text"]],
  ]),
  row("ts-command-focus", "command", [3, 4, 7, 3, 42, 101, 4, 1, null]),
  row("ts-command-selection", "command", [3, 4, 7, 3, 42, 103, 5, 3, [2, 4]]),
  row("ts-command-scroll-index", "command", [3, 4, 7, 3, 42, 104, 6, 4, [9, 0]]),
  row("ts-command-scroll-end", "command", [3, 4, 7, 3, 42, 105, 6, 5, null]),
  row("ts-command-title", "command", [3, 4, 7, 3, 42, 106, 1, 6, "Golden 😀 title"]),
  row("ts-command-resize", "command", [3, 4, 7, 3, 42, 107, 1, 7, [800, 600]]),
  row("ts-command-zoom", "command", [3, 4, 7, 3, 42, 108, 1, 8, null]),
  row("ts-command-fullscreen", "command", [3, 4, 7, 3, 42, 109, 1, 9, null]),
  row("ts-command-url", "command", [3, 4, 7, 3, 42, 110, 1, 10, "https://example.com/😀"]),
  row("ts-command-focus-next", "command", [3, 4, 7, 3, 42, 111, 1, 11, null]),
  row("ts-command-focus-prev", "command", [3, 4, 7, 3, 42, 112, 1, 12, null]),
  row("ts-command-get-window-size", "command", [3, 4, 7, 3, 42, 113, 1, 13, null]),
  row("ts-command-clipboard-write", "command", [3, 4, 7, 3, 42, 115, 1, 15, "clipboard text"]),
  row("ts-command-clipboard-read", "command", [3, 4, 7, 3, 42, 116, 1, 16, null]),
  row("ts-command-open-surface", "command", [3, 4, 7, 3, 42, 117, 1, 17, ["Child", [640, 480]]]),
  row("ts-command-file-dialog-open", "command", [3, 4, 7, 3, 42, 118, 1, 18, ["Choose", [1, 1]]]),
  row("ts-command-file-dialog-save", "command", [3, 4, 7, 3, 42, 119, 1, 19, "report.json"]),
  row("ts-command-notification", "command", [3, 4, 7, 3, 42, 120, 1, 20, ["Done", "Finished", [["open", "Open"]]]]),
  row("ts-command-set-menus", "command", [
    3,
    4,
    7,
    3,
    42,
    121,
    1,
    21,
    [["File", [[1, "open", [true, true]], [0], [2, ["More", [[1, "other"]]]]]]],
  ]),
  row("ts-command-get-focus", "command", [3, 4, 7, 3, 42, 114, 4, 14, null]),
  row("ts-command-blur", "command", [3, 4, 7, 3, 42, 102, 4, 2, null]),
].sort();

await Bun.write(
  `${outputDir}/ts_to_rust.hex`,
  `# protocol-golden-v1\n# id\tmessage\tpayload_hex\n${rows.join("\n")}\n`,
);
const invalid = [
  `ts-invalid-file-dialog-empty-paths\tevent\t${hex(
    encode([3, 2, 7, 3, 42, 21, 1, 0, 6, [2, 120, 18, 1, true, null, [5, []]]]),
  )}\terror\tevent-null`,
  `ts-invalid-style-box-shadow-negative-blur\tsnapshot\t${hex(encode(invalidShadowSnapshot))}\terror\traw`,
  `ts-invalid-event-type\tevent\t${hex(encode([3, 2, 7, 3, 42, 1, 1, 0, 99, null]))}\terror\tevent-null`,
  `ts-invalid-surface-close-node\tevent\t${hex(encode([3, 2, 7, 3, 42, 1, 1, 0, 16, null]))}\terror\tevent-null`,
  `ts-invalid-appearance-bool\tevent\t${hex(encode([3, 2, 7, 3, 42, 1, 1, 0, 18, true]))}\terror\tevent-null`,
  `ts-invalid-appearance-value\tevent\t${hex(encode([3, 2, 7, 3, 42, 2, 1, 0, 18, "system"]))}\terror\tevent-null`,
  `ts-invalid-event-payload-tag\tevent\t${hex(encode([3, 2, 7, 3, 42, 1, 1, 0, 12, [99]]))}\terror\tevent-null`,
  `ts-invalid-host-kind\tsnapshot\t${hex(encode([3, 1, 7, 3, 0, 1, [[2, 1, 0, 99, null, null, 0, null, null, false]]]))}\tok\traw`,
  "ts-invalid-messagepack\tevent\tc1\terror\tdecode-null",
];
await Bun.write(
  `${outputDir}/invalid.hex`,
  `# protocol-golden-v1\n# id\tmessage\tpayload_hex\trust_expected\tts_expected\n${invalid.join("\n")}\n`,
);
const frames = [
  "empty\t00000000\t\tok\tok",
  "truncated-header\t00\t\ttruncated\tpending",
  "truncated-payload\t04000000\t01\ttruncated\tpending",
  "maximum-plus-one\t01000001\t\toversize\toversize",
  "maximum-exact\t00000001\t\ttruncated\tpending",
];
await Bun.write(
  `${outputDir}/frames.hex`,
  `# protocol-golden-v1\n# id\theader_hex\tpayload_hex\trust_expected\tts_expected\n${frames.join("\n")}\n`,
);
