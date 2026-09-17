import { randomUUID } from "node:crypto";
import { realpathSync } from "node:fs";
import { mkdir, readFile, rename, rm, writeFile } from "node:fs/promises";
import { dirname, join, resolve, sep } from "node:path";

/** Cargo selection shared by the Vite plugin, the prepare CLI and packaging scripts. */
export interface NativeBuildOptions {
  /** Cargo profile: `dev` (default), `release`, or a custom profile name; `debug` is accepted for `dev`. */
  readonly profile?: string;
  /** Cargo `--target` triple. A cross target is built but never executed on the build host. */
  readonly target?: string;
  /** Pass `--locked` (default); `false` admits the initial lockfile resolution. */
  readonly locked?: boolean;
}

/** The Cargo-level facts of a configured native host. */
export interface NativeHostArtifacts {
  readonly manifestPath: string;
  readonly package?: string;
  readonly bin?: string;
  readonly features: readonly string[];
  readonly profile: string;
  readonly profileDirectory: string;
  readonly target?: string;
  readonly locked: boolean;
  readonly targetDirectory?: string;
  readonly executable?: string;
}

/** The host that runs the application. */
export interface HostArtifacts {
  readonly command: string;
  readonly args: readonly string[];
  readonly output: string;
}

/**
 * Authoritative locations for everything a consumer would otherwise re-derive:
 * generated bindings, prepared TypeScript config, the production bundle and the host executable.
 */
export interface NativeArtifacts {
  readonly version: 1;
  readonly root: string;
  readonly runtime: "web" | "bun" | "quickjs";
  readonly entry: string;
  /** The configured build output directory: the stable identity of where outputs belong. */
  readonly outDir: string;
  readonly bindings?: string;
  readonly tsconfig: string;
  readonly bundle?: string;
  readonly host?: HostArtifacts;
  readonly native?: NativeHostArtifacts;
}

const PROFILE_DIRECTORIES: Record<string, string> = { dev: "debug", release: "release" };

/** `debug` is what developers call Cargo's `dev` profile; Cargo only accepts the latter. */
export function normalizeCargoProfile(profile = "dev"): string {
  return profile === "debug" ? "dev" : profile;
}

/** Cargo's output directory for a profile: `dev` → `debug`, `release` → `release`, else the profile name. */
export function cargoProfileDirectory(profile = "dev"): string {
  const normalized = normalizeCargoProfile(profile);
  return PROFILE_DIRECTORIES[normalized] ?? normalized;
}

const HOST_ARCHITECTURES: Record<string, string> = { arm64: "aarch64", x64: "x86_64", ia32: "i686" };
const HOST_OPERATING_SYSTEMS: Record<string, string> = {
  darwin: "apple-darwin",
  linux: "linux",
  win32: "windows",
  freebsd: "freebsd",
};

function targetComponents(target: string): { readonly architecture: string; readonly system: string } {
  const [architecture = "", ...rest] = target.split("-");
  const remainder = rest.join("-");
  const system = ["apple-darwin", "windows", "linux", "freebsd", "none"].find((candidate) =>
    remainder.includes(candidate),
  );
  return {
    architecture: architecture === "arm64" ? "aarch64" : architecture,
    system: system ?? remainder,
  };
}

/**
 * Whether a `--target` artifact can be executed on this machine. Cargo cross builds produce
 * binaries for another system, and running one as the bindings exporter fails confusingly.
 */
export function targetRunsOnHost(
  target?: string,
  host: { readonly architecture: string; readonly platform: string } = {
    architecture: process.arch,
    platform: process.platform,
  },
): boolean {
  if (!target) return true;
  const artifact = targetComponents(target);
  return (
    artifact.architecture === (HOST_ARCHITECTURES[host.architecture] ?? host.architecture) &&
    artifact.system === (HOST_OPERATING_SYSTEMS[host.platform] ?? host.platform)
  );
}

/** The host Cargo actually built, exactly as the record holds it. Never derived, never guessed. */
export function recordedHostExecutable(artifacts: NativeArtifacts): string | undefined {
  return artifacts.host?.command ?? artifacts.native?.executable;
}

/**
 * Where another profile's executable *would* live, from the record's own Cargo facts
 * (`targetDirectory`, target triple, binary name). This is an expectation, not proof: custom Cargo
 * configuration can place artifacts elsewhere, so verify it exists (or build that profile) before
 * launching or shipping it. Use {@link recordedHostExecutable} when the built host itself is meant.
 */
export function expectedExecutablePath(
  artifacts: NativeArtifacts,
  profile = artifacts.native?.profile,
): string | undefined {
  const native = artifacts.native;
  if (!native?.bin || !native.targetDirectory || !profile) return undefined;
  const extension = (native.target ?? (process.platform === "win32" ? "windows" : "")).includes("windows")
    ? ".exe"
    : "";
  return join(
    native.targetDirectory,
    ...(native.target ? [native.target] : []),
    cargoProfileDirectory(profile),
    `${native.bin}${extension}`,
  );
}

/**
 * The target Cargo actually built for. `--target` is not the only way to cross compile:
 * `CARGO_BUILD_TARGET` and `build.target` in `.cargo/config.toml` do it invisibly, so the
 * effective triple is read from where Cargo wrote the artifact rather than from the options.
 */
export function artifactTargetTriple(executable: string, targetDirectory: string): string | undefined {
  const from = resolve(targetDirectory);
  const to = resolve(executable);
  if (!to.startsWith(from + sep)) return undefined;
  const segments = to.slice(from.length + 1).split(sep);
  return segments.length > 2 ? segments[0] : undefined;
}

/** Prepares derive every generated path from the Vite root, so nothing else has to guess it. */
export function generatedDirectory(root: string): string {
  return join(root, ".solid-gpui");
}

