/**
 * Static embedding product — the native half of an embedded-Bun application.
 *
 * `--mode=embed-native` runs the same codegen → deps → PCH → C/C++ phases as
 * `full` (see `emitBun`), then stops. It compiles no Rust staticlib and links
 * no executable: the host application is a separate Rust crate graph that
 * consumes the pinned `bun_bin` crate as an **rlib** and owns the final link.
 *
 * This module's whole job is to describe that native graph to the host build
 * system:
 *
 *   - one versioned JSON manifest (`embed-native.json`) naming every object,
 *     every dependency archive, the link policy and the Rust/cargo settings
 *     the host graph must reproduce, and
 *   - one ninja phony (`embed-native`) that materializes everything the
 *     manifest names before the host graph resolves.
 *
 * ## Why a manifest and not the CLI response file
 *
 * `full` mode writes `bun-debug.rsp`, which is the *inputs* of a link that also
 * carries the CLI's entry point, its PE resources, its map/order-file outputs
 * and a response-file syntax meant for one specific compiler driver. It is not
 * a build API: it contains no flags, it only exists after that link's command
 * line was already assembled, and its shape differs per platform. The embedding
 * host has a different entry point and different resources, so it gets the
 * object set plus the link *policy* (system libraries, exported-symbol lists,
 * delay-load flags, relocation model) as data.
 *
 * ## Constraints this file encodes
 *
 * - No separately compiled Bun Rust staticlib enters the manifest. Upstream's
 *   `staticlib` bundles its own copy of `std`; the host graph contains the same
 *   crate graph as an rlib so rustc reconciles one `std`, one panic strategy and
 *   one global allocator. `emitNativeGraphInvariants()` asserts this.
 * - No CLI entrypoint or PE resource: those are host-owned. The Windows
 *   `.bin/` shim *is* built here, because it is embedded data (`include_bytes!`
 *   in `bun_install`), not a link input — see `emitWindowsShim`.
 * The pinned prebuilt WebKit archives are used as-is; the manifest records
 * the WebKit mode so a consumer can refuse a source-WebKit product, and
 * configure refuses `--webkit=local` outright.
 *
 * ## Allocator ownership
 *
 * `bun_bin` keeps its mimalloc `#[global_allocator]`. rustc selects the
 * allocator from anywhere in the crate graph, so an rlib dependency's choice is
 * inherited by the final binary — and exactly one may exist across the crate and
 * its recursive dependencies, so the host root crate must NOT declare its own
 * (the build then fails with "conflicts with global allocator in: bun_bin").
 * Verified on the pinned nightly with a counting allocator: the rlib's allocator
 * serves the root binary's allocations, and a second declaration in the root is
 * a hard error. Nothing in this product file needs to restate that.
 */

import { isAbsolute, resolve } from "node:path";
import type { BunOutput } from "./bun.ts";
import type { CodegenOutputs } from "./codegen.ts";
import type { Config } from "./config.ts";
import { assert } from "./error.ts";
import { linkerMapPath, orderFilePath, symbolMapPath } from "./flags.ts";
import { writeIfChanged } from "./fs.ts";
import type { Ninja } from "./ninja.ts";
import { cargoBuildInvocation, cargoProfile, emitWindowsShim, rustTarget } from "./rust.ts";
import type { ResolvedDep } from "./source.ts";

/**
 * Ninja phony target that builds every input the manifest names.
 *
 * `ninja -C <buildDir> embed-native` is the whole preparation step: codegen,
 * vendored Rust path-dependency sources, dependency archives (libuv, WebKit
 * prebuilt, ...), every C/C++ object and the dependency `forbidUndefined`
 * checks. Run it *before* the host crate graph is resolved — the manifest's
 * `codegenDir` and `vendor/` path dependencies do not exist until it succeeds.
 */
export const embedNativeTarget = "embed-native";

/** Fixed manifest location: `<buildDir>/embed-native.json`. */
export function embedNativeManifestPath(cfg: Pick<Config, "buildDir">): string {
  return resolve(cfg.buildDir, "embed-native.json");
}

/**
 * The manifest. `schemaVersion` gates the whole contract: consumers must
 * refuse an unknown version rather than guess at fields.
 *
 * Link order the consumer must preserve: `objects` first (they create the
 * `Bun__*`/`Zig*`/`napi_*` undefined references), then `archives`, then
 * `linkArgs`. That is upstream's order in `emitBun`'s link step.
 */
