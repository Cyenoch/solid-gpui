/**
 * Generates the Cargo crate the embedded packager builds: one binary crate
 * whose dependency graph contains exactly one pinned Bun embedding library,
 * exactly one `std`, and exactly one global allocator.
 *
 * An application may own its host library instead of forking this packager. It
 * then supplies its own Cargo manifest, package name, features and `main`; the
 * packager merges that manifest's patches and profiles into the single graph
 * the embedded link requires, resolving every relative patch path against the
 * manifest that declared it.
 *
 * Invariants this module enforces instead of hoping for:
 *
 * - **One Bun graph.** `bun_bin` comes from the pinned checkout as an rlib, and
 *   no separate Bun Rust staticlib may enter the link (see
 *   `solid-gpui-bun-sys`'s native manifest validation). An application that
 *   names `bun_bin`, `bun_rust` or `solid-gpui-bun-sys` itself is rejected, and
 *   a `solid-gpui` dependency that does not resolve to the SDK checkout under
 *   build would link a second copy of the SDK and is rejected too.
 * - **One allocator.** `bun_bin` owns the `#[global_allocator]`; rustc refuses a
 *   second declaration across the crate graph. Declared Rust sources are
 *   scanned so the failure is reported as packaging input instead of a link
 *   error.
 * - **One panic strategy.** The pinned Bun build is compiled with
 *   `panic = "abort"`; unwinding across its frames is not a supported ABI. The
 *   merged profiles therefore keep `abort` for `dev` and `release`.
 */

import { mkdir, readFile, stat, writeFile } from "node:fs/promises";
import { dirname, isAbsolute, join, resolve } from "node:path";
import { fail } from "./errors.ts";

export type EmbeddedApplication = {
  /**
   * Absolute path to the application's Cargo manifest: the package, the
   * workspace root, or any manifest inside the workspace that owns it.
   */
  readonly manifest: string;
  /** Package in that workspace whose library is the embedded host. */
  readonly package: string;
  /** Features enabled on that package; each must exist in its manifest. */
  readonly features?: readonly string[];
  /**
   * Absolute path to the Rust source that defines `main`.
   *
   * It is `include!`d into the generated crate after the module-graph include,
   * so it can use `BUN_EMBEDDED_ENTRY` and `BUN_EMBEDDED_WORKERS` and call into
   * the application library.
   */
  readonly main: string;
};

type CargoDocument = Record<string, unknown>;

export type WriteGeneratedApplicationOptions = {
  /** SDK checkout: supplies `solid_gpui`, the backend pin and the default entry. */
  readonly sdkRoot: string;
  /** Pinned Bun source checkout: owns the `bun_bin` crate in the graph. */
  readonly bunSource: string;
  /** Directory that receives `Cargo.toml`, `src/main.rs` and `build.rs`. */
  readonly directory: string;
  /** Native manifest the generated build script replays into the final link. */
  readonly nativeManifest: string;
  /** Absolute path to the generated module-graph Rust include. */
  readonly graphSource: string;
  /** Application-owned host description; absent means the SDK default entry. */
  readonly application?: EmbeddedApplication;
  /**
   * Rust source defining `main` for the SDK's default dependency set, used when
   * the application owns no library of its own. Mutually exclusive with
   * `application`, whose `main` is part of the same description.
   */
  readonly main?: string;
};

export type GeneratedApplication = {
  readonly cargoManifest: string;
  readonly mainSource: string;
  readonly buildScript: string;
  /** Absolute package directory of the application host, when one is used. */
  readonly applicationPackageDir?: string;
  /** Cargo profile the generated manifest resolves `dev` to, for diagnostics. */
  readonly profiles: readonly string[];
};

export type ResolvedApplication = {
  readonly manifestPath: string;
  /** Manifest Cargo treats as the workspace root; patches and profiles come from it. */
  readonly workspaceManifestPath: string;
  readonly packageDir: string;
  readonly packageName: string;
  readonly features: readonly string[];
  readonly main: string;
};

async function exists(path: string): Promise<boolean> {
  try {
    await stat(path);
    return true;
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === "ENOENT") return false;
    throw error;
  }
}

async function readCargoDocument(path: string, what: string): Promise<CargoDocument> {
  let text: string;
  try {
    text = await readFile(path, "utf8");
  } catch (error) {
    fail(`${what} ${path} could not be read: ${error instanceof Error ? error.message : String(error)}`);
  }
  const document = Bun.TOML.parse(text) as CargoDocument | null;
  if (document === null || typeof document !== "object") fail(`${what} ${path} is not a TOML document`);
  return document;
}

