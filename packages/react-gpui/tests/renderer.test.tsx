import React, { useCallback, createRef, useState } from "react";
import { describe, expect, it } from "bun:test";
import {
  Image,
  MemoryTransport,
  Pressable,
  StyleSheet,
  Text,
  TextInput,
  TextInputHandle,
  View,
  ViewHandle,
  VirtualList,
  VirtualListHandle,
  createRoot,
} from "../src/index";
import { decode } from "@msgpack/msgpack";
import {
  COMMAND_BLUR,
  COMMAND_CLIPBOARD_READ,
  COMMAND_CLIPBOARD_WRITE,
  COMMAND_FILE_DIALOG_OPEN,
  COMMAND_FILE_DIALOG_SAVE,
  COMMAND_FOCUS,
  COMMAND_FOCUS_NEXT,
  COMMAND_FOCUS_PREV,
  COMMAND_GET_FOCUS,
  COMMAND_GET_WINDOW_SIZE,
  COMMAND_OPEN_URL,
  COMMAND_RESIZE_WINDOW,
  COMMAND_TOGGLE_FULLSCREEN,
  COMMAND_ZOOM_WINDOW,
  EVENT_HOVER,
  EVENT_KEY,
  EVENT_POINTER,
  EVENT_POINTER_DOWN,
  EVENT_POINTER_UP,
  EVENT_SCROLL,
  EVENT_SUBMIT,
  EVENT_WINDOW_ACTIVATION,
  EVENT_WINDOW_RESIZE,
  SCROLL_DELTA_LINES,
  SCROLL_DELTA_PIXELS,
  decodeEvent,
  encodeFrame,
  framePayload,
  FrameDecoder,
  PROTOCOL_VERSION,
  MAX_CLIPBOARD_TEXT_BYTES,
} from "../src/protocol";

type Snapshot = readonly [number, number, number, number, number, number, readonly unknown[][]];

function snapshots(transport: MemoryTransport): Snapshot[] {
  return transport.submitted.map((frame) => {
    const payload = frame.slice(4);
    return decode(payload) as Snapshot;
  });
}
function message(transport: MemoryTransport, index: number): readonly unknown[] {
  return decode(transport.submitted[index].slice(4)) as readonly unknown[];
}

describe("protocol framing", () => {
  it("decodes fragmented and coalesced frames", () => {
    const decoder = new FrameDecoder(1024);
    const first = encodeFrame([PROTOCOL_VERSION, 2, 1, 1, 1, 1, 9, 7, 1, null]);
    const second = encodeFrame([PROTOCOL_VERSION, 2, 1, 1, 1, 2, 10, 8, 1, null]);
    expect(decoder.push(first.slice(0, 2))).toEqual([]);
    expect(decoder.push(new Uint8Array([...first.slice(2), ...second]))).toEqual([first.slice(4), second.slice(4)]);
    expect(() => new FrameDecoder(2).push(first)).toThrow();
  });
  it("handles a large frame delivered one byte at a time", () => {
    const decoder = new FrameDecoder(100_000);
    const payload = new Uint8Array(80_000);
    payload.fill(7);
    const frame = framePayload(payload);
    let decoded: Uint8Array[] = [];
    for (const byte of frame) decoded = decoder.push(new Uint8Array([byte]));
    expect(decoded).toHaveLength(1);
    expect(decoded[0]).toEqual(payload);
  });
  it("rejects malformed tagged event payloads", () => {
    const malformed = encodeFrame([PROTOCOL_VERSION, 2, 1, 1, 1, 1, 2, 7, 6, [2, 1, 99, 2, true, null]] as never);
    expect(decodeEvent(malformed.slice(4))).toBeNull();
  });
  it("decodes pointer and hover payloads and rejects invalid pointer buttons", () => {
    const pointer = encodeFrame([PROTOCOL_VERSION, 2, 1, 1, 1, 1, 2, 7, EVENT_POINTER, [6, 4, ["cmd"], 1, 2]]);
    const hover = encodeFrame([PROTOCOL_VERSION, 2, 1, 1, 1, 2, 2, 7, EVENT_HOVER, null]);
    const invalid = encodeFrame([PROTOCOL_VERSION, 2, 1, 1, 1, 3, 2, 7, EVENT_POINTER, [6, 9, [], 1, 1]] as never);
    expect(decodeEvent(pointer.slice(4))).not.toBeNull();
    expect(decodeEvent(hover.slice(4))).not.toBeNull();
    expect(decodeEvent(invalid.slice(4))).toBeNull();
  });
  it("decodes pixel and line scroll payloads and rejects invalid scroll values", () => {
    const pixels = encodeFrame([
      PROTOCOL_VERSION,
      2,
      1,
      1,
      1,
      1,
      2,
      7,
      EVENT_SCROLL,
      [7, SCROLL_DELTA_PIXELS, 12.5, -8, 40, 24, ["shift"]],
    ]);
    const lines = encodeFrame([
      PROTOCOL_VERSION,
      2,
      1,
      1,
      1,
      2,
      2,
      7,
      EVENT_SCROLL,
      [7, SCROLL_DELTA_LINES, 2, -1.5, 40, 24, ["cmd"]],
    ]);
    const invalid = encodeFrame([
      PROTOCOL_VERSION,
      2,
      1,
      1,
      1,
      3,
      2,
      7,
      EVENT_SCROLL,
      [7, 99, Number.NaN, 0, 0, 0, []],
    ] as never);
    expect(decodeEvent(pixels.slice(4))).not.toBeNull();
    expect(decodeEvent(lines.slice(4))).not.toBeNull();
    expect(decodeEvent(invalid.slice(4))).toBeNull();
  });
  it("decodes window observation events and optional command values", () => {
    const resize = encodeFrame([PROTOCOL_VERSION, 2, 1, 1, 1, 1, 1, 0, EVENT_WINDOW_RESIZE, [800, 600]]);
    const activation = encodeFrame([PROTOCOL_VERSION, 2, 1, 1, 1, 2, 1, 0, EVENT_WINDOW_ACTIVATION, true]);
    const oldResult = encodeFrame([PROTOCOL_VERSION, 2, 1, 1, 1, 3, 1, 0, 6, [2, 1, COMMAND_FOCUS, 2, true, null]]);
    const sizeResult = encodeFrame([
      PROTOCOL_VERSION,
      2,
      1,
      1,
      1,
      4,
      1,
      0,
      6,
      [2, 2, COMMAND_GET_WINDOW_SIZE, 1, true, null, [2, [800, 600]]],
    ]);
    const focusResult = encodeFrame([
      PROTOCOL_VERSION,
      2,
      1,
      1,
      1,
      5,
      2,
      0,
      6,
      [2, 3, COMMAND_GET_FOCUS, 2, true, null, [3, true]],
    ]);
    const malformed = encodeFrame([PROTOCOL_VERSION, 2, 1, 1, 1, 6, 1, 0, EVENT_WINDOW_RESIZE, [800, -1]] as never);
    expect(decodeEvent(resize.slice(4))).not.toBeNull();
    expect(decodeEvent(activation.slice(4))).not.toBeNull();
    expect(decodeEvent(oldResult.slice(4))).not.toBeNull();
    expect(decodeEvent(sizeResult.slice(4))).not.toBeNull();
    expect(decodeEvent(focusResult.slice(4))).not.toBeNull();
    expect(decodeEvent(malformed.slice(4))).toBeNull();
  });
  it("validates file dialog command results and rejects empty path arrays", () => {
    const valid = encodeFrame([
      PROTOCOL_VERSION,
      2,
      1,
      1,
      1,
      7,
      1,
      0,
      6,
      [2, 4, COMMAND_FILE_DIALOG_OPEN, 1, true, null, [5, ["/tmp/file.txt"]]],
    ]);
    const empty = encodeFrame([
      PROTOCOL_VERSION,
      2,
      1,
      1,
      1,
      8,
      1,
      0,
      6,
      [2, 5, COMMAND_FILE_DIALOG_OPEN, 1, true, null, [5, []]],
    ] as never);
    const save = encodeFrame([
      PROTOCOL_VERSION,
      2,
      1,
      1,
      1,
      9,
      1,
      0,
      6,
      [2, 6, COMMAND_FILE_DIALOG_SAVE, 1, true, null, [4, "/tmp/file.txt"]],
    ]);
    expect(decodeEvent(valid.slice(4))).not.toBeNull();
    expect(decodeEvent(save.slice(4))).not.toBeNull();
    expect(decodeEvent(empty.slice(4))).toBeNull();
  });
});
describe("window observation and value commands", () => {
  it("dispatches resize and activation events to root callbacks", () => {
    const transport = new MemoryTransport();
    const resized: Array<[number, number]> = [];
    const active: boolean[] = [];
    const root = createRoot(transport, {
      surfaceId: 91,
      epoch: 92,
      onWindowResize: (width, height) => resized.push([width, height]),
      onWindowActivation: (value) => active.push(value),
    });
    root.render(<View />);
    transport.push(encodeFrame([PROTOCOL_VERSION, 2, 91, 92, 1, 1, 1, 0, EVENT_WINDOW_RESIZE, [640, 480]]));
    transport.push(encodeFrame([PROTOCOL_VERSION, 2, 91, 92, 1, 2, 1, 0, EVENT_WINDOW_ACTIVATION, false]));
    expect(resized).toEqual([[640, 480]]);
    expect(active).toEqual([false]);
    root.unmount();
  });

  it("reads window size and node focus through value command results", async () => {
    const transport = new MemoryTransport();
    const root = createRoot(transport, { surfaceId: 93, epoch: 94 });
    root.render(<View />);
    const sizePromise = root.getWindowSize();
    const sizeCommand = message(transport, 1);
    expect(sizeCommand[7]).toBe(COMMAND_GET_WINDOW_SIZE);
    transport.push(
      encodeFrame([
        PROTOCOL_VERSION,
        2,
        93,
        94,
        1,
        1,
        1,
        0,
        6,
        [2, sizeCommand[5] as number, COMMAND_GET_WINDOW_SIZE, 1, true, null, [2, [800, 600]]],
      ]),
    );
    await expect(sizePromise).resolves.toEqual([800, 600]);

    const focusTransport = new MemoryTransport();
    const focusRoot = createRoot(focusTransport, { surfaceId: 95, epoch: 96 });
    const ref = createRef<ViewHandle>();
    focusRoot.render(<View ref={ref} focusable onKeyDown={() => undefined} />);
    const nodeId = (
      snapshots(focusTransport)[0][6].find((node) => node[0] !== 1 && node[3] === 1) as readonly unknown[]
    )[0] as number;
    const focusPromise = ref.current!.isFocused();
    const focusCommand = message(focusTransport, 1);
    expect(focusCommand[7]).toBe(COMMAND_GET_FOCUS);
    focusTransport.push(
      encodeFrame([
        PROTOCOL_VERSION,
        2,
        95,
        96,
        1,
        1,
        nodeId,
        0,
        6,
        [2, focusCommand[5] as number, COMMAND_GET_FOCUS, nodeId, true, null, [3, true]],
      ]),
    );
    await expect(focusPromise).resolves.toBe(true);
    focusRoot.unmount();
  });
});

