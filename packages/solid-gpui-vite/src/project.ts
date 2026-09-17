import { spawn } from "node:child_process";
import { existsSync } from "node:fs";
import { basename, dirname, extname, join, resolve, sep } from "node:path";
import { loadConfigFromFile, normalizePath, type Plugin } from "vite";
import {
  artifactsFile,
  cargoProfileDirectory,
  expectedExecutablePath,
  normalizeCargoProfile,
  publishGeneratedFile,
  readNativeArtifacts,
  recordedHostExecutable,
  StaleGeneratedFileError,
  targetRunsOnHost,
  tsconfigFile,
  writeNativeArtifacts,
  type NativeArtifacts,
} from "./artifacts.ts";
import type { NativeHostOptions } from "./environment.ts";
import {
  exportHostBindings,
  prepareNativeHost,
  type NativeBuildReporter,
  type NativeExportOptions,
} from "./native-export.ts";
import { pluginSourceMappings } from "./source.ts";

/** Options for the one Vite plugin a Solid GPUI application configures. */
export interface SolidGpuiOptions {
  readonly entry: string;
  readonly runtime?: "bun" | "quickjs";
  readonly native?: Omit<NativeExportOptions, "output"> & { readonly output?: string };
  /**
   * An existing host executable whose exported catalog supplies component bindings,
   * or false for an application-owned module runner. `output` defaults to `.generated/native.ts`.
   */
  readonly host?: (NativeHostOptions & { readonly output?: string }) | false;
}

export type SolidGpuiTarget = SolidGpuiOptions | { readonly target: "web" };

export interface NativeHostBinding extends NativeHostOptions {
  readonly output: string;
}

/** The subset of a user Vite config that changes generated paths and the bundle location. */
export interface ProjectConfig {
  readonly root?: string;
  readonly build?: {
    readonly outDir?: string;
    readonly rolldownOptions?: {
      readonly output?: { readonly entryFileNames?: unknown } | readonly unknown[];
    };
  };
}

interface RegisteredPlugin extends Plugin {
  readonly [REGISTRATION]?: SolidGpuiTarget;
}

const REGISTRATION = Symbol.for("solid-gpui.project.registration");

/**
 * The plugin object carries the options every other tool must reuse, so editors, the CLI and the
 * dev session read one configuration instead of re-deriving paths. A symbol key keeps the plugin
 * object clean for Vite, and `Symbol.for` keeps the seam working when the config is bundled.
 */
export function registerSolidGpuiProject(plugin: Plugin, options: SolidGpuiTarget): Plugin {
  Object.defineProperty(plugin, REGISTRATION, {
    value: options,
    enumerable: false,
    configurable: true,
  });
  return plugin;
}

/**
 * Plugins own their mapping tables: `solidGpuiSource()` publishes the table its runtime aliases
 * use, and the prepared config consumes the same one, so editors and the runtime cannot disagree.
 */
export function registeredPathMappings(plugins: readonly unknown[]): Record<string, string> {
  const mappings: Record<string, string> = {};
  for (const entry of plugins) {
    if (Array.isArray(entry)) Object.assign(mappings, registeredPathMappings(entry));
    else Object.assign(mappings, pluginSourceMappings(entry));
  }
  return mappings;
}

function registeredOptions(plugins: readonly unknown[]): SolidGpuiTarget | undefined {
  for (const entry of plugins) {
    if (Array.isArray(entry)) {
      const nested = registeredOptions(entry);
      if (nested) return nested;
    } else if (entry && typeof entry === "object") {
      // Plugins arrive as `unknown[]` from a user config; only registered ones carry the symbol.
      const options = (entry as RegisteredPlugin)[REGISTRATION];
      if (options) return options;
    }
  }
  return undefined;
}

/** The conditional export that selects a package's TypeScript sources; see ./source.ts. */
const SOURCE_CONDITION = "solid-gpui-source";

function usesSourceMode(plugins: readonly unknown[]): boolean {
  for (const entry of plugins) {
    if (Array.isArray(entry)) {
      if (usesSourceMode(entry)) return true;
    } else if (entry && typeof entry === "object" && (entry as Plugin).name === SOURCE_CONDITION) return true;
  }
  return false;
}

