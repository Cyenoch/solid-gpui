import React, { useCallback, useState } from "react";
import { describe, expect, it } from "bun:test";
import {
  MemoryTransport,
  Pressable,
  StyleSheet,
  Text,
  TextInput,
  TextInputHandle,
  View,
  VirtualList,
  VirtualListHandle,
  createRoot,
} from "../src/index";
import { decode } from "@msgpack/msgpack";
import { decodeEvent, encodeFrame, framePayload, FrameDecoder, PROTOCOL_VERSION } from "../src/protocol";

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
    expect(decoder.push(new Uint8Array([...first.slice(2), ...second]))).toEqual([
      first.slice(4),
      second.slice(4),
    ]);
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
    const malformed = encodeFrame([
      PROTOCOL_VERSION,
      2,
      1,
      1,
      1,
      1,
      2,
      7,
      6,
      [2, 1, 99, 2, true, null],
    ] as never);
    expect(decodeEvent(malformed.slice(4))).toBeNull();
  });
});

describe("styles", () => {
  it("validates, freezes, and encodes RGBA styles", () => {
    const styles = StyleSheet.create({
      root: { width: 10, padding: 2, gap: 4, backgroundColor: "#12345678", color: "#abcdef" },
    });
    expect(Object.isFrozen(styles)).toBe(true);
    expect(Object.isFrozen(styles.root)).toBe(true);
    const transport = new MemoryTransport();
    const root = createRoot(transport, { surfaceId: 3, epoch: 4 });
    root.render(<View style={styles.root} />);
    expect(snapshots(transport)[0][6][1][4]).toEqual([10, null, 0, null, 2, 4, 0x12345678, 0xabcdefff, null, null]);
    root.render(<View style={{ opacity: 0, transition: { durationMs: 100 } }} />);
    expect(((message(transport, 1)[6] as readonly unknown[][])[0][3] as readonly unknown[])[9]).toEqual([100, 0, 3, 3]);
    expect(() => StyleSheet.create({ bad: { width: -1 } })).toThrow();
    expect(() => StyleSheet.create({ bad: { gap: Number.NaN } })).toThrow();
    expect(() => StyleSheet.create({ bad: { backgroundColor: "red" } })).toThrow();
    expect(() => StyleSheet.create({ bad: { flexGrow: -1 } })).toThrow();
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
    const event = encodeFrame([PROTOCOL_VERSION, 2, 7, 8, initial[5], 1, pressable[0] as number, pressable[6] as number, 1, null]);
    transport.push(event.slice(0, 3));
    transport.push(event.slice(3));
    transport.push(event);
    const update = (message(transport, 1)[6] as readonly unknown[][]).find((operation) => (operation[2] as number) & 2);
    expect(update?.[4]).toBe("1");
    const futureRevision = encodeFrame([PROTOCOL_VERSION, 2, 7, 8, initial[5] + 2, 2, pressable[0] as number, pressable[6] as number, 1, null]);
    transport.push(futureRevision);
    expect(transport.submitted).toHaveLength(2);
  });
  it("encodes keyed reorder, insertion, and subtree deletion as incremental operations", () => {
    const transport = new MemoryTransport();
    const root = createRoot(transport, { surfaceId: 17, epoch: 18 });
    const renderList = (items: string[]) => (
      <View>
        {items.map((item) => <Text key={item}>{item}</Text>)}
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
        <View key="new"><Text>new</Text></View>
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
    const renderItems = (items: string[]) => <>{items.map((item) => <Text key={item}>{item}</Text>)}</>;
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
          <View key={parent}>{children[parent].map((child) => <View key={child}><Text>{child}</Text></View>)}</View>
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
      <View>{Array.from({ length: 1000 }, (_, index) => <Text key={index}>{index === changed ? `changed-${index}` : index}</Text>)}</View>
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
        <TextInput onChangeText={(value) => changes.push(value)} onSelectionChange={(selection) => selections.push(selection.start)} />
        <TextInput onChangeText={(value) => changes.push(`second:${value}`)} />
      </View>,
    );
    const inputs = snapshots(transport)[0][6].filter((node) => node[3] === 5);
    const first = inputs[0];
    const second = inputs[1];
    const event = (node: readonly unknown[], text: string, sequence: number) =>
      encodeFrame([PROTOCOL_VERSION, 2, 25, 26, 1, sequence, node[0] as number, node[6] as number, 2, [1, text, 2, 2, null, null, sequence]]);
    transport.push(event(second, "second", 1));
    transport.push(event(first, "first", 2));
    transport.push(encodeFrame([PROTOCOL_VERSION, 2, 25, 26, 1, 3, first[0] as number, first[6] as number, 3, [1, "first", 2, 2, null, null, 2]]));
    expect(changes).toEqual(["second:second", "first"]);
    expect(selections).toEqual([2]);
  });
  it("preserves uncontrolled defaults and acknowledges controlled native edits", () => {
    const uncontrolledTransport = new MemoryTransport();
    const uncontrolledRoot = createRoot(uncontrolledTransport, { surfaceId: 27, epoch: 28 });
    uncontrolledRoot.render(<TextInput defaultValue="initial" />);
    uncontrolledRoot.render(<TextInput defaultValue="replacement" />);
    expect((message(uncontrolledTransport, 1)[6] as readonly unknown[][])).toHaveLength(0);

    const controlledTransport = new MemoryTransport();
    const controlledRoot = createRoot(controlledTransport, { surfaceId: 29, epoch: 30 });
    function Controlled() {
      const [value, setValue] = useState("");
      return <TextInput value={value} onChangeText={setValue} />;
    }
    controlledRoot.render(<Controlled />);
    const input = snapshots(controlledTransport)[0][6].find((node) => node[3] === 5) as readonly unknown[];
    controlledTransport.push(encodeFrame([PROTOCOL_VERSION, 2, 29, 30, 1, 1, input[0] as number, input[6] as number, 2, [1, "native", 6, 6, null, null, 1]]));
    const patch = message(controlledTransport, 1);
    const propertyUpdate = (patch[6] as readonly unknown[][]).find((operation) => (operation[2] as number) & 8);
    expect(propertyUpdate?.[6]).toEqual([1, "native", null, false, false, true, 1, 6, 6, null, null]);
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
      rawRoot.render(<View>orphan raw text</View>);
      expect(rawTransport.submitted).toHaveLength(0);

      const elementTransport = new MemoryTransport();
      const elementRoot = createRoot(elementTransport, { surfaceId: 15, epoch: 16 });
      elementRoot.render(<Text><View /></Text>);
      expect(elementTransport.submitted).toHaveLength(0);
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
    expect((message(transport, 1)[6] as readonly unknown[][]).filter((operation) => operation[0] === 1).length).toBeLessThan(40);
    const scroll = ref.current?.scrollToIndex(99_999);
    expect(message(transport, 2).slice(0, 9)).toEqual([3, 4, 41, 42, 2, 1, list[0], 4, [99_999, 0]]);
    const resultFrame = encodeFrame([3, 2, 41, 42, 2, 2, list[0] as number, 0, 6, [2, 1, 4, list[0] as number, true, null]]);
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
      duplicateRoot.render(
        <VirtualList
          data={[1, 2]}
          itemKey={() => "same"}
          renderItem={(item) => <Text>{item}</Text>}
          estimatedItemSize={20}
          initialNumToRender={2}
        />,
      );
    } finally {
      console.error = originalError;
    }
    expect(duplicateTransport.submitted).toHaveLength(1);
    expect(snapshots(duplicateTransport)[0][6].length).toBeLessThanOrEqual(2);
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

  it("encodes animation transitions in one commit without per-frame transport", () => {
    const transport = new MemoryTransport();
    const root = createRoot(transport, { surfaceId: 45, epoch: 46 });
    const onComplete = () => undefined;
    root.render(<View style={{ opacity: 0, transition: { durationMs: 120, easing: "easeOut", properties: ["opacity"], onComplete } }} />);
    root.render(<View style={{ opacity: 1, transition: { durationMs: 120, easing: "easeOut", properties: ["opacity"], onComplete } }} />);
    expect(transport.submitted).toHaveLength(2);
    const style = (message(transport, 1)[6] as readonly unknown[][])[0][3] as readonly unknown[];
    expect(style[8]).toBe(1);
    expect(style[9]).toEqual([120, 0, 2, 1]);
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
    expect((message(transport, 1)[6] as readonly unknown[][])).toHaveLength(1);
    expect((message(transport, 1)[6] as readonly unknown[][])[0][2]).toBe(1);
    expect(transport.submitted).toHaveLength(2);
    const completion = (sequence: number) => encodeFrame([
      PROTOCOL_VERSION,
      2,
      51,
      52,
      2,
      sequence,
      animated[0] as number,
      listener,
      8,
      [4, 1],
    ]);
    transport.push(completion(1));
    transport.push(completion(2));
    expect(completions).toEqual([1]);
    expect(transport.submitted).toHaveLength(2);
  });
});
