import { expect, test } from "bun:test";
import { copyFile, mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";

test("plain Bun commands run when the JSX compiler cannot be loaded", async () => {
  const directory = await mkdtemp(join(tmpdir(), "solid-jsx-preload-"));
  try {
    await mkdir(join(directory, "scripts"));
    await mkdir(join(directory, "packages/solid-gpui-vite/src"), { recursive: true });
    await copyFile(join(import.meta.dirname, "solid-jsx.ts"), join(directory, "scripts/solid-jsx.ts"));
    await writeFile(join(directory, "bunfig.toml"), 'preload = ["./scripts/solid-jsx.ts"]\n');
    await writeFile(
      join(directory, "packages/solid-gpui-vite/src/transform.ts"),
      'throw new Error("JSX compiler binding is unavailable");\nexport function transformJsx() {}\n',
    );
    const child = Bun.spawn([process.execPath, "--eval", 'console.log("plain command completed")'], {
      cwd: directory,
      stdin: "ignore",
      stdout: "pipe",
      stderr: "pipe",
      timeout: 5000,
    });
    const [code, stdout, stderr] = await Promise.all([
      child.exited,
      new Response(child.stdout).text(),
      new Response(child.stderr).text(),
    ]);
    expect(code, stderr).toBe(0);
    expect(stdout.trim()).toBe("plain command completed");
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});
