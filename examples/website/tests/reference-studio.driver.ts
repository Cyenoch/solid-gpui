import { NativeAcceptance, type NativeCleanup } from "@solid-gpui/core/testing";
import { mkdir } from "node:fs/promises";
import { resolve } from "node:path";
import { mountReferenceStudio } from "./reference-studio.application";
import { createClient } from "../src/generated/native";
import { runReferenceStudioNative, type ReferenceNativeDriver } from "./reference-studio.native";

export async function qualifyReferenceStudio(binary: string, mode: "deterministic" | "gpu", artifacts: string) {
  const host = await NativeAcceptance.launch({ command: [binary], mode, timeoutMs: 30_000 });
  const app = mountReferenceStudio(host.transport, 1);
  const native = createClient(app.root);
  const invoke = async <T>(request: Promise<T>): Promise<T> => {
    await host.flush();
    return request;
  };
  let cleanup: NativeCleanup | undefined;
  const locate = async (label: string) => {
    try {
      return await host.locate({ label });
    } catch (cause) {
      throw new Error(`Native reference locator failed: ${label}`, { cause });
    }
  };
  const driver: ReferenceNativeDriver = {
    capabilities: [
      "paintedBounds",
      "hitTestedClick",
      "nativeTyping",
      "nativeWheel",
      "nativeDragDrop",
      "nativeResize",
      "clipboard",
      "crossElementSelection",
      "cleanup",
    ],
    evidenceKind: "native-test-rendering",
    settle: () => host.flush(),
    async painted(label) {
      const node = await locate(label);
      const snapshot = await host.snapshot();
      const nodes = new Map(snapshot.nodes.map((item) => [item.id, item]));
      let ancestor = nodes.get(node.parentId);
      while (ancestor && !ancestor.label) ancestor = nodes.get(ancestor.parentId);
      const text = (id: number): string =>
        snapshot.nodes
          .filter((item) => item.parentId === id)
          .map((item) => item.text ?? text(item.id))
          .join("");
      return {
        id: node.id,
        ...node.bounds,
        text: node.text ?? text(node.id),
        inputValue: node.input?.value,
        parentLabel: ancestor?.label ?? undefined,
      };
    },
    async isPainted(label) {
      return (await host.snapshot()).nodes.some((node) => node.label === label);
    },
    async click(label) {
      await host.click(await locate(label));
    },
    async replaceText(label, text) {
      await host.click(await locate(label));
      await host.key("secondary-a");
      await host.key("backspace");
      await host.type(text);
    },
    async wheel(label, dy) {
      await host.wheel(await locate(label), { x: 0, y: -dy });
    },
    async drag(source, destination) {
      const target = await locate(destination);
      const node = await locate(source);
      await host.drag(node, {
        x: target.bounds.x + target.bounds.width - 12,
        y: target.bounds.y + target.bounds.height / 2,
      });
    },
    resize: (width, height) => host.resize(width, height),
    async scrollOffset(label) {
      const value = (await locate(label)).scrollOffset;
      if (value === null) throw new Error(`Missing native scroll offset: ${label}`);
      return value;
    },
    async selectText(from, to) {
      const destination = await locate(to);
      const source = await locate(from);
      await host.drag(
        source,
        {
          x: destination.bounds.x + destination.bounds.width - 2,
          y: destination.bounds.y + destination.bounds.height / 2,
        },
        { from: { x: source.bounds.x + 2, y: source.bounds.y + 8 } },
      );
    },
    copySelection: () => host.key("secondary-c"),
    async selection() {
      const result = await invoke(native.getTextSelection());
      return { text: result.text, paragraphs: result.spans.length };
    },
    async search(query) {
      const result = await invoke(native.searchText({ query }));
      if (result.matches.length)
        await invoke(
          native.selectTextSearchMatch({
            textRevision: result.textRevision,
            searchRevision: result.searchRevision,
            matchIndex: 0,
          }),
        );
      if (!query) await invoke(native.clearTextSelection());
      const current = await invoke(native.getTextSearch());
      return { matches: current.matches.length, activeMatch: current.activeMatch };
    },
    async clipboard() {
      return (await host.clipboardText()) ?? "";
    },
    async rowOwners() {
      return app.rowOwners();
    },
    ...(host.capabilities.screenshots
      ? {
          async screenshot(label: string) {
            await mkdir(artifacts, { recursive: true });
            const image = await host.screenshot();
            const path = resolve(artifacts, `${label}.png`);
            await Bun.write(path, image.png);
            return path;
          },
        }
      : {}),
    async previewSequence() {
      const state = await invoke(app.preview().getState());
      if (state.sequence === null) throw new Error("Preview has no committed frame");
      return state.sequence;
    },
    async cancelPreviewUpload() {
      const frame = app.preview();
      const before = await invoke(frame.getState());
      if (before.sequence === null) throw new Error("Preview has no committed frame");
      const sequence = before.sequence + 1;
      const during = await invoke(frame.beginFrame({ sequence, width: 64, height: 36 }));
      await invoke(frame.writeFrameChunk({ sequence, offset: 0, rgba: [255, 0, 0, 255] }));
      const after = await invoke(frame.cancelFrame());
      const released = await invoke(frame.clear());
      return {
        retainedBefore: before.retainedBytes,
        retainedDuring: during.retainedBytes,
        retainedAfter: after.retainedBytes,
        released: released.retainedBytes,
      };
    },
    async close() {
      app.dispose();
      cleanup = await host.close();
    },
    async resources() {
      if (!cleanup || cleanup.surfaces || cleanup.windows || cleanup.popups)
        throw new Error("Native host did not release surfaces and windows");
      return {
        rowOwners: app.rowOwners().live,
        ...cleanup,
      };
    },
  };
  try {
    await app.ready;
    return await runReferenceStudioNative(driver);
  } finally {
    app.dispose();
    await host.close();
  }
}
