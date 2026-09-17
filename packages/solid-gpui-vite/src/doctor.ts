import { existsSync, readFileSync, realpathSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { targetRunsOnHost } from "./artifacts.ts";
import { loadProject, type SolidGpuiProject } from "./project.ts";
import { SDK_PACKAGES, SOURCE_CONDITION, sdkPackage, sdkSource, type SdkSource } from "./source.ts";

/** Worst-first severity of a single check. */
export type DoctorStatus = "pass" | "warn" | "fail";

export type DoctorCheckId =
  | "vite-config"
  | "packages"
  | "solid-instance"
  | "source-mapping"
  | "host-package"
  | "cargo-graph"
  | "cargo-patches"
  | "cargo-profiles"
  | "runtime-capabilities";

export interface DoctorCheck {
  readonly id: DoctorCheckId;
  readonly status: DoctorStatus;
  readonly summary: string;
  readonly details?: readonly string[];
  /** The concrete edit that resolves a failed or warned check. */
  readonly hint?: string;
}

export interface DoctorReport {
  readonly root: string;
  /** Which SDK artifacts the runtime resolves: published `dist` or in-package sources. */
  readonly mode: "dist" | "source";
  readonly status: DoctorStatus;
  readonly checks: readonly DoctorCheck[];
}

/** The project facts doctor reads. A full `SolidGpuiProject` satisfies this. */
export interface DoctorConfig {
  readonly root?: string;
  readonly web?: boolean;
  readonly runtime?: "bun" | "quickjs";
  readonly entry?: string;
  /** Generated bindings path; `native.output` wins when both are present. */
  readonly bindings?: string;
  readonly native?: SolidGpuiProject["native"];
}

export interface DoctorOptions {
  readonly root?: string;
  readonly configFile?: string;
  /** A project from `loadProject` (or the same facts), instead of loading the config again. */
  readonly config?: DoctorConfig;
  readonly mode?: "dist" | "source";
  /** Native Cargo manifest, when no project is supplied. */
  readonly nativeManifest?: string;
}

const SEVERITY: Record<DoctorStatus, number> = { pass: 0, warn: 1, fail: 2 };
const REQUIRED_PACKAGES = ["@solid-gpui/core", "@solid-gpui/vite"] as const;
const PLATFORM_PATCH: Partial<Record<NodeJS.Platform, string>> = {
  darwin: "gpui-pre-macos",
  win32: "gpui-pre-windows",
  linux: "gpui-pre-linux",
};
/** Dev-profile entries that keep debug UI responsive; the QuickJS one is correctness-critical. */
const PERF_PROFILE_PACKAGES = ["gpui-pre", "taffy", "slotmap"] as const;
const QUICKJS_PROFILE_PACKAGE = "rquickjs-sys";
/**
 * The source workflow is declared by the condition itself, by the plugin factory, or by the
 * `sdkSource()` aliases; the resolved Vite config does not otherwise expose plugin internals.
 */
const SOURCE_PLUGIN = /solid-gpui-source|solidGpuiSource|sdkSource/;

interface TomlDocument {
  readonly path: string;
  readonly directory: string;
  readonly document: Record<string, unknown>;
}

interface CargoContext {
  readonly crate: TomlDocument;
  readonly workspace: TomlDocument;
  /** Manifests Cargo reads `[patch]`/`[profile]` from for this build: the workspace root and `.cargo/config.toml`. */
  readonly effective: readonly TomlDocument[];
  /** The SDK checkout's workspace manifest, when a path dependency on `solid-gpui` leads to it. */
  readonly sdk: TomlDocument | undefined;
}

interface CargoGraph {
  /** Packages Cargo took from a path, i.e. patches that took effect. */
  readonly patched: Readonly<Record<string, true>>;
  readonly packageNames: readonly string[];
  /** Features enabled on the resolved `solid-gpui` node. */
  readonly sdkFeatures: readonly string[];
}

interface DoctorContext {
  readonly root: string;
  readonly mode: "dist" | "source";
  readonly runtime: "bun" | "quickjs";
  readonly web: boolean;
  readonly configFile: string;
  readonly nativeOutput: string | undefined;
  readonly nativeTarget: string | undefined;
  /** An explicitly supplied host exporter, which admits a cross target. */
  readonly nativeExporter: boolean;
  readonly nativeManifest: string | undefined;
  readonly cargo: CargoContext | undefined;
  /** The resolved Cargo graph, when `cargo metadata --offline` succeeded. */
  readonly graph: CargoGraph | undefined;
  /** The Vite config declares the source condition. */
  readonly declaresSource: boolean;
  /** TypeScript resolves SDK sources: `customConditions` or a real SDK source mapping. */
  readonly typesAgree: boolean;
  /** The installed packages' source table, derived once for the mapping checks. */
  readonly mapped: SdkSource;
}

function table(value: unknown): Record<string, unknown> | undefined {
  return value !== null && typeof value === "object" && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : undefined;
}

function check(
  id: DoctorCheckId,
  status: DoctorStatus,
  summary: string,
  extras: { readonly details?: readonly string[]; readonly hint?: string } = {},
): DoctorCheck {
  return { id, status, summary, ...extras };
}

function messageOf(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function readJson(path: string): Record<string, unknown> | undefined {
  try {
    return JSON.parse(readFileSync(path, "utf8")) as Record<string, unknown>;
  } catch {
    return undefined;
  }
}

function readToml(path: string): TomlDocument | undefined {
  if (!existsSync(path)) return undefined;
  try {
    return {
      path,
      directory: dirname(path),
      document: Bun.TOML.parse(readFileSync(path, "utf8")) as Record<string, unknown>,
    };
  } catch {
    return undefined;
  }
}

function realpath(path: string): string {
  try {
    return realpathSync(path);
  } catch {
    return path;
  }
}

function satisfies(version: string, range: unknown): boolean {
  if (typeof range !== "string") return true;
  if (range === "*" || range === "" || range.startsWith("workspace:") || range.startsWith("file:")) return true;
  try {
    return Bun.semver.satisfies(version, range);
  } catch {
    return true;
  }
}

function findWorkspace(crate: TomlDocument): TomlDocument | undefined {
  let directory = crate.directory;
  for (;;) {
    const candidate = readToml(join(directory, "Cargo.toml"));
    const members = table(candidate?.document.workspace)?.members;
    if (candidate !== undefined && table(candidate.document.workspace) !== undefined) {
      const declaresCrate =
        candidate.path === crate.path ||
        (Array.isArray(members) &&
          members.some(
            (member) => typeof member === "string" && resolve(candidate.directory, member) === crate.directory,
          ));
      if (declaresCrate) return candidate;
    }
    const parent = dirname(directory);
    if (parent === directory) return undefined;
    directory = parent;
  }
}

/** Every dependency table of a manifest with the directory its `path` values are relative to. */
function dependencyTables(document: TomlDocument): readonly { readonly entries: unknown; readonly base: string }[] {
  return [
    { entries: table(document.document.workspace)?.dependencies, base: document.directory },
    { entries: document.document.dependencies, base: document.directory },
    { entries: document.document["dev-dependencies"], base: document.directory },
    { entries: document.document["build-dependencies"], base: document.directory },
  ];
}

function solidGpuiPath(documents: readonly TomlDocument[]): string | undefined {
  for (const document of documents) {
    for (const { entries, base } of dependencyTables(document)) {
      for (const [key, value] of Object.entries(table(entries) ?? {})) {
        const spec = table(value);
        if (typeof spec?.path !== "string") continue;
        if ((typeof spec.package === "string" ? spec.package : key) === "solid-gpui") return resolve(base, spec.path);
      }
    }
  }
  return undefined;
}

function cargoContext(nativeManifest: string): CargoContext | undefined {
  const crate = readToml(nativeManifest);
  if (crate === undefined) return undefined;
  const workspace = findWorkspace(crate) ?? crate;
  const read = (directory: string): readonly (TomlDocument | undefined)[] => [
    readToml(join(directory, ".cargo/config.toml")),
    readToml(join(directory, ".cargo/config")),
  ];
  const candidates = [workspace, ...read(workspace.directory), ...read(crate.directory)].filter(
    (entry): entry is TomlDocument => entry !== undefined,
  );
  const effective = candidates.filter(
    (entry, index) => candidates.findIndex((other) => other.path === entry.path) === index,
  );
  const sdkCrate = solidGpuiPath([workspace, crate]);
  let sdk: TomlDocument | undefined;
  for (let directory = sdkCrate === undefined ? undefined : sdkCrate; directory !== undefined;) {
    const candidate = readToml(join(directory, "Cargo.toml"));
    if (candidate !== undefined && table(candidate.document.workspace) !== undefined) {
      sdk = candidate;
      break;
    }
    const parent = dirname(directory);
    directory = parent === directory ? undefined : parent;
  }
  return { crate, workspace, effective, sdk };
}

/** `[patch.crates-io]` entries resolved to absolute paths across the manifests Cargo reads. */
function patchEntries(documents: readonly TomlDocument[]): Readonly<Record<string, string>> {
  const entries: Record<string, string> = {};
  for (const document of documents) {
    for (const [name, value] of Object.entries(table(table(document.document.patch)?.["crates-io"]) ?? {})) {
      const spec = table(value);
      if (typeof spec?.path === "string") entries[name] = resolve(document.directory, spec.path);
    }
  }
  return entries;
}

function profileTable(documents: readonly TomlDocument[], name: string): Record<string, unknown> | undefined {
  for (const document of documents) {
    const profile = table(table(document.document.profile)?.[name]);
    if (profile !== undefined) return profile;
  }
  return undefined;
}

function optLevel(dev: Record<string, unknown> | undefined, pkg: string): number | undefined {
  const value = table(table(dev?.package)?.[pkg])?.["opt-level"];
  return typeof value === "number" ? value : undefined;
}

function isPlatformPatch(name: string): boolean {
  return Object.values(PLATFORM_PATCH).includes(name);
}

function cargoUnavailable(context: DoctorContext, id: DoctorCheckId): DoctorCheck {
  return context.cargo === undefined && context.nativeManifest === undefined
    ? check(id, "pass", "no Cargo host configured; nothing to verify")
    : check(id, "warn", `skipped: no readable Cargo manifest at ${context.nativeManifest ?? context.root}`);
}

function readCargoGraph(cargo: CargoContext): { readonly check: DoctorCheck; readonly graph?: CargoGraph } {
  let stdout: string;
  let stderr: string;
  let exitCode: number;
  try {
    const result = Bun.spawnSync(
      ["cargo", "metadata", "--format-version", "1", "--offline", "--manifest-path", cargo.crate.path],
      { cwd: cargo.crate.directory, stdout: "pipe", stderr: "pipe", timeout: 120_000 },
    );
    stdout = result.stdout.toString();
    stderr = result.stderr.toString();
    exitCode = result.exitCode ?? 1;
  } catch (error) {
    return {
      check: check("cargo-graph", "warn", `cargo is unavailable: ${messageOf(error)}`, {
        hint: "install the Rust toolchain to compare the declared patches, profiles and features against the resolved graph",
      }),
    };
  }
  if (exitCode !== 0) {
    return {
      check: check("cargo-graph", "warn", "`cargo metadata --offline` failed; skipped resolved-graph verification", {
        details: stderr.trim().split("\n").slice(0, 3),
        hint: `run \`cargo fetch --locked\` in ${cargo.crate.directory} so the graph can be resolved offline`,
      }),
    };
  }
  let metadata: {
    readonly packages?: readonly {
      readonly id: string;
      readonly name: string;
      readonly source: string | null;
      readonly manifest_path: string;
    }[];
    readonly resolve?: { readonly nodes?: readonly { readonly id: string; readonly features?: readonly string[] }[] };
  };
  try {
    metadata = JSON.parse(stdout);
  } catch (error) {
    return { check: check("cargo-graph", "warn", `could not read cargo metadata: ${messageOf(error)}`) };
  }
  const packages = metadata.packages ?? [];
  const byId = new Map(packages.map((pkg) => [pkg.id, pkg]));
  const patched: Record<string, true> = {};
  for (const pkg of packages) {
    if (pkg.source === null) patched[pkg.name] = true;
  }
  // Resolve node ids are package ids, not manifest paths; only feature sets matter here.
  const featuresByName = new Map<string, readonly string[]>();
  for (const node of metadata.resolve?.nodes ?? []) {
    const pkg = byId.get(node.id);
    if (pkg === undefined || pkg.source !== null) continue;
    featuresByName.set(pkg.name, node.features ?? []);
  }
  return {
    check: check("cargo-graph", "pass", `resolved ${packages.length} Cargo packages from ${cargo.crate.path}`),
    graph: {
      patched,
      packageNames: packages.map((pkg) => pkg.name),
      sdkFeatures: featuresByName.get("solid-gpui") ?? [],
    },
  };
}

function packagesCheck(context: DoctorContext): DoctorCheck {
  const installed = SDK_PACKAGES.map((name) => sdkPackage(name, context.root)).filter(
    (info): info is NonNullable<typeof info> => info !== undefined,
  );
  const missing = REQUIRED_PACKAGES.filter((name) => !installed.some((info) => info.name === name));
  if (missing.length > 0) {
    return check("packages", "fail", `required SDK package(s) not installed: ${missing.join(", ")}`, {
      hint: `bun add ${missing.join(" ")}`,
    });
  }
  const details = installed.map((info) => `${info.name}@${info.version} at ${info.directory}`);
  const tooling = installed.find((info) => info.name === "@solid-gpui/vite");
  const seen = new Set<string>();
  const violations: string[] = [];
  for (const [name, range] of [
    ...Object.entries(table(tooling?.manifest.peerDependencies) ?? {}),
    ...installed.flatMap((info) => Object.entries(table(info.manifest.peerDependencies) ?? {})),
  ]) {
    if (name === "vite" || seen.has(name)) continue;
    seen.add(name);
    const found = sdkPackage(name, context.root);
    if (found === undefined || satisfies(found.version, range)) continue;
    violations.push(`${name}@${found.version} needs ${String(range)}`);
  }
  const unresolved: string[] = [];
  for (const info of installed) {
    for (const name of Object.keys(table(info.manifest.dependencies) ?? {})) {
      if (sdkPackage(name, info.directory) === undefined) unresolved.push(`${name} (dependency of ${info.name})`);
    }
  }
  const versions = [...new Set(installed.map((info) => info.version))];
  if (versions.length > 1) details.push(`mixed SDK versions: ${versions.join(", ")}`);
  if (violations.length > 0) {
    return check("packages", "fail", violations.join("; "), {
      details,
      hint: `bun add ${violations.map((entry) => entry.split(" needs ")[0]).join(" ")} at the SDK's required ranges`,
    });
  }
  if (unresolved.length > 0) {
    return check(
      "packages",
      "fail",
      `installed SDK packages cannot resolve their own dependencies: ${unresolved.join(", ")}`,
      {
        details,
        hint: "reinstall without overrides so each SDK package keeps its declared dependencies",
      },
    );
  }
  const app = readJson(join(context.root, "package.json"));
  const appDeps = Object.keys({ ...table(app?.dependencies), ...table(app?.devDependencies) });
  if (appDeps.includes("@solidjs/compiler")) {
    return check("packages", "warn", "the application declares its own @solidjs/compiler", {
      details,
      hint: "the compiler stack ships with @solid-gpui/vite; remove the application copy so both compile with one version",
    });
  }
  return check("packages", "pass", `SDK packages resolve cleanly (${versions.join(", ")})`, { details });
}

function solidInstanceCheck(context: DoctorContext): DoctorCheck {
  const sources = [
    context.root,
    ...SDK_PACKAGES.map((name) => sdkPackage(name, context.root)?.directory).filter(
      (directory): directory is string => directory !== undefined,
    ),
  ];
  const copies = new Map<string, string[]>();
  for (const directory of sources) {
    const info = sdkPackage("solid-js", directory);
    if (info === undefined) continue;
    const key = `${info.version} (${realpath(info.directory)})`;
    copies.set(key, [...(copies.get(key) ?? []), directory]);
  }
  const found = [...copies.keys()];
  if (found.length === 0) {
    return check("solid-instance", "fail", "solid-js is not installed", { hint: "bun add solid-js@^1.9.10" });
  }
  if (found.length > 1) {
    return check("solid-instance", "fail", `the graph resolves ${found.length} distinct solid-js copies`, {
      details: found,
      hint: 'keep one solid-js version and add `resolve.dedupe: ["solid-js"]` (solidGpuiSource() sets it for source mode)',
    });
  }
  return check("solid-instance", "pass", `one solid-js instance: ${found[0]}`);
}

function tablePaths(value: unknown): Readonly<Record<string, unknown>> {
  return table(value) ?? {};
}

/**
 * A generated `paths` entry is source mode only when it points at the SDK's own source file.
 * The host bindings (`#native`, `@solid-gpui/core/components`) are not SDK mappings, so an
 * ordinary dist workflow keeps them without ever looking like a source consumer.
 */
function mapsSpecifierToSource(
  paths: Readonly<Record<string, unknown>>,
  root: string,
  specifier: string,
  source: string,
): boolean {
  const declared = paths[specifier];
  if (!Array.isArray(declared)) return false;
  return declared.some(
    (candidate) =>
      typeof candidate === "string" &&
      // `paths` values are relative to the config that declares them; prepare writes absolute ones.
      [resolve(root, ".solid-gpui", candidate), resolve(root, candidate)].some(
        (path) => realpath(path) === realpath(source),
      ),
  );
}

function sourceMappingCheck(context: DoctorContext): DoctorCheck {
  const generated = readJson(join(context.root, ".solid-gpui/tsconfig.json"));
  const generatedPaths = tablePaths(table(generated?.compilerOptions)?.paths);
  const declaresSource = context.declaresSource;
  const typesAgree = context.typesAgree;
  const mapped = context.mapped;
  const unexported = SDK_PACKAGES.filter(
    (name) => sdkPackage(name, context.root) !== undefined && !mapped.packages.some((pkg) => pkg.name === name),
  );
  const entries = mapped.packages.flatMap((pkg) => pkg.entries);
  const missingSources = context.mode === "source" ? entries.filter((entry) => !existsSync(entry.source)) : [];
  const missingBindings = ["#native", "@solid-gpui/core/components"].flatMap((specifier) => {
    const target = generatedPaths[specifier];
    if (!Array.isArray(target) || typeof target[0] !== "string") return [];
    const file = resolve(context.root, target[0]);
    return existsSync(file) ? [] : [`${specifier} -> ${file}`];
  });
  if (missingBindings.length > 0) {
    return check("source-mapping", "fail", `generated host bindings are missing: ${missingBindings.join(", ")}`, {
      hint: "run `solid-gpui generate` to write the bindings for the selected host",
    });
  }
  if (context.mode === "source" && unexported.length > 0) {
    return check(
      "source-mapping",
      "fail",
      `installed package(s) declare no \`${SOURCE_CONDITION}\` export: ${unexported.join(", ")}`,
      {
        hint: "install SDK packages that ship sources (0.3.0 or newer) and `files` including `src`, or drop source mode",
      },
    );
  }
  if (missingSources.length > 0) {
    return check(
      "source-mapping",
      "fail",
      `${missingSources.length} source file(s) are missing from the installed packages`,
      {
        details: missingSources.map((entry) => `${entry.specifier} -> ${entry.source}`),
        hint: "the published `files` set must include the package `src` directory; reinstall or republish",
      },
    );
  }
  if (declaresSource !== typesAgree) {
    return declaresSource
      ? check(
          "source-mapping",
          "fail",
          "the runtime resolves SDK sources while TypeScript resolves the published build",
          {
            hint: `extend the generated \`.solid-gpui/tsconfig.json\`, or add "customConditions": ["${SOURCE_CONDITION}"] with moduleResolution bundler or nodenext`,
          },
        )
      : check(
          "source-mapping",
          "fail",
          "TypeScript resolves SDK sources while the runtime resolves the published build",
          {
            hint: "add `solidGpuiSource()` to the Vite plugins so both sides resolve the same files",
          },
        );
  }
  return check(
    "source-mapping",
    "pass",
    context.mode === "source"
      ? `source mappings agree: ${entries.length} specifier(s) across ${mapped.packages.length} package(s)`
      : "published build resolution; no source mappings required",
  );
}

function hostPackageCheck(context: DoctorContext): DoctorCheck {
  const manifestPath = join(context.root, "package.json");
  const app = readJson(manifestPath);
  if (app === undefined) {
    return check("host-package", "fail", `no package.json at ${manifestPath}`, {
      hint: "run doctor from the application root",
    });
  }
  const declared = { ...table(app.dependencies), ...table(app.devDependencies) };
  const absent = REQUIRED_PACKAGES.filter((name) => declared[name] === undefined);
  if (absent.length > 0) {
    return check("host-package", "fail", `application does not depend on ${absent.join(", ")}`, {
      details: [`${manifestPath}: ${Object.keys(declared).sort().join(", ")}`],
      hint: `bun add ${absent.join(" ")}`,
    });
  }
  if (context.nativeManifest === undefined) {
    return check("host-package", "pass", "application dependencies are complete; no native host configured");
  }
  if (!existsSync(context.nativeManifest)) {
    return check("host-package", "fail", `native Cargo manifest not found: ${context.nativeManifest}`, {
      hint: "point the native manifest option at the host crate's Cargo.toml (relative paths resolve from the Vite root)",
    });
  }
  if (context.nativeOutput !== undefined && !existsSync(resolve(context.root, context.nativeOutput))) {
    return check("host-package", "warn", `host bindings are not generated yet: ${context.nativeOutput}`, {
      hint: "run `solid-gpui generate`, or let the Vite plugin export them on the next build",
    });
  }
  return check("host-package", "pass", `native host manifest present: ${context.nativeManifest}`);
}

function cargoPatchesCheck(context: DoctorContext): DoctorCheck {
  const cargo = context.cargo;
  if (cargo === undefined) return cargoUnavailable(context, "cargo-patches");
  const required = cargo.sdk === undefined ? {} : patchEntries([cargo.sdk]);
  if (Object.keys(required).length === 0) {
    return check(
      "cargo-patches",
      "warn",
      "cannot verify platform patches: no SDK checkout with [patch.crates-io] was found",
      {
        details: [`searched up from ${cargo.crate.path}`],
        hint: "keep solid-gpui as a path dependency of the host crate, or copy its [patch.crates-io] block into your manifest",
      },
    );
  }
  const declared = patchEntries(cargo.effective);
  const platform = PLATFORM_PATCH[process.platform];
  const missing: string[] = [];
  const divergent: string[] = [];
  const ineffective: string[] = [];
  for (const [name, expected] of Object.entries(required)) {
    const actual = declared[name];
    if (actual === undefined || !existsSync(actual)) missing.push(name);
    else if (realpath(actual) !== realpath(expected)) divergent.push(`${name}: ${actual} (the SDK ships ${expected})`);
    else if (context.graph !== undefined && context.graph.patched[name] !== true) ineffective.push(name);
  }
  const block = [
    "[patch.crates-io]",
    ...Object.entries(required).map(([name, path]) => `${name} = { path = "${path}" }`),
  ].join("\n");
  const blocking = missing.filter((name) => name === platform || platform === undefined || !isPlatformPatch(name));
  if (blocking.length > 0) {
    return check(
      "cargo-patches",
      "fail",
      `required Cargo patch(es) missing for ${process.platform}: ${blocking.join(", ")}`,
      {
        hint: `add to your workspace manifest:\n${block}`,
      },
    );
  }
  if (missing.length > 0) {
    return check("cargo-patches", "warn", `Cargo patch(es) for other platforms are missing: ${missing.join(", ")}`, {
      hint: `add the remaining SDK patch entries:\n${block}`,
    });
  }
  if (ineffective.length > 0) {
    return check(
      "cargo-patches",
      "fail",
      `patch(es) declared but not applied to the resolved graph: ${ineffective.join(", ")}`,
      {
        hint: "declare the patches in the workspace root manifest or in the build directory's .cargo/config.toml; Cargo ignores [patch] in a non-root member",
      },
    );
  }
  if (divergent.length > 0) {
    return check("cargo-patches", "warn", `${divergent.length} patch(es) point at a different vendor copy`, {
      details: divergent,
      hint: "point the patches at the SDK checkout's vendor directories so one gpui source tree builds the app",
    });
  }
  return check("cargo-patches", "pass", `${Object.keys(required).length} Cargo patches match the SDK`);
}

function cargoProfilesCheck(context: DoctorContext): DoctorCheck {
  const cargo = context.cargo;
  if (cargo === undefined) return cargoUnavailable(context, "cargo-profiles");
  const dev = profileTable(cargo.effective, "dev");
  const details: string[] = [];
  if (context.runtime === "quickjs") {
    const level = optLevel(dev, QUICKJS_PROFILE_PACKAGE);
    if (level === undefined || level < 2) {
      return check(
        "cargo-profiles",
        "fail",
        level === undefined
          ? `[profile.dev.package.${QUICKJS_PROFILE_PACKAGE}] is missing, so the QuickJS interpreter stays unoptimized`
          : `[profile.dev.package.${QUICKJS_PROFILE_PACKAGE}] opt-level = ${level}, which under-optimizes the interpreter`,
        {
          hint: `add it to your own workspace manifest (profiles do not inherit from the SDK manifest; restart after changing it, hot reload cannot recompile the interpreter):\n[profile.dev.package.${QUICKJS_PROFILE_PACKAGE}]\nopt-level = 3`,
        },
      );
    }
    details.push(`${QUICKJS_PROFILE_PACKAGE} opt-level = ${level}`);
  }
  const packageEntries = table(table(dev?.package));
  const graph = context.graph;
  if (graph !== undefined) {
    const stale = Object.keys(packageEntries ?? {}).filter(
      (pkg) =>
        !graph.packageNames.includes(pkg) &&
        pkg !== QUICKJS_PROFILE_PACKAGE &&
        !PERF_PROFILE_PACKAGES.includes(pkg as (typeof PERF_PROFILE_PACKAGES)[number]),
    );
    if (stale.length > 0) {
      return check("cargo-profiles", "warn", `stale [profile.dev.package.*] entries: ${stale.join(", ")}`, {
        details,
        hint: `profile keys must name a package in the resolved graph (${graph.packageNames.length} packages resolved); remove or rename them`,
      });
    }
  }
  const missingPerf = PERF_PROFILE_PACKAGES.filter((pkg) => optLevel(dev, pkg) === undefined);
  if (missingPerf.length > 0) {
    return check("cargo-profiles", "warn", `dev profiles for ${missingPerf.join(", ")} are missing`, {
      details,
      hint: `unoptimized layout and scene code makes debug runs slow:\n${missingPerf
        .map((pkg) => `[profile.dev.package.${pkg}]\nopt-level = 3`)
        .join("\n")}`,
    });
  }
  return check("cargo-profiles", "pass", "dev profiles present", { details });
}

function runtimeCapabilitiesCheck(context: DoctorContext): DoctorCheck {
  const graph = context.graph;
  const details: string[] = [`runtime: ${context.runtime}`];
  if (context.web) {
    if (context.nativeManifest !== undefined || context.nativeTarget !== undefined) {
      return check("runtime-capabilities", "fail", "the web target must not declare a native host", {
        hint: "remove the native/host options from this config, or use a native build config for the host crate",
      });
    }
    return check("runtime-capabilities", "pass", "web target: no native host or runtime features required");
  }
  if (context.nativeTarget !== undefined && !targetRunsOnHost(context.nativeTarget) && !context.nativeExporter) {
    return check(
      "runtime-capabilities",
      "fail",
      `native target ${context.nativeTarget} cannot be exported on this host`,
      {
        details,
        hint: "point `native.exporter` at a host-built host executable, or build the host on the target platform; a cross-built artifact must never be executed here",
      },
    );
  }
  const quickjs = graph !== undefined && graph.sdkFeatures.includes("quickjs");
  if (context.runtime === "quickjs" && graph !== undefined && !quickjs) {
    return check(
      "runtime-capabilities",
      "fail",
      "the resolved solid-gpui dependency does not enable the `quickjs` feature",
      {
        details,
        hint: 'enable it on the host crate: solid-gpui = { path = "…", features = ["quickjs"] }',
      },
    );
  }
  if (context.runtime !== "quickjs" && quickjs) {
    return check("runtime-capabilities", "warn", "the `quickjs` feature is enabled but the runtime is Bun", {
      details,
      hint: "drop the feature unless the host also embeds QuickJS; it enlarges and slows the native build",
    });
  }
  return check("runtime-capabilities", "pass", `${context.runtime} runtime matches the declared host`, { details });
}

function worst(checks: readonly DoctorCheck[]): DoctorStatus {
  return checks.reduce<DoctorStatus>(
    (status, entry) => (SEVERITY[entry.status] > SEVERITY[status] ? entry.status : status),
    "pass",
  );
}

/** Printable report: one line per check plus the hint for anything that is not a pass. */
export function formatDoctorReport(report: DoctorReport): string {
  const lines = [`${report.status.toUpperCase()}  ${report.root} (${report.mode} mode)`];
  for (const entry of report.checks) {
    lines.push(`  ${entry.status.padEnd(4)} ${entry.id}: ${entry.summary}`);
    for (const detail of entry.details ?? []) lines.push(`       - ${detail}`);
    if (entry.status !== "pass" && entry.hint !== undefined) lines.push(`       > ${entry.hint}`);
  }
  return lines.join("\n");
}

/**
 * Report how the installed SDK packages, the TypeScript and runtime mappings, and the Cargo host
 * configuration line up. Read-only: it never writes, and it degrades to a warning when a check
 * cannot be verified (no cargo, no SDK checkout, no lockfile).
 *
 * Without `config` it reads the application's own `vite.config.ts` through `loadProject`, so the
 * runtime, native and bindings options are the ones the build uses.
 */
export async function doctor(options: DoctorOptions = {}): Promise<DoctorReport> {
  const root = resolve(options.root ?? options.config?.root ?? process.cwd());
  const configFile = resolve(options.configFile ?? join(root, "vite.config.ts"));
  const loaded = options.config === undefined ? await loadProjectFacts(root, configFile) : undefined;
  const config = options.config ?? loaded?.project;
  const configCheck = loaded?.check ?? check("vite-config", "pass", `using the supplied project facts for ${root}`);
  const native = config !== undefined && typeof config.native === "object" ? config.native : undefined;
  const web = config?.web === true;
  const declaredManifest =
    options.nativeManifest !== undefined
      ? resolve(options.nativeManifest)
      : native?.manifestPath !== undefined
        ? resolve(root, native.manifestPath)
        : existsSync(join(root, "Cargo.toml"))
          ? join(root, "Cargo.toml")
          : undefined;
  const nativeManifest = web ? undefined : declaredManifest;
  const cargo = nativeManifest === undefined || !existsSync(nativeManifest) ? undefined : cargoContext(nativeManifest);
  const declarations = readJson(join(root, ".solid-gpui/tsconfig.json"));
  const declarationsPaths = table(table(declarations?.compilerOptions)?.paths) ?? {};
  const conditions = [
    table(declarations?.compilerOptions)?.customConditions,
    table(readJson(join(root, "tsconfig.json"))?.compilerOptions)?.customConditions,
  ];
  const declaresSource = existsSync(configFile) && SOURCE_PLUGIN.test(readFileSync(configFile, "utf8"));
  const mapped = sdkSource({ root });
  const typesAgree =
    conditions.some((entry) => Array.isArray(entry) && entry.includes(SOURCE_CONDITION)) ||
    Object.entries(mapped.paths).some(([specifier, source]) =>
      mapsSpecifierToSource(declarationsPaths, root, specifier, source),
    );
  const graphResult = cargo === undefined ? undefined : readCargoGraph(cargo);
  const context: DoctorContext = {
    root,
    mode: options.mode ?? (declaresSource || typesAgree ? "source" : "dist"),
    runtime: config?.runtime === "quickjs" ? "quickjs" : "bun",
    web,
    configFile,
    nativeOutput: native?.output ?? config?.bindings,
    nativeTarget: native?.target,
    nativeExporter: native?.exporter !== undefined,
    nativeManifest,
    cargo,
    graph: graphResult?.graph,
    declaresSource,
    typesAgree,
    mapped,
  };
  const checks: DoctorCheck[] = [
    configCheck,
    packagesCheck(context),
    solidInstanceCheck(context),
    sourceMappingCheck(context),
    hostPackageCheck(context),
    graphResult?.check ?? cargoUnavailable(context, "cargo-graph"),
    cargoPatchesCheck(context),
    cargoProfilesCheck(context),
    runtimeCapabilitiesCheck(context),
  ];
  return { root, mode: context.mode, status: worst(checks), checks };
}

/**
 * The application config is the source of truth for the facts doctor cannot infer: the runtime,
 * the native host and the bindings path all come from the same `vite.config.ts` Vite loads.
 */
async function loadProjectFacts(
  root: string,
  configFile: string,
): Promise<{ readonly project?: SolidGpuiProject; readonly check: DoctorCheck }> {
  if (!existsSync(configFile)) {
    return {
      check: check("vite-config", "warn", `no Vite config at ${configFile}`, {
        hint: "run doctor from the application root, or pass the config path",
      }),
    };
  }
  try {
    return {
      project: await loadProject({ root, configFile }),
      check: check("vite-config", "pass", `loaded ${configFile}`),
    };
  } catch (error) {
    return {
      check: check("vite-config", "fail", `could not load ${configFile}: ${messageOf(error)}`, {
        hint: "fix the Vite config; doctor reads the runtime, native, host and bindings options from it",
      }),
    };
  }
}
