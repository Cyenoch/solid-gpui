#!/usr/bin/env bun
import { createHash, type Hash } from "node:crypto";
import { chmod, copyFile, cp, mkdir, mkdtemp, readFile, readdir, rename, rm, stat, writeFile } from "node:fs/promises";
import { basename, dirname, isAbsolute, join, resolve } from "node:path";
import { parseArgs } from "node:util";
import pin from "../crates/solid-gpui-bun-sys/bun-build.json";
import { extractGraphPayload, packageEmbeddedGraph, type EmbeddedGraphTarget } from "./bun-embedded-bundle";

const root = resolve(import.meta.dirname, "..");
const sys = join(root, "crates/solid-gpui-bun-sys");

interface NativeManifest {
  schemaVersion: number;
  target: string;
  codegenDir: string;
  objects: string[];
  archives: string[];
  linkArgs: string[];
  rustFlags: string[];
  cargoArgs: string[];
  environment: Record<string, string>;
  cargoProfile: string;
  bunRevision: string;
  webkitMode: string;
}

interface CargoWorkspace {
  patch?: Record<string, Record<string, { path?: string; [key: string]: unknown }>>;
  profile?: Record<string, unknown>;
}

async function run(args: string[], cwd: string, extra: Record<string, string> = {}, capture = false): Promise<string> {
  console.error(`$ ${args.map((arg) => JSON.stringify(arg)).join(" ")}`);
  const env = { ...process.env, ...extra };
  for (const key of [
    "RUSTC",
    "RUSTC_WRAPPER",
    "RUSTC_WORKSPACE_WRAPPER",
    "CLIPPY_ARGS",
    "RUSTFLAGS",
    "CARGO_ENCODED_RUSTFLAGS",
    "CARGO_MAKEFLAGS",
    "SOLID_GPUI_BUN_CHECK_ONLY",
  ]) {
    if (!(key in extra)) delete env[key];
  }
  const child = Bun.spawn(args, { cwd, env, stdin: "ignore", stdout: capture ? "pipe" : "inherit", stderr: "inherit" });
  const [output, status] = await Promise.all([capture ? new Response(child.stdout).text() : "", child.exited]);
  if (status !== 0) throw new Error(`${args[0]} failed with exit code ${status}`);
  return output.trim();
}

async function exists(path: string): Promise<boolean> {
  try {
    await stat(path);
    return true;
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === "ENOENT") return false;
    throw error;
  }
}

async function digestTree(path: string, hash: Hash): Promise<void> {
  for (const entry of (await readdir(path, { withFileTypes: true })).sort((a, b) => a.name.localeCompare(b.name))) {
    hash.update(entry.name);
    const child = join(path, entry.name);
    if (entry.isDirectory()) await digestTree(child, hash);
    else hash.update(await readFile(child));
  }
}

async function prepareSource(cache: string, localSource?: string): Promise<string> {
  const hash = createHash("sha256")
    .update(JSON.stringify(pin))
    .update(await readFile(join(sys, "bun_embed.patch")));
  await digestTree(join(sys, "embedded"), hash);
  const source = join(cache, `source-${hash.digest("hex").slice(0, 24)}`);
  if (!(await exists(join(source, ".solid-gpui-ready")))) {
    const staging = await mkdtemp(join(cache, "source-preparing-"));
    try {
      await run(
        [
          "git",
          "clone",
          "--no-checkout",
          ...(localSource ? ["--shared", resolve(localSource)] : ["--filter=blob:none", pin.repository]),
          staging,
        ],
        root,
      );
      if (!localSource) await run(["git", "fetch", "--depth=1", "origin", pin.revision], staging);
      await run(["git", "checkout", "--detach", pin.revision], staging);
      await run(["git", "apply", "--check", join(sys, "bun_embed.patch")], staging);
      await run(["git", "apply", join(sys, "bun_embed.patch")], staging);
      await mkdir(join(staging, "src/runtime/embedded"), { recursive: true });
      await copyFile(join(sys, "embedded/runtime.rs"), join(staging, "src/runtime/embedded.rs"));
      await copyFile(join(sys, "embedded/lifecycle.rs"), join(staging, "src/runtime/embedded/lifecycle.rs"));
      await cp(join(sys, "embedded/build"), join(staging, "scripts/build"), { recursive: true });
      await writeFile(join(staging, ".solid-gpui-ready"), pin.revision);
      await rename(staging, source);
    } catch (error) {
      await rm(staging, { recursive: true, force: true });
      throw error;
    }
  }
  const actual = await run(["git", "rev-parse", "HEAD"], source, {}, true);
  if (actual !== pin.revision) throw new Error(`Bun checkout drifted: ${actual}`);
  return source;
}

