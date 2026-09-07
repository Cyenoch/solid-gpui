import { mountApplication, Pressable, Text } from "@solid-gpui/core";
import { EmbeddedTransport } from "@solid-gpui/core/embedded";
import { createSignal } from "@solid-gpui/core/runtime";

mountApplication({
  transport: () => new EmbeddedTransport(),
  setup() {
    const [count, setCount] = createSignal(0);
    return {
      render: () => (
        <Pressable onPress={() => setCount((value) => value + 1)}>
          <Text>{`Count: ${count()} — 🌍`}</Text>
        </Pressable>
      ),
    };
  },
});
