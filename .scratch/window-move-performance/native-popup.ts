import { createComponent, createSignal } from "../../packages/solid-gpui/src/runtime";
import { mountApplication, Pressable, SystemPopover, Text, TextInput, View, type SolidElement } from "../../packages/solid-gpui/src/index";
import { StdioTransport } from "../../packages/solid-gpui/src/stdio";

const [open, setOpen] = createSignal(false);
const [nested, setNested] = createSignal(false);
const [name, setName] = createSignal("Ada");
const [count, setCount] = createSignal(0);
const text = (children: string | (() => string)) => createComponent(Text, { children, style: { color: "#f0f3f8", fontSize: 18 } });
const popup = (isOpen: () => boolean, update: (value: boolean) => void, label: string, content: () => SolidElement) =>
  createComponent(SystemPopover, {
    get open() { return isOpen(); },
    onOpenChange: update,
    width: 340,
    height: 230,
    placement: "bottom-start",
    gap: 8,
    accessibilityLabel: label,
    onError: (error) => { console.error("POPUP_ERROR", error); throw error; },
    slots: { trigger: text(label) },
    content,
  });

const application = mountApplication({
  transport: () => new StdioTransport(),
  setup: () => ({ render: () => createComponent(View, {
    style: { flexGrow: 1, flexDirection: "column", padding: 20, gap: 16, backgroundColor: "#18202c" },
    children: [
      text(() => `Counter: ${count()} | Saved name: ${name()}`),
      createComponent(Pressable, {
        accessibilityLabel: "Increment",
        style: { height: 40, padding: 8, backgroundColor: "#334866" },
        onPress: () => setCount(count() + 1),
        children: text("Increment"),
      }),
      popup(open, setOpen, "Edit name", () => createComponent(View, {
        style: { padding: 16, gap: 12, widthPercent: 100, heightPercent: 100, backgroundColor: "#25344a" },
        children: [
          text("Native popup follows its owner"),
          createComponent(TextInput, {
            accessibilityLabel: "Name",
            get value() { return name(); },
            onChangeText: setName,
            style: { height: 36, color: "#ffffff", backgroundColor: "#334866" },
          }),
          popup(nested, setNested, "Open nested popup", () => createComponent(View, {
            style: { padding: 16, backgroundColor: "#334866", widthPercent: 100, heightPercent: 100 },
            children: text("Nested popup content"),
          })),
        ],
      })),
      createComponent(View, {
        style: { flexGrow: 1, minHeight: 0, overflow: "scroll" },
        children: Array.from({ length: 500 }, (_, index) => text(`Retained content row ${index + 1}`)),
      }),
    ],
  }) }),
});
await application.root!.setTitle("Solid GPUI Popup Movement Probe");
await application.root!.resize(800, 600);
await application.root!.activateWindow();
console.error("POPUP_PROBE_READY");
if (process.argv.includes("--open-nested")) {
  setTimeout(() => setOpen(true), 1000);
  setTimeout(() => setNested(true), 2000);
}
setTimeout(() => application.quit(), 1_800_000);
