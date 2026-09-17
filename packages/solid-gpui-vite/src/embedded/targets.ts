/**
 * The explicit experimental matrix of targets the embedded packager accepts.
 *
 * Nothing here is a support statement. Every entry carries the strongest
 * evidence the repository actually has, and the packager refuses a
 * combination it cannot build instead of guessing: an unknown Rust triple
 * fails, and a target without a graph transport is only reachable through
 * `prepareOnly`.
 */

import { fail } from "./errors.ts";

/** Bun `--compile` target whose runtime will host the embedded module graph. */
export type EmbeddedGraphTarget = "bun-windows-x64" | "bun-windows-arm64" | "bun-darwin-arm64" | "bun-darwin-x64";

export type EmbeddedGraphTargetSpec = {
  /** Host platform whose Bun binary is the compile base unless one is supplied. */
  readonly host: string;
  readonly container: "pe" | "macho";
  /**
   * Machine type the image's own header must declare: `IMAGE_FILE_MACHINE_*`
   * for PE, `CPU_TYPE_*` for Mach-O.
   *
   * The graph key prefix distinguishes Windows from POSIX but *not* x64 from
   * arm64, so without this the serializer's own output — and therefore the
   * compile base behind it — would be accepted for either architecture.
   */
  readonly machine: number;
  /** Human-readable spelling of `machine`, for diagnostics. */
  readonly machineName: string;
  /** `StandaloneModuleGraph` key prefix written into every graph file name. */
  readonly prefix: string;
  /** Attribute value for the generated `#[unsafe(link_section = ...)]`. */
  readonly linkSection: string;
  /** Container suffix the serializer appends to a PE output file. */
  readonly executableSuffix: "" | ".exe";
};

export const EMBEDDED_GRAPH_TARGETS: Record<EmbeddedGraphTarget, EmbeddedGraphTargetSpec> = {
  "bun-windows-x64": {
    host: "win32-x64",
    container: "pe",
    machine: 0x8664,
    machineName: "x64",
    prefix: "B:/~BUN/root/",
    linkSection: ".bun",
    executableSuffix: ".exe",
  },
  "bun-windows-arm64": {
    host: "win32-arm64",
    container: "pe",
    machine: 0xaa64,
    machineName: "arm64",
    prefix: "B:/~BUN/root/",
    linkSection: ".bun",
    executableSuffix: ".exe",
  },
  "bun-darwin-x64": {
    host: "darwin-x64",
    container: "macho",
    machine: 0x0100_0007,
    machineName: "x86_64",
    prefix: "/$bunfs/root/",
    linkSection: "__BUN,__bun",
    executableSuffix: "",
  },
  "bun-darwin-arm64": {
    host: "darwin-arm64",
    container: "macho",
    machine: 0x0100_000c,
    machineName: "arm64",
    prefix: "/$bunfs/root/",
    linkSection: "__BUN,__bun",
    executableSuffix: "",
  },
};

export function graphTargetSpec(target: string): EmbeddedGraphTargetSpec {
  const spec = (EMBEDDED_GRAPH_TARGETS as Record<string, EmbeddedGraphTargetSpec | undefined>)[target];
  if (spec === undefined) {
    fail(
      `unsupported embedded graph target ${JSON.stringify(target)}; expected one of ${Object.keys(EMBEDDED_GRAPH_TARGETS).join(", ")}`,
    );
  }
  return spec;
}

/**
 * How far a target's evidence reaches. This is the only vocabulary the
 * packager and its docs may use:
 *
 * - `"prepare-only"` — the native embedding library can be prepared, but no
 *   ELF graph transport exists, so no application image can be produced.
 * - `"experimental"` — the full path exists and has produced fixture evidence,
 *   which is not support, portability, GUI, or distribution qualification.
 */
export type EmbeddedQualification = "prepare-only" | "experimental";

export type EmbeddedTargetSpec = {
  /** Rust target triple passed to Cargo and to the native build. */
  readonly triple: string;
  readonly os: "darwin" | "windows" | "linux";
  readonly arch: "aarch64" | "x64";
  readonly abi?: "gnu" | "musl";
  /** Graph target, absent when the platform has no graph transport. */
  readonly graph?: EmbeddedGraphTarget;
  readonly qualification: EmbeddedQualification;
  /** The strongest evidence this repository has for the combination. */
  readonly evidence: string;
};

const TARGETS: readonly EmbeddedTargetSpec[] = [
  {
    triple: "aarch64-apple-darwin",
    os: "darwin",
    arch: "aarch64",
    graph: "bun-darwin-arm64",
    qualification: "experimental",
    evidence:
      "debug builds run end to end (headless two-session probe and a real GPUI window); the debug executable is not standalone, and no release build is qualified",
  },
  {
    triple: "x86_64-apple-darwin",
    os: "darwin",
    arch: "x64",
    graph: "bun-darwin-x64",
    qualification: "experimental",
    evidence: "the full path exists but no build or run is recorded for this triple",
  },
  {
    triple: "x86_64-pc-windows-msvc",
    os: "windows",
    arch: "x64",
    graph: "bun-windows-x64",
    qualification: "experimental",
    evidence:
      "a debug image was cross-linked and run headlessly in a Windows 11 ARM64 VM; evidence is limited to fixture probes",
  },
  {
    triple: "aarch64-pc-windows-msvc",
    os: "windows",
    arch: "aarch64",
    graph: "bun-windows-arm64",
    qualification: "experimental",
    evidence:
      "debug and release images built natively and passed headless fixture probes, including a standard-user and read-only-installation run; GUI, signing and dependency closure stay unqualified",
  },
  {
    triple: "x86_64-unknown-linux-gnu",
    os: "linux",
    arch: "x64",
    abi: "gnu",
    qualification: "prepare-only",
    evidence: "native preparation only; ELF application graph transport is not implemented",
  },
  {
    triple: "aarch64-unknown-linux-gnu",
    os: "linux",
    arch: "aarch64",
    abi: "gnu",
    qualification: "prepare-only",
    evidence: "native preparation only; ELF application graph transport is not implemented",
  },
  {
    triple: "x86_64-unknown-linux-musl",
    os: "linux",
    arch: "x64",
    abi: "musl",
    qualification: "prepare-only",
    evidence: "native preparation only; ELF application graph transport is not implemented",
  },
  {
    triple: "aarch64-unknown-linux-musl",
    os: "linux",
    arch: "aarch64",
    abi: "musl",
    qualification: "prepare-only",
    evidence: "native preparation only; ELF application graph transport is not implemented",
  },
];

