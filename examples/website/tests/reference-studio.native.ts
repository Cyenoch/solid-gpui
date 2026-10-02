import manifest from "./reference-studio.acceptance.json";

export interface PaintedNode {
  id: number;
  x: number;
  y: number;
  width: number;
  height: number;
  text: string;
  inputValue?: string;
  parentLabel?: string;
}
/** Adapter over ticket01's native API, not a semantic TestHost or geometry emulator. */
export interface ReferenceNativeDriver {
  capabilities: readonly string[];
  evidenceKind: "native-test-rendering" | "physical-input";
  settle(): Promise<void>;
  painted(label: string): Promise<PaintedNode>;
  isPainted(label: string): Promise<boolean>;
  click(label: string): Promise<void>;
  replaceText(label: string, text: string): Promise<void>;
  wheel(label: string, dy: number): Promise<void>;
  drag(source: string, target: string): Promise<void>;
  resize(width: number, height: number): Promise<void>;
  scrollOffset(label: string): Promise<number>;
  selectText(from: string, to: string): Promise<void>;
  copySelection(): Promise<void>;
  selection(): Promise<{ text: string; paragraphs: number }>;
  search(query: string): Promise<{ matches: number; activeMatch: number | null }>;
  clipboard(): Promise<string>;
  rowOwners(): Promise<{ live: number; peak: number }>;
  screenshot?(label: string): Promise<string>;
  previewSequence(): Promise<number>;
  cancelPreviewUpload(): Promise<{
    retainedBefore: number;
    retainedDuring: number;
    retainedAfter: number;
    released: number;
  }>;
  close(): Promise<void>;
  resources(): Promise<{
    rowOwners: number;
    surfaces: number;
    windows: number;
    popups: number;
  }>;
}
function check(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message);
}

