export const examples = {
  Counter: `import { View, Text, Pressable } from "@solid-gpui/core";
import { createSignal } from "@solid-gpui/core/runtime";

export default function App() {
  const [count, setCount] = createSignal(0);
  return (
    <View style={{ padding: 32, gap: 24, backgroundColor: "#0a0a0a" }}>
      <Text style={{ fontSize: 28, color: "#fafafa" }}>Make every update count.</Text>
      <Text style={{ fontSize: 64, color: "#fafafa" }}>{count()}</Text>
      <Pressable onPress={() => setCount(count() + 1)}
        style={{ padding: 16, borderRadius: 8, backgroundColor: "#fafafa" }}>
        <Text style={{ color: "#0a0a0a" }}>Increment +</Text>
      </Pressable>
    </View>
  );
}`,
  Input: `import { View, Text, TextInput } from "@solid-gpui/core";
import { createSignal } from "@solid-gpui/core/runtime";

export default function App() {
  const [name, setName] = createSignal("World");
  return (
    <View style={{ padding: 32, gap: 24, backgroundColor: "#0a0a0a" }}>
      <Text style={{ color: "#fafafa", fontSize: 32 }}>Hello, {name()}!</Text>
      <TextInput value={name()} onChangeText={setName}
        placeholder="Your name" style={{ height: 48, padding: 12, color: "#fafafa",
          backgroundColor: "#1c1c1c", borderRadius: 8 }} />
    </View>
  );
}`,
  Layout: `import { View, Text, Pressable } from "@solid-gpui/core";
import { createSignal } from "@solid-gpui/core/runtime";

export default function App() {
  const [row, setRow] = createSignal(true);
  return (
    <View style={{ padding: 24, gap: 20, backgroundColor: "#0a0a0a" }}>
      <Pressable onPress={() => setRow(!row())} style={{ padding: 12, backgroundColor: "#fafafa" }}>
        <Text style={{ color: "#0a0a0a" }}>Toggle direction</Text>
      </Pressable>
      <View style={{ flexDirection: row() ? "row" : "column", gap: 12 }}>
        {["Overview", "Activity", "Settings"].map(label =>
          <View style={{ padding: 20, backgroundColor: "#1c1c1c", borderRadius: 8 }}>
            <Text style={{ color: "#fafafa" }}>{label}</Text>
          </View>
        )}
      </View>
    </View>
  );
}`,
} as const;
export type ExampleName = keyof typeof examples;
