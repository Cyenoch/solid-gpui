import { randomUUID } from "node:crypto";
import { mkdir, readFile, rename, rm, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import type { NativeHostOptions } from "./environment.ts";
import { runNativeCommand } from "./native-process.ts";

/** Where a host's exported TypeScript is published. */
export interface NativeBindingsOptions {
  readonly output: string;
  readonly check?: boolean;
}

export interface NativeExportOptions extends NativeBindingsOptions {
  readonly manifestPath: string;
  readonly package?: string;
  readonly bin?: string;
  readonly features?: readonly string[];
}

/** Cargo's artifact message is authoritative, including custom target directories. */
export async function buildNativeHost(
  options: Omit<NativeExportOptions, "output" | "check">,
  cwd: string,
  signal?: AbortSignal,
): Promise<string> {
  const args = [
    "build",
    "--message-format=json-render-diagnostics",
    "--manifest-path",
    resolve(cwd, options.manifestPath),
    ...(options.package ? ["--package", options.package] : []),
    ...(options.bin ? ["--bin", options.bin] : []),
    ...(options.features?.length ? ["--features", options.features.join(",")] : []),
  ];
  const { stdout, stderr } = await runNativeCommand("cargo", args, cwd, signal);
  if (stderr) process.stderr.write(stderr);
  const executables = new Set<string>();
  for (const line of stdout.trim().split("\n")) {
    const artifact = JSON.parse(line);
    if (artifact.reason === "compiler-artifact" && artifact.executable && artifact.target.kind.includes("bin")) {
      if (!options.bin || artifact.target.name === options.bin) executables.add(artifact.executable);
    }
  }
  if (executables.size !== 1) throw new Error("Select one native host executable with native.package and native.bin");
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

/** Build a Cargo host, then query the executable that will run the application. */
export async function exportNativeBindings(
  options: NativeExportOptions,
  cwd = process.cwd(),
  executable?: string,
  signal?: AbortSignal,
): Promise<string> {
  executable ??= await buildNativeHost(options, cwd, signal);
  return publishNativeBindings(executable, [], options, cwd, signal);
}

/** Query a host, then publish its bindings atomically so no partial file is ever loaded. */
async function publishNativeBindings(
  command: string,
  args: readonly string[],
  options: NativeBindingsOptions,
  cwd: string,
  signal?: AbortSignal,
): Promise<string> {
  const output = resolve(cwd, options.output);
  const { stdout, stderr } = await runNativeCommand(command, [...args, "--export-native"], cwd, signal);
  if (stderr) process.stderr.write(stderr);
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
  let previous: string | undefined;
  try {
    previous = await readFile(output, "utf8");
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
  }
  if (previous === generated) return output;
  if (options.check) throw new Error(`Native bindings are stale or missing: ${output}`);
  await mkdir(dirname(output), { recursive: true });
  const temporary = `${output}.${randomUUID()}.tmp`;
  try {
    signal?.throwIfAborted();
    await writeFile(temporary, generated, "utf8");
    signal?.throwIfAborted();
    await rename(temporary, output);
  } finally {
    await rm(temporary, { force: true });
  }
  return output;
}
