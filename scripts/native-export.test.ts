import { expect, test } from "bun:test";
import { mkdtemp, mkdir, readFile, rm, stat, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { exportNativeBindings } from "../packages/solid-gpui/src/vite/native-export";
import { solidGpui } from "../packages/solid-gpui/src/vite";

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
    const hook = solidGpui({ entry: "app.tsx", native: { manifestPath } }).config;
    if (typeof hook !== "function") throw new Error("expected Vite config hook");
    const config = await Reflect.apply(hook, {}, [{ root: directory }, { command: "build", mode: "production" }]);
    expect(config.resolve.alias).toEqual([{ find: "#native", replacement: output }]);
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
