import type { Absent } from "./type-only-module";
import { basename } from "node:path";
import { createRoot, Pressable, Text } from "@solid-gpui/core";
import { createSignal, Show } from "@solid-gpui/core/runtime";
import { TestHost } from "@solid-gpui/core/testing";

class Fields {
  declare erased: Absent;
  retained?: string;
}
if (Object.keys(new Fields()).join(",") !== "retained") throw new Error("TypeScript class fields changed");

const host = new TestHost();
const root = createRoot(host.transport, { surfaceId: 2 });
try {
  root.render(() => {
    const [count, setCount] = createSignal(0);
    return (
      <Pressable onPress={() => setCount(count() + 1)}>
        <Show when={count() > 0} fallback={<Text>Ready</Text>}>
          <Text>{`Count: ${count()}`}</Text>
        </Show>
      </Pressable>
    );
  });
  const button = host.surface(2)!.nodes.find((node) => node.kind === "Pressable")!;
  if (!host.surface(2)!.nodes.some((node) => node.text === "Ready")) throw new Error("Initial JSX did not mount");
  host.dispatch(button, { type: "press" });
  if (!host.surface(2)!.nodes.some((node) => node.text === "Count: 1")) throw new Error("Reactive JSX did not update");
  const current = host.surface(2)!.nodes.find((node) => node.kind === "Pressable")!;
  host.dispatch(current, { type: "press" });
  if (!host.surface(2)!.nodes.some((node) => node.text === "Count: 2"))
    throw new Error("Retained listener did not update");
  if (host.commits.map((commit) => commit.type).join(",") !== "snapshot,patch,patch")
    throw new Error("Compiler lost incremental publication");
} finally {
  root.unmount();
}