export interface EmbedNativeManifest {
  schemaVersion: 1;
  /** rustc target triple this product was compiled for. */
  target: string;
  /** `BUN_CODEGEN_DIR` for the Rust half: generated `.rs`/`.cpp`/`.bin` root. */
  codegenDir: string;
  /** Every compiled C/C++ object file — bun's own, generated, and dep objects. */
  objects: string[];
  /** Dependency archives (`.a`/`.lib`) — internal deps plus prebuilt WebKit. */
  archives: string[];
  /**
   * Link policy tokens for the host's final link.
   *
   * Unix: a clang/clang++ driver command line. Windows: an `lld-link`/`link.exe`
   * command line. Both are what rustc passes through as `-Clink-arg=`, which is
   * how the host appends them to its own link.
   */
  linkArgs: string[];
  /** rustc tokens (`CARGO_ENCODED_RUSTFLAGS`, split) for the host crate graph. */
  rustFlags: string[];
  /** Unstable cargo args the host cannot derive from its own manifest. */
  cargoArgs: string[];
  /** Environment for the host's cargo invocation (no `CARGO_ENCODED_RUSTFLAGS`). */
  environment: Record<string, string>;
  /** Cargo profile the native product was configured with (`dev`/`release`). */
  cargoProfile: string;
  // ── Provenance. Additive to the contract; consumers caching a built product
  //    must key on these, not on `schemaVersion` alone. ──
  bunVersion: string;
  bunRevision: string;
  /** `prebuilt` or `local` — a `local` product compiled WebKit from source. */
  webkitMode: string;
  buildDir: string;
}

/** Everything `emitBun` already assembled when it hands the graph over. */
export interface EmbedNativeInputs {
  codegen: CodegenOutputs;
  deps: ResolvedDep[];
  /** All C/C++ objects (bun's own + codegen + dep objects). */
  objects: string[];
  /** Dependency archives. */
  archives: string[];
  /** Dependency `forbidUndefined` stamps — they gate this object set. */
  checks: string[];
  /** `flags.ldflags` + `systemLibs(cfg)`. */
  ldflags: string[];
  /** `emitShims()` output: link flags/artifacts and their build inputs. */
  shims: { ldflags: string[]; implicitInputs: string[] };
  /** Globbed `*.rs` sources — staleness input for the Windows shim edge. */
  rustSources: string[];
  /** Fetch stamps of vendored Rust path deps (`vendor/lolhtml`, rust-argon2). */
  vendorStamps: string[];
}

/**
 * Emit the static embedding product. Called from `emitBun` after the shared
 * codegen/dep/compile phases, in place of the CLI archive+link+post-link steps.
 */
