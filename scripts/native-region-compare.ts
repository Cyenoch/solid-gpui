// Build and copy both production binaries first. This runner never compiles.
import { mkdir, writeFile } from "node:fs/promises";
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
      .update(await Bun.file(path).arrayBuffer())
      .digest("hex"),
  })),
);
await writeFile(
  resolve(directory, "binaries.json"),
  JSON.stringify(
    {
      identities,
      order: ["baseline", "candidate", "candidate", "baseline"],
      platform: process.platform,
      date: new Date().toISOString(),
      hud: false,
      workload: "360 commits; 500 rows; 800/560/1280/800 x 600; --regions",
    },
    null,
    2,
  ),
);
for (const [index, binary] of [baseline, candidate, candidate, baseline].entries()) {
  const child = Bun.spawn(
    [resolve(binary), "bun", "--conditions=browser", "scripts/native-commit-profile.ts", "--native", "--regions"],
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
}