export function artifactsFile(root: string): string {
  return join(generatedDirectory(root), "artifacts.json");
}

export function tsconfigFile(root: string): string {
  return join(generatedDirectory(root), "tsconfig.json");
}

/** A generated file that is not what its inputs produce right now. */
export class StaleGeneratedFileError extends Error {
  constructor(
    readonly path: string,
    description: string,
  ) {
    super(`${description} is stale or missing: ${path}`);
    this.name = "StaleGeneratedFileError";
  }
}

/**
 * Publish generated content atomically, and only when it differs. In check mode nothing is
 * written: a difference is reported instead, so verification never hides staleness by writing.
 */
export async function publishGeneratedFile(
  path: string,
  content: string,
  options: { readonly check?: boolean; readonly description?: string; readonly signal?: AbortSignal } = {},
): Promise<"unchanged" | "written"> {
  const { check, description = "Generated file", signal } = options;
  const previous = await readFileIfPresent(path);
  if (previous === content) return "unchanged";
  if (check) throw new StaleGeneratedFileError(path, description);
  signal?.throwIfAborted();
  await mkdir(dirname(path), { recursive: true });
  const temporary = `${path}.${randomUUID()}.tmp`;
  try {
    signal?.throwIfAborted();
    await writeFile(temporary, content, "utf8");
    signal?.throwIfAborted();
    await rename(temporary, path);
  } finally {
    await rm(temporary, { force: true });
  }
  return "written";
}

async function readFileIfPresent(path: string): Promise<string | undefined> {
  try {
    return await readFile(path, "utf8");
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === "ENOENT") return undefined;
    throw error;
  }
}

export function serializeNativeArtifacts(artifacts: NativeArtifacts): string {
  return `${JSON.stringify(artifacts, null, 2)}\n`;
}

export async function writeNativeArtifacts(artifacts: NativeArtifacts): Promise<"unchanged" | "written"> {
  return publishGeneratedFile(artifactsFile(artifacts.root), serializeNativeArtifacts(artifacts), {
    description: "Prepared native artifacts",
  });
}

/**
 * Read the record written by `prepare` or a production build. The fields a consumer acts on are
 * validated, so a corrupted file is reported with the command that recreates it instead of
 * surfacing later as an obscure preview failure.
 */
export async function readNativeArtifacts(root: string): Promise<NativeArtifacts | undefined> {
  const path = artifactsFile(root);
  const content = await readFileIfPresent(path);
  if (content === undefined) return undefined;
  let parsed: unknown;
  try {
    parsed = JSON.parse(content);
  } catch {
    throw invalidArtifacts(path, "it is not valid JSON");
  }
  assertArtifactRecord(parsed, path);
  if (realDirectory(resolve(parsed.root)) !== realDirectory(resolve(root))) {
    throw invalidArtifacts(path, `it records the root ${parsed.root}`);
  }
  return parsed;
}

function invalidArtifacts(path: string, reason: string): Error {
  return new Error(`${path} is not a usable artifact record (${reason}); regenerate it with \`solid-gpui prepare\``);
}

/** Only the fields consumers read are checked; this is not a schema, just a boundary. */
function assertArtifactRecord(value: unknown, path: string): asserts value is NativeArtifacts {
  if (!value || typeof value !== "object") throw invalidArtifacts(path, "it is not an object");
  const record = value as Record<string, unknown>;
  if (record.version !== 1) throw invalidArtifacts(path, "it was written by another tool version");
  for (const field of ["root", "runtime", "entry", "outDir", "tsconfig"] as const) {
    if (typeof record[field] !== "string" || record[field] === "") {
      throw invalidArtifacts(path, `its ${field} is missing or not a string`);
    }
  }
  const runtime = record.runtime;
  if (runtime !== "web" && runtime !== "bun" && runtime !== "quickjs") {
    throw invalidArtifacts(path, `its runtime "${String(runtime)}" is not one of web, bun or quickjs`);
  }
  for (const field of ["bindings", "bundle"] as const) {
    if (record[field] !== undefined && typeof record[field] !== "string") {
      throw invalidArtifacts(path, `its ${field} is not a string`);
    }
  }
  const host = record.host;
  if (host !== undefined) {
    if (!host || typeof host !== "object") throw invalidArtifacts(path, "its host entry is not an object");
    const entry = host as Record<string, unknown>;
    if (typeof entry.command !== "string" || entry.command === "") {
      throw invalidArtifacts(path, "its host entry has no command");
    }
    if (entry.args !== undefined && !Array.isArray(entry.args)) {
      throw invalidArtifacts(path, "its host args are not an array");
    }
  }
  const native = record.native;
  if (native !== undefined) {
    if (!native || typeof native !== "object") throw invalidArtifacts(path, "its native entry is not an object");
    const entry = native as Record<string, unknown>;
    for (const field of ["manifestPath", "profile", "profileDirectory"] as const) {
      if (typeof entry[field] !== "string" || entry[field] === "") {
        throw invalidArtifacts(path, `its native.${field} is missing or not a string`);
      }
    }
    if (typeof entry.locked !== "boolean") throw invalidArtifacts(path, "its native.locked is not a boolean");
    if (!Array.isArray(entry.features)) throw invalidArtifacts(path, "its native.features is not an array");
    for (const field of ["package", "bin", "target", "targetDirectory", "executable"] as const) {
      if (entry[field] !== undefined && typeof entry[field] !== "string") {
        throw invalidArtifacts(path, `its native.${field} is not a string`);
      }
    }
  }
}

/** A record and its project can be reached through different symlinks; compare the real targets. */
function realDirectory(path: string): string {
  try {
    return realpathSync(path);
  } catch {
    return path;
  }
}
