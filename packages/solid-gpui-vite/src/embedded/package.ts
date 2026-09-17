/**
 * The embedded application packager.
 *
 * One call turns a built Vite entry into a single executable that carries its
 * JavaScript as a mapped module-graph section and embeds the pinned Bun runtime
 * as an rlib in its own Cargo graph:
 *
 *   pinned Bun checkout (patched, prebuilt WebKit) → native embed library
 *   Vite output → validated standalone module graph + generated Rust include
 *   generated application crate (SDK + application manifest merge) → link
 *   image re-verification → atomic publish
 *
 * Everything is derived from the SDK checkout the caller names and the
 * serializer revision that checkout pins. Nothing is resolved from `PATH`
 * except the tools the caller asked for, and no step tolerates version drift:
 * the graph payload carries no version field, so compatibility is bound to the
 * serializer identity alone.
 */

import { createHash, type Hash } from "node:crypto";
import { chmod, copyFile, cp, mkdir, mkdtemp, readFile, readdir, rename, rm, stat, writeFile } from "node:fs/promises";
import { basename, dirname, isAbsolute, join, resolve } from "node:path";
import { fail } from "./errors.ts";
import { extractGraphPayload, packageEmbeddedGraph, type EmbeddedEntryIdentity } from "./graph.ts";
import { writeGeneratedApplication, type EmbeddedApplication } from "./manifest.ts";
import { assertSdkCheckout, readPinnedBunBuild, type PinnedBunBuild } from "./sdk.ts";
import {
  graphTargetSpec,
  resolveEmbeddedTarget,
  type EmbeddedGraphTarget,
  type EmbeddedQualification,
  type EmbeddedTargetSpec,
} from "./targets.ts";

export type EmbeddedProfile = "debug" | "release";

type EmbeddedPackagingBase = {
  /** SDK checkout that owns the pinned Bun/Rust backend. */
  readonly sdkRoot: string;
  /** Rust target triple; defaults to the pinned toolchain's host triple. */
  readonly target?: string;
  readonly profile?: EmbeddedProfile;
  /**
   * Application-owned host library. Absent means the SDK's default entry, which
   * starts the packaged entry with the default host profile.
   */
  readonly application?: EmbeddedApplication;
  /**
   * Absolute path to a Rust source that defines `main`, compiled against the
   * SDK's default dependency set. Mutually exclusive with `application`, whose
   * `main` belongs to the same description.
   */
  readonly main?: string;
  /** Absolute paths embedded into the graph as resources. */
  readonly assets?: readonly string[];
  /** Absolute paths to worker entry points embedded into the graph. */
  readonly workers?: readonly string[];
  /** Target-platform Bun executable used as the compile base for cross-target serialization. */
  readonly baseExecutable?: string;
  /** SDK-local cache for the pinned Bun source checkout; defaults to `<sdkRoot>/target/bun-static`. */
  readonly cacheDir?: string;
  /** Existing pinned Bun source checkout to clone from instead of the network. */
  readonly sourceCheckout?: string;
  /** Ninja executable; defaults to `PATH`. */
  readonly ninja?: string;
  readonly macosSdk?: string;
  readonly deploymentTarget?: string;
  readonly winsysroot?: string;
  /** Receives every external command before it runs. */
  readonly onCommand?: (command: readonly string[]) => void;
};

export type EmbeddedPackagingOptions = EmbeddedPackagingBase &
  (
    | {
        /** Prepare the native embedding library only; no application is linked. */
        readonly prepareOnly: true;
        readonly entry?: string;
        readonly output?: string;
        readonly bun?: string;
      }
    | {
        readonly prepareOnly?: false;
        /** Absolute path to the built Vite JS entry. */
        readonly entry: string;
        /** Absolute path of the application executable to write. */
        readonly output: string;
        /** Absolute path to the pinned-revision Bun executable used as serializer. */
        readonly bun: string;
      }
  );

/** Where the packager put everything it produced. */
export type EmbeddedPackagingArtifacts = {
  /** Pinned Bun source checkout the native library was built in. */
  readonly bunSource: string;
  /** Native embed manifest replays the linker inputs into the application. */
  readonly nativeManifest: string;
  /** Generated Cargo crate; its `Cargo.lock` is the resolved application graph. */
  readonly applicationDirectory: string;
  readonly graphSource?: string;
  readonly graphBytes?: string;
  /** Built executable inside the generated crate's target directory. */
  readonly executable?: string;
};