function validateManifest(value: NativeManifest, target: string, profile: string): void {
  if (value.schemaVersion !== 1 || value.target !== target)
    throw new Error("Native Bun manifest schema or target mismatch");
  if (value.bunRevision !== pin.revision)
    throw new Error(`Native Bun manifest was produced from Bun ${value.bunRevision}, not the pinned ${pin.revision}`);
  if (value.webkitMode !== "prebuilt")
    throw new Error("Native Bun manifest must use the pinned prebuilt WebKit product");
  if (value.cargoProfile !== (profile === "release" ? "release" : "dev")) {
    throw new Error(
      `Native Bun manifest carries cargo profile ${value.cargoProfile}, not the profile of --profile ${profile}`,
    );
  }
  for (const key of ["objects", "archives", "linkArgs", "rustFlags", "cargoArgs"] as const) {
    if (!Array.isArray(value[key]) || value[key].some((item) => typeof item !== "string"))
      throw new Error(`Invalid native manifest ${key}`);
  }
  if (!isAbsolute(value.codegenDir) || [...value.objects, ...value.archives].some((path) => !isAbsolute(path)))
    throw new Error("Native manifest paths must be absolute");
  if (value.archives.some((path) => /(?:lib)?bun_rust\.(?:a|lib)$/.test(path)))
    throw new Error("A separate Bun Rust staticlib cannot enter the application link");
}

async function prepareApplication(
  directory: string,
  source: string,
  manifestPath: string,
  graphSource: string,
  main?: string,
): Promise<void> {
  const host = Bun.TOML.parse(await readFile(join(root, "Cargo.toml"), "utf8")) as CargoWorkspace;
  const bun = Bun.TOML.parse(await readFile(join(source, "Cargo.toml"), "utf8")) as CargoWorkspace;
  const patches = structuredClone(host.patch ?? {});
  for (const entries of Object.values(patches)) {
    for (const dependency of Object.values(entries)) {
      if (dependency.path) dependency.path = resolve(root, dependency.path);
    }
  }
  const document = {
    workspace: { resolver: "2" },
    package: { name: "solid-gpui-embedded-app", version: "0.1.0", edition: "2024", publish: false },
    dependencies: {
      solid_gpui: { package: "solid-gpui", path: join(root, "crates/solid-gpui"), features: ["embedded-bun"] },
      bun_rust: { package: "bun_bin", path: join(source, "src/bun_bin"), features: ["solid-gpui-embed"] },
    },
    profile: bun.profile,
    patch: patches,
  };
  await mkdir(join(directory, "src"), { recursive: true });
  const cargoToml = Bun.TOML.stringify(document);
  if (!cargoToml) throw new Error("Failed to serialize application Cargo manifest");
  await writeFile(join(directory, "Cargo.toml"), cargoToml);
  await writeFile(
    join(directory, "src/main.rs"),
    [
      '#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]',
      "extern crate bun_rust;",
      `include!(${JSON.stringify(graphSource)});`,
      `include!(${JSON.stringify(main ? resolve(main) : join(sys, "static-application.rs"))});`,
      "",
    ].join("\n"),
  );
  const manifest: NativeManifest = JSON.parse(await readFile(manifestPath, "utf8"));
  const args = [...manifest.objects, ...manifest.archives, ...manifest.linkArgs];
  await writeFile(
    join(directory, "build.rs"),
    [
      "fn main() {",
      `    println!("cargo:rerun-if-changed={}", ${JSON.stringify(manifestPath)});`,
      ...args.map((arg) => `    println!("cargo:rustc-link-arg={}", ${JSON.stringify(arg)});`),
      ...[...manifest.objects, ...manifest.archives].map(
        (path) => `    println!("cargo:rerun-if-changed={}", ${JSON.stringify(path)});`,
      ),
      "}",
      "",
    ].join("\n"),
  );
}

