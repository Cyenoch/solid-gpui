#!/usr/bin/env bun
/**
 * Build the upstream Solid compiler binding for Windows ARM64 and pack it as
 * @solid-gpui/solid-compiler-win32-arm64-msvc. Upstream publishes no win32-arm64
 * binding; the pinned @solidjs/compiler loads ours through its SOLID_COMPILER_NATIVE
 * override (no fork, no fallback).
 */

import { Command } from "commander";
import { createHash } from "node:crypto";
import { createReadStream, existsSync } from "node:fs";
import { copyFile, mkdir, open, readFile, readdir, stat, writeFile } from "node:fs/promises";
import { basename, dirname, isAbsolute, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");

/** Bump together with the @solidjs/compiler pin in packages/solid-gpui-vite/package.json. */
const COMPILER_VERSION = "2.0.0-rc.9";
/** Exact solidjs/solid revision; the published @solidjs/compiler gitHead for this version. */
const UPSTREAM_REVISION = "9a29b1a07aa3e06ee32afd1fc4c18414b4a558bb";
const UPSTREAM_REPOSITORY = "https://github.com/solidjs/solid";
const PACKAGE_NAME = "@solid-gpui/solid-compiler-win32-arm64-msvc";
const TARGET = "aarch64-pc-windows-msvc";
const BINARY_FILE = "compiler.win32-arm64-msvc.node";
const CDYLIB_ARTIFACT = "solidjs_compiler.dll";
const ARM64_PE_MACHINE = 0xaa64;

async function run(command: readonly string[], cwd: string, extraEnv: Record<string, string> = {}): Promise<void> {
  console.error(`\n$ ${command.join(" ")}`);
  const child = Bun.spawn([...command], {
    cwd,
    env: { ...process.env, ...extraEnv },
    stdio: ["inherit", "inherit", "inherit"],
  });
  const exitCode = await child.exited;
  if (exitCode !== 0 || child.signalCode !== null)
    throw new Error(`command failed with exit code ${exitCode}: ${command.join(" ")}`);
}

async function capture(command: readonly string[], cwd: string): Promise<string> {
  const child = Bun.spawn([...command], { cwd, stdout: "pipe", stderr: "pipe" });
  const [stdout, stderr, exitCode] = await Promise.all([
    new Response(child.stdout).text(),
    new Response(child.stderr).text(),
    child.exited,
  ]);
  if (exitCode !== 0)
    throw new Error(`command failed with exit code ${exitCode}: ${command.join(" ")}\n${(stderr || stdout).trim()}`);
  return stdout.trim();
}

async function sha256File(path: string): Promise<string> {
  const hash = createHash("sha256");
  for await (const chunk of createReadStream(path)) hash.update(chunk);
  return hash.digest("hex");
}

async function verifyPin(): Promise<void> {
  for (const manifestPath of [
    join(repoRoot, "packages/solid-gpui-vite/package.json"),
    join(repoRoot, "package.json"),
  ]) {
    const manifest = JSON.parse(await readFile(manifestPath, "utf8")) as {
      dependencies?: Record<string, string>;
      devDependencies?: Record<string, string>;
    };
    const pin = manifest.dependencies?.["@solidjs/compiler"] ?? manifest.devDependencies?.["@solidjs/compiler"];
    if (pin !== COMPILER_VERSION) {
      throw new Error(
        `${relative(repoRoot, manifestPath)} pins @solidjs/compiler ${pin ?? "(missing)"}, but this builder packages ` +
          `${COMPILER_VERSION}; bump the pin and COMPILER_VERSION together.`,
      );
    }
  }
}

function verifyHost(): void {
  if (process.platform !== "win32" || process.arch !== "arm64") {
    throw new Error(
      `${TARGET} builds natively on Windows ARM64 only; this machine is ${process.platform}-${process.arch}. ` +
        "Run the builder on the Windows ARM64 machine with the Rust MSVC toolchain and Visual Studio ARM64 build tools.",
    );
  }
}

async function verifySource(
  sourceDir: string,
): Promise<{ manifestPath: string; lockPath: string; licensePath: string }> {
  const compilerDir = join(sourceDir, "packages/compiler");
  const plan = {
    manifestPath: join(compilerDir, "Cargo.toml"),
    lockPath: join(compilerDir, "Cargo.lock"),
    licensePath: join(sourceDir, "LICENSE"),
  };
  for (const file of [
    plan.manifestPath,
    plan.lockPath,
    join(compilerDir, "package.json"),
    join(compilerDir, "index.js"),
    plan.licensePath,
  ]) {
    if (!existsSync(file)) {
      throw new Error(
        `upstream checkout is incomplete: missing ${relative(sourceDir, file)}; clone ${UPSTREAM_REPOSITORY} and check out ${UPSTREAM_REVISION}`,
      );
    }
  }
  const compiler = JSON.parse(await readFile(join(compilerDir, "package.json"), "utf8")) as { version: string };
  if (compiler.version !== COMPILER_VERSION) {
    throw new Error(
      `upstream packages/compiler is ${compiler.version}, expected ${COMPILER_VERSION}; the checkout does not match ${UPSTREAM_REVISION}`,
    );
  }
  if (!Bun.which("git")) throw new Error("git was not found on PATH; it verifies the source revision");
  let revision: string;
  try {
    revision = await capture(["git", "rev-parse", "HEAD"], sourceDir);
  } catch {
    throw new Error(
      `${sourceDir} is not a git checkout, so its revision cannot be verified; clone ${UPSTREAM_REPOSITORY} and check out ${UPSTREAM_REVISION}`,
    );
  }
  if (revision !== UPSTREAM_REVISION) {
    throw new Error(
      `source checkout is at ${revision}, expected ${UPSTREAM_REVISION}; run git -C ${sourceDir} checkout ${UPSTREAM_REVISION}`,
    );
  }
  const changes = await capture(["git", "status", "--porcelain", "--", "packages/compiler", "LICENSE"], sourceDir);
  if (changes) throw new Error(`compiler source must match the pinned revision without local changes:\n${changes}`);
  return plan;
}

/** Cheap structural check that the artifact is an ARM64 PE image. */
async function verifyArm64Image(binaryPath: string): Promise<void> {
  const handle = await open(binaryPath, "r");
  try {
    const header = Buffer.alloc(0x40);
    await handle.read(header, 0, header.length, 0);
    const peOffset = header.readUInt32LE(0x3c);
    if (peOffset <= 0 || peOffset + 6 > (await handle.stat()).size) throw new Error(`${binaryPath} is not a PE image`);
    const coff = Buffer.alloc(6);
    await handle.read(coff, 0, coff.length, peOffset);
    if (!coff.subarray(0, 4).equals(Buffer.from("PE\0\0", "latin1")))
      throw new Error(`${binaryPath} is not a PE image`);
    const machine = coff.readUInt16LE(4);
    if (machine !== ARM64_PE_MACHINE) {
      throw new Error(
        `${binaryPath} targets machine 0x${machine.toString(16)}, expected ARM64 (0xaa64); the cargo build used the wrong target`,
      );
    }
  } finally {
    await handle.close();
  }
}

/** Copy the license files of the shipped normal-dependency closure (target-filtered cargo metadata). */
async function packageDependencyLicenses(
  manifestPath: string,
  compilerDir: string,
  stagingDir: string,
  sourceRoot: string,
): Promise<{ crates: number; files: number; missing: number }> {
  const metadata = JSON.parse(
    await capture(
      [
        "cargo",
        "metadata",
        "--format-version",
        "1",
        "--locked",
        "--manifest-path",
        manifestPath,
        "--filter-platform",
        TARGET,
      ],
      compilerDir,
    ),
  ) as {
    resolve: { root: string; nodes: { id: string; deps: { pkg: string; dep_kinds: { kind: string | null }[] }[] }[] };
    packages: {
      id: string;
      name: string;
      version: string;
      license?: string;
      license_file?: string;
      manifest_path: string;
      source?: string | null;
    }[];
  };
  const byId = new Map(metadata.packages.map((pkg) => [pkg.id, pkg]));
  const shipped = new Set<string>([metadata.resolve.root]);
  for (const id of shipped) {
    const node = metadata.resolve.nodes.find((candidate) => candidate.id === id);
    for (const dep of node?.deps ?? []) {
      if (dep.dep_kinds.some((kind) => (kind.kind ?? "normal") === "normal") && !shipped.has(dep.pkg))
        shipped.add(dep.pkg);
    }
  }
  shipped.delete(metadata.resolve.root); // the upstream MIT LICENSE ships at the package root

  const licenseLike = (name: string): boolean => /^(license|licence|copying|copyright|notice|unlicense)/i.test(name);
  // Git checkouts inherit the repository root LICENSE, so the search may walk above
  // the crate; it stops at the dependency's own repository or cache root.
  const searchBound = (pkg: (typeof metadata.packages)[number]): string => {
    if (pkg.source?.startsWith("registry+")) return dirname(pkg.manifest_path);
    const segments = pkg.manifest_path.split(/[\\/]/);
    const checkouts = segments.lastIndexOf("checkouts");
    if (pkg.source?.startsWith("git+") && checkouts > 0) return join(...segments.slice(0, checkouts + 3));
    return sourceRoot;
  };

  const supplements: { source: string; packages: string[]; text: string }[] = JSON.parse(
    await readFile(join(repoRoot, "scripts/solid-compiler-licenses.json"), "utf8"),
  );
  await mkdir(join(stagingDir, "licenses"), { recursive: true });
  const entries: { name: string; version: string; license: string | null; files: string[]; missing?: boolean }[] = [];
  let files = 0;
  let missing = 0;
  for (const id of [...shipped].sort()) {
    const pkg = byId.get(id);
    if (!pkg) throw new Error(`cargo metadata listed ${id} without package details`);
    const licenseDir = join(stagingDir, "licenses", `${pkg.name}-${pkg.version}`);
    const copied: string[] = [];
    const collect = async (from: string): Promise<void> => {
      await mkdir(licenseDir, { recursive: true });
      await copyFile(from, join(licenseDir, basename(from)));
      copied.push(`${pkg.name}-${pkg.version}/${basename(from)}`);
    };
    const declared = pkg.license_file
      ? isAbsolute(pkg.license_file)
        ? pkg.license_file
        : join(dirname(pkg.manifest_path), pkg.license_file)
      : null;
    if (declared && existsSync(declared) && (await stat(declared)).isFile()) {
      await collect(declared);
    } else {
      let dir = dirname(pkg.manifest_path);
      const bound = searchBound(pkg);
      const within = (candidate: string): boolean => {
        const rel = relative(bound, candidate);
        return rel === "" || (!rel.startsWith("..") && !isAbsolute(rel));
      };
      while (within(dir) && copied.length === 0) {
        for (const name of (await readdir(dir)).sort()) {
          const file = join(dir, name);
          if (licenseLike(name) && (await stat(file)).isFile()) await collect(file);
        }
        const parent = dirname(dir);
        if (parent === dir) break;
        dir = parent;
      }
    }
    if (copied.length === 0) {
      const notice = supplements.find((entry) => entry.packages.includes(`${pkg.name}@${pkg.version}`));
      if (!notice) throw new Error(`No license notice found for ${pkg.name}@${pkg.version}`);
      await mkdir(licenseDir, { recursive: true });
      await writeFile(join(licenseDir, "LICENSE"), `Source: ${notice.source}\n\n${notice.text}\n`);
      copied.push(`${pkg.name}-${pkg.version}/LICENSE`);
    }
    entries.push({ name: pkg.name, version: pkg.version, license: pkg.license ?? null, files: copied });
    files += copied.length;
  }
  await writeFile(join(stagingDir, "licenses", "index.json"), `${JSON.stringify(entries, null, 2)}\n`);
  return { crates: entries.length, files, missing };
}

async function main(): Promise<void> {
  const program = new Command()
    .name("bun scripts/build-solid-compiler.ts")
    .description(
      `Build @solidjs/compiler ${COMPILER_VERSION} (solidjs/solid ${UPSTREAM_REVISION}) for ${TARGET} and pack it as ${PACKAGE_NAME}`,
    )
    .requiredOption("--source <path>", "git checkout of solidjs/solid at the pinned revision")
    .requiredOption("--output <path>", "directory receiving the packed tgz, unpacked package and provenance")
    .showHelpAfterError("(run with --help for usage and prerequisites)")
    .addHelpText(
      "after",
      `
Prerequisites: native Windows ARM64 host; Rust MSVC toolchain (rustc >= 1.95) and cargo on PATH;
Visual Studio ARM64 build tools and the Windows SDK (the binding dynamically links the ARM64 VC++
runtime, vcruntime140.dll); git and bun on PATH.
Output: <output>/package/ (unpacked package) and <output>/*.tgz (bun pm pack); cargo artifacts stay
in the checkout's packages/compiler/target and are reused across runs.
Consumption: set SOLID_COMPILER_NATIVE to the absolute path of ${BINARY_FILE}.`,
    );
  program.parse(Bun.argv);
  const { source: rawSource, output } = program.opts<{ source: string; output: string }>();
  const sourceDir = resolve(rawSource);
  const outputDir = resolve(output);

  await verifyPin();
  verifyHost();
  const source = await verifySource(sourceDir);

  if (!Bun.which("cargo"))
    throw new Error("cargo was not found on PATH; install Rust via rustup with the MSVC toolchain");
  const [cargo, rustc] = await Promise.all([
    capture(["cargo", "--version"], repoRoot),
    capture(["rustc", "--version"], repoRoot),
  ]);
  // Builds in the checkout's own packages/compiler/target so prior native builds are reused.
  await run(
    [
      "cargo",
      "build",
      "--release",
      "--locked",
      "--target",
      TARGET,
      "--target-dir",
      join(sourceDir, "packages/compiler/target"),
      "--manifest-path",
      source.manifestPath,
    ],
    join(sourceDir, "packages/compiler"),
    // CARGO_PROFILE_RELEASE_STRIP reproduces upstream `napi build --strip`.
    { CARGO_PROFILE_RELEASE_STRIP: "symbols" },
  );

  const binaryArtifact = join(sourceDir, "packages/compiler/target", TARGET, "release", CDYLIB_ARTIFACT);
  if (!existsSync(binaryArtifact)) throw new Error(`cargo produced no ${CDYLIB_ARTIFACT} at ${binaryArtifact}`);
  await verifyArm64Image(binaryArtifact);
  const binarySha256 = await sha256File(binaryArtifact);

  // Stage the napi platform-package shape plus MIT LICENSE, provenance, locked cargo inputs and dependency licenses.
  const stagingDir = join(outputDir, "package");
  if (existsSync(stagingDir)) throw new Error(`${stagingDir} already exists; remove it or choose a different --output`);
  await mkdir(join(stagingDir, "cargo"), { recursive: true });
  await copyFile(binaryArtifact, join(stagingDir, BINARY_FILE));
  await copyFile(source.licensePath, join(stagingDir, "LICENSE"));
  await copyFile(source.manifestPath, join(stagingDir, "cargo/Cargo.toml"));
  await copyFile(source.lockPath, join(stagingDir, "cargo/Cargo.lock"));
  const licenses = await packageDependencyLicenses(
    source.manifestPath,
    join(sourceDir, "packages/compiler"),
    stagingDir,
    sourceDir,
  );
  await writeFile(
    join(stagingDir, "package.json"),
    `${JSON.stringify(
      {
        name: PACKAGE_NAME,
        version: COMPILER_VERSION,
        cpu: ["arm64"],
        main: BINARY_FILE,
        files: [BINARY_FILE, "LICENSE", "provenance.json", "cargo", "licenses"],
        description: "Solid native Oxc JSX compiler (win32-arm64-msvc)",
        author: "Ryan Carniato",
        license: "MIT",
        repository: { type: "git", url: "git+https://github.com/solidjs/solid.git", directory: "packages/compiler" },
        os: ["win32"],
      },
      null,
      2,
    )}\n`,
  );
  await writeFile(
    join(stagingDir, "provenance.json"),
    `${JSON.stringify(
      {
        packageName: PACKAGE_NAME,
        version: COMPILER_VERSION,
        upstream: {
          repository: UPSTREAM_REPOSITORY,
          revision: UPSTREAM_REVISION,
          directory: "packages/compiler",
          compilerVersion: COMPILER_VERSION,
          license: "MIT",
          licenseFile: "LICENSE",
        },
        target: TARGET,
        binary: { file: BINARY_FILE, sha256: binarySha256 },
        cargoLock: { file: "cargo/Cargo.lock", sha256: await sha256File(source.lockPath) },
        dependencyLicenses: { manifest: "licenses/index.json", ...licenses },
        build: {
          cargo,
          rustc,
          host: `${process.platform}-${process.arch}`,
          flags: `cargo build --release --locked --target ${TARGET} (checkout packages/compiler/target, reused across runs)`,
          features: "default (node, tsrx)",
          locked: true,
          strip: "symbols (CARGO_PROFILE_RELEASE_STRIP, matching upstream napi --strip)",
          runtime:
            "dynamically links the ARM64 VC++ runtime (vcruntime140.dll); target machines need the Visual C++ ARM64 redistributable",
        },
        loader: "SOLID_COMPILER_NATIVE must point at the binary",
        packagedAt: new Date().toISOString(),
      },
      null,
      2,
    )}\n`,
  );

  const archivePath = join(outputDir, `${PACKAGE_NAME.replace(/^@/, "").replace("/", "-")}-${COMPILER_VERSION}.tgz`);
  await run(["bun", "pm", "pack", "--filename", archivePath, "--quiet"], stagingDir);
  const packed = await new Bun.Archive(await Bun.file(archivePath).bytes()).files();
  for (const entry of [
    "package/package.json",
    `package/${BINARY_FILE}`,
    "package/LICENSE",
    "package/provenance.json",
    "package/cargo/Cargo.lock",
    "package/licenses/index.json",
  ]) {
    if (!packed.has(entry)) throw new Error(`packed archive is missing ${entry}`);
  }

  console.log(
    JSON.stringify(
      {
        package: PACKAGE_NAME,
        version: COMPILER_VERSION,
        revision: UPSTREAM_REVISION,
        target: TARGET,
        binary: { file: BINARY_FILE, sha256: binarySha256 },
        archive: { file: archivePath, sha256: await sha256File(archivePath) },
        licenses: { crates: licenses.crates, files: licenses.files, missing: licenses.missing },
        unpacked: stagingDir,
      },
      null,
      2,
    ),
  );
}

if (import.meta.main)
  await main().catch((error: Error) => {
    console.error(error instanceof Error ? error.message : error);
    process.exitCode = 1;
  });