describe("clipboard commands", () => {
  it("writes and reads clipboard text through root commands", async () => {
    const transport = new MemoryTransport();
    const root = createRoot(transport, { surfaceId: 97, epoch: 98 });
    root.render(<View />);

    const writePromise = root.setClipboardText("copied text");
    const writeCommand = message(transport, 1);
    expect(writeCommand[7]).toBe(COMMAND_CLIPBOARD_WRITE);
    expect(writeCommand[8]).toBe("copied text");
    transport.push(
      encodeFrame([
        PROTOCOL_VERSION,
        2,
        97,
        98,
        1,
        1,
        1,
        0,
        6,
        [2, writeCommand[5] as number, COMMAND_CLIPBOARD_WRITE, 1, true, null],
      ]),
    );
    await expect(writePromise).resolves.toBeUndefined();

    const readPromise = root.getClipboardText();
    const readCommand = message(transport, 2);
    expect(readCommand[7]).toBe(COMMAND_CLIPBOARD_READ);
    expect(readCommand[8]).toBeNull();
    transport.push(
      encodeFrame([
        PROTOCOL_VERSION,
        2,
        97,
        98,
        1,
        2,
        1,
        0,
        6,
        [2, readCommand[5] as number, COMMAND_CLIPBOARD_READ, 1, true, null, [4, "pasted text"]],
      ]),
    );
    await expect(readPromise).resolves.toBe("pasted text");

    const failedRead = root.getClipboardText();
    const failedCommand = message(transport, 3);
    transport.push(
      encodeFrame([
        PROTOCOL_VERSION,
        2,
        97,
        98,
        1,
        3,
        1,
        0,
        6,
        [2, failedCommand[5] as number, COMMAND_CLIPBOARD_READ, 1, false, "clipboard has no text content"],
      ]),
    );
    await expect(failedRead).rejects.toThrow("clipboard has no text content");

    const before = transport.submitted.length;
    await expect(root.setClipboardText("x".repeat(MAX_CLIPBOARD_TEXT_BYTES + 1))).rejects.toThrow(
      "clipboard text exceeds the supported size",
    );
    expect(transport.submitted).toHaveLength(before);
    root.unmount();
  });
});