export async function runReferenceStudioNative(driver: ReferenceNativeDriver) {
  const missing = manifest.requiredCapabilities.filter((capability) => !driver.capabilities.includes(capability));
  check(!missing.length, `Unsupported required native capabilities: ${missing.join(", ")}`);
  const started = Date.now();
  const step = async (action: () => Promise<void>) => {
    check(Date.now() - started < manifest.deadlineMs, "Reference workload exceeded five minutes");
    await action();
    await driver.settle();
  };
  const visible = async (label: string) => {
    const node = await driver.painted(label);
    check(node.width > 0 && node.height > 0, `No painted bounds: ${label}`);
    return node;
  };
  const evidence: Record<string, unknown> = { evidenceKind: driver.evidenceKind, screenshot: "unsupported" };
  try {
    await step(() => driver.resize(1280, 720));
    const identities = await Promise.all(
      manifest.retainedLocators.map(async (label) => [label, (await visible(label)).id] as const),
    );
    await visible("studio.track.track-0");
    await visible("studio.history.history-0");
    await visible("studio.preview.recording");
    await visible("studio.preview.frame");
    const initialFrame = await driver.previewSequence();
    await step(() => driver.click("studio.preview.advance"));
    check((await driver.previewSequence()) > initialFrame, "Advance preview did not publish a new native frame");
    check(
      (await visible("studio.preview.status")).text.includes("9216 bytes"),
      "Native frame upload did not acknowledge retained pixels",
    );
    await step(() => driver.replaceText("studio.clip.title", "字幕 😀 é"));
    await step(() => driver.click("studio.clip.save"));
    check((await visible("studio.status")).text.includes("Saved 字幕 😀 é"), "Unicode save did not reach native paint");
    await step(() => driver.drag("studio.clip.clip-0-a", "studio.track.track-2"));
    check(
      (await visible("studio.status")).text.includes("Moved clip-0-a to track-2"),
      "Native clip drop missed its target",
    );
    check(
      (await visible("studio.clip.clip-0-a")).parentLabel === "studio.track.track-2",
      "Moved clip retained its previous track",
    );
    await step(() => driver.click("studio.clip.clip-0-a"));
    const beforeWheel = await visible("studio.track.track-4");
    const clickedTitle = (await visible("studio.clip.title")).inputValue;
    check(clickedTitle === "字幕 😀 é", `Follow-up click lost edited clip: ${JSON.stringify(clickedTitle)}`);
    await step(() => driver.drag("studio.reorder.track-2", "studio.track.track-0"));
    const moved = await visible("studio.track.track-2");
    const old = await visible("studio.track.track-0");
    check(moved.y < old.y, "Track reorder did not change painted order");
    await step(() => driver.click("studio.clip.clip-0-a"));
    await step(() => driver.wheel("studio.tracks", 180));
    const offset = await driver.scrollOffset("studio.tracks");
    check(offset > 0, "Wheel did not move native scroll offset");
    const displaced = await visible("studio.track.track-4");
    check(displaced.y !== beforeWheel.y, "Wheel did not change painted content");
    await step(() => driver.click("studio.nav.history"));
    await step(() => driver.resize(560, 720));
    check((await visible("studio.history.pane")).width >= 500, "Narrow history did not occupy the viewport");
    check(!(await driver.isPainted("studio.timeline.pane")), "Inactive narrow timeline remained painted");
    await step(() => driver.click("studio.nav.timeline"));
    await step(() => driver.resize(1280, 720));
    for (const [label, id] of identities)
      check((await visible(label)).id === id, `Retained native identity changed: ${label}`);
    check(Math.abs((await driver.scrollOffset("studio.tracks")) - offset) < 1, "Route/resize reset timeline offset");
    check((await visible("studio.clip.title")).inputValue === "字幕 😀 é", "Route/resize lost the edited title");
    await step(() => driver.click("studio.history.last"));
    await visible("studio.history.history-9999");
    await step(() => driver.replaceText("studio.history.search", "Review 10000:"));
    await visible("studio.history.history-9999");
    check((await visible("studio.history.count")).text.includes("1 reviews"), "End-of-list filter lost its row");
    await step(() => driver.replaceText("studio.history.search", ""));
    check(
      (await visible("studio.history.count")).text.includes("10000 reviews"),
      "Clearing search did not restore history",
    );
    await step(() => driver.click("studio.history.first"));
    await step(() => driver.selectText("studio.text.history-0.0", "studio.text.history-0.1"));
    await step(() => driver.copySelection());
    const selected = await driver.clipboard();
    check(
      selected.includes("opening cut") && selected.includes("dialogue clear"),
      "Cross-element selection did not copy both paragraphs",
    );
    const selection = await driver.selection();
    check(
      selection.paragraphs >= 2 && selection.text === selected,
      "Generated selection service disagrees with native copy",
    );
    const search = await driver.search("opening cut");
    check(search.matches > 0 && search.activeMatch === 0, "Generated search did not select a committed native match");
    await driver.search("");
    await step(() => driver.replaceText("studio.note.draft", "Review 字幕 😀 é"));
    await step(() => driver.click("studio.note.post"));
    await visible("studio.history.note-0");
    await step(() => driver.wheel("studio.inspector", 300));
    await step(() => driver.click("studio.review.copy"));
    check((await driver.clipboard()) === "Review 字幕 😀 é", "Review copy did not reach native clipboard");
    const owners = await driver.rowOwners();
    check(owners.peak <= manifest.data.maxLiveRowOwners, `Unbounded row owners: ${owners.peak}`);
    evidence.rowOwners = owners;
    if (driver.screenshot) evidence.screenshot = await driver.screenshot("reference-studio-complete");
    const preview = await driver.cancelPreviewUpload();
    check(
      preview.retainedBefore === 9216 &&
        preview.retainedDuring === 18432 &&
        preview.retainedAfter === 9216 &&
        preview.released === 0,
      `Preview staging cancellation/release failed: ${JSON.stringify(preview)}`,
    );
    evidence.previewResources = preview;
  } finally {
    await driver.close();
  }
  const resources = await driver.resources();
  check(
    Object.values(resources).every((count) => count === 0),
    `Native close leaked resources: ${JSON.stringify(resources)}`,
  );
  return { ...evidence, resources, elapsedMs: Date.now() - started };
}
