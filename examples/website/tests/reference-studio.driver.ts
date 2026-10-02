import { NativeAcceptance, type NativeCleanup } from "@solid-gpui/core/testing";
import { mkdir } from "node:fs/promises";
import { resolve } from "node:path";
import { mountReferenceStudio } from "./reference-studio.application";
import { runReferenceStudioNative, type ReferenceNativeDriver } from "./reference-studio.native";

export async function qualifyReferenceStudio(binary: string, mode: "deterministic" | "gpu", artifacts: string) {
  const host = await NativeAcceptance.launch({ command: [binary], mode, timeoutMs: 30_000 });
  const app = mountReferenceStudio(host.transport, 1);
  let cleanup: NativeCleanup | undefined;
  const locate = (label: string) => host.locate({ label });
  const driver: ReferenceNativeDriver = {
    capabilities: ["paintedBounds", "hitTestedClick", "nativeTyping", "nativeWheel", "nativeDragDrop", "nativeResize", "clipboard", "crossElementSelection", "cleanup"],
    evidenceKind: "native-test-rendering",
    settle: () => host.flush(),
    async painted(label) {
      const node = await locate(label);
      const snapshot = await host.snapshot();
      const text = (id: number): string => snapshot.nodes.filter((item) => item.parentId === id).map((item) => item.text ?? text(item.id)).join("");
      return { id: node.id, ...node.bounds, text: node.text ?? text(node.id), inputValue: node.input?.value };
    },
    async click(label) { await host.click(await locate(label)); },
    async replaceText(label, text) { await host.click(await locate(label)); await host.key("secondary-a"); await host.type(text); },
    async wheel(label, dy) { await host.wheel(await locate(label), { x: 0, y: -dy }); },
    async drag(source, destination) {
      const target = await locate(destination);
      const node = await locate(source);
      await host.drag(node, { x: target.bounds.x + target.bounds.width - 12, y: target.bounds.y + target.bounds.height / 2 });
    },
    async resize(width, height) { const pending = app.root.resize(width, height); await host.flush(); await pending; },
    async scrollOffset(label) {
      const value = (await locate(label)).scrollOffset;
      if (value === null) throw new Error(`Missing native scroll offset: ${label}`);
      return value;
    },
    async selectText(from, to) {
      const destination = await locate(to);
      const source = await locate(from);
      await host.drag(source, { x: destination.bounds.x + destination.bounds.width - 2, y: destination.bounds.y + destination.bounds.height / 2 },
        { from: { x: source.bounds.x + 2, y: source.bounds.y + 8 } });
    },
    copySelection: () => host.key("secondary-c"),
    async clipboard() { return await host.clipboardText() ?? ""; },
    async rowOwners() { return app.rowOwners(); },
    ...(host.capabilities.screenshots ? { async screenshot(label: string) {
      await mkdir(artifacts, { recursive: true });
      const image = await host.screenshot();
      const path = resolve(artifacts, `${label}.png`);
      await Bun.write(path, image.png);
      return path;
    } } : {}),
    async close() { app.dispose(); cleanup = await host.close(); },
    async resources() {
      if (!cleanup || cleanup.surfaces || cleanup.windows || cleanup.popups) throw new Error("Native host did not release surfaces and windows");
      // Native registries and their input/list/selection/media owners are dropped by verified host closure.
      return { rowOwners: app.rowOwners().live, nativeInputs: cleanup.surfaces, nativeLists: cleanup.surfaces, selectionOwners: cleanup.surfaces, previewResources: cleanup.surfaces };
    },
  };
  try { return await runReferenceStudioNative(driver); }
  finally { app.dispose(); await host.close(); }
}
