import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join, resolve } from "node:path";
import type { Plugin } from "vite";

/** Conditional export that selects a package's TypeScript sources instead of its build output. */
export const SOURCE_CONDITION = "solid-gpui-source";

/** Published packages that declare `solid-gpui-source` exports. */
export const SDK_PACKAGES = [
  "@solid-gpui/core",
  "@solid-gpui/vite",
  "@solid-gpui/router",
  "@solid-gpui/shiki",
] as const;

const DEDUPE = ["solid-js"] as const;
const SOURCE_MAPPINGS = Symbol.for("solid-gpui.source.mappings");

interface SourcePlugin extends Plugin {
  readonly [SOURCE_MAPPINGS]?: Readonly<Record<string, string>>;
}

/**
 * Path mappings a source-mode plugin publishes, for the generated tsconfig. Tooling that writes
 * `compilerOptions.paths` reads this from the configured plugin list, so editor resolution and the
 * runtime aliases share one table instead of a hand-maintained copy.
 */
export function pluginSourceMappings(plugin: unknown): Readonly<Record<string, string>> | undefined {
  return plugin !== null && typeof plugin === "object" ? (plugin as SourcePlugin)[SOURCE_MAPPINGS] : undefined;
}

/**
 * The `solidGpui` plugin rewrites `@solid-gpui/core/components` to the host bindings generated for
 * the selected host, so source mappings leave that specifier to the plugin (and to the generated
 * tsconfig, which points it at the same file). Web-only apps pass `exclude: []`.
 */
const HOST_OWNED_SPECIFIERS = ["@solid-gpui/core/components"] as const;

export interface SdkSourceOptions {
  /** Directory the SDK packages are resolved from. Defaults to the current working directory. */
  readonly root?: string;
  /** Package names to map. Defaults to {@link SDK_PACKAGES}; uninstalled names are skipped. */
  readonly packages?: readonly string[];
  /** Specifiers left to other tooling. Defaults to the host-owned bindings entry point. */
  readonly exclude?: readonly string[];
}

export interface SdkPackageInfo {
  readonly name: string;
  readonly directory: string;
  readonly version: string;
  readonly manifest: Readonly<Record<string, unknown>>;
}

export interface SdkSourceEntry {
  readonly specifier: string;
  readonly source: string;
}

export interface SdkSourcePackage {
  readonly name: string;
  readonly directory: string;
  readonly entries: readonly SdkSourceEntry[];
}

export interface SdkSourceAlias {
  readonly find: string;
  readonly replacement: string;
}

export interface SdkSource {
  readonly condition: string;
  readonly root: string;
  readonly packages: readonly SdkSourcePackage[];
  /** Longest specifier first, so a subpath entry wins over its package root. */
  readonly aliases: readonly SdkSourceAlias[];
  /** Specifier to absolute source file: the table shared with the generated tsconfig `paths`. */
  readonly paths: Readonly<Record<string, string>>;
  readonly dedupe: readonly string[];
  /** Mapped package names, for `optimizeDeps.exclude`. */
  readonly names: readonly string[];
}

function resolveFrom(specifier: string, root: string): string | undefined {
  try {
    return Bun.resolveSync(specifier, root);
  } catch {
    // Fall through to Node's resolver for a non-Bun host.
  }
  try {
    return createRequire(join(root, "package.json")).resolve(specifier);
  } catch {
    return undefined;
  }
}

function readManifest(directory: string): Readonly<Record<string, unknown>> | undefined {
  try {
    return JSON.parse(readFileSync(join(directory, "package.json"), "utf8")) as Record<string, unknown>;
  } catch {
    return undefined;
  }
}

/** Locate an installed package by walking up from its resolved entry point to its own manifest. */
export function sdkPackage(name: string, root?: string): SdkPackageInfo | undefined {
  const from = resolve(root ?? process.cwd());
  const entry = resolveFrom(name, from);
  if (entry === undefined) return undefined;
  let directory = dirname(entry);
  for (;;) {
    const manifest = readManifest(directory);
    if (manifest?.name === name) {
      return {
        name,
        directory,
        version: typeof manifest.version === "string" ? manifest.version : "0.0.0",
        manifest,
      };
    }
    const parent = dirname(directory);
    if (parent === directory) return undefined;
    directory = parent;
  }
}

