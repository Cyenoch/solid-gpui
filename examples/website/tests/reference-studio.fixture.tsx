import { MemoryTransport, createRoot } from "@solid-gpui/core";
import { TestHost } from "@solid-gpui/core/testing";
import { createSignal } from "@solid-gpui/core/runtime";
import { ReferenceStudio, createReferenceStudioState } from "../src/showcase/ReferenceStudio";

export async function verifyReferenceStudio() {
  const transport = new MemoryTransport();
  const host = new TestHost(transport);
  const surfaceId = 91;
  const root = createRoot(transport, { surfaceId });
  const living = new Set<object>();
  let peak = 0;
  let copy = "";
  const [view, setView] = createSignal<"timeline" | "history">("timeline");
  const [width, setWidth] = createSignal(1000);
  const state = createReferenceStudioState();
  const find = (label: string) => {
    const matches = host.surface(surfaceId)!.nodes.filter((node) => node.accessibilityLabel === label);
    if (matches.length !== 1) throw new Error(`Expected one locator ${label}; got ${matches.length}`);
    return matches[0]!;
  };
  const tick = async () => {
    await Promise.resolve();
    await Promise.resolve();
  };
  try {
    root.render(() => (
      <ReferenceStudio
        state={state}
        width={width()}
        view={view()}
        navigate={setView}
        copyText={async (text) => {
          copy = text;
        }}
        onRowLifetime={(_kind, _id, mounted, owner) => {
          if (mounted) living.add(owner);
          else living.delete(owner);
          peak = Math.max(peak, living.size);
        }}
      />
    ));
    const retained = [
      "studio.shell",
      "studio.navigation",
      "studio.timeline.pane",
      "studio.history.pane",
      "studio.tracks",
      "studio.history.list",
      "studio.clip.title",
      "studio.note.draft",
    ].map((label) => [label, find(label).id] as const);
    host.dispatch(find("studio.clip.title"), { type: "input", text: "字幕 😀 é" });
    await tick();
    host.dispatch(find("studio.clip.save"), { type: "press" });
    await tick();
    if (state.selectedClip().label !== "字幕 😀 é") throw new Error("Controlled title did not save");
    host.dispatch(find("studio.nav.history"), { type: "press" });
    setWidth(520);
    await tick();
    host.dispatch(find("studio.nav.timeline"), { type: "press" });
    setWidth(1000);
    await tick();
    for (const [label, id] of retained) if (find(label).id !== id) throw new Error(`Native identity changed: ${label}`);
    host.dispatch(find("studio.history.list"), { type: "visible-range", start: 9994, end: 10000 });
    await tick();
    if (!host.surface(surfaceId)!.nodes.some((node) => node.accessibilityLabel === "studio.history.history-9999"))
      throw new Error("Final review is missing from the requested row window");
    host.dispatch(find("studio.history.search"), { type: "input", text: "Review 10000:" });
    await tick();
    if (
      state.visibleHistory().length !== 1 ||
      !host.surface(surfaceId)!.nodes.some((node) => node.accessibilityLabel === "studio.history.history-9999")
    )
      throw new Error("Filtering at the end lost the matching row");
    host.dispatch(find("studio.note.post"), { type: "press" });
    await tick();
    if (state.selectedHistory().paragraphs[0] !== "Review 字幕 😀 é" || state.history().length !== 10001)
      throw new Error("Unicode note was lost");
    const scroll = host.scrollCommands.at(-1);
    if (!scroll || scroll.type !== "scroll-to-index" || scroll.index !== 0)
      throw new Error("Post did not request the first native row");
    host.replyScroll(scroll);
    host.dispatch(find("studio.review.copy"), { type: "press" });
    await tick();
    if (copy !== "Review 字幕 😀 é") throw new Error("Copy did not use selected review text");
    if (peak > 40) throw new Error(`Row owners were not bounded: ${peak}`);
  } finally {
    root.unmount();
  }
  if (living.size !== 0) throw new Error(`Unmount retained ${living.size} row owners`);
}
