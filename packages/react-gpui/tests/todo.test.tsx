import React from "react";
import { describe, expect, it } from "bun:test";
import { decode } from "@msgpack/msgpack";
import {
  COMMAND_FOCUS_NEXT,
  EVENT_CHANGE,
  EVENT_KEY,
  EVENT_PRESS,
  EVENT_SUBMIT,
  EVENT_WINDOW_RESIZE,
  PROTOCOL_VERSION,
  encodeFrame,
} from "../src/protocol";
import { MemoryTransport, Text, VirtualList, createRoot, createWindowSizeStore } from "../src/index";
import { TodoApp } from "../examples/todo";

type WireNode = readonly unknown[];
type WireMessage = readonly unknown[];

function messages(transport: MemoryTransport): WireMessage[] {
  return transport.submitted.map((frame) => decode(frame.slice(4)) as WireMessage);
}

function initialNodes(transport: MemoryTransport): Map<number, WireNode> {
  const snapshot = messages(transport).find((message) => message[1] === 1);
  if (snapshot === undefined) throw new Error("missing initial snapshot");
  const nodes = new Map<number, WireNode>();
  for (const node of snapshot[6] as readonly WireNode[]) nodes.set(Number(node[0]), node);
  return nodes;
}

function nodeWithText(nodes: Map<number, WireNode>, value: string): WireNode {
  const node = [...nodes.values()].find((candidate) => Number(candidate[3]) === 4 && candidate[5] === value);
  if (node === undefined) throw new Error(`missing text node ${value}`);
  return node;
}
function pressableForText(nodes: Map<number, WireNode>, value: string): WireNode {
  let current = nodes.get(Number(nodeWithText(nodes, value)[1]));
  while (current !== undefined && Number(current[3]) !== 3) {
    current = nodes.get(Number(current[1]));
  }
  if (current === undefined) throw new Error(`text ${value} is not inside Pressable`);
  return current;
}

function textInput(nodes: Map<number, WireNode>): WireNode {
  const input = [...nodes.values()].find((node) => Number(node[3]) === 5);
  if (input === undefined) throw new Error("missing TextInput node");
  return input;
}

function eventFrame(
  surfaceId: number,
  epoch: number,
  sequence: number,
  node: WireNode,
  eventType: number,
  payload: unknown,
): Uint8Array {
  return encodeFrame([
    PROTOCOL_VERSION,
    2,
    surfaceId,
    epoch,
    1,
    sequence,
    Number(node[0]),
    Number(node[6]),
    eventType,
    payload,
  ] as never);
}

function commandResult(surfaceId: number, epoch: number, requestId: number, command: number): Uint8Array {
  return encodeFrame([
    PROTOCOL_VERSION,
    2,
    surfaceId,
    epoch,
    1,
    requestId + 1,
    1,
    0,
    6,
    [2, requestId, command, 1, true, null],
  ]);
}
function rowAncestor(nodes: Map<number, WireNode>, node: WireNode): number {
  let current = Number(node[1]);
  while (current !== 0) {
    const candidate = nodes.get(current);
    if (candidate === undefined) break;
    const parent = nodes.get(Number(candidate[1]));
    if (Number(candidate[3]) === 1 && parent !== undefined && Number(parent[3]) === 6) return current;
    current = Number(candidate[1]);
  }
  return 0;
}

function pressableForRowAction(nodes: Map<number, WireNode>, rowText: string, action: string): WireNode {
  const row = rowAncestor(nodes, nodeWithText(nodes, rowText));
  const text = [...nodes.values()].find(
    (candidate) => Number(candidate[3]) === 4 && candidate[5] === action && rowAncestor(nodes, candidate) === row,
  );
  if (text === undefined) throw new Error(`missing ${action} action for ${rowText}`);
  let parent = nodes.get(Number(text[1]));
  while (parent !== undefined && Number(parent[3]) !== 3) parent = nodes.get(Number(parent[1]));
  if (parent === undefined) throw new Error(`${action} is not inside Pressable`);
  return parent;
}
function latestPatch(transport: MemoryTransport): WireMessage {
  const patch = [...messages(transport)].reverse().find((message) => message[1] === 3);
  if (patch === undefined) throw new Error("missing patch");
  return patch;
}

function latestCreatedNode(transport: MemoryTransport, kind: number): WireNode {
  for (const message of [...messages(transport)].reverse()) {
    if (message[1] === 1) {
      const node = [...(message[6] as readonly WireNode[])]
        .reverse()
        .find((candidate) => Number(candidate[3]) === kind);
      if (node !== undefined) return node;
    }
    if (message[1] !== 3) continue;
    const node = [...(message[6] as readonly WireNode[])]
      .reverse()
      .find((operation) => Number(operation[0]) === 1 && Number(operation[4]) === kind);
    if (node !== undefined) return node.slice(1) as WireNode;
  }

  throw new Error(`missing created node kind ${kind}`);
}
function EmptyStateList({ items }: { readonly items: readonly string[] }) {
  return (
    <VirtualList
      data={items}
      itemKey={(item) => item}
      renderItem={(item) => <Text>{item}</Text>}
      estimatedItemSize={24}
      style={{ height: 200 }}
      emptyState={<Text>No todos yet</Text>}
    />
  );
}