/** The runtime file a `solid-gpui-source` condition (or its nested types/import map) selects. */
function sourceTarget(value: unknown): string | undefined {
  if (typeof value === "string") return value;
  if (value === null || typeof value !== "object") return undefined;
  const record = value as Record<string, unknown>;
  for (const key of ["import", "default", "source", "types"]) {
    const target = sourceTarget(record[key]);
    if (target !== undefined) return target;
  }
  return undefined;
}

function packageEntries(info: SdkPackageInfo): SdkSourceEntry[] {
  const exportsField = info.manifest.exports;
  if (exportsField === null || typeof exportsField !== "object") return [];
  const entries: SdkSourceEntry[] = [];
  for (const [key, value] of Object.entries(exportsField as Record<string, unknown>)) {
    if (key !== "." && !key.startsWith("./")) continue;
    const conditions = value === null || typeof value !== "object" ? undefined : (value as Record<string, unknown>);
    const target = sourceTarget(conditions?.[SOURCE_CONDITION]);
    if (target === undefined) continue;
    entries.push({
      specifier: key === "." ? info.name : `${info.name}/${key.slice(2)}`,
      source: resolve(info.directory, target),
    });
  }
  return entries;
}

/**
 * Derive source-mode mappings from the installed packages' own export maps, so consumers never
 * hand-maintain an alias or `paths` table. Mapping a package contributes its declared public
 * subpaths whose export map carries a {@link SOURCE_CONDITION} entry.
 */
export function sdkSource(options: SdkSourceOptions = {}): SdkSource {
  const root = resolve(options.root ?? process.cwd());
  const excluded = new Set(options.exclude ?? HOST_OWNED_SPECIFIERS);
  const packages: SdkSourcePackage[] = [];
  for (const name of options.packages ?? SDK_PACKAGES) {
    const info = sdkPackage(name, root);
    if (info === undefined) continue;
    const entries = packageEntries(info).filter((entry) => !excluded.has(entry.specifier));
    if (entries.length > 0) packages.push({ name, directory: info.directory, entries });
  }
  const mapped = packages.flatMap((pkg) => pkg.entries);
  const paths: Record<string, string> = {};
  for (const entry of mapped) paths[entry.specifier] = entry.source;
  return {
    condition: SOURCE_CONDITION,
    root,
    packages,
    aliases: Object.entries(paths)
      .sort(([left], [right]) => right.length - left.length)
      .map(([find, replacement]) => ({ find, replacement })),
    paths,
    dedupe: DEDUPE,
    names: packages.map((pkg) => pkg.name),
  };
}

/**
 * Resolve SDK imports to the packages' TypeScript sources for the explicit source workflow.
 *
 * Consumers enable it by adding this plugin, and agree with the runtime in one of two ways:
 * the {@link SOURCE_CONDITION} condition (`tsconfig` `customConditions` plus `bun --conditions`)
 * or the aliases/paths returned by {@link sdkSource}. Ordinary dist consumption needs neither.
 * The plugin publishes {@link pluginSourceMappings} so `prepare` writes the same table into the
 * generated tsconfig.
 */
export function solidGpuiSource(options: SdkSourceOptions = {}): Plugin {
  const source = sdkSource(options);
  const plugin: Plugin = {
    name: "solid-gpui-source",
    enforce: "pre",
    config() {
      return {
        resolve: { conditions: [source.condition], dedupe: [...source.dedupe] },
        optimizeDeps: { exclude: [...source.names] },
      };
    },
  };
  Object.defineProperty(plugin, SOURCE_MAPPINGS, {
    value: source.paths,
    enumerable: false,
    configurable: true,
  });
  return plugin;
}
