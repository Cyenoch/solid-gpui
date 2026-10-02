import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import provenance from "../src/showcase/reference/provenance.json";

/** Verify exact upstream bytes from an explicit checkout pair or immutable raw URLs. */
export async function verifyReferenceProvenance(checkouts?: Record<string, string>) {
  const results = [];
  for (const fixture of provenance.fixtures) {
    const checkout = checkouts?.[fixture.repository];
    const url = `https://raw.githubusercontent.com/${fixture.repository}/${fixture.commit}/${fixture.path}`;
    let bytes: Uint8Array;
    if (checkout) {
      const result = Bun.spawnSync(["git", "-C", checkout, "show", `${fixture.commit}:${fixture.path}`]);
      if (result.exitCode !== 0) throw new Error(`Cannot read pinned fixture: ${fixture.path}`);
      bytes = result.stdout;
    } else {
      const response = await fetch(url, { signal: AbortSignal.timeout(20_000) });
      if (!response.ok) throw new Error(`${response.status}: ${url}`);
      bytes = new Uint8Array(await response.arrayBuffer());
    }
    const sha256 = createHash("sha256").update(bytes).digest("hex");
    const gitBlob = createHash("sha1").update(`blob ${bytes.length}\0`).update(bytes).digest("hex");
    if (sha256 !== fixture.sha256 || gitBlob !== fixture.gitBlob) throw new Error(`Reference changed: ${fixture.path}`);
    results.push({ ...fixture, verified: true });
  }
  const root = resolve(import.meta.dirname, "../src/showcase");
  for (const source of provenance.sources) await readFile(resolve(root, source));
  return { mode: provenance.mode, fixtures: results };
}
if (import.meta.main) {
  const args = Bun.argv.slice(2);
  if (args.length !== 0 && args.length !== 2)
    throw new Error("Provide GPUIX and gpuix-solid checkout paths, or omit both for immutable HTTP reads.");
  console.log(
    JSON.stringify(
      await verifyReferenceProvenance(
        args.length ? { "remorses/gpuix": args[0]!, "jhomra21/gpuix-solid": args[1]! } : undefined,
      ),
      null,
      2,
    ),
  );
}