function table(document: CargoDocument, key: string): CargoDocument | undefined {
  const value = document[key];
  if (value === undefined || value === null) return undefined;
  if (typeof value !== "object" || Array.isArray(value)) fail(`Cargo manifest field ${key} is not a table`);
  return value as CargoDocument;
}

function stringField(document: CargoDocument, key: string): string | undefined {
  const value = document[key];
  if (value === undefined) return undefined;
  if (typeof value !== "string") fail(`Cargo manifest field ${key} is not a string`);
  return value;
}

/** The parts of `cargo metadata --no-deps` this packager consumes. */
type CargoMetadataPackage = {
  readonly name: string;
  readonly manifest_path: string;
  readonly features: Record<string, readonly string[]>;
  readonly targets: readonly {
    readonly kind: readonly string[];
    readonly crate_types: readonly string[];
    readonly src_path: string;
  }[];
  readonly dependencies: readonly {
    readonly name: string;
    readonly rename?: string | null;
    readonly path?: string | null;
    /** `null` for a normal dependency, `"dev"` or `"build"` otherwise. */
    readonly kind?: string | null;
  }[];
};

type CargoMetadata = {
  readonly workspace_root: string;
  readonly packages: readonly CargoMetadataPackage[];
};

/**
 * Asks Cargo what the application's workspace actually contains.
 *
 * Workspace membership, `workspace = true` inheritance, path resolution,
 * feature names and target sources are all Cargo's answers here instead of a
 * second, partial Cargo parser inside the packager. That is what makes a
 * `members = ["crates/*"]` workspace and an inherited dependency path behave
 * exactly as `cargo build` will, and it needs no registry access
 * (`--no-deps`).
 */
async function readCargoMetadata(manifestPath: string): Promise<CargoMetadata> {
  const child = Bun.spawn(
    ["cargo", "metadata", "--no-deps", "--format-version", "1", "--manifest-path", manifestPath],
    { stdin: "ignore", stdout: "pipe", stderr: "pipe" },
  );
  const [stdout, stderr, exitCode] = await Promise.all([
    new Response(child.stdout).text(),
    new Response(child.stderr).text(),
    child.exited,
  ]);
  if (exitCode !== 0) {
    fail(`cargo metadata for ${manifestPath} failed with exit code ${exitCode}: ${stderr.trim() || stdout.trim()}`);
  }
  const metadata = JSON.parse(stdout) as CargoMetadata;
  if (typeof metadata.workspace_root !== "string" || !Array.isArray(metadata.packages)) {
    fail(`cargo metadata for ${manifestPath} did not report a workspace root and packages`);
  }
  return metadata;
}

/**
 * Resolves the application's package inside its own workspace.
 *
 * The package must expose a library target: the generated crate is the binary
 * that owns the final link, and it calls into the application's library.
 */
export async function resolveEmbeddedApplication(
  application: EmbeddedApplication,
  sdkRoot: string,
): Promise<ResolvedApplication> {
  if (!isAbsolute(application.manifest)) {
    fail(`application.manifest must be an absolute path, got ${JSON.stringify(application.manifest)}`);
  }
  const manifestPath = resolve(application.manifest);
  const metadata = await readCargoMetadata(manifestPath);
  const workspaceManifestPath = join(resolve(metadata.workspace_root), "Cargo.toml");
  const pkg = metadata.packages.find((candidate) => candidate.name === application.package);
  if (pkg === undefined) {
    fail(
      `package ${JSON.stringify(application.package)} is not a member of the workspace at ${metadata.workspace_root}; ` +
        `members: ${metadata.packages.map((candidate) => candidate.name).join(", ") || "(none)"}`,
    );
  }
  const packageDir = dirname(pkg.manifest_path);

  const features = application.features ?? [];
  for (const feature of features) {
    if (!(feature in pkg.features)) {
      fail(
        `package ${JSON.stringify(application.package)} does not declare feature ${JSON.stringify(feature)}; ` +
          `available: ${Object.keys(pkg.features).join(", ") || "(none)"}`,
      );
    }
  }

  const library = pkg.targets.find((target) => target.crate_types.includes("lib"));
  if (library === undefined) {
    fail(
      `package ${JSON.stringify(application.package)} declares no library target; the embedded host application must ` +
        `expose one (src/lib.rs or an explicit [lib] path), because the generated binary crate owns the final link`,
    );
  }

  if (!isAbsolute(application.main)) {
    fail(`application.main must be an absolute path, got ${JSON.stringify(application.main)}`);
  }
  const main = resolve(application.main);
  if (!(await exists(main))) fail(`application.main ${main} does not exist`);

  assertSingleBunGraph(pkg, sdkRoot);
  await assertNoGlobalAllocator([main, library.src_path]);

  return {
    manifestPath,
    workspaceManifestPath,
    packageDir,
    packageName: application.package,
    features,
    main,
  };
}

