import { mountApplication, Pressable, Text } from "@solid-gpui/core";
import { EmbeddedTransport } from "@solid-gpui/core/embedded";
import { createSignal } from "@solid-gpui/core/runtime";

mountApplication<number>({
  transport: () => new EmbeddedTransport(),
  setup(previous = 0) {
    const [count, setCount] = createSignal(previous);
    return {
      captureState: () => count(),
      render: () => (
        <Pressable onPress={() => setCount((value) => value + 1)}>
          <Text>{`Count: ${count()} — 🌍`}</Text>
        </Pressable>
      ),
    };
  },
});
