import { expect, test } from "bun:test";
import { mkdtemp, mkdir, readFile, rm, stat, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { build } from "vite";
import { exportNativeBindings } from "../packages/solid-gpui-vite/src/native-export";
import { solidGpui } from "../packages/solid-gpui-vite/src";

test("Vite exports an actual Rust host before resolving #native, preserves stable outputs and rejects stale/failed generation", async () => {
  const directory = await mkdtemp(join(tmpdir(), "solid-native-export-"));
  const manifestPath = join(directory, "Cargo.toml");
  const source = join(directory, "src/main.rs");
  const output = join(directory, ".generated/native.ts");
  try {
    await mkdir(join(directory, "src"));
    await writeFile(
      manifestPath,
      '[package]\nname = "native-export-fixture"\nversion = "0.0.0"\nedition = "2024"\n[workspace]\n',
    );
    await writeFile(
      source,
      'fn main() { assert_eq!(std::env::args().nth(1).as_deref(), Some("--export-native")); println!("export const answer=42;"); }',
    );
    await writeFile(join(directory, "app.js"), 'import { answer } from "#native"; console.log(answer);');
    await writeFile(join(directory, "package.json"), '{"type":"module"}');
    await build({
      configFile: false,
      root: directory,
      logLevel: "silent",
      plugins: [solidGpui({ entry: "app.js", native: { manifestPath } })],
    });
    const child = Bun.spawn(["bun", join(directory, "dist/app.js")], { stdout: "pipe", stderr: "pipe" });
    const [code, stdout, stderr] = await Promise.all([
      child.exited,
      new Response(child.stdout).text(),
      new Response(child.stderr).text(),
    ]);
    expect(code, stderr).toBe(0);
    expect(stdout.trim()).toBe("42");
    expect(await readFile(output, "utf8")).toBe("export const answer = 42;\n");
    const before = await stat(output);
    await exportNativeBindings({ manifestPath, output });
    expect((await stat(output)).ino).toBe(before.ino);
    await exportNativeBindings({ manifestPath, output, check: true });
    await writeFile(output, "export const stale = true;\n");
    await expect(exportNativeBindings({ manifestPath, output, check: true })).rejects.toThrow("stale or missing");
    expect(await readFile(output, "utf8")).toContain("stale");
    await writeFile(source, 'fn main() { println!("export const =;"); }');
    await expect(exportNativeBindings({ manifestPath, output })).rejects.toThrow("Failed to format native bindings");
    expect(await readFile(output, "utf8")).toBe("export const stale = true;\n");
    await writeFile(source, "fn main() { std::process::exit(7); }");
    await expect(exportNativeBindings({ manifestPath, output })).rejects.toThrow();
    expect(await readFile(output, "utf8")).toContain("stale");
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
}, 30000);
