import { createRoot, View } from "@solid-gpui/core";
import { createSignal } from "@solid-gpui/core/runtime";
import { WebTransport } from "@solid-gpui/core/web";
import catalog from "virtual:component-catalog";
import previews from "virtual:component-previews";
import { loadHost } from "../src/wasm";
import { ComponentPreview } from "../src/ComponentPreview";
import { configureSite } from "../src/site";
import { Copy, colors } from "../src/ui";

// Development-only harness: mount each documented composition inside its production preview shell.
const entries = catalog.flatMap((page) => [
  { id: page.name, title: page.name, source: page.source, compact: false, page },
  ...page.examples.map((example) => ({ ...example, title: `${page.name}: ${example.title}`, compact: true, page })),
]);
const [selection, setSelection] = createSignal(location.hash.slice(1) || entries[0].id);
const [width, setWidth] = createSignal(innerWidth);
window.addEventListener("hashchange", () => setSelection(location.hash.slice(1)));
window.addEventListener("resize", () => setWidth(innerWidth));
configureSite({
  copyText: (text) => navigator.clipboard.writeText(text),
  openUrl: async () => {},
  saveLanguage: () => {},
});
const transport = new WebTransport(await loadHost());
const root = createRoot(transport);
transport.onTermination((error) => {
  document.title = `ERROR: ${String(error)}`;
});
root.render(() => (
  <View
    style={{
      width: width(),
      height: innerHeight,
      backgroundColor: colors.bg,
      color: colors.text,
      fontFamily: "Inter Variable",
      padding: 20,
      gap: 20,
      overflow: "scroll",
    }}
  >
    {() => {
      const entry = entries.find((entry) => entry.id === selection());
      if (!entry) throw new Error(`Unknown audit preview: ${selection()}`);
      document.title = entry.id;
      return (
        <View style={{ width: Math.min(680, width() - 40), gap: 20 }}>
          <Copy translate={false} size={22}>
            {entry.title}
          </Copy>
          <ComponentPreview
            width={Math.min(680, width() - 40)}
            source={entry.source}
            compact={entry.compact}
            unavailable={entry.page.previewNote}
            preview={previews[entry.id]}
            previewWidth={entry.page.previewWidth}
            minPreviewWidth={entry.page.previewMinWidth}
            centered={entry.page.previewCentered}
          />
        </View>
      );
    }}
  </View>
));
window.addEventListener(
  "pagehide",
  () => {
    root.unmount();
    transport.dispose();
  },
  { once: true },
);
