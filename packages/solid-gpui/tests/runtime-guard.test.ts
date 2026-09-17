import { expect, test } from "bun:test";
import { cp, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { MemoryTransport, Text, View, createRoot } from "../src/index";

const source = resolve(import.meta.dirname, "../src");
const packageDirectory = resolve(import.meta.dirname, "..");

const SERVER_SOLID = "solid-js resolved to its server build";
const DUPLICATE_SOLID = "two copies of solid-js are loaded";

async function run(script: string, conditions: readonly string[] = []): Promise<{ code: number; output: string }> {
  const directory = await mkdtemp(join(tmpdir(), "solid-gpui-guard-"));
  const file = join(directory, "probe.ts");
  await writeFile(file, script);
  try {
    const child = Bun.spawn([process.execPath, ...[...conditions].map((c) => `--conditions=${c}`), file], {
      cwd: packageDirectory,
      stdout: "pipe",
      stderr: "pipe",
    });
    const [code, stdout, stderr] = await Promise.all([
      child.exited,
      new Response(child.stdout).text(),
      new Response(child.stderr).text(),
    ]);
    return { code, output: stdout + stderr };
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
}

test("a root renders while Solid's client build is loaded", async () => {
  const transport = new MemoryTransport();
  const root = createRoot(transport, { surfaceId: 1 });
  try {
    root.render(() => Text({ children: "guard" }));
    expect(transport.submitted.length).toBeGreaterThan(0);
  } finally {
    root.unmount();
  }
  expect(View).toBeDefined();
});

test("a renderer started on Solid's server build fails loudly instead of rendering nothing", async () => {
  const { code, output } = await run(
    `import { MemoryTransport, createRoot } from ${JSON.stringify(join(source, "index.ts"))};
     createRoot(new MemoryTransport()).render(() => null);`,
  );
  expect(output).toContain(SERVER_SOLID);
  expect(code).not.toBe(0);
});

test("a second Solid copy is reported instead of silently splitting the graph", async () => {
  const directory = await mkdtemp(join(tmpdir(), "solid-gpui-duplicate-"));
  try {
    // A second copy of Solid, and of the guard that has to notice it. The copy
    // is dereferenced so it is a genuinely different module, not a symlink to
    // the same file.
    const copy = join(directory, "sdk");
    await cp(join(source, "runtime-guard.ts"), join(directory, "sdk-runtime-guard.ts"));
    await cp(join(process.cwd(), "node_modules", "solid-js"), copy, { recursive: true, dereference: true });
    await writeFile(join(directory, "package.json"), JSON.stringify({ name: "duplicate-solid-probe", type: "module" }));
    const guard = await readFile(join(directory, "sdk-runtime-guard.ts"), "utf8");
    await writeFile(
      join(directory, "guard.ts"),
      guard.split('from "solid-js"').join(`from "${join(copy, "dist", "solid.js")}"`),
    );
    const { code, output } = await run(
      `import { createRoot, MemoryTransport } from ${JSON.stringify(join(source, "index.ts"))};
       createRoot(new MemoryTransport());
       const second = await import(${JSON.stringify(join(directory, "guard.ts"))});
       second.assertSolidRuntime();`,
      ["browser"],
    );
    expect(output).toContain(DUPLICATE_SOLID);
    expect(code).not.toBe(0);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});