export type EmbeddedPackagingReport = {
  readonly rustTriple: string;
  /** Graph transport platform for the triple, absent when the platform has none. */
  readonly graphTarget?: EmbeddedGraphTarget;
  readonly profile: EmbeddedProfile;
  readonly qualification: EmbeddedQualification;
  readonly evidence: string;
  readonly prepareOnly: boolean;
  /** Application entry identity, absent when only the native library was prepared. */
  readonly entry?: EmbeddedEntryIdentity;
  readonly workers?: readonly EmbeddedEntryIdentity[];
  readonly graphSha256?: string;
  /** Absolute path of the published executable, absent when only the native library was prepared. */
  readonly output?: string;
  readonly sha256?: string;
  readonly artifacts: EmbeddedPackagingArtifacts;
};

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

type CommandContext = {
  readonly cwd: string;
  readonly environment?: Record<string, string>;
  readonly onCommand?: (command: readonly string[]) => void;
  /** Capture stdout instead of inheriting it, for commands whose output is data. */
  readonly capture?: boolean;
};

async function run(args: readonly string[], context: CommandContext): Promise<string> {
  context.onCommand?.(args);
  const env = { ...process.env, ...context.environment };
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
    if (!(context.environment && key in context.environment)) delete env[key];
  }
  const child = Bun.spawn([...args], {
    cwd: context.cwd,
    env,
    stdin: "ignore",
    stdout: context.capture === true ? "pipe" : "inherit",
    stderr: "inherit",
  });
  const [output, status] = await Promise.all([
    context.capture === true ? new Response(child.stdout).text() : "",
    child.exited,
  ]);
  if (status !== 0) throw new Error(`${args[0]} failed with exit code ${status}`);
  return output.trim();
}

/**
 * Materializes the pinned Bun checkout, patched and overlaid with this SDK's
 * embedding runtime, and validates the cache it reuses.
 *
 * The cache key covers every input that changes the product: the pin file, the
 * embedding patch, and the overlay sources. A marker records the revision, and a
 * reused checkout whose revision drifted is deleted rather than trusted.
 */
async function prepareSource(
  sdkRoot: string,
  pin: PinnedBunBuild,
  cache: string,
  localSource?: string,
): Promise<string> {
  const sys = join(sdkRoot, "crates/solid-gpui-bun-sys");
  const hash = createHash("sha256")
    .update(JSON.stringify(pin))
    .update(await readFile(join(sys, "bun_embed.patch")));
  await digestTree(join(sys, "embedded"), hash);
  const source = join(cache, `source-${hash.digest("hex").slice(0, 24)}`);
  const context: CommandContext = { cwd: sdkRoot };
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
        context,
      );
      if (!localSource) await run(["git", "fetch", "--depth=1", "origin", pin.revision], { ...context, cwd: staging });
      await run(["git", "checkout", "--detach", pin.revision], { ...context, cwd: staging });
      await run(["git", "apply", "--check", join(sys, "bun_embed.patch")], { ...context, cwd: staging });
      await run(["git", "apply", join(sys, "bun_embed.patch")], { ...context, cwd: staging });
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
  const actual = await run(["git", "rev-parse", "HEAD"], { ...context, cwd: source, capture: true });
  if (actual !== pin.revision) {
    await rm(source, { recursive: true, force: true });
    fail(`pinned Bun checkout ${source} is at ${actual}, not ${pin.revision}; the cache was discarded, run again`);
  }
  return source;
}

/**
 * Validates the native embed manifest before it reaches the link.
 *
 * The manifest is the contract between the patched Bun build and the
 * application's Cargo graph: the same pinned revision, the same profile, and an
 * object/archive set that contains no separately compiled Bun Rust staticlib —
 * such a library would bring its own `std` and allocator into the link.
 */