export interface ProjectOptions {
  readonly root?: string;
  readonly configFile?: string;
  readonly mode?: string;
}

/** Cargo's facts after a build: what was built, and where every profile of it lands. */
export interface NativePreparation {
  readonly host: NativeHostOptions;
  readonly executable?: string;
  readonly bindings?: string;
  readonly targetDirectory?: string;
  /** The target Cargo actually built for, which may come from the environment or config. */
  readonly target?: string;
}

export interface SolidGpuiProject {
  readonly root: string;
  /** `web` builds an ordinary browser application; the others run inside a native host. */
  readonly runtime: "web" | "bun" | "quickjs";
  readonly web: boolean;
  readonly entry?: string;
  /** Generated bindings the runtime aliases resolve to. */
  readonly bindings?: string;
  /** Prepared TypeScript config consumers extend. */
  readonly tsconfig: string;
  /** Authoritative artifact record for packaging and preview. */
  readonly artifactsPath: string;
  readonly native?: NativeExportOptions;
  readonly host?: NativeHostBinding;
  readonly aliases: readonly { readonly find: string; readonly replacement: string }[];
  /** Path mappings registered by other plugins, kept in step with their runtime aliases. */
  readonly pathMappings: Readonly<Record<string, string>>;
  /** Conditional exports the runtime selects, which TypeScript must select too. */
  readonly customConditions: readonly string[];
  /** Explicit additional native watch paths, relative to the Vite root as configured. */
  readonly watch: readonly string[];
  /** Configured build output directory: where every emitted entry of this project belongs. */
  readonly outDir: string;
  /** Production bundle the runtime loads, derived from the resolved build configuration. */
  readonly bundle?: string;
  readonly artifacts: NativeArtifacts;
  /** Build the host and publish bindings: a fresh attempt per call, as a session needs. */
  prepare(
    signal?: AbortSignal,
    options?: { readonly check?: boolean; readonly log?: NativeBuildReporter },
  ): Promise<NativePreparation>;
}

/**
 * The single implementation both the Vite plugin and the CLI build on. Everything that decides a
 * path, an alias or a bundle name is computed here from the same user configuration.
 */
