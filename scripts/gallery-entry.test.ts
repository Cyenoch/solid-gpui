import { expect, test } from "bun:test";
import { join, resolve } from "node:path";

const repoRoot = resolve(import.meta.dir, "..");
const preload = join(repoRoot, "scripts/solid-jsx.ts");
const entry = join(repoRoot, "examples/gallery/src/main.tsx");
const maxFramePayload = 16 * 1024 * 1024;

async function readFrameLength(stream: ReadableStream<Uint8Array>): Promise<number> {
  const reader = stream.getReader();
  const prefix = new Uint8Array(4);
  let offset = 0;
  while (offset < prefix.length) {
    const { done, value } = await reader.read();
    if (done || value === undefined) throw new Error("gallery renderer closed before its first frame");
    const consumed = Math.min(value.length, prefix.length - offset);
    prefix.set(value.subarray(0, consumed), offset);
    offset += consumed;
  }
  return new DataView(prefix.buffer).getUint32(0, true);
}

test("gallery renderer writes a bounded binary frame before any text", async () => {
  const child = Bun.spawn(["bun", "run", "--conditions=browser", "--preload", preload, entry], {
    cwd: repoRoot,
    stdin: "pipe",
    stdout: "pipe",
    stderr: "pipe",
  });

  try {
    const frameLength = await readFrameLength(child.stdout);
    expect(frameLength).toBeGreaterThan(0);
    expect(frameLength).toBeLessThanOrEqual(maxFramePayload);
  } finally {
    child.kill("SIGTERM");
    await child.exited;
  }
});
