import { readFile, readdir } from "node:fs/promises";
import type { Dirent } from "node:fs";
import { dirname, join, resolve, sep } from "node:path";
import { runNativeCommand } from "./native-process.ts";
import type { NativeExportOptions } from "./native-export.ts";

/** A local Cargo package whose changes require another native build. */
export interface NativeWatchPackage {
  readonly name: string;
  readonly directory: string;
}

export interface NativeWatchInputs {
  /** Local Cargo package roots: sources and embedded assets are tracked under these. */
  readonly roots: readonly string[];
  /** Cargo configuration that changes the whole build graph. */
  readonly configuration: readonly string[];
  /** Paths Cargo itself tracks, from its dep-info: build-script inputs and included assets. */
  readonly declared: readonly string[];
  /** Explicit `native.watch` paths and directories. */
  readonly explicit: readonly string[];
  readonly packages: readonly NativeWatchPackage[];
  /** Cargo fingerprint directories, re-read after each build without another Cargo call. */
  readonly fingerprints: readonly string[];
  readonly targetDirectory?: string;
}

const NATIVE_INPUT_EXTENSIONS =
  /\.(?:rs|svg|png|jpe?g|gif|webp|ico|bmp|ttf|otf|ttc|woff2?|wgsl|glsl|vert|frag|metal|h|c|cc|cpp|hpp|m|mm)$/i;

const NATIVE_CONFIGURATION_FILE = /(?:^|[/\\])(?:Cargo\.(?:toml|lock)|\.cargo[/\\]config(?:\.toml)?)$/;

const CRATE_NAME_SUFFIX = /-[0-9a-f]{16}$/;

/** A build script's fingerprint: it holds the inputs Cargo knows but rustc dep-info does not. */
const BUILD_SCRIPT_FINGERPRINT = /^run-build-script-.*\.json$/;

async function readDirectory(path: string): Promise<Dirent[]> {
  try {
    return await readdir(path, { withFileTypes: true });
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === "ENOENT") return [];
    throw error;
  }
}

/** Cargo writes profile and cross-target fingerprints side by side; all of them are inputs. */
async function fingerprintDirectories(targetDirectory: string): Promise<string[]> {
  const bases = [targetDirectory];
  for (const entry of await readDirectory(targetDirectory)) {
    if (entry.isDirectory() && entry.name.includes("-")) bases.push(join(targetDirectory, entry.name));
  }
  const directories: string[] = [];
  for (const base of bases) {
    for (const entry of await readDirectory(base)) {
      if (entry.isDirectory()) directories.push(join(base, entry.name, ".fingerprint"));
    }
  }
  return directories;
}

/**
 * Cargo's dep-info files are the authoritative list of a compilation's inputs: they include
 * build-script `rerun-if-changed` paths and `include_str!`/`include_bytes!` assets, which no
 * manifest exposes. Entries are `\0` + u32le length + UTF-8 path, and only local packages are
 * interesting because registry sources cannot change.
 */
function depInfoInputs(content: Buffer, directory: string): string[] {
  const inputs: string[] = [];
  for (let index = 0; index + 5 <= content.length; index++) {
    if (content[index] !== 0) continue;
    const length = content.readUInt32LE(index + 1);
    if (length === 0 || length > 4096 || index + 5 + length > content.length) continue;
    const text = content.subarray(index + 5, index + 5 + length).toString("utf8");
    if (!text || text.includes("\uFFFD")) continue;
    inputs.push(resolve(directory, text));
    index += 4 + length;
  }
  return inputs;
}

async function declaredNativeInputs(
  fingerprints: readonly string[],
  packages: readonly NativeWatchPackage[],
): Promise<string[]> {
  const directories = new Map(packages.map((entry) => [entry.name.replaceAll("_", "-"), entry.directory]));
  const declared = new Set<string>();
  for (const fingerprint of fingerprints) {
    for (const entry of await readDirectory(fingerprint)) {
      if (!entry.isDirectory()) continue;
      const directory = directories.get(entry.name.replace(CRATE_NAME_SUFFIX, "").replaceAll("_", "-"));
      if (!directory) continue;
      for (const file of await readDirectory(join(fingerprint, entry.name))) {
        const depInfo = file.name.startsWith("dep-");
        if (!depInfo && !BUILD_SCRIPT_FINGERPRINT.test(file.name)) continue;
        const content = await readFile(join(fingerprint, entry.name, file.name)).catch((error) => {
          if ((error as NodeJS.ErrnoException).code === "ENOENT") return undefined;
          throw error;
        });
        if (!content) continue;
        for (const input of depInfo ? depInfoInputs(content, directory) : rerunIfChangedPaths(content, directory)) {
          declared.add(input);
        }
      }
    }
  }
  return [...declared];
}

/**
 * A build script's own inputs, which have no dep-info: Cargo stores the paths it was told to
 * rerun on in the fingerprint of the running build script. They may be files or whole directories.
 */
function rerunIfChangedPaths(content: Buffer, directory: string): string[] {
  let parsed: unknown;
  try {
    parsed = JSON.parse(content.toString("utf8"));
  } catch {
    return [];
  }
  if (!parsed || typeof parsed !== "object" || !("local" in parsed) || !Array.isArray(parsed.local)) return [];
  const paths: string[] = [];
  for (const entry of parsed.local) {
    if (!entry || typeof entry !== "object" || !("RerunIfChanged" in entry)) continue;
    const directive = entry.RerunIfChanged;
    if (!directive || typeof directive !== "object" || !("paths" in directive) || !Array.isArray(directive.paths)) {
      continue;
    }
    for (const path of directive.paths) {
      if (typeof path === "string" && path !== "") paths.push(resolve(directory, path));
    }
  }
  return paths;
}

