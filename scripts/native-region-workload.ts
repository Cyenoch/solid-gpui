import { createComponent, createSignal } from "../packages/solid-gpui/src/runtime";
import { Pressable, Text, TextInput, View } from "../packages/solid-gpui/src/index";

/** Shared production measurement and native paint/input qualification workload. */
export function createRegionWorkload(rows: number, regions: boolean) {
  const [count, setCount] = createSignal(0);
  const [text, setText] = createSignal("Edit here to verify native input");
  let appRuns = 0;
  const render = () => {
    appRuns++;
    const staticRows = () =>
      Array.from({ length: rows }, (_, index) =>
        createComponent(Text, {
          accessibilityLabel: `profile.row.${index + 1}`,
          style: { color: "#d8e4f2", fontSize: 14, lineHeight: 24 },
          children: `Unchanged row ${index + 1}`,
        }),
      );
    return createComponent(View, {
      style: { flexGrow: 1, flexDirection: "column", padding: 20, gap: 12, backgroundColor: "#18202c" },
      children: [
        createComponent(Text, {
          accessibilityLabel: "profile.counter",
          style: { color: "#ffffff", fontSize: 24 },
          children: () => `Count: ${count()}`,
        }),
        createComponent(Pressable, {
          accessibilityLabel: "profile.increment",
          style: { height: 36, backgroundColor: "#334866", padding: 8 },
          onPress: () => setCount((value) => value + 1),
          children: createComponent(Text, { style: { color: "#ffffff" }, children: "Increment" }),
        }),
        createComponent(TextInput, {
          accessibilityLabel: "profile.draft",
          style: { height: 36, padding: 6, backgroundColor: "#25344a", color: "#ffffff" },
          get value() {
            return text();
          },
          onChangeText: setText,
        }),
        createComponent(View, {
          accessibilityLabel: "profile.scroll",
          style: { height: 0, flexGrow: 1, minHeight: 0, overflow: "scroll" },
          children: regions
            ? createComponent(View, {
                style: { width: 480, height: rows * 24, flexShrink: 0, overflow: "hidden" },
                children: staticRows(),
              })
            : staticRows(),
        }),
      ],
    });
  };
  return { render, count, setCount, text, appRuns: () => appRuns };
}
