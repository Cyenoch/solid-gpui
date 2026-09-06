import { Pressable, StdioTransport, Text, View, createRoot } from "../packages/solid-gpui/dist/index.js";
import { createComponent, createSignal } from "../packages/solid-gpui/dist/runtime.js";

function PressRoundtrip() {
  const [count, setCount] = createSignal(0);
  return createComponent(View, {
    style: { padding: 24, gap: 12 },
    get children() {
      return [
        createComponent(Text, {
          style: { color: "#FAFAFA", fontSize: 28, fontWeight: "semibold" },
          children: () => `Count: ${count()}`,
        }),
        createComponent(Pressable, {
          style: { padding: 12, backgroundColor: "#2563EB", borderRadius: 8 },
          accessibilityRole: "button",
          accessibilityLabel: "Increment",
          onPress: () => setCount((value) => value + 1),
          children: createComponent(Text, {
            style: { color: "#FFFFFF", fontWeight: "medium" },
            children: "Increment",
          }),
        }),
      ];
    },
  });
}

const root = createRoot(new StdioTransport(), { surfaceId: 1, epoch: 1 });
root.render(() => createComponent(PressRoundtrip, {}));
