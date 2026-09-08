import { execFile } from "node:child_process";
import { randomUUID } from "node:crypto";
import { mkdir, readFile, rename, rm, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { promisify } from "node:util";

export interface NativeExportOptions {
  readonly manifestPath: string;
  readonly package?: string;
  readonly bin?: string;
  readonly features?: readonly string[];
  readonly output: string;
  readonly check?: boolean;
}

/** Cargo's artifact message is authoritative, including custom target directories. */
export async function buildNativeHost(
  options: Omit<NativeExportOptions, "output" | "check">,
  cwd: string,
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
  const { stdout, stderr } = await promisify(execFile)("cargo", args, { cwd, maxBuffer: 16 * 1024 * 1024 });
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

/** Query the executable that will run the application, then publish atomically. */
export async function exportNativeBindings(
  options: NativeExportOptions,
  cwd = process.cwd(),
  executable?: string,
): Promise<string> {
  const output = resolve(cwd, options.output);
  executable ??= await buildNativeHost(options, cwd);
  const { stdout, stderr } = await promisify(execFile)(executable, ["--export-native"], {
    cwd,
    maxBuffer: 16 * 1024 * 1024,
  });
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
    await writeFile(temporary, generated, "utf8");
    await rename(temporary, output);
  } finally {
    await rm(temporary, { force: true });
  }
  return output;
}