describe("keyboard reachability", () => {
  it("routes Pressable key events and press activation, while disabled blocks interaction", () => {
    const transport = new MemoryTransport();
    const keys: string[] = [];
    let presses = 0;
    const root = createRoot(transport, { surfaceId: 99, epoch: 100 });
    root.render(
      <Pressable
        focusable
        onKeyDown={(event) => keys.push(`${event.key}:${event.action}`)}
        onPress={() => {
          presses += 1;
        }}
      />,
    );
    const pressable = snapshots(transport)[0][6].find((node) => node[3] === 3) as readonly unknown[];
    expect(pressable[9]).toBe(true);
    expect(pressable[6]).not.toBe(0);
    const listenerId = pressable[6] as number;
    transport.push(
      encodeFrame([
        PROTOCOL_VERSION,
        2,
        99,
        100,
        1,
        1,
        pressable[0] as number,
        listenerId,
        EVENT_KEY,
        [5, "Enter", [], 1],
      ]),
    );
    transport.push(encodeFrame([PROTOCOL_VERSION, 2, 99, 100, 1, 2, pressable[0] as number, listenerId, 1, null]));
    expect(keys).toEqual(["Enter:down"]);
    expect(presses).toBe(1);
    root.unmount();

    const disabledTransport = new MemoryTransport();
    let disabledPresses = 0;
    const disabledRoot = createRoot(disabledTransport, { surfaceId: 101, epoch: 102 });
    disabledRoot.render(
      <Pressable
        focusable
        disabled
        onKeyDown={() => {
          throw new Error("disabled Pressable received key input");
        }}
        onPress={() => {
          disabledPresses += 1;
        }}
      />,
    );
    const disabled = snapshots(disabledTransport)[0][6].find((node) => node[3] === 3) as readonly unknown[];
    expect(disabled[9]).toBe(false);
    expect(disabled[6]).toBe(0);
    disabledRoot.unmount();
    expect(disabledPresses).toBe(0);
  });

  it("routes TextInput onKeyDown through the existing key event wire", () => {
    const transport = new MemoryTransport();
    const keys: string[] = [];
    const root = createRoot(transport, { surfaceId: 103, epoch: 104 });
    root.render(<TextInput value="" onKeyDown={(event) => keys.push(`${event.key}:${event.action}`)} />);
    const input = snapshots(transport)[0][6].find((node) => node[3] === 5) as readonly unknown[];
    transport.push(
      encodeFrame([
        PROTOCOL_VERSION,
        2,
        103,
        104,
        1,
        1,
        input[0] as number,
        input[6] as number,
        EVENT_KEY,
        [5, "Escape", [], 1],
      ]),
    );
    expect(keys).toEqual(["Escape:down"]);
    root.unmount();
  });
});

describe("styles", () => {
  it("validates, freezes, and encodes RGBA styles", () => {
    const styles = StyleSheet.create({
      root: {
        width: 10,
        padding: 2,
        gap: 4,
        justifyContent: "space-between",
        alignItems: "baseline",
        borderRadius: 3,
        borderWidth: 2,
        borderColor: "#11223344",
        fontSize: 14,
        fontWeight: "semibold",
        backgroundColor: "#12345678",
        color: "#abcdef",
        overflow: "scroll",
        lineClamp: 3,
        textOverflow: "ellipsis",
        marginTop: 1,
        marginRight: 2,
        marginBottom: 3,
        marginLeft: 4,
        fontStyle: "italic",
        textDecoration: "lineThrough",
        lineHeight: 20,
        minWidth: 5,
        maxWidth: 500,
        minHeight: 6,
        maxHeight: 600,
        flexShrink: 0.5,
        alignSelf: "center",
      },
    });
    expect(Object.isFrozen(styles)).toBe(true);
    expect(Object.isFrozen(styles.root)).toBe(true);
    const transport = new MemoryTransport();
    const root = createRoot(transport, { surfaceId: 3, epoch: 4 });
    root.render(<View style={styles.root} />);
    expect(snapshots(transport)[0][6][1][4]).toEqual([
      10,
      null,
      0,
      null,
      2,
      4,
      0x12345678,
      0xabcdefff,
      null,
      null,
      4,
      5,
      3,
      2,
      0x11223344,
      14,
      600,
      3,
      3,
      2,
      1,
      2,
      3,
      4,
      1,
      2,
      20,
      5,
      500,
      6,
      600,
      0.5,
      5,
    ]);
    root.render(<View style={{ opacity: 0, transition: { durationMs: 100 } }} />);
    expect(((message(transport, 1)[6] as readonly unknown[][])[0][3] as readonly unknown[])[9]).toEqual([
      100, 0, 3, 15,
    ]);
    expect(() => StyleSheet.create({ bad: { width: -1 } })).toThrow();
    expect(() => StyleSheet.create({ bad: { gap: Number.NaN } })).toThrow();
    expect(() => StyleSheet.create({ bad: { backgroundColor: "red" } })).toThrow();
    expect(() => StyleSheet.create({ bad: { flexGrow: -1 } })).toThrow();
    expect(() => StyleSheet.create({ bad: { justifyContent: "invalid" as never } })).toThrow();
    expect(() => StyleSheet.create({ bad: { alignItems: "invalid" as never } })).toThrow();
    expect(() => StyleSheet.create({ bad: { borderRadius: -1 } })).toThrow();
    expect(() => StyleSheet.create({ bad: { fontSize: 0 } })).toThrow();
    expect(() => StyleSheet.create({ bad: { fontWeight: "heavyweight" as never } })).toThrow();
    expect(() => StyleSheet.create({ bad: { overflow: "invalid" as never } })).toThrow();
    expect(() => StyleSheet.create({ bad: { lineClamp: 0 } })).toThrow();
    expect(() => StyleSheet.create({ bad: { lineClamp: -1 } })).toThrow();
    expect(() => StyleSheet.create({ bad: { lineClamp: 3.5 } })).toThrow();
    expect(() => StyleSheet.create({ bad: { lineClamp: 101 } })).toThrow();
    expect(() => StyleSheet.create({ bad: { textOverflow: "middle" as never } })).toThrow();
    expect(() => StyleSheet.create({ bad: { marginTop: -1 } })).toThrow();
    expect(() => StyleSheet.create({ bad: { lineHeight: Number.NaN } })).toThrow();
    expect(() => StyleSheet.create({ bad: { minWidth: -1 } })).toThrow();
    expect(() => StyleSheet.create({ bad: { maxHeight: Number.POSITIVE_INFINITY } })).toThrow();
    expect(() => StyleSheet.create({ bad: { flexShrink: -1 } })).toThrow();
    expect(() => StyleSheet.create({ bad: { fontStyle: "oblique" as never } })).toThrow();
    expect(() => StyleSheet.create({ bad: { textDecoration: "double" as never } })).toThrow();
    expect(() => StyleSheet.create({ bad: { alignSelf: "invalid" as never } })).toThrow();
  });
});
it("encodes Image host properties and rejects Image children", () => {
  const transport = new MemoryTransport();
  const root = createRoot(transport, { surfaceId: 79, epoch: 80 });
  root.render(<Image source="assets/logo.png" objectFit="cover" style={{ width: 120, height: 48 }} />);
  const image = snapshots(transport)[0][6].find((node) => node[3] === 7) as readonly unknown[];
  expect(image[3]).toBe(7);
  expect(image[7]).toEqual([3, "assets/logo.png", 3]);
  root.unmount();
  const unicodeTransport = new MemoryTransport();
  const unicodeRoot = createRoot(unicodeTransport, { surfaceId: 83, epoch: 84 });
  const unicodeSource = "é".repeat(512);
  unicodeRoot.render(<Image source={unicodeSource} />);
  expect((snapshots(unicodeTransport)[0][6][1][7] as readonly unknown[])[1]).toBe(unicodeSource);
  unicodeRoot.unmount();

  const oversizedRoot = createRoot(new MemoryTransport(), { surfaceId: 85, epoch: 86 });
  expect(() => oversizedRoot.render(<Image source={"é".repeat(513)} />)).toThrow("UTF-8 bytes");

  const invalidTransport = new MemoryTransport();
  const invalidRoot = createRoot(invalidTransport, { surfaceId: 81, epoch: 82 });
  expect(() =>
    invalidRoot.render(React.createElement("Image", { source: "assets/logo.png" }, React.createElement("View"))),
  ).toThrow("Image nodes cannot contain children");
  expect(invalidTransport.submitted).toHaveLength(0);
});
describe("validation errors", () => {
  it("Error Boundary captures render validation and later commits recover", () => {
    let captured = false;
    class Boundary extends React.Component<{ children?: React.ReactNode }, { failed: boolean }> {
      state = { failed: false };

      static getDerivedStateFromError(): { failed: boolean } {
        captured = true;
        return { failed: true };
      }

      render(): React.ReactNode {
        return this.state.failed ? <Text>recovered</Text> : this.props.children;
      }
    }

    const transport = new MemoryTransport();
    const root = createRoot(transport, { surfaceId: 71, epoch: 72 });
    const originalError = console.error;
    console.error = () => {};
    try {
      root.render(
        <Boundary>
          <View style={{ width: -1 }} />
        </Boundary>,
      );
    } finally {
      console.error = originalError;
    }
    expect(captured).toBe(true);
    expect(transport.submitted).toHaveLength(0);

    root.render(
      <View>
        <Text>healthy</Text>
      </View>,
    );
    expect(transport.submitted).toHaveLength(1);
    expect(message(transport, 0)[1] as number).toBe(1);
    root.unmount();
  });

  it("without an Error Boundary render validation throws and emits no invalid frame", () => {
    const transport = new MemoryTransport();
    const root = createRoot(transport, { surfaceId: 73, epoch: 74 });
    const originalError = console.error;
    console.error = () => {};
    try {
      expect(() => root.render(<View style={{ width: -1 }} />)).toThrow();
    } finally {
      console.error = originalError;
    }
    expect(transport.submitted).toHaveLength(0);
    root.unmount();
  });
});