export function createSolidGpuiProject(
  config: ProjectConfig,
  options: SolidGpuiTarget,
  extra: {
    /** An enclosing session already built and exported this host, e.g. QuickJS development. */
    readonly preparedHost?: NativeHostOptions;
    /** The configured plugins: they contribute path mappings and active conditions. */
    readonly plugins?: readonly unknown[];
  } = {},
): SolidGpuiProject {
  const web = "target" in options;
  const root = resolve(config.root ?? process.cwd());
  if (!web && options.runtime && options.runtime !== "bun" && options.runtime !== "quickjs") {
    throw new Error("Runtime must be bun or quickjs");
  }
  const runtime = web ? "web" : options.runtime === "quickjs" ? "quickjs" : "bun";
  const configured = web ? undefined : options;
  const output = config.build?.rolldownOptions?.output;
  if (runtime === "quickjs" && Array.isArray(output)) {
    throw new Error("QuickJS requires a single Vite output configuration");
  }
  const entry = configured ? normalizePath(resolve(root, configured.entry)) : undefined;
  const bindingsPath = (path?: string) => normalizePath(resolve(root, path ?? ".generated/native.ts"));
  const native = configured?.native
    ? { ...configured.native, output: bindingsPath(configured.native.output) }
    : undefined;
  const host = configured?.host ? { ...configured.host, output: bindingsPath(configured.host.output) } : undefined;
  // `host` next to `native` is a runtime override for local development, not a second catalog: the
  // native build owns the bindings file, so a configured host output may only agree with it.
  const configuredHostOutput = configured?.host ? configured.host.output : undefined;
  if (native && host && configuredHostOutput !== undefined && host.output !== native.output) {
    throw new Error(
      `native.output and host.output must name the same file when both are configured; the native build publishes the catalog`,
    );
  }
  const bindings = native?.output ?? host?.output;
  const outDir = resolve(root, config.build?.outDir ?? "dist");
  const entryFile = entry ? basename(entry, extname(entry)) : undefined;
  // The prediction is only as good as the configuration: QuickJS names its bundle "app.js" unless
  // told otherwise, the Bun runtime follows Vite's own naming, and a pattern keeps its placeholders
  // so a wrong name can never point at an unrelated existing file.
  const pattern = outputEntryFileName(config);
  const predictedName = pattern ?? (runtime === "quickjs" ? "app.js" : `${entryFile}.js`);
  const bundle = entryFile
    ? normalizePath(
        resolve(outDir, runtime === "quickjs" ? predictedName : predictedName.replaceAll("[name]", entryFile)),
      )
    : undefined;

  const prepare = async (
    signal?: AbortSignal,
    prepareOptions: { readonly check?: boolean; readonly log?: NativeBuildReporter } = {},
  ): Promise<NativePreparation> => {
    if (extra.preparedHost) return { host: extra.preparedHost, bindings };
    if (native) {
      const prepared = await prepareNativeHost(
        { ...native, check: native.check === true || prepareOptions.check === true },
        root,
        signal,
        prepareOptions.log,
        host ? { command: host.command, args: host.args } : undefined,
      );
      return {
        host: prepared.host,
        executable: prepared.executable,
        bindings: prepared.bindings,
        targetDirectory: prepared.targetDirectory,
        target: prepared.target,
      };
    }
    if (!host) return { host: { command: "solid-gpui-host" } };
    // A configured host owns its component catalog; its exporter replaces the SDK's checked-in copy.
    await exportHostBindings(host, { output: host.output, check: prepareOptions.check }, root, signal);
    return { host: { command: host.command, args: host.args }, bindings: host.output };
  };

  return {
    root,
    runtime,
    web,
    entry,
    bindings,
    tsconfig: tsconfigFile(root),
    artifactsPath: artifactsFile(root),
    native,
    host,
    aliases: bindings
      ? [
          { find: "#native", replacement: bindings },
          { find: "@solid-gpui/core/components", replacement: bindings },
        ]
      : [],
    pathMappings: registeredPathMappings(extra.plugins ?? []),
    customConditions: usesSourceMode(extra.plugins ?? []) ? [SOURCE_CONDITION] : [],
    watch: native?.watch ? [...native.watch] : [],
    outDir,
    bundle,
    artifacts: {
      version: 1,
      root,
      runtime,
      entry: entry ?? root,
      outDir,
      bindings,
      tsconfig: tsconfigFile(root),
      bundle,
      host: host && { command: host.command, args: host.args ?? [], output: host.output },
      native: native && {
        manifestPath: resolve(root, native.manifestPath),
        package: native.package,
        bin: native.bin,
        features: native.features ?? [],
        profile: normalizeCargoProfile(native.profile),
        profileDirectory: cargoProfileDirectory(native.profile),
        target: native.target,
        locked: native.locked !== false,
      },
    },
    prepare,
  };
}

/**
 * Resolve the application's own Vite config. Plugin hooks are not run, so nothing is built and
 * nothing is written: the result is the same configuration Vite itself would use.
 */
export async function loadProject(options: ProjectOptions = {}): Promise<SolidGpuiProject> {
  const loaded = await loadConfigFromFile(
    { command: "build", mode: options.mode ?? "production" },
    options.configFile,
    options.root,
  );
  if (!loaded) {
    const searched = resolve(options.configFile ?? options.root ?? process.cwd());
    throw new Error(
      `No Vite config found at ${searched}.\n` +
        `Solid GPUI applications configure solidGpui() in vite.config.ts; pass --config <file> to point at another config.`,
    );
  }
  const plugins = loaded.config.plugins ?? [];
  const target = registeredOptions(plugins);
  if (!target) {
    throw new Error(
      `The Vite config ${loaded.path} does not configure solidGpui().\n` +
        `Add solidGpui({ entry }) (or solidGpui({ target: "web" })) to its plugins, or pass the config that does.`,
    );
  }
  return createSolidGpuiProject(
    { ...loaded.config, root: resolve(loaded.config.root ?? options.root ?? process.cwd()) },
    target,
    { plugins },
  );
}

export interface PreparedProject {
  readonly project: SolidGpuiProject;
  readonly artifacts: NativeArtifacts;
  /** What happened to each generated file, in publication order. */
  readonly files: readonly { readonly path: string; readonly status: "written" | "unchanged" }[];
}