describe("todo example", () => {
  it("switches between emptyState and a native VirtualList cleanly", () => {
    const transport = new MemoryTransport();
    const root = createRoot(transport, { surfaceId: 300, epoch: 301 });
    root.render(<EmptyStateList items={[]} />);
    const emptyNodes = initialNodes(transport);
    expect([...emptyNodes.values()].some((node) => Number(node[3]) === 6)).toBe(false);
    expect([...emptyNodes.values()].some((node) => node[5] === "No todos yet")).toBe(true);

    root.render(<EmptyStateList items={["First todo"]} />);
    const populatedPatch = latestPatch(transport);
    const virtualListCreate = (populatedPatch[6] as readonly WireNode[]).find(
      (operation) => Number(operation[0]) === 1 && Number(operation[4]) === 6,
    );
    expect(virtualListCreate).toBeDefined();

    root.render(<EmptyStateList items={[]} />);
    const emptyPatch = latestPatch(transport);
    expect(
      (emptyPatch[6] as readonly WireNode[]).some(
        (operation) => Number(operation[0]) === 4 && Number(operation[1]) === Number(virtualListCreate?.[1]),
      ),
    ).toBe(true);
    expect(JSON.stringify(emptyPatch)).toContain("No todos yet");
    root.unmount();
  });

  it("adds, toggles, edits, and deletes todos through public events", () => {
    const transport = new MemoryTransport();
    const store = createWindowSizeStore();
    const root = createRoot(transport, { surfaceId: 301, epoch: 302 });
    root.render(<TodoApp windowSizeStore={store} />);

    const nodes = initialNodes(transport);
    const input = textInput(nodes);
    transport.push(eventFrame(301, 302, 1, input, EVENT_CHANGE, [1, "Schedule demo", 13, 13, null, null, 1]));
    transport.push(eventFrame(301, 302, 2, input, EVENT_SUBMIT, "Schedule demo"));
    const firstTitleId = Number(nodes.get(Number(nodeWithText(nodes, "Review the GPUI renderer API")[1]))?.[0]);

    const firstTodo = pressableForText(nodes, "Review the GPUI renderer API");
    transport.push(eventFrame(301, 302, 3, firstTodo, EVENT_PRESS, null));
    const togglePatch = latestPatch(transport);
    const toggle = (togglePatch[6] as readonly WireNode[]).find(
      (operation) => Number(operation[0]) === 2 && Number(operation[1]) === firstTitleId,
    );
    expect((toggle?.[3] as readonly unknown[])[25]).toBe(2);

    const editButton = pressableForRowAction(nodes, "Try the todo app with a narrow window", "Edit");
    transport.push(eventFrame(301, 302, 4, editButton, EVENT_PRESS, null));
    const editInput = latestCreatedNode(transport, 5);
    transport.push(
      eventFrame(301, 302, 5, editInput, EVENT_CHANGE, [1, "Try the app on a phone", 22, 22, null, null, 2]),
    );
    transport.push(eventFrame(301, 302, 6, editInput, EVENT_SUBMIT, "Try the app on a phone"));
    expect(JSON.stringify(latestPatch(transport))).toContain("Try the app on a phone");

    const deleteButton = pressableForRowAction(nodes, "Try the todo app with a narrow window", "Delete");
    transport.push(eventFrame(301, 302, 7, deleteButton, EVENT_PRESS, null));
    const deletePatch = latestPatch(transport);
    expect(JSON.stringify(deletePatch)).not.toContain("Try the app on a phone");
    root.unmount();
  });

  it("updates compact layout from the root onWindowResize callback", () => {
    const transport = new MemoryTransport();
    const store = createWindowSizeStore({ width: 1000, height: 700 });
    const root = createRoot(transport, {
      surfaceId: 303,
      epoch: 304,
      onWindowResize: (width, height) => store.set(width, height),
    });
    root.render(<TodoApp windowSizeStore={store} />);
    expect(JSON.stringify(messages(transport))).not.toContain("Compact layout for a narrow window");

    transport.push(
      eventFrame(303, 304, 1, [1, 0, 0, 1, null, null, 0, null, null, false], EVENT_WINDOW_RESIZE, [600, 500]),
    );
    expect(store.getSnapshot()).toEqual({ width: 600, height: 500 });
    expect(JSON.stringify(latestPatch(transport))).toContain("Compact layout for a narrow window");
    root.unmount();
  });

  it("uses Tab key events to request a focusNext cycle", async () => {
    const transport = new MemoryTransport();
    const store = createWindowSizeStore();
    let focusPromise: Promise<void> | undefined;
    const root = createRoot(transport, { surfaceId: 305, epoch: 306 });
    root.render(
      <TodoApp
        windowSizeStore={store}
        onFocusNext={() => {
          focusPromise = root.focusNext();
        }}
      />,
    );
    const nodes = initialNodes(transport);
    const focusable = [...nodes.values()].find(
      (node) => node[9] === true && (node[8] as readonly unknown[] | null)?.[0] === 1,
    );
    if (focusable === undefined) throw new Error("missing focusable Tab cycle node");
    transport.push(eventFrame(305, 306, 1, focusable, EVENT_KEY, [5, "Tab", [], 1]));
    const command = messages(transport).at(-1);
    expect(command?.[7]).toBe(COMMAND_FOCUS_NEXT);
    const requestId = Number(command?.[5]);
    transport.push(commandResult(305, 306, requestId, COMMAND_FOCUS_NEXT));
    await focusPromise;
    root.unmount();
  });
});