describe("renderer commits", () => {
  it("submits once per commit and preserves host and listener identity", () => {
    const transport = new MemoryTransport();
    const root = createRoot(transport, { surfaceId: 5, epoch: 6 });
    const onPress = () => undefined;
    root.render(
      <View>
        <Text>before</Text>
        <Pressable onPress={onPress}>
          <Text>button</Text>
        </Pressable>
      </View>,
    );
    expect(transport.submitted).toHaveLength(1);
    const before = snapshots(transport)[0][6];
    root.render(
      <View>
        <Text>after</Text>
        <Pressable onPress={() => undefined}>
          <Text>button</Text>
        </Pressable>
      </View>,
    );
    expect(transport.submitted).toHaveLength(2);
    const patch = message(transport, 1);
    const operations = patch[6] as readonly unknown[][];
    expect(patch.slice(0, 6)).toEqual([3, 3, 5, 6, 1, 2]);
    expect(operations).toHaveLength(1);
    expect(operations[0][0]).toBe(2);
    expect(operations[0][4]).toBe("after");
  });

  it("dispatches press updates in a batched commit", () => {
    const transport = new MemoryTransport();
    const root = createRoot(transport, { surfaceId: 7, epoch: 8 });
    function Counter() {
      const [count, setCount] = useState(0);
      const increment = useCallback(() => setCount((current) => current + 1), []);
      return (
        <View>
          <Text>{count}</Text>
          <Pressable onPress={increment} />
        </View>
      );
    }
    root.render(<Counter />);
    const initial = snapshots(transport)[0];
    const pressable = initial[6].find((node) => node[3] === 3) as readonly unknown[];
    const event = encodeFrame([
      PROTOCOL_VERSION,
      2,
      7,
      8,
      initial[5],
      1,
      pressable[0] as number,
      pressable[6] as number,
      1,
      null,
    ]);
    transport.push(event.slice(0, 3));
    transport.push(event.slice(3));
    transport.push(event);
    const update = (message(transport, 1)[6] as readonly unknown[][]).find((operation) => (operation[2] as number) & 2);
    expect(update?.[4]).toBe("1");
    const futureRevision = encodeFrame([
      PROTOCOL_VERSION,
      2,
      7,
      8,
      initial[5] + 2,
      2,
      pressable[0] as number,
      pressable[6] as number,
      1,
      null,
    ]);
    transport.push(futureRevision);
    expect(transport.submitted).toHaveLength(2);
  });
  it("encodes keyed reorder, insertion, and subtree deletion as incremental operations", () => {
    const transport = new MemoryTransport();
    const root = createRoot(transport, { surfaceId: 17, epoch: 18 });
    const renderList = (items: string[]) => (
      <View>
        {items.map((item) => (
          <Text key={item}>{item}</Text>
        ))}
      </View>
    );
    root.render(renderList(["a", "b", "c"]));
    const first = snapshots(transport)[0][6];
    root.render(renderList(["c", "a", "b"]));
    const reorder = message(transport, 1)[6] as readonly unknown[][];
    expect(reorder.some((operation) => operation[0] === 3)).toBe(true);
    expect(reorder.every((operation) => operation[0] !== 1 && operation[0] !== 4)).toBe(true);

    root.render(
      <View>
        {renderList(["c", "a", "b"])}
        <View key="new">
          <Text>new</Text>
        </View>
      </View>,
    );
    const insertion = message(transport, 2)[6] as readonly unknown[][];
    expect(insertion.some((operation) => operation[0] === 1)).toBe(true);
    const createdIds = insertion.filter((operation) => operation[0] === 1).map((operation) => operation[1]);
    root.render(<View>{renderList(["c", "a", "b"])}</View>);
    const deletion = message(transport, 3)[6] as readonly unknown[][];
    expect(deletion.some((operation) => operation[0] === 4 && createdIds.includes(operation[1]))).toBe(true);
    expect(first.length).toBeGreaterThan(1);
  });

  it("uses synthetic root id 1 for direct fragment creates and moves", () => {
    const transport = new MemoryTransport();
    const root = createRoot(transport, { surfaceId: 21, epoch: 22 });
    const renderItems = (items: string[]) => (
      <>
        {items.map((item) => (
          <Text key={item}>{item}</Text>
        ))}
      </>
    );
    root.render(renderItems(["a", "b"]));
    root.render(renderItems(["a", "b", "c"]));
    const createOperations = message(transport, 1)[6] as readonly unknown[][];
    expect(createOperations.some((operation) => operation[0] === 1 && operation[2] === 1)).toBe(true);
    root.render(renderItems(["c", "a", "b"]));
    const moveOperations = message(transport, 2)[6] as readonly unknown[][];
    expect(moveOperations.some((operation) => operation[0] === 3 && operation[2] === 1)).toBe(true);
  });

  it("emits nested ancestor and child moves in parent-before-child order", () => {
    const transport = new MemoryTransport();
    const root = createRoot(transport, { surfaceId: 23, epoch: 24 });
    const renderNested = (parents: string[], children: Record<string, string[]>) => (
      <View>
        {parents.map((parent) => (
          <View key={parent}>
            {children[parent].map((child) => (
              <View key={child}>
                <Text>{child}</Text>
              </View>
            ))}
          </View>
        ))}
      </View>
    );
    root.render(renderNested(["a", "b"], { a: ["a1", "a2"], b: ["b1", "b2"] }));
    root.render(renderNested(["b", "a"], { a: ["a2", "a1"], b: ["b2", "b1"] }));
    const operations = message(transport, 1)[6] as readonly unknown[][];
    const moves = operations.filter((operation) => operation[0] === 3);
    expect(moves.length).toBeGreaterThanOrEqual(2);
    expect(moves.every((operation) => operation[2] !== 0)).toBe(true);
  });

  it("encodes one update for a leaf in a large committed tree", () => {
    const transport = new MemoryTransport();
    const root = createRoot(transport, { surfaceId: 19, epoch: 20 });
    const renderItems = (changed: number) => (
      <View>
        {Array.from({ length: 1000 }, (_, index) => (
          <Text key={index}>{index === changed ? `changed-${index}` : index}</Text>
        ))}
      </View>
    );
    root.render(renderItems(-1));
    root.render(renderItems(777));
    const operations = message(transport, 1)[6] as readonly unknown[][];
    expect(operations).toHaveLength(1);
    expect(operations[0][0]).toBe(2);
    expect(operations[0][4]).toBe("changed-777");
  });

  it("routes text input change and selection events by stable listener token", () => {
    const transport = new MemoryTransport();
    const root = createRoot(transport, { surfaceId: 25, epoch: 26 });
    const changes: string[] = [];
    const selections: number[] = [];
    root.render(
      <View>
        <TextInput
          onChangeText={(value) => changes.push(value)}
          onSelectionChange={(selection) => selections.push(selection.start)}
        />
        <TextInput onChangeText={(value) => changes.push(`second:${value}`)} />
      </View>,
    );
    const inputs = snapshots(transport)[0][6].filter((node) => node[3] === 5);
    const first = inputs[0];
    const second = inputs[1];
    const event = (node: readonly unknown[], text: string, sequence: number) =>
      encodeFrame([
        PROTOCOL_VERSION,
        2,
        25,
        26,
        1,
        sequence,
        node[0] as number,
        node[6] as number,
        2,
        [1, text, 2, 2, null, null, sequence],
      ]);
    transport.push(event(second, "second", 1));
    transport.push(event(first, "first", 2));
    transport.push(
      encodeFrame([
        PROTOCOL_VERSION,
        2,
        25,
        26,
        1,
        3,
        first[0] as number,
        first[6] as number,
        3,
        [1, "first", 2, 2, null, null, 2],
      ]),
    );
    expect(changes).toEqual(["second:second", "first"]);
    expect(selections).toEqual([2]);
  });
  it("preserves uncontrolled defaults and acknowledges controlled native edits", () => {
    const uncontrolledTransport = new MemoryTransport();
    const uncontrolledRoot = createRoot(uncontrolledTransport, { surfaceId: 27, epoch: 28 });
    uncontrolledRoot.render(<TextInput defaultValue="initial" />);
    uncontrolledRoot.render(<TextInput defaultValue="replacement" />);
    expect(message(uncontrolledTransport, 1)[6] as readonly unknown[][]).toHaveLength(0);

    const controlledTransport = new MemoryTransport();
    const controlledRoot = createRoot(controlledTransport, { surfaceId: 29, epoch: 30 });
    function Controlled() {
      const [value, setValue] = useState("");
      return <TextInput value={value} onChangeText={setValue} />;
    }
    controlledRoot.render(<Controlled />);
    const input = snapshots(controlledTransport)[0][6].find((node) => node[3] === 5) as readonly unknown[];
    controlledTransport.push(
      encodeFrame([
        PROTOCOL_VERSION,
        2,
        29,
        30,
        1,
        1,
        input[0] as number,
        input[6] as number,
        2,
        [1, "native", 6, 6, null, null, 1],
      ]),
    );
    const patch = message(controlledTransport, 1);
    const propertyUpdate = (patch[6] as readonly unknown[][]).find((operation) => (operation[2] as number) & 8);
    expect(propertyUpdate?.[6]).toEqual([1, "native", null, false, false, true, 1, 6, 6, null, null, null]);
  });
  it("encodes TextInput maxLength, clamps controlled edits, and dispatches submit", () => {
    const transport = new MemoryTransport();
    const submits: string[] = [];
    const changes: string[] = [];
    const root = createRoot(transport, { surfaceId: 33, epoch: 34 });
    root.render(
      <TextInput
        maxLength={4}
        onChangeText={(value) => changes.push(value)}
        onSubmitEditing={(value) => submits.push(value)}
      />,
    );
    const input = snapshots(transport)[0][6].find((node) => node[3] === 5) as readonly unknown[];
    expect(input[7]).toEqual([1, "", null, false, false, false, 0, 0, 0, null, null, 4]);
    transport.push(
      encodeFrame([
        PROTOCOL_VERSION,
        2,
        33,
        34,
        1,
        1,
        input[0] as number,
        input[6] as number,
        2,
        [1, "12345", 5, 5, null, null, 1],
      ]),
    );
    transport.push(
      encodeFrame([PROTOCOL_VERSION, 2, 33, 34, 1, 2, input[0] as number, input[6] as number, EVENT_SUBMIT, null]),
    );
    transport.push(
      encodeFrame([PROTOCOL_VERSION, 2, 33, 34, 1, 3, input[0] as number, input[6] as number, EVENT_SUBMIT, "1234"]),
    );
    expect(changes).toEqual(["1234"]);
    expect(submits).toEqual(["", "1234"]);
    root.unmount();
  });

  it("frames TextInput commands and resolves typed command results", async () => {
    const transport = new MemoryTransport();
    const root = createRoot(transport, { surfaceId: 31, epoch: 32 });
    const ref = React.createRef<TextInputHandle>();
    root.render(<TextInput ref={ref} />);
    const commandPromise = ref.current?.setSelection(1, 1);
    const command = message(transport, 1);
    const commandNodeId = Number(command[6]);
    expect(command).toEqual([3, 4, 31, 32, 1, 1, commandNodeId, 3, [1, 1]]);
    transport.push(encodeFrame([3, 2, 31, 32, 1, 1, commandNodeId, 0, 6, [2, 1, 3, commandNodeId, true, null]]));
    await commandPromise;
  });

  it("rejects invalid text trees before transport", () => {
    const originalError = console.error;
    console.error = () => {};
    try {
      const rawTransport = new MemoryTransport();
      const rawRoot = createRoot(rawTransport, { surfaceId: 13, epoch: 14 });
      expect(() => rawRoot.render(<View>orphan raw text</View>)).toThrow("Raw text must be a direct child of Text");

      const elementTransport = new MemoryTransport();
      const elementRoot = createRoot(elementTransport, { surfaceId: 15, epoch: 16 });
      expect(() =>
        elementRoot.render(
          <Text>
            <View />
          </Text>,
        ),
      ).toThrow("Text children must be raw text");
    } finally {
      console.error = originalError;
    }
  });

  it("always emits one synthetic root for fragments and unmount", () => {
    const transport = new MemoryTransport();
    const root = createRoot(transport, { surfaceId: 11, epoch: 12 });
    root.render(
      <>
        <Text>first</Text>
        <Text>second</Text>
      </>,
    );
    const initial = snapshots(transport)[0][6];
    const textIds = initial.filter((node) => node[3] === 2).map((node) => node[0]);
    expect(initial.slice(1).map((node) => [node[1], node[2]])).toEqual([
      [1, 0],
      [3, 0],
      [1, 1],
      [5, 0],
    ]);
    root.unmount();
    const patch = message(transport, 1);
    expect(patch.slice(0, 6)).toEqual([3, 3, 11, 12, 1, 2]);
    expect((patch[6] as readonly unknown[][]).map((operation) => operation[0])).toEqual([4, 4]);
    expect((patch[6] as readonly unknown[][]).map((operation) => operation[1])).toEqual(textIds);
  });

  it("ignores stale events and submits an explicit synthetic-root unmount snapshot", () => {
    const transport = new MemoryTransport();
    const root = createRoot(transport, { surfaceId: 9, epoch: 10 });
    let presses = 0;
    root.render(<Pressable onPress={() => presses++} />);
    const initial = snapshots(transport)[0];
    const pressable = initial[6].find((node) => node[3] === 3) as readonly unknown[];
    const stale = (surfaceId: number, listenerId: number) =>
      encodeFrame([PROTOCOL_VERSION, 2, surfaceId, 10, initial[5], 1, pressable[0] as number, listenerId, 1, null]);
    transport.push(stale(99, pressable[6] as number));
    transport.push(stale(9, 999));
    expect(presses).toBe(0);
    root.unmount();
    const patch = message(transport, 1);
    expect(patch[6]).toEqual([[4, pressable[0]]]);
    transport.push(stale(9, pressable[6] as number));
    expect(presses).toBe(0);
  });
  it("keeps 100k VirtualList data bounded to the committed range", async () => {
    const transport = new MemoryTransport();
    const root = createRoot(transport, { surfaceId: 41, epoch: 42 });
    const ref = React.createRef<VirtualListHandle>();
    const data = Array.from({ length: 100_000 }, (_, index) => index);
    root.render(
      <VirtualList
        ref={ref}
        data={data}
        itemKey={(item) => item}
        renderItem={(item) => <Text>{item}</Text>}
        estimatedItemSize={24}
        overscan={2}
        initialNumToRender={8}
      />,
    );
    const initial = snapshots(transport)[0][6];
    const list = initial.find((node) => node[3] === 6) as readonly unknown[];
    expect(list[7]).toEqual([2, 100_000, 0, 8, 24, 2]);
    expect(initial.length).toBeLessThan(40);
    const listener = list[6] as number;
    transport.push(encodeFrame([PROTOCOL_VERSION, 2, 41, 42, 1, 1, list[0] as number, listener, 7, [3, 40, 48]]));
    const update = message(transport, 1);
    const updatedList = (update[6] as readonly unknown[][]).find((operation) => (operation[2] as number) & 8);
    expect(
      (message(transport, 1)[6] as readonly unknown[][]).filter((operation) => operation[0] === 1).length,
    ).toBeLessThan(40);
    const scroll = ref.current?.scrollToIndex(99_999);
    expect(message(transport, 2).slice(0, 9)).toEqual([3, 4, 41, 42, 2, 1, list[0], 4, [99_999, 0]]);
    const resultFrame = encodeFrame([
      3,
      2,
      41,
      42,
      2,
      2,
      list[0] as number,
      0,
      6,
      [2, 1, 4, list[0] as number, true, null],
    ]);
    transport.push(resultFrame);
    await scroll;
  });
  it("deduplicates repeated VisibleRange commits and calls onEndReached once", () => {
    const transport = new MemoryTransport();
    const root = createRoot(transport, { surfaceId: 53, epoch: 54 });
    let endReached = 0;
    root.render(
      <VirtualList
        data={[0, 1, 2, 3]}
        itemKey={(item) => item}
        renderItem={(item) => <Text>{item}</Text>}
        estimatedItemSize={20}
        initialNumToRender={2}
        onEndReached={() => {
          endReached += 1;
        }}
      />,
    );
    const list = snapshots(transport)[0][6].find((node) => node[3] === 6) as readonly unknown[];
    const rangeEvent = (sequence: number, start: number, end: number) =>
      encodeFrame([PROTOCOL_VERSION, 2, 53, 54, 1, sequence, list[0] as number, list[6] as number, 7, [3, start, end]]);
    transport.push(rangeEvent(1, 0, 2));
    expect(transport.submitted).toHaveLength(1);
    transport.push(rangeEvent(2, 2, 4));
    expect(transport.submitted).toHaveLength(2);
    expect(endReached).toBe(1);
    transport.push(rangeEvent(3, 2, 4));
    expect(transport.submitted).toHaveLength(2);
    expect(endReached).toBe(1);
  });
  it("shrinks VirtualList ranges safely and rejects duplicate committed keys", () => {
    const transport = new MemoryTransport();
    const root = createRoot(transport, { surfaceId: 47, epoch: 48 });
    const renderList = (data: readonly number[]) => (
      <VirtualList
        data={data}
        itemKey={(item) => item}
        renderItem={(item) => <Text>{item}</Text>}
        estimatedItemSize={20}
        initialNumToRender={8}
      />
    );
    root.render(renderList(Array.from({ length: 20 }, (_, index) => index)));
    root.render(renderList([0, 1]));
    const operations = message(transport, 1)[6] as readonly unknown[][];
    const propertyUpdate = operations.find((operation) => (operation[2] as number) & 8);
    expect(propertyUpdate?.[6]).toEqual([2, 2, 0, 2, 20, 2]);

    const duplicateTransport = new MemoryTransport();
    const duplicateRoot = createRoot(duplicateTransport, { surfaceId: 49, epoch: 50 });
    const originalError = console.error;
    console.error = () => {};
    try {
      expect(() =>
        duplicateRoot.render(
          <VirtualList
            data={[1, 2]}
            itemKey={() => "same"}
            renderItem={(item) => <Text>{item}</Text>}
            estimatedItemSize={20}
            initialNumToRender={2}
          />,
        ),
      ).toThrow("VirtualList itemKey must be unique");
    } finally {
      console.error = originalError;
    }
  });

  it("emits accessibility-only updates without a style or listener change", () => {
    const transport = new MemoryTransport();
    const root = createRoot(transport, { surfaceId: 43, epoch: 44 });
    root.render(<View accessibilityLabel="before" />);
    root.render(<View accessibilityLabel="after" />);
    const operations = message(transport, 1)[6] as readonly unknown[][];
    expect(operations).toHaveLength(1);
    expect(operations[0][2]).toBe(16);
    expect(operations[0][7]).toEqual([0, "after", null, false, null, null, null]);
  });

  it("encodes width and height transition masks without per-frame transport", () => {
    const transport = new MemoryTransport();
    const root = createRoot(transport, { surfaceId: 45, epoch: 46 });
    const onComplete = () => undefined;
    root.render(
      <View
        style={{
          width: 100,
          height: 50,
          opacity: 0,
          transition: { durationMs: 120, easing: "easeOut", properties: ["opacity", "width", "height"], onComplete },
        }}
      />,
    );
    root.render(
      <View
        style={{
          width: 200,
          height: 100,
          opacity: 1,
          transition: { durationMs: 120, easing: "easeOut", properties: ["opacity", "width", "height"], onComplete },
        }}
      />,
    );
    expect(transport.submitted).toHaveLength(2);
    const style = (message(transport, 1)[6] as readonly unknown[][])[0][3] as readonly unknown[];
    expect(style[0]).toBe(200);
    expect(style[1]).toBe(100);
    expect(style[8]).toBe(1);
    expect(style[9]).toEqual([120, 0, 2, 13]);
  });
  it("routes one native animation completion per generation without JS tick commits", () => {
    const transport = new MemoryTransport();
    const completions: number[] = [];
    const onComplete = (generation: number) => completions.push(generation);
    const root = createRoot(transport, { surfaceId: 51, epoch: 52 });
    const style = (opacity: number) => ({
      opacity,
      transition: { durationMs: 120, easing: "easeInOut" as const, properties: ["opacity"] as const, onComplete },
    });
    root.render(<View style={style(0)} />);
    const initial = snapshots(transport)[0][6];
    const animated = initial.find((node) => node[0] !== 1 && node[3] === 1) as readonly unknown[];
    const listener = animated[6] as number;
    root.render(<View style={style(1)} />);
    expect(message(transport, 1)[6] as readonly unknown[][]).toHaveLength(1);
    expect((message(transport, 1)[6] as readonly unknown[][])[0][2]).toBe(1);
    expect(transport.submitted).toHaveLength(2);
    const completion = (sequence: number) =>
      encodeFrame([PROTOCOL_VERSION, 2, 51, 52, 2, sequence, animated[0] as number, listener, 8, [4, 1]]);
    transport.push(completion(1));
    transport.push(completion(2));
    expect(completions).toEqual([1]);
    expect(transport.submitted).toHaveLength(2);
  });
  it("dispatches focused View key down, repeat, and up notifications", () => {
    const transport = new MemoryTransport();
    const events: Array<{ key: string; modifiers: readonly string[]; action: "down" | "repeat" | "up" }> = [];
    const root = createRoot(transport, { surfaceId: 61, epoch: 62 });
    root.render(<View focusable onKeyDown={(event) => events.push(event)} />);
    const view = snapshots(transport)[0][6].find((node) => node[0] !== 1) as readonly unknown[];
    for (const [sequence, action] of [
      [1, 1],
      [2, 2],
      [3, 3],
    ] as const) {
      transport.push(
        encodeFrame([
          PROTOCOL_VERSION,
          2,
          61,
          62,
          1,
          sequence,
          view[0] as number,
          view[6] as number,
          EVENT_KEY,
          [5, "ArrowLeft", ["shift", "cmd"], action],
        ]),
      );
    }
    expect(events).toEqual([
      { key: "ArrowLeft", modifiers: ["shift", "cmd"], action: "down" },
      { key: "ArrowLeft", modifiers: ["shift", "cmd"], action: "repeat" },
      { key: "ArrowLeft", modifiers: ["shift", "cmd"], action: "up" },
    ]);
  });
  it("dispatches pointer buttons and hover edge notifications", () => {
    const transport = new MemoryTransport();
    const received: Array<unknown> = [];
    const root = createRoot(transport, { surfaceId: 65, epoch: 66 });
    root.render(
      <Pressable
        onPointerDown={(event) => received.push([event.type, event.button, event.modifiers, event.clickCount])}
        onPointerUp={(event) => received.push([event.type, event.button, event.modifiers, event.clickCount])}
        onHoverChange={(hovered) => received.push(["hover", hovered])}
      />,
    );
    const node = snapshots(transport)[0][6].find((entry) => entry[0] !== 1) as readonly unknown[];
    const listener = node[6] as number;
    const nodeId = node[0] as number;
    transport.push(
      encodeFrame([PROTOCOL_VERSION, 2, 65, 66, 1, 1, nodeId, listener, EVENT_POINTER, [6, 1, ["cmd"], 1, 2]]),
    );
    transport.push(encodeFrame([PROTOCOL_VERSION, 2, 65, 66, 1, 2, nodeId, listener, EVENT_POINTER, [6, 4, [], 2, 1]]));
    transport.push(encodeFrame([PROTOCOL_VERSION, 2, 65, 66, 1, 3, nodeId, listener, EVENT_HOVER, null]));
    transport.push(encodeFrame([PROTOCOL_VERSION, 2, 65, 66, 1, 4, nodeId, listener, EVENT_HOVER, null]));
    expect(received).toEqual([
      ["pointerdown", "left", ["cmd"], 2],
      ["pointerup", "back", [], 1],
      ["hover", true],
      ["hover", false],
    ]);
    root.unmount();
  });
  it("dispatches View scroll notifications for pixel and line deltas", () => {
    const transport = new MemoryTransport();
    const received: Array<unknown> = [];
    const root = createRoot(transport, { surfaceId: 69, epoch: 70 });
    root.render(
      <View
        onScroll={(event) => received.push([event.deltaKind, event.dx, event.dy, event.x, event.y, event.modifiers])}
      />,
    );
    const node = snapshots(transport)[0][6].find((entry) => entry[0] !== 1) as readonly unknown[];
    const nodeId = node[0] as number;
    const listener = node[6] as number;
    transport.push(
      encodeFrame([
        PROTOCOL_VERSION,
        2,
        69,
        70,
        1,
        1,
        nodeId,
        listener,
        EVENT_SCROLL,
        [7, SCROLL_DELTA_PIXELS, 12.5, -8, 40, 24, ["shift"]],
      ]),
    );
    transport.push(
      encodeFrame([
        PROTOCOL_VERSION,
        2,
        69,
        70,
        1,
        2,
        nodeId,
        listener,
        EVENT_SCROLL,
        [7, SCROLL_DELTA_LINES, 2, -1.5, 40, 24, ["cmd"]],
      ]),
    );
    expect(received).toEqual([
      ["pixels", 12.5, -8, 40, 24, ["shift"]],
      ["lines", 2, -1.5, 40, 24, ["cmd"]],
    ]);
    root.unmount();
  });

  it("exposes focus and blur commands on focusable View refs", () => {
    const transport = new MemoryTransport();
    const ref = createRef<ViewHandle>();
    const root = createRoot(transport, { surfaceId: 67, epoch: 68 });
    root.render(<View ref={ref} focusable />);
    const focusPromise = ref.current?.focus();
    focusPromise?.catch(() => undefined);
    expect((message(transport, 1) as readonly unknown[])[7]).toBe(COMMAND_FOCUS);
    const blurPromise = ref.current?.blur();
    blurPromise?.catch(() => undefined);
    expect((message(transport, 2) as readonly unknown[])[7]).toBe(COMMAND_BLUR);
    const titlePromise = root.setTitle("React GPUI");
    titlePromise.catch(() => undefined);
    expect((message(transport, 3) as readonly unknown[])[7]).toBe(6);
    expect((message(transport, 3) as readonly unknown[])[8]).toBe("React GPUI");
    root.unmount();
  });
  it("frames root-level resize, zoom, fullscreen, and URL commands", async () => {
    const transport = new MemoryTransport();
    const root = createRoot(transport, { surfaceId: 75, epoch: 76 });
    root.render(<View />);
    const complete = (sequence: number, requestId: number, command: number) =>
      encodeFrame([3, 2, 75, 76, 1, sequence, 1, 0, 6, [2, requestId, command, 1, true, null]]);

    const resize = root.resize(800, 600);
    expect(message(transport, 1)).toEqual([3, 4, 75, 76, 1, 1, 1, COMMAND_RESIZE_WINDOW, [800, 600]]);
    transport.push(complete(1, 1, COMMAND_RESIZE_WINDOW));
    await resize;

    const zoom = root.zoom();
    expect((message(transport, 2) as readonly unknown[])[7]).toBe(COMMAND_ZOOM_WINDOW);
    transport.push(complete(2, 2, COMMAND_ZOOM_WINDOW));
    await zoom;

    const fullscreen = root.toggleFullscreen();
    expect((message(transport, 3) as readonly unknown[])[7]).toBe(COMMAND_TOGGLE_FULLSCREEN);
    transport.push(complete(3, 3, COMMAND_TOGGLE_FULLSCREEN));
    await fullscreen;

    const openUrl = root.openUrl("https://example.com/docs");
    expect((message(transport, 4) as readonly unknown[])[7]).toBe(COMMAND_OPEN_URL);
    expect((message(transport, 4) as readonly unknown[])[8]).toBe("https://example.com/docs");
    transport.push(complete(4, 4, COMMAND_OPEN_URL));
    await openUrl;

    await expect(root.resize(0, 600)).rejects.toThrow();
    await expect(root.openUrl("file:///tmp/example")).rejects.toThrow();
    root.unmount();
  });
  it("frames file dialog commands and maps async results and cancellation", async () => {
    const transport = new MemoryTransport();
    const root = createRoot(transport, { surfaceId: 79, epoch: 80 });
    root.render(<View />);
    const complete = (
      sequence: number,
      requestId: number,
      command: number,
      value: readonly unknown[] | undefined = undefined,
    ) =>
      encodeFrame([
        3,
        2,
        79,
        80,
        1,
        sequence,
        1,
        0,
        6,
        value === undefined ? [2, requestId, command, 1, true, null] : [2, requestId, command, 1, true, null, value],
      ] as never);

    const files = root.pickFiles({ title: "Open files", multiple: true });
    expect(message(transport, 1)).toEqual([3, 4, 79, 80, 1, 1, 1, COMMAND_FILE_DIALOG_OPEN, ["Open files", [0, 1]]]);
    transport.push(complete(1, 1, COMMAND_FILE_DIALOG_OPEN, [5, ["/tmp/a.txt", "/tmp/b.txt"]]));
    await expect(files).resolves.toEqual(["/tmp/a.txt", "/tmp/b.txt"]);

    const canceled = root.pickFiles({ directories: true });
    expect(message(transport, 2)).toEqual([3, 4, 79, 80, 1, 2, 1, COMMAND_FILE_DIALOG_OPEN, ["", [1, 0]]]);
    transport.push(complete(2, 2, COMMAND_FILE_DIALOG_OPEN));
    await expect(canceled).resolves.toBeNull();

    const save = root.pickSavePath({ defaultName: "report.json" });
    expect(message(transport, 3)).toEqual([3, 4, 79, 80, 1, 3, 1, COMMAND_FILE_DIALOG_SAVE, "report.json"]);
    transport.push(complete(3, 3, COMMAND_FILE_DIALOG_SAVE, [4, "/tmp/report.json"]));
    await expect(save).resolves.toBe("/tmp/report.json");
    root.unmount();
  });
  it("frames root focus traversal commands", async () => {
    const transport = new MemoryTransport();
    const root = createRoot(transport, { surfaceId: 77, epoch: 78 });
    root.render(<View />);
    const complete = (sequence: number, requestId: number, command: number) =>
      encodeFrame([3, 2, 77, 78, 1, sequence, 1, 0, 6, [2, requestId, command, 1, true, null]]);

    const next = root.focusNext();
    expect((message(transport, 1) as readonly unknown[])[7]).toBe(COMMAND_FOCUS_NEXT);
    transport.push(complete(1, 1, COMMAND_FOCUS_NEXT));
    await next;

    const previous = root.focusPrev();
    expect((message(transport, 2) as readonly unknown[])[7]).toBe(COMMAND_FOCUS_PREV);
    transport.push(complete(2, 2, COMMAND_FOCUS_PREV));
    await previous;
    root.unmount();
  });

  it("rejects a View key listener without focusable opt-in", () => {
    const originalError = console.error;
    console.error = () => {};
    try {
      const transport = new MemoryTransport();
      const root = createRoot(transport, { surfaceId: 63, epoch: 64 });
      expect(() => root.render(<View onKeyDown={() => undefined} />)).toThrow("requires focusable=true");
    } finally {
      console.error = originalError;
    }
  });
});
