import { existsSync } from "node:fs";
import { resolve } from "node:path";
import type { NativeHostOptions } from "./environment.ts";
import { runNativeCommand } from "./native-process.ts";
import {
  artifactTargetTriple,
  normalizeCargoProfile,
  publishGeneratedFile,
  targetRunsOnHost,
  type NativeBuildOptions,
} from "./artifacts.ts";
import { cargoTargetDirectory } from "./native-watch.ts";

/** Where a host's exported TypeScript is published. */
export interface NativeBindingsOptions {
  readonly output: string;
  readonly check?: boolean;
}

export interface NativeExportOptions extends NativeBindingsOptions, NativeBuildOptions {
  readonly manifestPath: string;
  readonly package?: string;
  readonly bin?: string;
  readonly features?: readonly string[];
  /**
   * Host executable that exports the catalog. Required when `target` cannot run on this machine:
   * a Cargo cross build produces an artifact for another system, which must never be executed here.
   */
  readonly exporter?: NativeHostOptions;
  /** Additional paths whose changes must rebuild the native host. */
  readonly watch?: readonly string[];
}

/** Cargo's status and diagnostic output, forwarded while it is produced. */
export type NativeBuildReporter = (line: string) => void;

/** What a prepared Cargo host is, and where every profile of it lands. */
export interface NativeHostPreparation {
  /** The host that runs the application: the configured override, else the executable Cargo built. */
  readonly host: NativeHostOptions;
  /** The executable Cargo built for the configured target, cross targets included. */
  readonly executable: string;
  /** Cargo's target directory, resolved from its own configuration. */
  readonly targetDirectory: string;
  /** The target the artifact is for, including an implicit one, when it is known. */
  readonly target?: string;
  /** Published bindings path. */
  readonly bindings: string;
}

function cargoSelection(options: NativeBuildOptions): string[] {
  const profile = normalizeCargoProfile(options.profile);
  return [
    ...(profile === "release" ? ["--release"] : profile === "dev" ? [] : ["--profile", profile]),
    ...(options.target ? ["--target", options.target] : []),
    ...(options.locked === false ? [] : ["--locked"]),
  ];
}

/**
 * Cargo's lockfile requirement is a build-order problem, not a code problem: report the option
 * that admits the first resolution instead of only Cargo's raw complaint.
 */
function describeFailure(error: unknown): Error {
  const message = error instanceof Error ? error.message : String(error);
  if (!message.includes("because --locked was passed")) return error instanceof Error ? error : new Error(message);
  return new Error(
    `${message}\nCargo is invoked with --locked by default. Run \`solid-gpui prepare\` once with locked: false ` +
      `(native.locked: false) to resolve and commit Cargo.lock, then keep the default.`,
  );
}

/** A host path typo must not surface as an opaque spawn failure. */
function assertHostPresent(command: string, args: readonly string[], cwd: string): void {
  if (!command.includes("/") && !command.includes("\\")) return;
  const path = resolve(cwd, command);
  if (existsSync(path)) return;
  throw new Error(
    `Host executable not found: ${path}${args.length ? ` (args: ${args.join(" ")})` : ""}.\n` +
      `Build it first, point host.command at the built executable, or configure native.manifestPath so Vite builds it.`,
  );
}

/** Cargo's artifact message is authoritative, including custom target directories. */
export async function buildNativeHost(
  options: Omit<NativeExportOptions, "output" | "check">,
  cwd: string,
  signal?: AbortSignal,
  report?: NativeBuildReporter,
): Promise<string> {
  const args = [
    "build",
    "--message-format=json-render-diagnostics",
    "--manifest-path",
    resolve(cwd, options.manifestPath),
    ...(options.package ? ["--package", options.package] : []),
    ...(options.bin ? ["--bin", options.bin] : []),
    ...(options.features?.length ? ["--features", options.features.join(",")] : []),
    ...cargoSelection(options),
  ];
  let stdout: string;
  let stderr: string;
  try {
    ({ stdout, stderr } = await runNativeCommand("cargo", args, cwd, signal, report ? { onStderrLine: report } : {}));
  } catch (error) {
    throw describeFailure(error);
  }
  if (stderr && !report) process.stderr.write(stderr);
  const executables = new Set<string>();
  for (const line of stdout.trim().split("\n")) {
    const artifact = JSON.parse(line);
    if (artifact.reason === "compiler-artifact" && artifact.executable && artifact.target.kind.includes("bin")) {
      if (!options.bin || artifact.target.name === options.bin) executables.add(artifact.executable);
    }
  }
  if (executables.size !== 1) {
    throw new Error(
      `Select one native host executable with native.package and native.bin; Cargo built ${
        executables.size === 0 ? "none" : `several: ${[...executables].join(", ")}`
      }`,
    );
  }
  return [...executables][0]!;
}