function validateNativeManifest(
  value: NativeManifest,
  target: string,
  profile: EmbeddedProfile,
  pin: PinnedBunBuild,
): void {
  if (value.schemaVersion !== 1 || value.target !== target) {
    fail("native Bun manifest schema or target mismatch");
  }
  if (value.bunRevision !== pin.revision) {
    fail(`native Bun manifest was produced from Bun ${value.bunRevision}, not the pinned ${pin.revision}`);
  }
  if (value.webkitMode !== "prebuilt") {
    fail("native Bun manifest must use the pinned prebuilt WebKit product");
  }
  if (value.cargoProfile !== (profile === "release" ? "release" : "dev")) {
    fail(`native Bun manifest carries cargo profile ${value.cargoProfile}, not the profile of ${profile}`);
  }
  for (const key of ["objects", "archives", "linkArgs", "rustFlags", "cargoArgs"] as const) {
    if (!Array.isArray(value[key]) || value[key].some((item) => typeof item !== "string")) {
      fail(`invalid native manifest ${key}`);
    }
  }
  if (!isAbsolute(value.codegenDir) || [...value.objects, ...value.archives].some((path) => !isAbsolute(path))) {
    fail("native manifest paths must be absolute");
  }
  if (value.archives.some((path) => /(?:lib)?bun_rust\.(?:a|lib)$/.test(path))) {
    fail("a separate Bun Rust staticlib cannot enter the application link");
  }
}

async function hostTriple(pin: PinnedBunBuild, sdkRoot: string, context: CommandContext): Promise<string> {
  const rustc = await run(["rustup", "run", pin.toolchain, "rustc", "-vV"], {
    ...context,
    cwd: sdkRoot,
    capture: true,
  });
  const host = /^host: (.+)$/m.exec(rustc)?.[1];
  if (host === undefined) fail(`${pin.toolchain} rustc did not report a host triple`);
  return host;
}

/**
 * Packages the application, or prepares only the native embedding library.
 *
 * Every unsupported combination fails before any build work: an unknown triple,
 * a target whose platform has no graph transport, a cross-target serialization
 * without a compile base. Nothing is published until the produced image has
 * been re-verified against the graph that was serialized.
 */
