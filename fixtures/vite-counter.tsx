import { mountApplication, Pressable, Text, View } from "@solid-gpui/core";
import { createSignal } from "@solid-gpui/core/runtime";
import { StdioTransport } from "@solid-gpui/core/stdio";

mountApplication<number>({
  transport: () => new StdioTransport(),
  hotKey: import.meta.hot ? import.meta.url : undefined,
  setup(previous = 0) {
    const [count, setCount] = createSignal(previous);
    return {
      render: () => (
        <View style={{ padding: 24, gap: 12 }}>
          <Text>Count: {count()}</Text>
          <Pressable accessibilityRole="button" onPress={() => setCount((value) => value + 1)}>
            <Text>Increment</Text>
          </Pressable>
        </View>
      ),
      captureState: count,
    };
  },
});