/**
 * Rejects an application dependency graph that would contain a second Bun
 * embedding library or a second copy of the SDK.
 *
 * Both failures are silent at the JS level and only appear as link or type
 * errors far from their cause, so they are packaging input errors here. Cargo
 * has already resolved inheritance and renames, so each dependency's `name` is
 * the package it really is and `path` is absolute.
 */
function assertSingleBunGraph(pkg: CargoMetadataPackage, sdkRoot: string): void {
  const bunPackages = new Set(["bun_bin", "bun_rust", "solid-gpui-bun-sys", "bun-sys", "bun_core"]);
  const sdkCrates = join(resolve(sdkRoot), "crates/solid-gpui");
  for (const dependency of pkg.dependencies) {
    if (bunPackages.has(dependency.name)) {
      fail(
        `package ${JSON.stringify(pkg.name)} depends on ${JSON.stringify(dependency.name)}` +
          `${dependency.rename === null || dependency.rename === undefined ? "" : ` as ${JSON.stringify(dependency.rename)}`}; ` +
          `the embedded application receives exactly one pinned Bun graph from the packager, so it must not depend on the ` +
          `embedding library itself`,
      );
    }
    if (dependency.name === "solid-gpui") {
      if (dependency.path === null || dependency.path === undefined) {
        fail(
          `package ${JSON.stringify(pkg.name)} depends on solid-gpui without a path; the embedded host must use the SDK ` +
            `checkout the packager builds (${sdkCrates}), or the link would contain two SDK copies`,
        );
      }
      if (resolve(dependency.path) !== sdkCrates) {
        fail(
          `package ${JSON.stringify(pkg.name)} depends on solid-gpui at ${resolve(dependency.path)}, not the SDK checkout ` +
            `under build (${sdkCrates}); two copies would not share types`,
        );
      }
    }
  }
}

/** Rejects a declared global allocator; `bun_bin` owns the process's only one. */
async function assertNoGlobalAllocator(sources: readonly string[]): Promise<void> {
  for (const source of sources) {
    if (!(await exists(source))) continue;
    const text = await readFile(source, "utf8");
    for (const [index, line] of text.split("\n").entries()) {
      if (line.trim().startsWith("#[global_allocator]")) {
        fail(
          `${source}:${index + 1} declares #[global_allocator]; the embedded application's allocator comes from the pinned ` +
            `bun_bin crate in the same graph, and rustc rejects a second declaration`,
        );
      }
    }
  }
}

type PatchEntry = { readonly spec: CargoDocument; readonly origin: string };

/**
 * Merges `[patch]` tables from every manifest that declares one.
 *
 * Relative paths are resolved against the manifest that declared them, so a
 * patch that points into the application's own vendored checkout and one that
 * points into the SDK stay meaningful in the generated crate's directory.
 * Identical entries dedupe; two different targets for one crate refuse,
 * because Cargo would silently take one of them.
 */
export function mergePatchTables(
  sources: readonly { readonly path: string; readonly document: CargoDocument }[],
): Record<string, Record<string, CargoDocument>> {
  const merged: Record<string, Record<string, PatchEntry>> = {};
  for (const { path, document } of sources) {
    const patch = table(document, "patch");
    if (patch === undefined) continue;
    const directory = dirname(path);
    for (const [registry, entries] of Object.entries(patch)) {
      if (entries === null || typeof entries !== "object" || Array.isArray(entries)) {
        fail(`${path}: [patch.${registry}] is not a table`);
      }
      const target = (merged[registry] ??= {});
      for (const [name, spec] of Object.entries(entries as CargoDocument)) {
        const resolved = structuredClone(spec) as CargoDocument;
        const relative = stringField(resolved, "path");
        if (relative !== undefined) resolved["path"] = resolve(directory, relative);
        const existing = target[name];
        if (existing === undefined) {
          target[name] = { spec: resolved, origin: path };
          continue;
        }
        if (JSON.stringify(existing.spec) !== JSON.stringify(resolved)) {
          fail(
            `${name} is patched to two different targets: ${JSON.stringify(existing.spec)} from ${existing.origin} and ` +
              `${JSON.stringify(resolved)} from ${path}; pass one patch table or make them identical`,
          );
        }
      }
    }
  }
  const document: Record<string, Record<string, CargoDocument>> = {};
  for (const [registry, entries] of Object.entries(merged)) {
    document[registry] = {};
    for (const [name, entry] of Object.entries(entries)) document[registry]![name] = entry.spec;
  }
  return document;
}

