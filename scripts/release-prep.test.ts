import { expect, test } from "bun:test";
import { chmod, copyFile, mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";

const releaseFiles = {
  "Cargo.toml": '[workspace.package]\nversion = "0.2.0"\n',
  "packages/solid-gpui/package.json": '{\n  "version": "0.2.0"\n}\n',
  "packages/solid-gpui-router/package.json":
    '{\n  "version": "0.2.0",\n  "peerDependencies": {\n    "@solid-gpui/core": "^0.2.0"\n  }\n}\n',
  "Cargo.lock": "original cargo lock\n",
  "bun.lock": "original bun lock\n",
  "THIRD-PARTY-NOTICES.md": "original notices\n",
};

async function withReleaseFixture(run: (directory: string) => Promise<void>): Promise<void> {
  const directory = await mkdtemp(join(tmpdir(), "solid-gpui-release-test-"));
  try {
    for (const [file, source] of Object.entries(releaseFiles)) {
      await mkdir(dirname(join(directory, file)), { recursive: true });
      await writeFile(join(directory, file), source);
    }
    await mkdir(join(directory, "scripts"));
    await copyFile(join(import.meta.dir, "release-prep.sh"), join(directory, "scripts/release-prep.sh"));
    await writeFile(join(directory, "CHANGELOG.md"), "# Changelog\n\n## [0.2.0]\n\n## [0.3.0]\n");
    await mkdir(join(directory, "bin"));
    // Exercise the actual release transaction while keeping package resolution
    // and compiler failures deterministic and isolated from this checkout.
    const commands = {
      cargo: '#!/bin/sh\nif [ "$1" = update ]; then echo updated > Cargo.lock; fi\n',
      bun: '#!/bin/sh\nif [ "$2" != --frozen-lockfile ]; then echo updated > bun.lock; fi\n',
      git: "#!/bin/sh\nexit 0\n",
    };
    for (const [name, source] of Object.entries(commands)) {
      await writeFile(join(directory, "bin", name), source);
      await chmod(join(directory, "bin", name), 0o755);
    }
    await writeFile(join(directory, "scripts/third-party-notices.sh"), "echo updated > THIRD-PARTY-NOTICES.md\n");
    await run(directory);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
}

async function prepare(directory: string, version: string): Promise<{ code: number; output: string }> {
  const child = Bun.spawn(["bash", "scripts/release-prep.sh", version], {
    cwd: directory,
    env: { ...process.env, PATH: `${join(directory, "bin")}:${process.env.PATH}` },
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

test("an already synchronized release still requires its changelog and frozen locks", async () => {
  await withReleaseFixture(async (directory) => {
    await writeFile(join(directory, "CHANGELOG.md"), "# Changelog\n");
    const missingChangelog = await prepare(directory, "0.2.0");
    expect(missingChangelog.code).toBe(1);
    expect(missingChangelog.output).toContain("has no ## [0.2.0] section");

    await writeFile(join(directory, "CHANGELOG.md"), "## [0.2.0]\n");
    await writeFile(join(directory, "bin/bun"), '#!/bin/sh\necho "stale frozen lock" >&2\nexit 9\n');
    const staleLock = await prepare(directory, "0.2.0");
    expect(staleLock.code).toBe(9);
    expect(staleLock.output).toContain("stale frozen lock");
  });
});

test("a failed final release step restores every manifest, lock and notices file", async () => {
  await withReleaseFixture(async (directory) => {
    await writeFile(
      join(directory, "scripts/third-party-notices.sh"),
      'echo partial > THIRD-PARTY-NOTICES.md\necho "notice generation failed" >&2\nexit 7\n',
    );
    const result = await prepare(directory, "0.3.0");
    expect(result.code).toBe(7);
    expect(result.output).toContain("notice generation failed");
    for (const [file, source] of Object.entries(releaseFiles)) {
      expect(await readFile(join(directory, file), "utf8")).toBe(source);
    }
  });
});

test("a successful release synchronizes package versions, the router peer and generated files", async () => {
  await withReleaseFixture(async (directory) => {
    const result = await prepare(directory, "0.3.0");
    expect(result.code, result.output).toBe(0);
    expect(await readFile(join(directory, "Cargo.toml"), "utf8")).toContain('version = "0.3.0"');
    const core = await Bun.file(join(directory, "packages/solid-gpui/package.json")).json();
    const router = await Bun.file(join(directory, "packages/solid-gpui-router/package.json")).json();
    expect(core.version).toBe("0.3.0");
    expect(router.version).toBe("0.3.0");
    expect(router.peerDependencies["@solid-gpui/core"]).toBe("^0.3.0");
    for (const file of ["Cargo.lock", "bun.lock", "THIRD-PARTY-NOTICES.md"]) {
      expect(await readFile(join(directory, file), "utf8")).toBe("updated\n");
    }
  });
});
