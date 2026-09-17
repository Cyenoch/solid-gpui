import { expect, test } from "bun:test";
import { cp, mkdir, mkdtemp, rm, symlink, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { runTests } from "../packages/solid-gpui-vite/src/test.ts";
import { startTestPipeline, type TestPipeline } from "../packages/solid-gpui-vite/src/test-pipeline.ts";

const fixture = resolve(import.meta.dirname, "../packages/solid-gpui-vite/fixtures/test-app");
const quickjsFixture = resolve(import.meta.dirname, "../packages/solid-gpui-vite/fixtures/quickjs-app");
const modeFixture = resolve(import.meta.dirname, "../packages/solid-gpui-vite/fixtures/mode-app");
const repository = resolve(import.meta.dirname, "..");
const sdkDirectory = join(repository, "packages/solid-gpui");
const viteDirectory = join(repository, "packages/solid-gpui-vite");

test("runs an application's tests through its own Vite config", async () => {
  expect(await runTests({ root: fixture })).toBe(0);
}, 30_000);

test("runs a single requested test file", async () => {
  expect(await runTests({ root: fixture, args: ["counter.test.tsx"] })).toBe(0);
}, 30_000);

test("refuses a project whose config does not register the plugin", async () => {
  await expect(runTests({ root: resolve(import.meta.dirname, ".."), args: ["--version"] })).rejects.toThrow(
    /does not register the solidGpui plugin/,
  );
}, 30_000);

test("a linked SDK source with its own Solid copy still shares the consumer's graph", async () => {
  const consumer = await mkdtemp(join(tmpdir(), "solid-gpui-linked-"));
  const root = join(consumer, "app");
  const sdk = join(root, "node_modules", "@solid-gpui", "core");
  try {
    // The linked SDK checkout carries its own solid-js, and the consumer has its
    // own copy: the two must not become two reactive graphs.
    await mkdir(join(root, "node_modules", "@solid-gpui"), { recursive: true });
    await cp(join(sdkDirectory, "package.json"), join(sdk, "package.json"));
    for (const part of ["src", "dist"]) await cp(join(sdkDirectory, part), join(sdk, part), { recursive: true });
    for (const target of [join(sdk, "node_modules", "solid-js"), join(root, "node_modules", "solid-js")])
      await cp(join(repository, "node_modules", "solid-js"), target, { recursive: true, dereference: true });
    await symlink(join(repository, "node_modules", "vite"), join(root, "node_modules", "vite"));
    await symlink(join(sdkDirectory, "node_modules", "bebop"), join(root, "node_modules", "bebop"));
    await symlink(viteDirectory, join(root, "node_modules", "@solid-gpui", "vite"));
    await writeFile(join(root, "package.json"), JSON.stringify({ name: "linked-consumer", type: "module" }));
    await writeFile(
      join(root, "vite.config.ts"),
      `import { solidGpui, solidGpuiSource } from "@solid-gpui/vite";
       export default { root: import.meta.dirname, plugins: [solidGpuiSource({ root: import.meta.dirname, exclude: [] }), solidGpui({ target: "web" })] };
      `,
    );
    await writeFile(
      join(root, "panel.tsx"),
      `import { Text, View } from "@solid-gpui/core";
       import { createEffect } from "@solid-gpui/core/runtime";
       import type { Accessor } from "solid-js";
       export function Panel(props: { value: Accessor<number>; observe: (value: number) => void }) {
         createEffect(() => props.observe(props.value()));
         return <View><Text>{String(props.value())}</Text></View>;
       }
      `,
    );
    await writeFile(
      join(root, "panel.test.tsx"),
      `import { beforeEach, expect, test } from "bun:test";
       import { createSignal } from "solid-js";
       import { MemoryTransport, createRoot } from "@solid-gpui/core";
       import { Panel } from "./panel";
       const seen: number[] = [];
       beforeEach(() => { seen.length = 0; });
       test("consumer Solid and linked SDK source share one reactive graph", async () => {
         const [value, setValue] = createSignal(0);
         const transport = new MemoryTransport();
         const root = createRoot(transport, { surfaceId: 5 });
         try {
           root.render(() => <Panel value={value} observe={(next) => seen.push(next)} />);
           await Promise.resolve();
           expect(seen).toEqual([0]);
           expect(transport.submitted.length).toBe(1);
           setValue(1);
           await Promise.resolve();
           expect(seen).toEqual([0, 1]);
           expect(transport.submitted.length).toBe(2);
         } finally {
           root.unmount();
         }
       });
      `,
    );
    expect(await runTests({ root, args: ["panel.test.tsx"] })).toBe(0);
  } finally {
    await rm(consumer, { recursive: true, force: true });
  }
}, 60_000);

test("two runs in one process open no conflicting sockets", async () => {
  const script = `
    const { runTests } = await import(${JSON.stringify(resolve(import.meta.dirname, "../packages/solid-gpui-vite/src/test.ts"))});
    const codes = await Promise.all([
      runTests({ root: ${JSON.stringify(fixture)}, args: ["counter.test.tsx"] }),
      runTests({ root: ${JSON.stringify(fixture)}, args: ["counter.test.tsx"] }),
    ]);
    console.log("CODES " + codes.join(","));
  `;
  const child = Bun.spawn([process.execPath, "-e", script], {
    cwd: resolve(import.meta.dirname, ".."),
    stdout: "pipe",
    stderr: "pipe",
  });
  const [code, stdout, stderr] = await Promise.all([
    child.exited,
    new Response(child.stdout).text(),
    new Response(child.stderr).text(),
  ]);
  expect(code).toBe(0);
  expect(stdout + stderr).toContain("CODES 0,0");
  expect(stdout + stderr).not.toContain("already in use");
}, 60_000);

test("a QuickJS application's tests read, mutate and spawn with the real environment", async () => {
  // The fixture asserts this value, PATH, its own mutation and the environment a
  // subprocess resolves through PATH; the runner is the only one that can set it.
  const previous = process.env.SOLID_GPUI_FIXTURE_SENTINEL;
  process.env.SOLID_GPUI_FIXTURE_SENTINEL = "inherited-sentinel";
  try {
    expect(await runTests({ root: quickjsFixture })).toBe(0);
  } finally {
    if (previous === undefined) delete process.env.SOLID_GPUI_FIXTURE_SENTINEL;
    else process.env.SOLID_GPUI_FIXTURE_SENTINEL = previous;
  }
}, 30_000);

test("a requested mode reaches a real test run, and the default stays development", async () => {
  const requested = async (mode: string | undefined, expected: string) => {
    const previous = process.env.SOLID_GPUI_FIXTURE_MODE;
    process.env.SOLID_GPUI_FIXTURE_MODE = expected;
    try {
      return await runTests({ root: modeFixture, ...(mode === undefined ? {} : { mode }) });
    } finally {
      if (previous === undefined) delete process.env.SOLID_GPUI_FIXTURE_MODE;
      else process.env.SOLID_GPUI_FIXTURE_MODE = previous;
    }
  };
  // The fixture's own test reads the mode back out of the config alias, the config
  // `define` and `import.meta.env`, and fails on any of them disagreeing.
  expect(await requested(undefined, "development")).toBe(0);
  expect(await requested("staging", "staging")).toBe(0);
  const cli = Bun.spawn({
    cmd: [
      process.execPath,
      join(viteDirectory, "src/cli.ts"),
      "test",
      "--root",
      modeFixture,
      "--mode",
      "staging",
      "--",
      "mode.test.ts",
    ],
    env: { ...process.env, SOLID_GPUI_FIXTURE_MODE: "staging" },
    stdout: "inherit",
    stderr: "inherit",
  });
  expect(await cli.exited).toBe(0);
}, 60_000);
