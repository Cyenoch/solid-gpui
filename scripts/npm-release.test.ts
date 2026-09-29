import { expect, test } from "bun:test";
import { mkdtemp, mkdir, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { checkRelease, publicationState, releasePackages, releaseVersion } from "./npm-release";

test("release tags must match every committed package, Rust version, peer and changelog", async () => {
  const root = await mkdtemp(join(tmpdir(), "solid-npm-release-"));
  try {
    await Bun.write(join(root, "Cargo.toml"), '[workspace.package]\nversion = "1.2.3"\n');
    await Bun.write(join(root, "CHANGELOG.md"), "# Changelog\n\n## [1.2.3]\n");
    for (const folder of releasePackages) {
      await mkdir(join(root, "packages", folder), { recursive: true });
      await Bun.write(
        join(root, "packages", folder, "package.json"),
        JSON.stringify({
          name: `@solid-gpui/${folder === "solid-gpui" ? "core" : folder.slice("solid-gpui-".length)}`,
          version: "1.2.3",
          publishConfig: { access: "public", registry: "https://registry.npmjs.org/" },
          peerDependencies: { "@solid-gpui/core": "^1.2.3" },
        }),
      );
    }
    expect(await checkRelease(root, "v1.2.3")).toBe("1.2.3");
    await expect(checkRelease(root, "v1.2.4")).rejects.toThrow("Rust workspace");
    const path = join(root, "packages/solid-gpui-shiki/package.json");
    const manifest = await Bun.file(path).json();
    await Bun.write(path, JSON.stringify({ ...manifest, version: "1.2.2" }));
    await expect(checkRelease(root, "v1.2.3")).rejects.toThrow("solid-gpui-shiki");
    await Bun.write(path, JSON.stringify({ ...manifest, peerDependencies: { "@solid-gpui/core": "^1.2.2" } }));
    await expect(checkRelease(root, "v1.2.3")).rejects.toThrow("require core");
    await Bun.write(path, JSON.stringify(manifest));
    await Bun.write(join(root, "CHANGELOG.md"), "# Unreleased\n");
    await expect(checkRelease(root, "v1.2.3")).rejects.toThrow("CHANGELOG");
    for (const tag of ["main", "v01.2.3", "v1.2.3-beta.1", "v0.0.0", "v1.2.3\n"])
      expect(() => releaseVersion(tag)).toThrow();
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});

test("publication retries skip only identical bytes and never treat registry failures as unpublished", async () => {
  expect(await publicationState(new Response(null, { status: 404 }), "sha512-a")).toBe("missing");
  expect(await publicationState(Response.json({ dist: { integrity: "sha512-a" } }), "sha512-a")).toBe("published");
  await expect(publicationState(Response.json({ dist: { integrity: "sha512-b" } }), "sha512-a")).rejects.toThrow(
    "different bytes",
  );
  for (const status of [401, 403, 429, 500])
    await expect(publicationState(new Response(null, { status }), "sha512-a")).rejects.toThrow(`HTTP ${status}`);
});