export async function packageEmbeddedApplication(options: EmbeddedPackagingOptions): Promise<EmbeddedPackagingReport> {
  const sdkRoot = resolve(options.sdkRoot);
  await assertSdkCheckout(sdkRoot);
  const pin = await readPinnedBunBuild(sdkRoot);
  const context: CommandContext = { cwd: sdkRoot, onCommand: options.onCommand };
  const profile: EmbeddedProfile = options.profile ?? "release";
  if (profile !== "debug" && profile !== "release")
    fail(`profile must be debug or release, got ${JSON.stringify(profile)}`);

  const triple = options.target ?? (await hostTriple(pin, sdkRoot, context));
  const target: EmbeddedTargetSpec = resolveEmbeddedTarget(triple);
  const prepareOnly = options.prepareOnly === true;
  if (!prepareOnly && target.graph === undefined) {
    fail(
      `${triple} has no application graph transport (${target.evidence}); ` +
        `the native embedding library can still be prepared with prepareOnly`,
    );
  }
  if (!prepareOnly && (options.entry === undefined || options.output === undefined || options.bun === undefined)) {
    fail("entry, output and bun (the pinned serializer) are required unless only the native library is prepared");
  }
  if (!prepareOnly && options.main !== undefined) {
    if (options.application !== undefined) {
      fail("pass the Rust main through application.main, not alongside an application manifest");
    }
    if (!isAbsolute(options.main)) fail(`main must be an absolute path, got ${JSON.stringify(options.main)}`);
  }

  const ninja = options.ninja ?? Bun.which("ninja");
  if (!ninja) fail(`Ninja ${pin.ninjaVersion} is required; install it or pass ninja explicitly`);

  const cache = resolve(options.cacheDir ?? join(sdkRoot, "target/bun-static"));
  await mkdir(cache, { recursive: true });
  const bunSource = await prepareSource(sdkRoot, pin, cache, options.sourceCheckout);

  const buildDirectory = join(bunSource, "build", `solid-gpui-${triple}-${profile}`);
  const environment = { RUSTUP_TOOLCHAIN: pin.toolchain };
  const sdkArgs = [
    ...(options.macosSdk ? [`--macos-sdk=${resolve(options.macosSdk)}`] : []),
    ...(options.deploymentTarget ? [`--osx-deployment-target=${options.deploymentTarget}`] : []),
    ...(options.winsysroot ? [`--winsysroot=${resolve(options.winsysroot)}`] : []),
  ];
  await run(
    [
      process.execPath,
      "scripts/build.ts",
      `--profile=${profile === "debug" ? "debug-no-asan" : "release"}`,
      `--os=${target.os}`,
      `--arch=${target.arch}`,
      ...(target.abi ? [`--abi=${target.abi}`] : []),
      ...sdkArgs,
      "--webkit=prebuilt",
      "--mode=embed-native",
      "--configure-only",
      "--build-dir",
      buildDirectory,
    ],
    { ...context, cwd: bunSource, environment },
  );
  await run([ninja, "-C", buildDirectory, "embed-native"], { ...context, cwd: bunSource, environment });

  const nativeManifestPath = join(buildDirectory, "embed-native.json");
  const nativeManifest: NativeManifest = JSON.parse(await readFile(nativeManifestPath, "utf8"));
  validateNativeManifest(nativeManifest, triple, profile, pin);

  const artifacts: EmbeddedPackagingArtifacts = {
    bunSource,
    nativeManifest: nativeManifestPath,
    applicationDirectory: join(buildDirectory, "application"),
  };
  const base = {
    rustTriple: triple,
    ...(target.graph === undefined ? {} : { graphTarget: target.graph }),
    profile,
    qualification: target.qualification,
    evidence: target.evidence,
    artifacts,
  };
  if (prepareOnly) return { ...base, prepareOnly: true };
  if (target.graph === undefined) fail(`${triple} has no application graph target`);

  const graph = await packageEmbeddedGraph({
    sdkRoot,
    bun: resolve(options.bun!),
    entry: resolve(options.entry!),
    target: target.graph,
    outDir: join(artifacts.applicationDirectory, "graph"),
    assets: options.assets,
    workers: options.workers,
    baseExecutable: options.baseExecutable,
  });
  const graphTarget = target.graph;

  await writeGeneratedApplication({
    sdkRoot,
    bunSource,
    directory: artifacts.applicationDirectory,
    nativeManifest: nativeManifestPath,
    graphSource: graph.rustSource,
    application: options.application,
    main: options.main,
  });

  const cargoEnvironment = {
    ...nativeManifest.environment,
    RUSTUP_TOOLCHAIN: pin.toolchain,
    CARGO_ENCODED_RUSTFLAGS: nativeManifest.rustFlags.join("\x1f"),
    SOLID_GPUI_BUN_LINK_MANIFEST: nativeManifestPath,
  };
  const cargoContext: CommandContext = {
    cwd: artifacts.applicationDirectory,
    environment: cargoEnvironment,
    onCommand: options.onCommand,
  };
  if (!(await exists(join(artifacts.applicationDirectory, "Cargo.lock")))) {
    await run(["rustup", "run", pin.toolchain, "cargo", "generate-lockfile"], cargoContext);
  }
  await run(
    [
      "rustup",
      "run",
      pin.toolchain,
      "cargo",
      "build",
      "--locked",
      "--target",
      triple,
      "--profile",
      nativeManifest.cargoProfile,
      ...nativeManifest.cargoArgs,
    ],
    cargoContext,
  );

  const executable = join(
    artifacts.applicationDirectory,
    "target",
    triple,
    nativeManifest.cargoProfile === "dev" ? "debug" : nativeManifest.cargoProfile,
    `solid-gpui-embedded-app${graphTargetSpec(graphTarget).container === "pe" ? ".exe" : ""}`,
  );
  const output = resolve(options.output!);
  // Verify the final link — machine type, graph survival, and payload identity —
  // before writing the destination. Publish exactly the bytes that were checked.
  const image = await readFile(executable);
  const payload = extractGraphPayload(image, graphTarget);
  const graphDigest = createHash("sha256").update(payload).digest("hex");
  if (graphDigest !== graph.graphSha256) {
    fail(`${basename(output)} carries the module graph ${graphDigest}, not the validated payload ${graph.graphSha256}`);
  }
  const sha256 = createHash("sha256").update(image).digest("hex");
  await mkdir(dirname(output), { recursive: true });
  const staging = `${output}.staging-${process.pid}`;
  await writeFile(staging, image);
  await chmod(staging, (await stat(executable)).mode);
  await rename(staging, output);

  return {
    ...base,
    prepareOnly: false,
    entry: graph.entry,
    workers: graph.workers,
    graphSha256: graph.graphSha256,
    output,
    sha256,
    artifacts: {
      ...artifacts,
      graphSource: graph.rustSource,
      graphBytes: graph.sectionBytes,
      executable,
    },
  };
}
