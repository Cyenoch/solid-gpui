import { expect, test } from "bun:test";
import { readdir, readFile } from "node:fs/promises";
import { resolve } from "node:path";

const repoRoot = resolve(import.meta.dir, "..");

async function task(...args: string[]): Promise<{ code: number; output: string }> {
  const child = Bun.spawn([process.execPath, "scripts/tasks.ts", ...args], {
    cwd: repoRoot,
    stdout: "pipe",
    stderr: "pipe",
  });
  const [code, stdout, stderr] = await Promise.all([
    child.exited,
    new Response(child.stdout).text(),
    new Response(child.stderr).text(),
  ]);
  return { code, output: stdout + stderr };
}

test("every GitHub workflow task is a callable CLI entrypoint", async () => {
  const directory = resolve(repoRoot, ".github/workflows");
  const commands = new Set<string>();
  for (const file of await readdir(directory)) {
    if (!/\.ya?ml$/.test(file)) continue;
    const source = await readFile(resolve(directory, file), "utf8");
    for (const match of source.matchAll(/\bbun run task ([a-z][a-z0-9-]*)/g)) commands.add(match[1]!);
  }
  expect(commands.size).toBeGreaterThan(0);
  for (const command of commands) {
    const result = await task(command, "--help");
    expect(result.code, `${command}: ${result.output}`).toBe(0);
    expect(result.output).toContain(`Usage: task ${command} `);
  }
});

test("release version operand reaches the release script's validation", async () => {
  const result = await task("release-prep", "0.0.0");
  expect(result.code).toBe(2);
  expect(result.output).toContain("release-prep: VERSION must be strict MAJOR.MINOR.PATCH");
});
