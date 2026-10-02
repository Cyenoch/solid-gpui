#!/usr/bin/env bun
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { join, resolve } from "node:path";
import { tmpdir } from "node:os";
import { parseArgs } from "node:util";
import { generateKeyPairSync, sign } from "node:crypto";
import { packageSignedUpdate } from "../packages/solid-gpui-vite/src/package.ts";

const { values } = parseArgs({
  options: { root: { type: "string" }, acceptance: { type: "string" }, mode: { type: "string" } },
  strict: true,
});
if (!values.root || !values.acceptance)
  throw new Error("Pass the external stock consumer --root and --acceptance executable");
const root = resolve(values.root);
const source = `import { expect, test } from "bun:test";
import { createRoot } from "@solid-gpui/core";
import { NativeAcceptance } from "@solid-gpui/core/testing";
import { Counter } from "./src/counter.tsx";
test("the packed standalone counter paints and accepts hit-tested Unicode input", async () => {
  const host = await NativeAcceptance.launch({ command: [${JSON.stringify(resolve(values.acceptance))}], mode: ${JSON.stringify(values.mode ?? "deterministic")} });
  const root = createRoot(host.transport, { surfaceId: 1 });
  try {
    root.render(() => <Counter />);
    const before = host.capabilities.screenshots ? await host.screenshot() : undefined;
    await host.click(await host.locate({ label: "Increment" }));
    expect((await host.locate({ text: "Count: 1" })).bounds.width).toBeGreaterThan(0);
    const snapshot = await host.snapshot();
    const editor = snapshot.nodes.find(node => node.input !== null)!;
    await host.click(await host.locate({ id: editor.id }));
    await host.key("secondary-a");
    await host.type("新值🙂");
    expect((await host.snapshot()).nodes.find(node => node.input !== null)!.input!.value).toBe("新值🙂");
    if (before) expect(Buffer.compare(before.png, (await host.screenshot()).png)).not.toBe(0);
  } finally { root.unmount(); expect(await host.close()).toEqual({ surfaces: 0, windows: 0, popups: 0 }); }
});
`;
const test = join(root, "delivery-native.test.tsx");
await writeFile(test, source);
try {
  const child = Bun.spawn(
    [process.execPath, join(root, "node_modules/@solid-gpui/vite/dist/cli.js"), "test", "delivery-native.test.tsx"],
    { cwd: root, stdio: ["ignore", "inherit", "inherit"] },
  );
  if ((await child.exited) !== 0) throw new Error("Clean packed consumer native acceptance failed");
} finally {
  await rm(test, { force: true });
}

if (process.platform === "darwin") {
  const directory = await mkdtemp(join(process.env.SOLID_GPUI_CONSUMER_TEMP ?? tmpdir(), "delivery-update-"));
  try {
    const archive = join(root, "packages/stock-quickjs-darwin-arm64.tar.gz");
    const extract = Bun.spawn(["tar", "-xzf", archive, "-C", directory], { stdio: ["ignore", "inherit", "inherit"] });
    if ((await extract.exited) !== 0) throw new Error("Update fixture extraction failed");
    const { privateKey, publicKey } = generateKeyPairSync("ed25519");
    const result = await packageSignedUpdate({
      bundle: join(directory, "stock-quickjs/stock-quickjs.app"),
      executable: "Contents/MacOS/stock-quickjs",
      appId: "dev.solidgpui.stock-quickjs",
      channel: "fixture",
      sequence: 2,
      version: "0.1.0",
      url: "https://updates.example.test/candidate.tar",
      output: join(directory, "updates"),
      publicKey: publicKey.export({ type: "spki", format: "pem" }).toString(),
      sign: async (bytes) => new Uint8Array(sign(null, bytes, privateKey)),
    });
    const feed = JSON.parse(await readFile(result.feed, "utf8"));
    const manifest = JSON.parse(Buffer.from(feed.payload, "hex").toString());
    if (manifest.archiveFormat !== "app-tar-v1" || manifest.sha256 !== result.sha256)
      throw new Error("Update packaging contract changed");
    console.log("Final extracted app USTAR and Ed25519 feed verified with an ephemeral local fixture key");
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
}
