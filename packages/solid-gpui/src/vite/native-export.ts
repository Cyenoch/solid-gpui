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

/** Compile and query the actual host before any JavaScript entry is loaded. */
export async function exportNativeBindings(options: NativeExportOptions, cwd = process.cwd()): Promise<string> {
  const output = resolve(cwd, options.output);
  const args = [
    "run",
    "--quiet",
    "--manifest-path",
    resolve(cwd, options.manifestPath),
    ...(options.package ? ["--package", options.package] : []),
    ...(options.bin ? ["--bin", options.bin] : []),
    ...(options.features?.length ? ["--features", options.features.join(",")] : []),
    "--",
    "--export-native",
  ];
  const { stdout, stderr } = await promisify(execFile)("cargo", args, { cwd, maxBuffer: 16 * 1024 * 1024 });
  if (stderr) process.stderr.write(stderr);
  const [{ format }, typescript, estree] = await Promise.all([
    import("prettier/standalone"),
    import("prettier/plugins/typescript"),
    import("prettier/plugins/estree"),
  ]);
  const generated = await format(stdout, {
    parser: "typescript",
    plugins: [typescript, estree.default],
    printWidth: 120,
    singleQuote: false,
    semi: true,
    trailingComma: "all",
  });
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
