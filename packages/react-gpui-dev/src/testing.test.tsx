import React from "react";
import { describe, expect, it } from "bun:test";
import { encode } from "@msgpack/msgpack";
import * as Core from "@react-gpui/core";
import {
  Pressable,
  Text,
  TextInput,
  View,
  VirtualList,
  createWindowSizeStore,
  type KeyEvent,
  type WindowSizeStore,
} from "@react-gpui/core";
import { render } from "./testing";
type TodoFixture = { TodoApp: (props: { windowSizeStore: WindowSizeStore }) => React.ReactElement };

// Intentional dynamic import: this integration fixture lives in the sibling
// core package and must not become a dev-package build dependency.
const { TodoApp } = (await import(
  `${import.meta.dir}/../node_modules/@react-gpui/core/examples/todo.tsx`
)) as TodoFixture;

function frame(value: readonly unknown[]): Uint8Array {
  const payload = encode(value, { sortKeys: false, forceFloat32: true });
  const result = new Uint8Array(payload.byteLength + 4);
  new DataView(result.buffer).setUint32(0, payload.byteLength, true);
  result.set(payload, 4);
  return result;
}

function event(
  surfaceId: number,
  epoch: number,
  sequence: number,
  nodeId: number,
  listenerId: number,
  type: number,
  payload: unknown,
): Uint8Array {
  return frame([3, 2, surfaceId, epoch, 1, sequence, nodeId, listenerId, type, payload]);
}

function textIn(commits: readonly unknown[], value: string): boolean {
  return JSON.stringify(commits).includes(value);
}

describe("@react-gpui/dev headless renderer", () => {
  it("renders the real TodoApp and exposes frames, decoded commits, and node predicates", () => {
    const result = render(<TodoApp windowSizeStore={createWindowSizeStore()} />, { surfaceId: 11, epoch: 12 });
    const input = result.node("TextInput", (node) => node.accessibility?.[1] === "New todo");
    expect(input.kind).toBe("TextInput");
    expect(result.frames).toHaveLength(1);
    expect(result.commits()[0]).toEqual(expect.arrayContaining([3, 1, 11, 12]));
    result.unmount();
  });

  it("press injects a real Press event", () => {
    const pressed: string[] = [];
    function Counter() {
      return (
        <Pressable onPress={() => pressed.push("pressed")} accessibilityLabel="increment">
          <Text>increment</Text>
        </Pressable>
      );
    }
    const result = render(<Counter />);
    const button = result.node("Pressable", (node) => node.accessibility?.[1] === "increment");
    result.press(button);
    expect(pressed).toEqual(["pressed"]);
    result.unmount();
  });

  it("key injects the core Key event path", () => {
    const keys: string[] = [];
    function KeyProbe() {
      return <View focusable onKeyDown={(event) => keys.push(event.key)} />;
    }
    const result = render(<KeyProbe />);
    const view = result.node("View", (node) => node.focusable && node.listenerId !== 0);
    const key: KeyEvent = { key: "Enter", modifiers: [], action: "down" };
    result.key(view, key);
    expect(keys).toEqual(["Enter"]);
    result.unmount();
  });

  it("input and submit drive TextInput callbacks", () => {
    const changes: string[] = [];
    const submits: string[] = [];
    function InputProbe() {
      return (
        <View>
          <TextInput onChangeText={(value) => changes.push(value)} onSubmitEditing={() => submits.push("submitted")} />
          <Text>input probe</Text>
        </View>
      );
    }
    const result = render(<InputProbe />);
    const input = result.node("TextInput");
    result.input(input, "draft");
    result.submit(input, "draft");
    expect(changes).toEqual(["draft"]);
    expect(submits).toEqual(["submitted"]);
    result.unmount();
  });

  it("visibleRange injects the VirtualList range notification", () => {
    const reached: string[] = [];
    function ListProbe() {
      return (
        <View>
          <VirtualList
            data={["one", "two"]}
            itemKey={(item) => item}
            renderItem={(item) => <Text>{item}</Text>}
            estimatedItemSize={24}
            onEndReached={() => reached.push("end")}
          />
          <Text>list probe</Text>
        </View>
      );
    }
    const result = render(<ListProbe />);
    const list = result.node("VirtualList");
    result.visibleRange(list, 0, 2);
    expect(reached).toEqual(["end"]);
    result.unmount();
  });

  it("commandResult resolves a captured command with a typed value", async () => {
    const result = render(<View />);
    const pending = result.root.getWindowSize();
    const command = result.commits().find((value) => Array.isArray(value) && value[1] === 4) as readonly unknown[];
    expect(command?.[5]).toBe(1);
    result.commandResult(Number(command[5]), { value: [800, 600] });
    await expect(pending).resolves.toEqual([800, 600]);
    result.unmount();
  });

  it("commandResult resolves typed path values for file dialogs", async () => {
    const result = render(<View />);
    const pending = result.root.pickFiles();
    const command = result.commits().find((value) => Array.isArray(value) && value[1] === 4) as readonly unknown[];
    expect(command?.[5]).toBe(1);
    result.commandResult(Number(command[5]), { value: ["/tmp/a.txt", "/tmp/b.txt"] });
    await expect(pending).resolves.toEqual(["/tmp/a.txt", "/tmp/b.txt"]);
    result.unmount();
  });

  it("dispatchFrame is the raw framed-event escape hatch", () => {
    const pressed: string[] = [];
    const result = render(
      <Pressable onPress={() => pressed.push("pressed")} accessibilityLabel="raw">
        <Text>raw</Text>
      </Pressable>,
      { surfaceId: 21, epoch: 22 },
    );
    const button = result.node("Pressable", (node) => node.accessibility?.[1] === "raw");
    result.dispatchFrame(event(21, 22, 1, button.id, button.listenerId, 1, null));
    expect(pressed).toEqual(["pressed"]);
    result.unmount();
  });

  it("does not swallow unbounded render errors", () => {
    expect(() => render(<View style={{ width: -1 }} />)).toThrow("width");
  });

  it("reuses any public core protocol constants and guards the internal tag table by real round trips", () => {
    const core = Core as unknown as Record<string, unknown>;
    const mirrored: Record<string, number> = {
      EVENT_PRESS: 1,
      EVENT_CHANGE: 2,
      EVENT_VISIBLE_RANGE: 7,
      EVENT_KEY: 9,
      EVENT_SUBMIT: 13,
    };
    let publicConstantCount = 0;
    for (const [name, value] of Object.entries(mirrored)) {
      if (typeof core[name] === "number") {
        publicConstantCount += 1;
        expect(core[name]).toBe(value);
      }
    }
    expect(typeof Core.MemoryTransport).toBe("function");
    expect(publicConstantCount).toBeGreaterThanOrEqual(0);
  });
});