async function main(): Promise<void> {
  const { values } = parseArgs({
    options: {
      entry: { type: "string" },
      output: { type: "string" },
      target: { type: "string" },
      profile: { type: "string", default: "release" },
      cache: { type: "string" },
      source: { type: "string" },
      bun: { type: "string" },
      ninja: { type: "string" },
      main: { type: "string" },
      "base-executable": { type: "string" },
      assets: { type: "string", multiple: true },
      workers: { type: "string", multiple: true },
      "macos-sdk": { type: "string" },
      "deployment-target": { type: "string" },
      winsysroot: { type: "string" },
      "prepare-only": { type: "boolean", default: false },
      help: { type: "boolean", default: false },
    },
  });
  if (values.help) {
    console.log(
      "Usage: bun scripts/bun-static-package.ts --entry <Vite JS> --bun <pinned Bun executable> --output <application> [--target <Rust triple>] [--profile debug|release] [--source <pinned checkout>] [--main <Rust main>] [--assets <path>] [--workers <entry>] [--base-executable <target Bun>] [--macos-sdk <path>] [--deployment-target <version>] [--winsysroot <path>] [--prepare-only]",
    );
    return;
  }
  if (!values.entry || !values.output || !values.bun)
    throw new Error("--entry, --output and --bun (pinned serializer) are required");
  if (values.profile !== "debug" && values.profile !== "release") throw new Error("--profile must be debug or release");
  const rustc = await run(["rustup", "run", pin.toolchain, "rustc", "-vV"], root, {}, true);
  const target = values.target ?? /^host: (.+)$/m.exec(rustc)?.[1];
  const targets: Record<string, { os: string; arch: string; abi?: string; graph?: EmbeddedGraphTarget }> = {
    "aarch64-apple-darwin": { os: "darwin", arch: "aarch64", graph: "bun-darwin-arm64" },
    "x86_64-apple-darwin": { os: "darwin", arch: "x64", graph: "bun-darwin-x64" },
    "x86_64-pc-windows-msvc": { os: "windows", arch: "x64", graph: "bun-windows-x64" },
    "aarch64-pc-windows-msvc": { os: "windows", arch: "aarch64", graph: "bun-windows-arm64" },
    "x86_64-unknown-linux-gnu": { os: "linux", arch: "x64", abi: "gnu" },
    "aarch64-unknown-linux-gnu": { os: "linux", arch: "aarch64", abi: "gnu" },
    "x86_64-unknown-linux-musl": { os: "linux", arch: "x64", abi: "musl" },
    "aarch64-unknown-linux-musl": { os: "linux", arch: "aarch64", abi: "musl" },
  };
  if (!target || !targets[target]) throw new Error(`Unsupported static application target: ${target}`);
  const selected = targets[target]!;
  if (!values["prepare-only"] && !selected.graph)
    throw new Error(
      "ELF application graph transport is not implemented; Linux native preparation requires --prepare-only",
    );
  const ninja = values.ninja ?? Bun.which("ninja");
  if (!ninja) throw new Error(`Ninja ${pin.ninjaVersion} is required; install it or pass --ninja`);
  const cache = resolve(values.cache ?? join(root, "target/bun-static"));
  await mkdir(cache, { recursive: true });
  const source = await prepareSource(cache, values.source);
  const build = join(source, "build", `solid-gpui-${target}-${values.profile}`);
  const environment = { RUSTUP_TOOLCHAIN: pin.toolchain };
  const sdkArgs = [
    ...(values["macos-sdk"] ? [`--macos-sdk=${resolve(values["macos-sdk"])}`] : []),
    ...(values["deployment-target"] ? [`--osx-deployment-target=${values["deployment-target"]}`] : []),
    ...(values.winsysroot ? [`--winsysroot=${resolve(values.winsysroot)}`] : []),
  ];
  await run(
    [
      process.execPath,
      "scripts/build.ts",
      `--profile=${values.profile === "debug" ? "debug-no-asan" : "release"}`,
      `--os=${selected.os}`,
      `--arch=${selected.arch}`,
      ...(selected.abi ? [`--abi=${selected.abi}`] : []),
      ...sdkArgs,
      "--webkit=prebuilt",
      "--mode=embed-native",
      "--configure-only",
      "--build-dir",
      build,
    ],
    source,
    environment,
  );
  await run([ninja, "-C", build, "embed-native"], source, environment);
  const manifestPath = join(build, "embed-native.json");
  const manifest: NativeManifest = JSON.parse(await readFile(manifestPath, "utf8"));
  validateManifest(manifest, target, values.profile);
  if (values["prepare-only"]) {
    console.log(manifestPath);
    return;
  }
  const directory = join(build, "application");
  if (!selected.graph) throw new Error("Missing application graph target");
  const graph = await packageEmbeddedGraph({
    bun: resolve(values.bun),
    entry: resolve(values.entry),
    target: selected.graph,
    outDir: join(directory, "graph"),
    assets: values.assets,
    workers: values.workers,
    baseExecutable: values["base-executable"],
  });
  await prepareApplication(directory, source, manifestPath, graph.rustSource, values.main);
  const cargoEnvironment = {
    ...manifest.environment,
    RUSTUP_TOOLCHAIN: pin.toolchain,
    CARGO_ENCODED_RUSTFLAGS: manifest.rustFlags.join("\x1f"),
    SOLID_GPUI_BUN_LINK_MANIFEST: manifestPath,
  };
  if (!(await exists(join(directory, "Cargo.lock"))))
    await run(["rustup", "run", pin.toolchain, "cargo", "generate-lockfile"], directory, cargoEnvironment);
  await run(
    [
      "rustup",
      "run",
      pin.toolchain,
      "cargo",
      "build",
      "--locked",
      "--target",
      target,
      "--profile",
      manifest.cargoProfile,
      ...manifest.cargoArgs,
    ],
    directory,
    cargoEnvironment,
  );
  const executable = join(
    directory,
    "target",
    target,
    manifest.cargoProfile === "dev" ? "debug" : manifest.cargoProfile,
    `solid-gpui-embedded-app${target.includes("windows") ? ".exe" : ""}`,
  );
  const output = resolve(values.output);
  // Verify the final link, including target identity and graph survival, before
  // writing the destination. Publish exactly the bytes that were checked.
  const image = await readFile(executable);
  const payload = extractGraphPayload(image, selected.graph);
  const graphDigest = createHash("sha256").update(payload).digest("hex");
  if (graphDigest !== graph.graphSha256) {
    throw new Error(
      `${basename(output)} carries the module graph ${graphDigest}, not the validated payload ${graph.graphSha256}`,
    );
  }
  await mkdir(dirname(output), { recursive: true });
  await writeFile(output, image);
  await chmod(output, (await stat(executable)).mode);
  const digest = createHash("sha256").update(image).digest("hex");
  console.log(`${digest}  ${basename(output)}`);
  console.log(`Static application: ${output}`);
}

if (import.meta.main) await main();
