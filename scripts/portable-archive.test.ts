import { expect, test } from "bun:test";
import { chmod, mkdir, mkdtemp, rm, stat } from "node:fs/promises";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { writePortableArchive } from "../packages/solid-gpui-vite/src/portable-archive.ts";

test("portable archives preserve executables, empty resource directories and long Unicode paths", async () => {
  const root = await mkdtemp(join(tmpdir(), "solid-portable-archive-"));
  try {
    const stage = join(root, "stage");
    await mkdir(join(stage, "empty"), { recursive: true });
    const executable = join(stage, "host");
    await Bun.write(executable, "#!/bin/sh\nexit 0\n");
    await chmod(executable, 0o755);
    const name = "long-" + "🙂".repeat(40) + ".txt";
    await Bun.write(join(stage, name), "新值🙂\n");
    const archive = join(root, "app.tar.gz");
    await writePortableArchive(stage, "application", archive);
    const bytes = await Bun.file(archive).bytes();
    expect(Array.from(bytes.subarray(0, 2))).toEqual([31, 139]);
    await new Bun.Archive(bytes).extract(join(root, "extracted"));
    const app = join(root, "extracted/application");
    expect(await Bun.file(join(app, name)).text()).toBe("新值🙂\n");
    expect((await stat(join(app, "empty"))).isDirectory()).toBe(true);
    if (process.platform !== "win32") expect((await stat(join(app, "host"))).mode & 0o777).toBe(0o755);
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});