/**
 * Merges `[profile]` tables in increasing precedence: the pinned Bun source is
 * the base the embedded link is built against, the SDK checkout states what its
 * own crates need, and the application overrides both.
 */
export function mergeProfileTables(sources: readonly CargoDocument[]): Record<string, CargoDocument> {
  const merged: Record<string, CargoDocument> = {};
  for (const document of sources) {
    const profiles = table(document, "profile");
    if (profiles === undefined) continue;
    for (const [name, value] of Object.entries(profiles)) {
      if (value === null || typeof value !== "object" || Array.isArray(value)) {
        fail(`[profile.${name}] is not a table`);
      }
      merged[name] = deepMerge(merged[name] ?? {}, value as CargoDocument);
    }
  }
  return merged;
}

function deepMerge(base: CargoDocument, override: CargoDocument): CargoDocument {
  const merged: CargoDocument = { ...base };
  for (const [key, value] of Object.entries(override)) {
    const previous = merged[key];
    if (
      previous !== null &&
      typeof previous === "object" &&
      !Array.isArray(previous) &&
      value !== null &&
      typeof value === "object" &&
      !Array.isArray(value)
    ) {
      merged[key] = deepMerge(previous as CargoDocument, value as CargoDocument);
    } else {
      merged[key] = value;
    }
  }
  return merged;
}

/**
 * Keeps the embedded link's panic strategy.
 *
 * The pinned Bun objects are compiled with `panic = "abort"`; unwinding across
 * them is not a supported ABI, so an application profile that asks for unwind
 * is refused rather than linked.
 */
export function assertAbortPanicStrategy(profiles: Record<string, CargoDocument>): void {
  for (const name of ["dev", "release"]) {
    const profile = profiles[name];
    if (profile === undefined) continue;
    const panic = profile["panic"];
    if (panic !== undefined && panic !== "abort") {
      fail(
        `[profile.${name}] sets panic = ${JSON.stringify(panic)}; the pinned Bun graph is compiled with panic = "abort" and ` +
          `unwinding across its frames is not a supported ABI`,
      );
    }
    profile["panic"] = "abort";
  }
}

const GENERATED_MANIFEST_HEADER = `# @generated by @solid-gpui/vite/embedded -- do not edit.
#
# The embedded application crate graph. It contains exactly one Bun embedding
# library (bun_bin, from the pinned checkout) as an rlib, so rustc reconciles one
# std, one panic strategy and one global allocator across the final link. No
# separate Bun Rust staticlib may enter: the native manifest validation rejects
# one. The binary is the link root; everything else is a dependency.
`;

/**
 * Writes the generated crate: manifest, entry source, and the build script that
 * replays the native manifest into the final link.
 */