/**
 * A single output config may rename the QuickJS bundle; array outputs are rejected earlier. The
 * plugin's build output and the artifact record must read this name the same way.
 */
export function outputEntryFileName(config: ProjectConfig): string | undefined {
  const output = config.build?.rolldownOptions?.output;
  if (!output || typeof output !== "object" || Array.isArray(output)) return undefined;
  const name = "entryFileNames" in output ? output.entryFileNames : undefined;
  return typeof name === "string" ? name : undefined;
}

/**
 * The generated base must be usable on its own, because consumers extend it with only the few
 * options that are theirs to decide (strictness, ambient types, include). Everything here is what
 * the runtime and the SDK actually require; a consumer's own value always overrides it.
 */
function projectTsconfig(project: SolidGpuiProject): string {
  const compilerOptions: Record<string, unknown> = {
    target: "ES2024",
    module: "preserve",
    moduleResolution: "bundler",
    jsx: "preserve",
    jsxImportSource: "@solid-gpui/core",
    // Sources are compiled by Vite, and the paths below point at TypeScript files.
    noEmit: true,
    allowImportingTsExtensions: true,
  };
  const paths: Record<string, readonly string[]> = {};
  for (const [specifier, target] of Object.entries(project.pathMappings)) {
    paths[specifier] = [normalizePath(resolve(project.root, target))];
  }
  if (project.bindings) {
    paths["#native"] = [project.bindings];
    paths["@solid-gpui/core/components"] = [project.bindings];
  }
  if (Object.keys(paths).length > 0) compilerOptions.paths = paths;
  if (project.customConditions.length > 0) compilerOptions.customConditions = [...project.customConditions];
  if (project.runtime === "quickjs") {
    // The QuickJS runtime has no DOM, and its globals come from the SDK's types entry, which
    // resolves only through package exports — hence the explicit bundler resolution above.
    compilerOptions.lib = ["ES2024"];
    if (installedPackage(project.root, "@solid-gpui/core")) compilerOptions.types = ["@solid-gpui/core/quickjs"];
  }
  return `${JSON.stringify({ compilerOptions }, null, 2)}\n`;
}

/** TypeScript resolves `types` entries as modules; only emit one that is really installed. */
function installedPackage(root: string, name: string): boolean {
  for (let directory = root; ;) {
    if (existsSync(join(directory, "node_modules", name, "package.json"))) return true;
    const parent = dirname(directory);
    if (parent === directory) return false;
    directory = parent;
  }
}

function currentArtifacts(project: SolidGpuiProject, preparation?: NativePreparation): NativeArtifacts {
  const native = project.artifacts.native;
  if (!native) return project.artifacts;
  const host = preparation?.host;
  return {
    ...project.artifacts,
    // The recorded host is the one that runs here; a cross artifact is described, not launched.
    host: host
      ? {
          command: host.command,
          args: host.args ?? [],
          output: project.host?.output ?? project.native?.output ?? host.command,
        }
      : undefined,
    native: {
      ...native,
      // An implicit build target changes where the artifact lives, so record what was built.
      target: preparation?.target ?? native.target,
      targetDirectory: preparation?.targetDirectory ?? native.targetDirectory,
      executable: preparation?.executable,
    },
  };
}

export interface PublicationOptions {
  /** Verify freshness without writing anything. */
  readonly check?: boolean;
  readonly signal?: AbortSignal;
  /** Native build output, forwarded as Cargo produces it. */
  readonly log?: NativeBuildReporter;
  /** A preparation the caller already ran, such as a Vite build's own host build. */
  readonly preparation?: NativePreparation;
}

/**
 * Publish the prepared TypeScript config and the artifact record for a project, preparing the host
 * first unless the caller already did. Check mode writes nothing and reports what is stale.
 */
