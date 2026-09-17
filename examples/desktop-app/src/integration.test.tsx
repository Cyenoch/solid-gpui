import { expect, test } from "bun:test";
import { Column, Image, MemoryTransport, Text, createRoot } from "@solid-gpui/core";
import { Button } from "@solid-gpui/core/components";
import { createEffect } from "@solid-gpui/core/runtime";
import { createSignal } from "solid-js";
import cover from "../assets/cover.png?inline";

test("the application configuration shares reactivity across JSX, native controls and assets", async () => {
  const transport = new MemoryTransport();
  const root = createRoot(transport);
  const observations: number[] = [];
  let increment!: () => void;
  function Counter() {
    const [count, setCount] = createSignal(0);
    increment = () => setCount((value) => value + 1);
    createEffect(() => observations.push(count()));
    return (
      <Column>
        <Text>{count()}</Text>
        <Image source={cover} />
        <Button label="Increment" onPress={() => increment()} />
      </Column>
    );
  }
  try {
    root.render(() => <Counter />);
    await Promise.resolve();
    expect(observations).toEqual([0]);
    expect(transport.submitted.length).toBe(1);
    increment();
    await Promise.resolve();
    expect(observations).toEqual([0, 1]);
    expect(transport.submitted.length).toBe(2);
    expect(cover.startsWith("data:image/png")).toBe(true);
  } finally {
    root.unmount();
  }
});
