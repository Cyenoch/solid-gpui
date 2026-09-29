import { createHash } from "node:crypto";
import { resolve, join } from "node:path";

export const releasePackages = ["solid-gpui", "solid-gpui-vite", "solid-gpui-router", "solid-gpui-shiki"] as const;
const registry = "https://registry.npmjs.org";

export function releaseVersion(tag: string): string {
  if (tag !== tag.trim() || !/^v(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/.test(tag) || tag === "v0.0.0")
    throw new Error("npm release requires a vMAJOR.MINOR.PATCH tag (not v0.0.0)");
  return tag.slice(1);
}

/** Validate committed release inputs without changing versions or lockfiles in CI. */
export async function checkRelease(root: string, tag: string): Promise<string> {
  const version = releaseVersion(tag);
  const cargo = Bun.TOML.parse(await Bun.file(join(root, "Cargo.toml")).text()) as {
    workspace: { package: { version: string } };
  };
  if (cargo.workspace.package.version !== version) throw new Error(`Rust workspace does not match ${tag}`);
  const changelog = await Bun.file(join(root, "CHANGELOG.md")).text();
  if (!changelog.split(/\r?\n/).includes(`## [${version}]`)) throw new Error(`CHANGELOG.md needs ## [${version}]`);
  for (const directory of releasePackages) {
    const manifest = await Bun.file(join(root, "packages", directory, "package.json")).json();
    const expectedName = `@solid-gpui/${directory === "solid-gpui" ? "core" : directory.slice("solid-gpui-".length)}`;
    if (manifest.name !== expectedName || manifest.version !== version || manifest.private === true)
      throw new Error(`${directory} must be public ${expectedName}@${version}`);
    if (manifest.publishConfig?.access !== "public" || manifest.publishConfig?.registry !== `${registry}/`)
      throw new Error(`${expectedName} must publish publicly to npm`);
    if (directory !== "solid-gpui" && manifest.peerDependencies?.["@solid-gpui/core"] !== `^${version}`)
      throw new Error(`${expectedName} must require core ^${version}`);
  }
  return version;
}

/** Only an explicit 404 means unpublished; outages and authentication errors stop the release. */
export async function publicationState(response: Response, integrity: string): Promise<"missing" | "published"> {
  if (response.status === 404) return "missing";
  if (!response.ok) throw new Error(`npm registry lookup failed: HTTP ${response.status}`);
  const metadata = (await response.json()) as { dist?: { integrity?: string } };
  if (metadata.dist?.integrity !== integrity) throw new Error("npm already contains different bytes for this version");
  return "published";
}

async function publish(root: string, tag: string, directory: string): Promise<void> {
  const version = await checkRelease(root, tag);
  // Inspect all four verified archives before publishing any package.
  const candidates = await Promise.all(
    releasePackages.map(async (folder) => {
      const manifest = await Bun.file(join(root, "packages", folder, "package.json")).json();
      const filename = `${manifest.name.replace("@", "").replace("/", "-")}-${version}.tgz`;
      const path = resolve(directory, filename);
      const bytes = await Bun.file(path).bytes();
      const files = await new Bun.Archive(bytes).files();
      const packed = files.get("package/package.json");
      if (!packed) throw new Error(`${filename} has no package.json`);
      const identity = JSON.parse(await packed.text());
      if (identity.name !== manifest.name || identity.version !== version)
        throw new Error(`${filename} does not match ${manifest.name}@${version}`);
      const integrity = `sha512-${createHash("sha512").update(bytes).digest("base64")}`;
      const url = `${registry}/${encodeURIComponent(manifest.name)}/${version}`;
      const state = await publicationState(await fetch(url), integrity);
      return { name: manifest.name, path, state };
    }),
  );
  for (const candidate of candidates) {
    if (candidate.state === "published") {
      console.log(`${candidate.name}@${version}: identical archive already published`);
      continue;
    }
    const child = Bun.spawn(
      [
        "npm",
        "publish",
        candidate.path,
        "--access",
        "public",
        "--tag",
        "latest",
        "--provenance",
        "--ignore-scripts",
        "--registry",
        registry,
      ],
      { cwd: root, stdio: ["inherit", "inherit", "inherit"] },
    );
    if ((await child.exited) !== 0) throw new Error(`npm publish failed for ${candidate.name}`);
  }
}

if (import.meta.main) {
  const [command, tag, directory] = process.argv.slice(2);
  const root = resolve(import.meta.dirname, "..");
  if (!tag || (command !== "check" && command !== "publish") || (command === "publish" && !directory))
    throw new Error("Usage: bun scripts/npm-release.ts check <tag> | publish <tag> <archive-directory>");
  if (command === "check") console.log(`Validated npm release ${await checkRelease(root, tag)}`);
  else await publish(root, tag, directory!);
}