export function emitEmbedNativeProduct(n: Ninja, cfg: Config, inputs: EmbedNativeInputs): BunOutput {
  assert(cfg.mode === "embed-native", `emitEmbedNativeProduct called with mode=${cfg.mode}`);

  n.comment("════════════════════════════════════════════════════════════════");
  n.comment("  Static embedding product");
  n.comment("════════════════════════════════════════════════════════════════");
  n.blank();

  // The Windows `.bin/` shim PE is `include_bytes!`d by `bun_install`, so it
  // must exist in the source tree before the *host* graph compiles that crate.
  // Upstream emits it inside emitRust(); the embedding product skips that step,
  // so it is emitted here. Its stamp is a preparation input, never a link input.
  const shimStamps = emitWindowsShim(n, cfg, {
    codegenInputs: inputs.codegen.rustInputs,
    codegenOrderOnly: inputs.codegen.rustOrderOnly,
    rustSources: inputs.rustSources,
    vendorStamps: inputs.vendorStamps,
  });

  const { tokens: linkArgs, artifacts: shimArtifacts } = splitLinkFlags(cfg, [
    ...inputs.ldflags,
    ...inputs.shims.ldflags,
  ]);
  const objects = [...new Set([...inputs.objects, ...shimArtifacts])];
  const archives = [...new Set(inputs.archives)];
  assertEmbedNativeProduct(cfg, objects, archives);

  const { rustFlags, cargoArgs, environment, cargoProfile: profile } = hostCargoSettings(cfg);

  const manifest: EmbedNativeManifest = {
    schemaVersion: 1,
    target: rustTarget(cfg),
    codegenDir: cfg.codegenDir,
    objects,
    archives,
    linkArgs,
    rustFlags,
    cargoArgs,
    environment,
    cargoProfile: profile,
    bunVersion: cfg.version,
    bunRevision: cfg.revision,
    webkitMode: cfg.webkit,
    buildDir: cfg.buildDir,
  };
  const manifestPath = embedNativeManifestPath(cfg);
  writeIfChanged(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`);

  // Everything the manifest names, plus the prerequisites it cannot name: the
  // codegen outputs (the Rust half `include!`s them), the vendored Rust path
  // dependencies (cargo cannot resolve the host graph without them) and the
  // shim build inputs. Dep `forbidUndefined` checks are included so a dep that
  // regrows a forbidden reference fails here rather than in the host link.
  const target = [...new Set([
    ...objects,
    ...archives,
    ...inputs.checks,
    ...inputs.shims.implicitInputs,
    ...shimStamps,
    ...inputs.codegen.all,
    ...inputs.vendorStamps,
  ])].sort();
  n.phony(embedNativeTarget, target);
  n.default([embedNativeTarget]);
  n.blank();
  n.comment(`Manifest: ${manifestPath}`);

  return {
    deps: inputs.deps,
    codegen: inputs.codegen,
    rustObjects: [],
    objects,
  };
}

/**
 * Bare absolute paths with these suffixes are link *inputs* rather than flags.
 * Suffix-matched on purpose — a bare absolute path is also how a flag argument
 * arrives (`-isysroot <sdk>`, `-exported_symbols_list <file>`).
 */
const LINK_ARTIFACT_SUFFIX = /\.(o|obj|a|lib|dylib|so|tbd)$/;

/**
 * Split the CLI link flags into (a) tokens the host link inherits and (b) bare
 * artifact paths it must treat as link inputs, and drop the tokens that belong
 * to the CLI executable's own output.
 *
 * The linker map and symbol-ordering file are *products of bun-debug's link*:
 * they are named after that binary and written into bun's build directory, and
 * the ordering file was traced from the CLI entry point. The host executable
 * owns its own map path and its own entry point, so replaying them would write
 * a stale map next to nothing relevant (and, on darwin/windows, make the host
 * link depend on a file traced for another program). Everything else —
 * exported-symbol lists, version scripts, delay-load flags, system libraries,
 * the relocation model, the stack size — is policy for the very same object
 * set and is preserved verbatim.
 *
 * Windows `-Wl,`-less tokens (`/DEF:`, `/STACK:`, `winmm.lib`, …) pass through
 * unchanged: they are already linker tokens there.
 */
function splitLinkFlags(cfg: Config, ldflags: string[]): { tokens: string[]; artifacts: string[] } {
  // `slash()` normalizes separators in some flag spellings (`/lldmap:`), so
  // compare separator-insensitively — the flags must be dropped on a native
  // Windows host too, where path.join produces backslashes.
  const normalize = (path: string) => path.replaceAll("\\", "/");
  const exeProductPaths = [linkerMapPath(cfg), symbolMapPath(cfg), orderFilePath(cfg)].map(normalize);
  const tokens: string[] = [];
  const artifacts: string[] = [];
  for (const token of ldflags) {
    if (exeProductPaths.some(path => normalize(token).includes(path))) continue;
    // Shim outputs are bare paths on the link line (see shims.ts) — a host
    // build system wants those as inputs, not as opaque "flags".
    if (isAbsolute(token) && LINK_ARTIFACT_SUFFIX.test(token)) {
      artifacts.push(token);
      continue;
    }
    tokens.push(token);
  }
  return { tokens, artifacts };
}

/**
 * The Rust/cargo half of the product, derived from `cargoBuildInvocation()`
 * so the host graph compiles the same crate graph with the same compiler
 * policy as the native graph it is linked against.
 */
function hostCargoSettings(cfg: Config): {
  rustFlags: string[];
  cargoArgs: string[];
  environment: Record<string, string>;
  cargoProfile: string;
} {
  const { args, env } = cargoBuildInvocation(cfg);
  const { CARGO_ENCODED_RUSTFLAGS: encoded, ...environment } = env;
  const rustFlags = encoded === undefined ? [] : encoded.split("\x1f").filter(flag => flag.length > 0);

  // A packaged application carries its JavaScript internally: there is no
  // build-machine codegen directory at runtime. Upstream gates
  // `bun_codegen_embed` on the profile only because a locally built `bun`
  // binary can read the codegen cache; force it for every embedded profile or
  // a debug host panics in `bun_runtime::bake` / `bun_resolver::node_fallbacks`
  // on a machine that has no build cache.
  // The upstream rlib build never performs the application's final link.
  // Its defensive lld override conflicts with the platform linker selected
  // by inputs.ldflags, notably Apple's native -ld_new on Darwin.
  const embedded = rustFlags.filter(flag => flag !== "--cfg=bun_codegen_embed" && flag !== "-Clink-arg=-fuse-ld=lld");
  embedded.push("--cfg=bun_codegen_embed");
  // rustc normally adds -nodefaultlibs. The final C++ application also needs
  // the compiler driver's C++ and sanitizer runtime selection.
  if (!cfg.windows) embedded.push("-Cdefault-linker-libraries=yes");

  const targetEnvironment = rustTarget(cfg).replace(/-/g, "_");
  for (const tool of ["CC", "CXX", "AR"]) {
    const executable = environment[tool];
    if (executable !== undefined) {
      environment[`${tool}_${targetEnvironment}`] = executable;
      delete environment[tool];
    }
  }
  if (cfg.windows) {
    const flags = [
      ...(cfg.winsysroot === undefined ? [] : ["/winsysroot", JSON.stringify(cfg.winsysroot)]),
      cfg.debug ? "/MTd" : "/MT",
    ].join(" ");
    environment.CC_SHELL_ESCAPED_FLAGS = "1";
    environment[`CFLAGS_${targetEnvironment}`] = flags;
    environment[`CXXFLAGS_${targetEnvironment}`] = flags;
  }

  // Windows: the C/C++ half is compiled against the static MSVC runtime
  // (`/MTd` in debug, `/MT` in release, plus `/U_DLL` — flags.ts), per the
  // no-install product contract. The Rust half must match, or the final image
  // imports the VC runtime DLLs and the two halves end up with different CRT
  // heaps and different `FILE*`/locale state. Target-only: these flags reach
  // only target crates when the host passes `--target <manifest.target>`, so
  // build scripts and proc-macros stay on the dynamic CRT.
  if (cfg.windows) embedded.push("-Ctarget-feature=+crt-static");

  return {
    rustFlags: embedded,
    // The host graph supplies `-p <its own crate> --lib --target-dir … --profile
    // …`; what it cannot derive is upstream's unstable-cargo policy
    // (`-Zbuild-std`/`-Zbuild-std-features` on release, asan and tier-3). Carry
    // exactly those args.
    cargoArgs: args.filter(arg => arg.startsWith("-Z")),
    environment,
    cargoProfile: cargoProfile(cfg).name,
  };
}