export async function publishProject(
  project: SolidGpuiProject,
  options: PublicationOptions = {},
): Promise<PreparedProject> {
  const { check, signal } = options;
  if (project.web) return { project, artifacts: project.artifacts, files: [] };
  const stale: string[] = [];
  let preparation = options.preparation;
  if (!preparation) {
    try {
      preparation = await project.prepare(signal, { check, log: options.log });
    } catch (error) {
      // A stale binding is one finding among several; verify the rest before reporting.
      if (!check || !(error instanceof StaleGeneratedFileError)) throw error;
      stale.push(error.message);
    }
  }
  const files: { path: string; status: "written" | "unchanged" }[] = [];
  try {
    files.push({
      path: project.tsconfig,
      status: await publishGeneratedFile(project.tsconfig, projectTsconfig(project), {
        check,
        description: "Prepared TypeScript config",
        signal,
      }),
    });
  } catch (error) {
    if (!check || !(error instanceof StaleGeneratedFileError)) throw error;
    stale.push(error.message);
  }
  const artifacts = currentArtifacts(project, preparation);
  if (!check) await writeNativeArtifacts(artifacts);
  if (stale.length > 0) {
    throw new Error(
      `Generated files are stale; run \`solid-gpui prepare\` to update them:\n${stale
        .map((entry) => `  - ${entry}`)
        .join("\n")}`,
    );
  }
  return { project, artifacts, files };
}

export interface PrepareProjectOptions extends ProjectOptions, PublicationOptions {}

/**
 * Build the configured host, publish bindings and the prepared TypeScript config, and record the
 * artifact locations. No application bundle is produced and no application process is started.
 */
export async function prepareProject(options: PrepareProjectOptions = {}): Promise<PreparedProject> {
  return publishProject(await loadProject(options), options);
}

/** Verify that generated files match their hosts without writing anything. */
export async function checkProject(options: PrepareProjectOptions = {}): Promise<PreparedProject> {
  return publishProject(await loadProject(options), { ...options, check: true });
}

/**
 * A host built for another platform cannot be launched for development or preview. Refusing here
 * names the cause, instead of leaving an opaque "exec format error" at the operating system. A
 * configured runtime host (the plugin's `host` option) is not the artifact, so it is never refused.
 */
export function assertHostRunsLocally(preparation: NativePreparation, purpose: string): void {
  if (preparation.host.command !== preparation.executable) return;
  if (targetRunsOnHost(preparation.target)) return;
  throw new Error(
    `The prepared host is built for ${preparation.target} and cannot run on this machine for ${purpose}.\n` +
      `Run this platform's host instead: drop native.target for development, or point host.command at a host ` +
      `built here while native.target stays for packaging.`,
  );
}

export interface PreviewOptions extends ProjectOptions {
  /** Extra arguments for the host executable, after the runtime arguments. */
  readonly args?: readonly string[];
  readonly env?: Readonly<Record<string, string | undefined>>;
  readonly signal?: AbortSignal;
}

/**
 * Run the already-built application: the recorded host against the production bundle. Nothing is
 * rebuilt, watched or re-exported, so this shows real production composition rather than another
 * development server.
 */
export async function previewApplication(options: PreviewOptions = {}): Promise<number> {
  const project = await loadProject(options);
  if (project.web) {
    throw new Error(`This config targets the web (target: "web"); preview the web build with \`vite preview\``);
  }
  const artifacts = (await readNativeArtifacts(project.root)) ?? project.artifacts;
  assertRecordedConfiguration(project, artifacts);
  // Only the artifact itself is platform-bound; a configured runtime host runs here by definition.
  const artifact = artifacts.native;
  if (artifact?.target && !targetRunsOnHost(artifact.target) && artifacts.host?.command === artifact.executable) {
    throw new Error(
      `The recorded artifacts are built for ${artifact.target} and cannot run on this machine.\n` +
        `Preview runs the host for this platform: drop native.target for that run, or point host.command at a host built here.`,
    );
  }
  const bundle = artifacts.bundle;
  if (!bundle) throw new Error("No application entry is configured, so there is nothing to preview");
  if (!existsSync(bundle)) {
    throw new Error(
      `Production bundle not found: ${bundle}\n` +
        `Run \`vite build\` first (or \`solid-gpui prepare && vite build\`); preview never builds for you.`,
    );
  }
  // The record names the host that runs here; an exporter is a bindings source, never a runtime.
  const command = recordedHostExecutable(artifacts) ?? expectedExecutablePath(artifacts);
  if (!command) {
    throw new Error(
      "No runnable host found.\nConfigure host.command or native.manifestPath, then run `solid-gpui prepare` so the host exists.",
    );
  }
  // A command with a path is resolved the way the launch resolves it (relative to the project root);
  // a bare name is a PATH lookup that happens at spawn time and must not be treated as a file here.
  const executable = command.includes("/") || command.includes("\\") ? resolve(project.root, command) : undefined;
  if (executable && !existsSync(executable)) {
    throw new Error(
      `Host executable not found: ${executable}.\nBuild it, or run \`solid-gpui prepare\` before previewing.`,
    );
  }
  const environment: NodeJS.ProcessEnv = { ...process.env, ...options.env };
  // The Bun runtime selects its runner through the host's argv contract, not a host CLI flag; a
  // QuickJS host must not inherit an enclosing session's runner.
  const runtimeArguments = artifacts.runtime === "quickjs" ? ["--runtime", "quickjs", bundle] : [];
  if (artifacts.runtime === "quickjs") delete environment.SOLID_GPUI_VITE_RUNNER;
  else environment.SOLID_GPUI_VITE_RUNNER = JSON.stringify([process.execPath, "--conditions=browser", bundle]);
  return runApplication(command, [...(artifacts.host?.args ?? []), ...runtimeArguments, ...(options.args ?? [])], {
    cwd: project.root,
    env: environment,
    signal: options.signal,
  });
}

