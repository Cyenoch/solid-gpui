// Build and copy both production binaries first. This runner never compiles.
import { chmod, copyFile, mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { resolve } from "node:path";
import { createHash } from "node:crypto";

const [baseline, candidate, output] = process.argv.slice(2);
if (!baseline || !candidate || !output) {
  throw new Error(
    "Usage: bun scripts/native-region-compare.ts BASELINE_BINARY CANDIDATE_BINARY NEW_ARTIFACT_DIRECTORY",
  );
}
const directory = resolve(output);
await mkdir(directory); // Refuse to overwrite an earlier comparison.
const identities = await Promise.all(
  [baseline, candidate].map(async (path) => ({
    path: resolve(path),
    sha256: createHash("sha256")
      .update(new Uint8Array(await Bun.file(path).arrayBuffer()))
      .digest("hex"),
  })),
);
const bundleRoot =
  process.platform === "darwin" ? await mkdtemp(resolve(tmpdir(), "solid-gpui-region-measure-")) : undefined;
try {
  const executables: string[] = [];
  for (const [index, binary] of [baseline, candidate].entries()) {
    if (!bundleRoot) {
      executables.push(resolve(binary));
      continue;
    }
    const contents = resolve(bundleRoot, `${index}.app`, "Contents");
    await mkdir(resolve(contents, "MacOS"), { recursive: true });
    const executable = resolve(contents, "MacOS/profile");
    await copyFile(resolve(binary), executable);
    await chmod(executable, 0o555);
    await writeFile(
      resolve(contents, "Info.plist"),
      `<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleExecutable</key><string>profile</string>
<key>CFBundleIdentifier</key><string>dev.solid-gpui.region-measurement</string>
<key>CFBundleName</key><string>Region Measurement</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleVersion</key><string>1</string>
<key>NSHighResolutionCapable</key><true/>
</dict></plist>`,
    );
    executables.push(executable);
  }
  await writeFile(
    resolve(directory, "binaries.json"),
    JSON.stringify(
      {
        identities,
        order: ["baseline", "candidate", "candidate", "baseline"],
        platform: process.platform,
        date: new Date().toISOString(),
        hud: false,
        launch: bundleRoot ? "matched temporary macOS app bundles" : "native executable",
        workload: "360 commits; 500 rows; 800/801 continuous native resize plus 560/1280/800 x 600; --regions",
      },
      null,
      2,
    ),
  );
  for (const [index, binary] of [executables[0]!, executables[1]!, executables[1]!, executables[0]!].entries()) {
    const child = Bun.spawn(
      [
        resolve(binary),
        "bun",
        "--conditions=browser",
        "scripts/native-commit-profile.ts",
        "--native",
        "--regions",
        "--resize-each-update",
      ],
      {
        env: { ...process.env, SOLID_GPUI_PROFILE_HUD: "0" },
        stdout: "pipe",
        stderr: "pipe",
      },
    );
    const timer = setTimeout(() => child.kill("SIGTERM"), 60_000);
    const [stdout, stderr, code] = await Promise.all([
      new Response(child.stdout).text(),
      new Response(child.stderr).text(),
      child.exited,
    ]);
    clearTimeout(timer);
    await writeFile(
      resolve(directory, `${index}-${index === 0 || index === 3 ? "baseline" : "candidate"}.log`),
      stdout + stderr,
    );
    if (code !== 0) throw new Error(`Native run ${index} failed with ${code}; discard its timing comparison`);
    const renders = [...stderr.matchAll(/solid_commit_stages:[^\n]* root_renders=(\d+)/g)].reduce(
      (count, match) => count + Number(match[1]),
      0,
    );
    if (renders < 180)
      throw new Error(
        `Native run ${index} reported only ${renders} root renders for 360 updates; discard its timing comparison`,
      );
  }
} finally {
  if (bundleRoot) await rm(bundleRoot, { recursive: true, force: true });
}