/** Every triple the packager accepts, in matrix order. */
export const EMBEDDED_TARGETS: readonly EmbeddedTargetSpec[] = Object.freeze(TARGETS);

/** Resolves a Rust triple against the matrix; an unknown triple fails early. */
export function resolveEmbeddedTarget(triple: string): EmbeddedTargetSpec {
  const spec = EMBEDDED_TARGETS.find((target) => target.triple === triple);
  if (spec === undefined) {
    fail(
      `unsupported embedded application target ${JSON.stringify(triple)}; the matrix accepts ${EMBEDDED_TARGETS.map((target) => target.triple).join(", ")}`,
    );
  }
  return spec;
}

/**
 * Graph key of the application entry point.
 *
 * The serializer keys the first entry point after the output file's basename,
 * with the target-format suffix (`.exe`) appended for PE targets and then
 * removed again from the key. The packager names that intermediate after the
 * entry itself, so the key is derived from the application rather than from a
 * temporary file name. Deriving it here — instead of letting an application
 * guess it — is what makes an entry identity checkable.
 */
export function embeddedEntryPointIdentity(target: EmbeddedGraphTarget, outfile: string): string {
  const spec = graphTargetSpec(target);
  const name = fileLeaf(outfile);
  const bare =
    spec.executableSuffix !== "" && name.endsWith(spec.executableSuffix)
      ? name.slice(0, -spec.executableSuffix.length)
      : name;
  if (bare.length === 0) fail(`embedded entry ${JSON.stringify(outfile)} has an empty file name`);
  return `${spec.prefix}${bare}`;
}

/** Extensions the bundler rewrites to `.js` when an entry becomes a graph file. */
const SCRIPT_EXTENSIONS = [".ts", ".tsx", ".mts", ".cts", ".mjs", ".cjs", ".jsx", ".js"] as const;

function fileLeaf(path: string): string {
  const name = path
    .replace(/[\\/]+$/, "")
    .split(/[\\/]/)
    .pop();
  if (name === undefined || name.length === 0) fail(`embedded entry ${JSON.stringify(path)} has no file name`);
  return name;
}

/**
 * The serializer's source root: the longest directory every entry shares.
 *
 * Bun keys an additional entry point after its path relative to that root, so
 * the packager computes the same value, passes it as `--root`, and can then
 * state each worker's identity exactly instead of guessing it from a file name.
 */
export function embeddedSourceRoot(entries: readonly string[]): string {
  if (entries.length === 0) fail("an embedded application needs at least one entry point");
  const components = entries.map((entry) => entry.split(/[\\/]+/));
  let shared = components[0]!.slice(0, -1);
  for (const other of components.slice(1)) {
    const directory = other.slice(0, -1);
    let index = 0;
    while (index < shared.length && index < directory.length && directory[index] === shared[index]) index += 1;
    shared = shared.slice(0, index);
  }
  if (shared.length === 0) fail(`embedded entries ${entries.join(", ")} share no common directory`);
  const joined = shared.join("/");
  return joined.length === 0 ? "/" : joined;
}

/**
 * Graph key of an additional entry point (a worker), as the serializer keys it.
 *
 * Bun writes the entry's path relative to the source root with the emitted
 * `.js` extension — every script extension (`.ts`, `.tsx`, `.mts`, `.cts`,
 * `.mjs`, `.cjs`, `.jsx`) becomes `.js`, verified against the pinned
 * serializer. `packageEmbeddedGraph` verifies the result against the serialized
 * graph, so a rule that ever drifted would fail packaging with the actual keys
 * rather than hand an application a specifier nothing resolves.
 */
export function embeddedWorkerIdentity(target: EmbeddedGraphTarget, source: string, sourceRoot: string): string {
  const spec = graphTargetSpec(target);
  const root = sourceRoot.replace(/[\\/]+$/, "").split(/[\\/]+/);
  const components = source.split(/[\\/]+/);
  let shared = 0;
  while (shared < root.length && shared < components.length && components[shared] === root[shared]) shared += 1;
  if (shared < root.length) {
    fail(
      `worker entry ${JSON.stringify(source)} is not inside the serializer source root ${JSON.stringify(sourceRoot)}; ` +
        `every entry point must share a directory`,
    );
  }
  const relative = components.slice(shared).join("/");
  if (relative.length === 0)
    fail(`worker entry ${JSON.stringify(source)} has no path inside ${JSON.stringify(sourceRoot)}`);
  const extension = SCRIPT_EXTENSIONS.find((candidate) => relative.endsWith(candidate));
  const emitted = extension === undefined ? relative : `${relative.slice(0, -extension.length)}.js`;
  return `${spec.prefix}${emitted}`;
}
