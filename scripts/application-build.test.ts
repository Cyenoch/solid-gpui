import { expect, test } from "bun:test";
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { buildApplication } from "../packages/solid-gpui/src/vite/build";

test("runtime bundles reject unavailable QuickJS imports without overwriting a working output", async () => {
  const directory = await mkdtemp(join(tmpdir(), "solid-gpui-build-"));
  const entry = join(directory, "app.ts");
  const outfile = join(directory, "app.js");
  try {
    await writeFile(outfile, "previous successful build");
    for (const module of ["fs", "node:fs", "bun", "bun:sqlite"]) {
      await writeFile(entry, `import * as service from ${JSON.stringify(module)}; console.log(service);`);
      await expect(buildApplication({ runtime: "quickjs", entry, outfile })).rejects.toThrow("QuickJS cannot import");
      expect(await readFile(outfile, "utf8")).toBe("previous successful build");
    }
    await writeFile(entry, "const module = globalThis.moduleName; await import(module);");
    await expect(buildApplication({ runtime: "quickjs", entry, outfile })).rejects.toThrow();
    expect(await readFile(outfile, "utf8")).toBe("previous successful build");

    await writeFile(
      entry,
      'import { basename } from "node:path"; console.log(basename("/native/bun") + ":" + process.env.NODE_ENV);',
    );
    await buildApplication({ runtime: "bun", entry, outfile });
    const child = Bun.spawn(["bun", outfile], { stdout: "pipe", stderr: "pipe" });
    const [status, stdout, stderr] = await Promise.all([
      child.exited,
      new Response(child.stdout).text(),
      new Response(child.stderr).text(),
    ]);
    expect(status, stderr).toBe(0);
    expect(stdout.trim()).toBe("bun:production");
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test("production QuickJS bundles omit source maps and run without their authored modules", async () => {
  const directory = await mkdtemp(join(tmpdir(), "solid-gpui-production-build-"));
  const entry = join(directory, "app.ts");
  const dependency = join(directory, "message.ts");
  const outfile = join(directory, "app.js");
  try {
    await writeFile(dependency, 'export const message = "quickjs:" + process.env.NODE_ENV;');
    await writeFile(entry, 'import { message } from "./message"; console.log(message);');
    await buildApplication({ runtime: "quickjs", entry, outfile, sourcemap: "none" });
    expect(await readFile(outfile, "utf8")).not.toContain("sourceMappingURL");
    await rm(entry);
    await rm(dependency);
    // The native package check exercises this output mode in QuickJS itself;
    // this test protects bundling and source-map policy without a Rust build.
    const child = Bun.spawn(["bun", outfile], { stdout: "pipe", stderr: "pipe" });
    const [status, stdout, stderr] = await Promise.all([
      child.exited,
      new Response(child.stdout).text(),
      new Response(child.stderr).text(),
    ]);
    expect(status, stderr).toBe(0);
    expect(stdout.trim()).toBe("quickjs:production");
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test("native async composition runs in a production Bun bundle", async () => {
  const directory = await mkdtemp(join(tmpdir(), "solid-gpui-async-"));
  try {
    const outfile = join(directory, "app.js");
    const preload = join(directory, "host.js");
    await writeFile(
      preload,
      `globalThis.__solidGpuiHost = {
      subscribe() { return () => {}; },
      submit(frame) { process.stdout.write(frame); }
    };`,
    );
    await buildApplication({ runtime: "bun", entry: "fixtures/quickjs-async.tsx", outfile });
    const child = Bun.spawn(["bun", "--preload", preload, outfile], { stdout: "pipe", stderr: "pipe" });
    const [status, bytes, stderr] = await Promise.all([
      child.exited,
      new Response(child.stdout).bytes(),
      new Response(child.stderr).text(),
    ]);
    expect(status, stderr).toBe(0);
    const { Envelope } = await import("../packages/solid-gpui/src/protocol/generated/protocol");
    const body = Envelope.decode(bytes.subarray(4)).body;
    expect(body?.tag).toBe(1);
    expect(body?.tag === 1 && body.value.nodes?.some((node) => node.text === "Async: passed")).toBe(true);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});