export async function writeGeneratedApplication(
  options: WriteGeneratedApplicationOptions,
): Promise<GeneratedApplication> {
  const sdkRoot = resolve(options.sdkRoot);
  const bunSource = resolve(options.bunSource);
  const application =
    options.application === undefined ? undefined : await resolveEmbeddedApplication(options.application, sdkRoot);
  let main = application?.main ?? join(sdkRoot, "crates/solid-gpui-bun-sys/static-application.rs");
  if (options.main !== undefined) {
    if (application !== undefined) {
      fail("pass the Rust main through application.main, not alongside an application manifest");
    }
    if (!isAbsolute(options.main)) {
      fail(`main must be an absolute path, got ${JSON.stringify(options.main)}`);
    }
    if (!(await exists(options.main))) fail(`main ${options.main} does not exist`);
    await assertNoGlobalAllocator([options.main]);
    main = options.main;
  }

  const bunDocument = await readCargoDocument(join(bunSource, "Cargo.toml"), "pinned Bun Cargo manifest");
  const sdkDocument = await readCargoDocument(join(sdkRoot, "Cargo.toml"), "SDK Cargo manifest");
  const profileSources = [bunDocument, sdkDocument];
  const patchSources = [{ path: join(sdkRoot, "Cargo.toml"), document: sdkDocument }];
  if (application !== undefined) {
    const applicationDocument = await readCargoDocument(
      application.workspaceManifestPath,
      "application workspace Cargo manifest",
    );
    profileSources.push(applicationDocument);
    patchSources.push({ path: application.workspaceManifestPath, document: applicationDocument });
  }

  const mergedProfiles = mergeProfileTables(profileSources);
  // The pinned Bun graph states the link's panic strategy; nothing downstream
  // may change it.
  assertAbortPanicStrategy(mergedProfiles);

  const patches = mergePatchTables(patchSources);

  const dependencies: Record<string, CargoDocument> = {
    solid_gpui: {
      package: "solid-gpui",
      path: join(sdkRoot, "crates/solid-gpui"),
      features: ["embedded-bun"],
    },
    bun_rust: { package: "bun_bin", path: join(bunSource, "src/bun_bin"), features: ["solid-gpui-embed"] },
  };
  if (application !== undefined) {
    dependencies[application.packageName] = {
      path: application.packageDir,
      ...(application.features.length > 0 ? { features: [...application.features] } : {}),
    };
  }

  const document: CargoDocument = {
    workspace: { resolver: "2" },
    package: { name: "solid-gpui-embedded-app", version: "0.1.0", edition: "2024", publish: false },
    dependencies,
  };
  if (Object.keys(mergedProfiles).length > 0) document["profile"] = mergedProfiles;
  if (Object.keys(patches).length > 0) document["patch"] = patches;

  const serialized = Bun.TOML.stringify(document);
  if (!serialized) fail("failed to serialize the generated application Cargo manifest");

  await mkdir(join(options.directory, "src"), { recursive: true });
  const cargoManifest = join(options.directory, "Cargo.toml");
  const mainSource = join(options.directory, "src/main.rs");
  const buildScript = join(options.directory, "build.rs");
  await writeFile(cargoManifest, `${GENERATED_MANIFEST_HEADER}\n${serialized}`);
  await writeFile(
    mainSource,
    [
      '#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]',
      "// @generated by @solid-gpui/vite/embedded -- do not edit.",
      "//",
      "// `bun_rust` is the pinned Bun embedding library from the same graph as the",
      "// graph section below. Do not declare a global allocator here: bun_bin owns",
      "// the process's only one, and rustc rejects a second declaration.",
      "extern crate bun_rust;",
      `include!(${JSON.stringify(options.graphSource)});`,
      `include!(${JSON.stringify(main)});`,
      "",
    ].join("\n"),
  );

  const nativeManifestDocument = await readFile(options.nativeManifest, "utf8");
  const nativeManifest: { objects: string[]; archives: string[]; linkArgs: string[] } =
    JSON.parse(nativeManifestDocument);
  const args = [...nativeManifest.objects, ...nativeManifest.archives, ...nativeManifest.linkArgs];
  await writeFile(
    buildScript,
    [
      "fn main() {",
      `    println!("cargo:rerun-if-changed={}", ${JSON.stringify(resolve(options.nativeManifest))});`,
      `    println!("cargo:rerun-if-changed={}", ${JSON.stringify(options.graphSource)});`,
      ...(application === undefined
        ? [`    println!("cargo:rerun-if-changed={}", ${JSON.stringify(main)});`]
        : [
            `    println!("cargo:rerun-if-changed={}", ${JSON.stringify(application.manifestPath)});`,
            `    println!("cargo:rerun-if-changed={}", ${JSON.stringify(application.main)});`,
          ]),
      ...args.map((arg) => `    println!("cargo:rustc-link-arg={}", ${JSON.stringify(arg)});`),
      ...[...nativeManifest.objects, ...nativeManifest.archives].map(
        (path) => `    println!("cargo:rerun-if-changed={}", ${JSON.stringify(path)});`,
      ),
      "}",
      "",
    ].join("\n"),
  );

  return {
    cargoManifest,
    mainSource,
    buildScript,
    ...(application === undefined ? {} : { applicationPackageDir: application.packageDir }),
    profiles: Object.keys(mergedProfiles).sort(),
  };
}