/**
 * Re-read Cargo's tracked inputs after a build. Fingerprint directories are discovered again here,
 * not just re-read: the first build creates them, so a session that started on a clean target would
 * otherwise never see any of Cargo's own inputs.
 */
export async function refreshNativeWatchInputs(inputs: NativeWatchInputs): Promise<NativeWatchInputs> {
  const fingerprints = inputs.targetDirectory
    ? await fingerprintDirectories(inputs.targetDirectory)
    : inputs.fingerprints;
  return { ...inputs, fingerprints, declared: await declaredNativeInputs(fingerprints, inputs.packages) };
}

/**
 * Declared inputs the watcher does not already cover. A build script may track a file or a whole
 * directory outside every package root, so those paths have to be watched themselves; a directory
 * already covers the declared files inside it.
 */
export function externalWatchInputs(inputs: NativeWatchInputs): string[] {
  const external = inputs.declared.filter(
    (path) =>
      !inputs.configuration.includes(path) &&
      !inputs.explicit.some((entry) => path === entry || path.startsWith(entry + sep)) &&
      !inputs.roots.some((root) => path.startsWith(root + sep)),
  );
  return external.filter((path) => !external.some((other) => other !== path && path.startsWith(other + sep)));
}

/**
 * Cargo decides where artifacts land, through configuration this process cannot see. Asking an
 * unresolved (dependency-free) query keeps it cheap and never touches the lockfile.
 */
export async function cargoTargetDirectory(manifestPath: string, cwd: string, signal?: AbortSignal): Promise<string> {
  const { stdout } = await runNativeCommand(
    "cargo",
    ["metadata", "--format-version", "1", "--no-deps", "--manifest-path", resolve(cwd, manifestPath)],
    cwd,
    signal,
  );
  return resolve((JSON.parse(stdout) as { target_directory: string }).target_directory);
}

/** Watch local Cargo inputs, including path dependencies outside the Vite root. */
export async function nativeWatchInputs(
  options: NativeExportOptions,
  cwd: string,
  signal: AbortSignal,
): Promise<NativeWatchInputs> {
  const { stdout } = await runNativeCommand(
    "cargo",
    [
      "metadata",
      "--format-version",
      "1",
      "--manifest-path",
      resolve(cwd, options.manifestPath),
      ...(options.locked === false ? [] : ["--locked"]),
      ...(options.features?.length
        ? [
            "--features",
            options.features
              .map((feature) => (options.package && !feature.includes("/") ? `${options.package}/${feature}` : feature))
              .join(","),
          ]
        : []),
    ],
    cwd,
    signal,
  );
  const metadata = JSON.parse(stdout) as {
    workspace_root: string;
    target_directory: string;
    workspace_default_members: string[];
    packages: { id: string; name: string; source: string | null; manifest_path: string }[];
    resolve: { root: string | null; nodes: { id: string; dependencies: string[] }[] };
  };
  const pending = options.package
    ? metadata.packages.filter((entry) => entry.name === options.package).map((entry) => entry.id)
    : metadata.resolve.root
      ? [metadata.resolve.root]
      : [...metadata.workspace_default_members];
  const dependencies = new Map(metadata.resolve.nodes.map((entry) => [entry.id, entry.dependencies]));
  const reachable = new Set<string>();
  while (pending.length) {
    const id = pending.pop()!;
    if (reachable.has(id)) continue;
    reachable.add(id);
    pending.push(...(dependencies.get(id) ?? []));
  }
  const packages = metadata.packages
    .filter((entry) => !entry.source && reachable.has(entry.id))
    .map((entry) => ({ name: entry.name, directory: resolve(dirname(entry.manifest_path)) }));
  const targetDirectory = resolve(metadata.target_directory);
  const fingerprints = await fingerprintDirectories(targetDirectory);
  return {
    roots: packages.map((entry) => entry.directory),
    configuration: [
      ...["Cargo.toml", "Cargo.lock", ".cargo/config", ".cargo/config.toml"].map((file) =>
        join(metadata.workspace_root, file),
      ),
      join(cwd, ".cargo/config"),
      join(cwd, ".cargo/config.toml"),
    ],
    declared: await declaredNativeInputs(fingerprints, packages),
    explicit: [...(options.watch ?? [])].map((path) => resolve(cwd, path)),
    packages,
    fingerprints,
    targetDirectory,
  };
}

/**
 * Whether a changed file must rebuild the native host. Cargo's own inputs come first, then
 * asset extensions under local package roots; everything Vite compiles stays with HMR.
 */
export function isNativeInput(file: string, inputs: NativeWatchInputs): boolean {
  if (inputs.configuration.includes(file)) return true;
  // A declared input can be a directory: build scripts may track a whole resource tree.
  if (inputs.declared.some((entry) => file === entry || file.startsWith(entry + sep))) return true;
  if (inputs.explicit.some((path) => file === path || file.startsWith(path + sep))) return true;
  return (
    inputs.roots.some((root) => file.startsWith(root + sep)) &&
    (NATIVE_INPUT_EXTENSIONS.test(file) || NATIVE_CONFIGURATION_FILE.test(file))
  );
}
