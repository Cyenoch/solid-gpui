import { createRoot, Pressable, Text, View } from "../packages/solid-gpui/src/index";
import { StdioTransport } from "../packages/solid-gpui/src/stdio";
import { createComponent, createSignal } from "../packages/solid-gpui/src/runtime";

// Run: solid-gpui-host bun --conditions=browser fixtures/style-contract.ts
const root = createRoot(new StdioTransport());
root.render(() => {
  const [saved, setSaved] = createSignal(0);
  return createComponent(View, {
    style: { paddingX: 24, paddingY: 16, gap: 12, backgroundColor: "#101827", color: "#ffffff" },
    children: [
      createComponent(Text, { children: "Native style contract" }),
      createComponent(Pressable, {
        focusable: true,
        accessibilityLabel: "Save changes",
        onPress: () => setSaved((count) => count + 1),
        style: {
          width: { unit: "rem", value: 14 },
          paddingX: 16,
          paddingY: 8,
          borderWidth: 2,
          borderColor: "#00000000",
          backgroundColor: "#26344a",
          hover: { backgroundColor: "#344866" },
          active: { backgroundColor: "#182235" },
          focusVisible: { borderColor: "#82b4ff" },
        },
        get children() {
          return createComponent(Text, { children: `Saved ${saved()} times` });
        },
      }),
      createComponent(View, {
        style: { width: 280, height: 120, overflowX: "hidden", overflowY: "scroll" },
        children: createComponent(View, {
          style: { width: { unit: "percent", value: 100 }, height: 320, paddingY: 12, backgroundColor: "#25344a" },
          children: createComponent(Text, { children: "Scroll this vertical pane" }),
        }),
      }),
    ],
  });
});
setTimeout(() => root.unmount(), 5000);