/**
 * Contract checks that must fail at configure time, not at the host link.
 *
 * The research constraint is explicit: one Rust graph, therefore no separately
 * compiled Bun Rust archive, and no CLI entry point or PE resource riding along
 * in the object set; and the pinned prebuilt WebKit archives, never a source
 * JSC build, which would be a different engine configuration than the qualified
 * one (and would silently replace "reuse the pinned prebuilt" with a multi-hour
 * compile).
 */
function assertEmbedNativeProduct(cfg: Config, objects: string[], archives: string[]): void {
  assert(
    cfg.webkit === "prebuilt",
    `the static embedding product requires the pinned prebuilt WebKit (webkit=${cfg.webkit})`,
    {
      hint:
        "Configure without --webkit=local (the default). A source WebKit build is a different " +
        "engine configuration than the qualified prebuilt archives and cannot be silently substituted.",
    },
  );
  const rustArchive = [...objects, ...archives].find(path => /bun_rust\.(a|lib)$/.test(path));
  assert(
    rustArchive === undefined,
    `embedding product must not carry a separately compiled Bun Rust archive (saw ${rustArchive})`,
    { hint: "bun_bin is consumed as an rlib by the host crate graph; drop the staticlib from this mode's inputs" },
  );
  const resource = objects.find(path => path.endsWith(".res") || path.endsWith(".rc"));
  assert(
    resource === undefined,
    `embedding product must not carry CLI PE resources (saw ${resource}) — the host owns its icon/manifest`,
  );
}

