import { strict as assert } from "node:assert";
import { mkdir } from "node:fs/promises";
import { resolve } from "node:path";
import { createRoot } from "../packages/solid-gpui/src/index";
import { NativeAcceptance } from "../packages/solid-gpui/src/testing/native-acceptance";
import { createRegionWorkload } from "./native-region-workload";

const [binary, output] = Bun.argv.slice(2);
if (!binary || !output)
  throw new Error("Usage: bun --conditions=browser qualify-native-region.ts <acceptance binary> <output>");
const host = await NativeAcceptance.launch({ command: [resolve(binary)], mode: "gpu" });
const workload = createRegionWorkload(500, true);
const root = createRoot(host.transport, { surfaceId: 1 });
try {
  root.render(workload.render);
  await host.resize(800, 600);
  workload.setCount(360);
  await host.flush();
  assert.equal((await host.locate({ label: "profile.counter" })).text, "Count: 360");
  await host.click(await host.locate({ label: "profile.increment" }));
  assert.equal((await host.locate({ label: "profile.counter" })).text, "Count: 361");
  await host.click(await host.locate({ label: "profile.draft" }));
  await host.key("secondary-a");
  await host.type("Native edit 字幕 🙂");
  assert.equal(workload.text(), "Native edit 字幕 🙂");
  assert.equal((await host.locate({ label: "profile.draft" })).input?.value, workload.text());
  const first = await host.locate({ label: "profile.row.1" });
  await host.wheel(await host.locate({ label: "profile.scroll" }), { x: 0, y: -180 });
  assert(!(await host.snapshot()).nodes.some((node) => node.id === first.id && node.bounds.y === first.bounds.y));
  await host.wheel(await host.locate({ label: "profile.scroll" }), { x: 0, y: -20_000 });
  const last = await host.locate({ label: "profile.row.500" });
  assert.equal(last.text, "Unchanged row 500");
  assert(last.bounds.width > 0 && last.bounds.height > 0);
  for (const width of [560, 1280, 800]) {
    await host.resize(width, 600);
    assert.equal((await host.locate({ label: "profile.counter" })).text, "Count: 361");
    assert.equal((await host.locate({ label: "profile.draft" })).input?.value, workload.text());
  }
  await mkdir(resolve(output), { recursive: true });
  const screenshot = await host.screenshot();
  await Bun.write(resolve(output, "region-correctness.png"), screenshot.png);
  await Bun.write(
    resolve(output, "correctness.json"),
    JSON.stringify(
      {
        kind: "macOS GPU native test rendering",
        rows: 500,
        counter: "Count: 361",
        input: workload.text(),
        finalRow: last.text,
        appRuns: workload.appRuns(),
        screenshot: "region-correctness.png",
        productionWindowAutomation:
          "unbundled/temp app window unavailable to computer-use service; no timing from failed hold run retained",
      },
      null,
      2,
    ) + "\n",
  );
  console.log(
    "Shared 500-row workload passed native GPU paint, click, typing, displacement, final-row access, and resize",
  );
} finally {
  root.unmount();
  assert.deepEqual(await host.close(), { surfaces: 0, windows: 0, popups: 0 });
}