/** Export the catalog of a host executable Vite does not build. */
export function exportHostBindings(
  host: NativeHostOptions,
  options: NativeBindingsOptions,
  cwd = process.cwd(),
  signal?: AbortSignal,
): Promise<string> {
  return publishNativeBindings(host.command, host.args ?? [], options, cwd, signal);
}

/**
 * A cross-compiled host must never be executed to export bindings. The failure mode is either a
 * confusing "exec format error" or, worse, a successful run under an emulator that exports a
 * catalog for the wrong platform. Both explicit `native.target` and the implicit build target
 * (`CARGO_BUILD_TARGET`, `build.target` in `.cargo/config.toml`) are checked against this machine.
 */
export function assertRunnableHost(
  options: { readonly target?: string },
  executable: string,
  targetDirectory?: string,
): void {
  const foreign =
    options.target && !targetRunsOnHost(options.target) ? { triple: options.target, how: "native.target" } : undefined;
  const implicit = targetDirectory ? artifactTargetTriple(executable, targetDirectory) : undefined;
  const mismatch =
    foreign ??
    (implicit && !targetRunsOnHost(implicit)
      ? {
          triple: implicit,
          how: "the Cargo build target (`CARGO_BUILD_TARGET` or `build.target` in .cargo/config.toml)",
        }
      : undefined);
  if (!mismatch) return;
  throw new Error(
    `Refusing to run the ${mismatch.triple} host ${executable} on this machine to export bindings, because it was built for another platform by ${mismatch.how}.\n` +
      `Set native.exporter to a host executable that runs here, or drop the cross target when generating bindings ` +
      `and keep it for packaging: Cargo builds it, only the export step must run locally.`,
  );
}

/**
 * Prepare the configured Cargo host: build the artifact for the configured target (cross targets
 * included, because the plan promises a native artifact), publish its catalog, and name the host
 * that runs the application.
 *
 * An `exporter` is a *bindings* exporter, not a runtime: it supplies the catalog when the built
 * artifact cannot run here, and it is never launched as the application. A `runtimeHost` override
 * (the plugin's `host` option) is what local development and preview should launch instead.
 */
export async function prepareNativeHost(
  options: NativeExportOptions,
  cwd: string,
  signal?: AbortSignal,
  report?: NativeBuildReporter,
  runtimeHost?: NativeHostOptions,
): Promise<NativeHostPreparation> {
  const executable = await buildNativeHost(options, cwd, signal, report);
  const targetDirectory = await cargoTargetDirectory(options.manifestPath, cwd, signal);
  const exporter = options.exporter;
  // Without an exporter the artifact itself has to export the catalog, so it must run here.
  if (!exporter) assertRunnableHost(options, executable, targetDirectory);
  return {
    host: runtimeHost ?? { command: executable },
    executable,
    targetDirectory,
    target: artifactTargetTriple(executable, targetDirectory),
    bindings: await publishNativeBindings(exporter?.command ?? executable, exporter?.args ?? [], options, cwd, signal),
  };
}

/** Build a Cargo host, then query the executable that will run the application. */
export async function exportNativeBindings(
  options: NativeExportOptions,
  cwd = process.cwd(),
  executable?: string,
  signal?: AbortSignal,
  report?: NativeBuildReporter,
): Promise<string> {
  if (options.exporter) {
    // An explicitly supplied exporter is the supported way to generate bindings for a cross target.
    return publishNativeBindings(options.exporter.command, options.exporter.args ?? [], options, cwd, signal);
  }
  if (executable) {
    assertRunnableHost(options, executable, await cargoTargetDirectory(options.manifestPath, cwd, signal));
    return publishNativeBindings(executable, [], options, cwd, signal);
  }
  return (await prepareNativeHost(options, cwd, signal, report)).bindings;
}

/** Query a host, then publish its bindings atomically so no partial file is ever loaded. */
async function publishNativeBindings(
  command: string,
  args: readonly string[],
  options: NativeBindingsOptions,
  cwd: string,
  signal?: AbortSignal,
): Promise<string> {
  assertHostPresent(command, args, cwd);
  const output = resolve(cwd, options.output);
  const { stdout, stderr } = await runNativeCommand(command, [...args, "--export-native"], cwd, signal);
  if (stderr) process.stderr.write(stderr);
  // Formatting runs only when bindings are actually published; loading the formatter statically
  // would pay its parse cost on every Vite start.
  const { format } = await import("oxfmt");
  // Rust exports TypeScript regardless of the destination's extension.
  const { code: generated, errors } = await format("native-bindings.ts", stdout, {
    printWidth: 120,
    singleQuote: false,
    semi: true,
    trailingComma: "all",
    sortImports: false,
  });
  if (errors.length > 0) {
    throw new Error(
      `Failed to format native bindings for ${output}:\n${errors.map((error) => error.codeframe ?? error.message).join("\n")}`,
    );
  }
  await publishGeneratedFile(output, generated, {
    check: options.check,
    description: "Generated native bindings file",
    signal,
  });
  return output;
}