/**
 * A record written for another configuration must not launch an old entry or runtime in silence:
 * the whole point of the record is that it describes what was built. Only what is stable is
 * compared — the application entry and the configured output directory — because Vite owns the file
 * name: a pattern, a nested pattern or an output function decides where the entry lands, and the
 * recorded bundle is the file that really exists inside that directory.
 */
function assertRecordedConfiguration(project: SolidGpuiProject, artifacts: NativeArtifacts): void {
  const differences: string[] = [];
  const compare = (label: string, recorded: unknown, configured: unknown): void => {
    if (recorded !== undefined && configured !== undefined && recorded !== configured) {
      differences.push(`${label}: recorded ${String(recorded)}, configured ${String(configured)}`);
    }
  };
  compare("runtime", artifacts.runtime, project.runtime);
  compare("application entry", artifacts.entry, project.entry);
  compare("build output directory", artifacts.outDir, project.outDir);
  if (artifacts.bundle && !artifacts.bundle.startsWith(project.outDir + sep)) {
    differences.push(`bundle: recorded ${artifacts.bundle}, outside ${project.outDir}`);
  }
  compare("host package", artifacts.native?.package, project.artifacts.native?.package);
  compare("host binary", artifacts.native?.bin, project.artifacts.native?.bin);
  compare("Cargo profile", artifacts.native?.profile, project.artifacts.native?.profile);
  compare("Cargo target", artifacts.native?.target, project.artifacts.native?.target);
  if (differences.length > 0) {
    throw new Error(
      `The recorded artifacts do not match this configuration (${differences.join("; ")}).\n` +
        `Run \`solid-gpui prepare\` and \`vite build\` again, or pass --config for the config they were built with.`,
    );
  }
}

function runApplication(
  command: string,
  args: readonly string[],
  options: { readonly cwd: string; readonly env: NodeJS.ProcessEnv; readonly signal?: AbortSignal },
): Promise<number> {
  const { promise, resolve, reject } = Promise.withResolvers<number>();
  const child = spawn(command, [...args], { cwd: options.cwd, env: options.env, stdio: "inherit" });
  const onAbort = () => child.kill("SIGTERM");
  const onInterrupt = () => child.kill("SIGINT");
  const onTerminate = () => child.kill("SIGTERM");
  const detach = () => {
    process.off("SIGINT", onInterrupt);
    process.off("SIGTERM", onTerminate);
    options.signal?.removeEventListener("abort", onAbort);
  };
  options.signal?.addEventListener("abort", onAbort, { once: true });
  process.once("SIGINT", onInterrupt);
  process.once("SIGTERM", onTerminate);
  child.on("error", (error) => {
    detach();
    reject(new Error(`Failed to start ${command}: ${error.message}`));
  });
  child.on("close", (code) => {
    detach();
    resolve(code ?? 1);
  });
  return promise;
}
