/**
 * Command-line route for the embedded packager.
 *
 * This is the same implementation the `@solid-gpui/vite/embedded` API exposes,
 * driven by arguments so a build system can call it without writing
 * TypeScript. The SDK checkout is required here — the packager builds pinned
 * Bun and Rust sources — and is discovered from the running module when the
 * command is executed inside a checkout.
 */

import { stat } from "node:fs/promises";
import { join, resolve } from "node:path";
import { parseArgs } from "node:util";
import { EmbeddedPackagingError } from "./errors.ts";
import type { EmbeddedApplication } from "./manifest.ts";
import { packageEmbeddedApplication, type EmbeddedPackagingReport, type EmbeddedProfile } from "./package.ts";

export const EMBEDDED_USAGE = `Usage: solid-gpui embedded package --entry <Vite JS> --bun <pinned Bun> --output <application> [options]

Packages a built Vite entry into one executable that embeds the pinned Bun
runtime and the serialized application module graph. Experimental: see
docs/distribution.md for the target matrix and its evidence.

Required (unless --prepare-only):
  --entry <path>              built Vite JS entry to serialize
  --bun <path>                pinned-revision Bun executable used as serializer
  --output <path>             application executable to write

Application (custom host instead of the SDK default entry):
  --manifest <Cargo.toml>     application-owned Cargo manifest
  --package <name>            package in that workspace that provides the host library
  --feature <name>            feature to enable on that package (repeatable)
  --main <file.rs>            Rust source that defines main; required with --manifest, and
                              usable alone for a custom entry over the default dependencies

Graph contents:
  --worker <path>             worker entry point embedded in the graph (repeatable)
  --asset <path>              resource embedded in the graph (repeatable)

Target and profile:
  --target <triple>           Rust target triple; defaults to the pinned host
  --profile <debug|release>   Cargo and native profile; default release
  --base-executable <path>    target-platform Bun used as the compile base when cross-target
  --macos-sdk <path>          macOS SDK for a cross-target Apple build
  --deployment-target <ver>   macOS deployment target for a cross-target Apple build
  --winsysroot <path>         MSVC sysroot for a cross-target Windows build

Environment:
  --sdk-root <path>           SDK checkout that owns the pinned Bun/Rust backend
  --cache <path>              cache for the pinned Bun source checkout
  --source <path>             existing pinned Bun checkout to clone from
  --ninja <path>              ninja executable; defaults to PATH
  --prepare-only              build the native embedding library only, link nothing

Other:
  --help                      print this message

The packager also accepts "solid-gpui embedded --help". Unsupported targets and
missing prerequisites fail before any build work.`;

export type EmbeddedCommandContext = {
  readonly cwd: string;
  /**
   * SDK checkout to build against. Omitted means the command looks for a
   * checkout above the module it is running from, which resolves inside the
   * repository and fails with an explicit message once the package is
   * installed elsewhere.
   */
  readonly sdkRoot?: string;
  /** Progress sink; defaults to stderr. */
  readonly write?: (line: string) => void;
};

async function discoverSdkRoot(from: string): Promise<string> {
  let directory = from;
  for (;;) {
    try {
      await stat(join(directory, "crates/solid-gpui-bun-sys/bun-build.json"));
      return directory;
    } catch {
      const parent = resolve(directory, "..");
      if (parent === directory) {
        throw new Error(
          "no SDK checkout found above the running command; the embedded packager builds pinned Bun and Rust sources, " +
            "so pass --sdk-root <solid-gpui checkout>",
        );
      }
      directory = parent;
    }
  }
}

/**
 * Runs one `solid-gpui embedded` invocation.
 *
 * Usage errors are reported and returned as exit code 2; a real packaging
 * failure throws so the caller prints its message and exits non-zero.
 */
export async function runEmbeddedCommand(argv: readonly string[], context: EmbeddedCommandContext): Promise<number> {
  const write = context.write ?? ((line: string) => console.error(line));
  let values: Record<string, unknown>;
  try {
    ({ values } = parseArgs({
      args: [...argv],
      options: {
        entry: { type: "string" },
        output: { type: "string" },
        bun: { type: "string" },
        target: { type: "string" },
        profile: { type: "string" },
        manifest: { type: "string" },
        package: { type: "string" },
        feature: { type: "string", multiple: true },
        main: { type: "string" },
        worker: { type: "string", multiple: true },
        asset: { type: "string", multiple: true },
        "base-executable": { type: "string" },
        "macos-sdk": { type: "string" },
        "deployment-target": { type: "string" },
        winsysroot: { type: "string" },
        "sdk-root": { type: "string" },
        cache: { type: "string" },
        source: { type: "string" },
        ninja: { type: "string" },
        "prepare-only": { type: "boolean", default: false },
        help: { type: "boolean", default: false },
      },
    }));
  } catch (error) {
    write(error instanceof Error ? error.message : String(error));
    write(EMBEDDED_USAGE);
    return 2;
  }

  if (values.help === true) {
    write(EMBEDDED_USAGE);
    return 0;
  }
  const prepareOnly = values["prepare-only"] === true;
  if (values.manifest !== undefined && values.package === undefined) {
    write("--package <name> is required with --manifest");
    write(EMBEDDED_USAGE);
    return 2;
  }
  if (values.manifest !== undefined && values.main === undefined) {
    write(
      "--main <file.rs> is required with --manifest: the generated binary crate includes it as the application entry",
    );
    write(EMBEDDED_USAGE);
    return 2;
  }
  if (!prepareOnly && (values.entry === undefined || values.output === undefined || values.bun === undefined)) {
    write("--entry, --output and --bun are required unless --prepare-only is given");
    write(EMBEDDED_USAGE);
    return 2;
  }
  if (values.profile !== undefined && values.profile !== "debug" && values.profile !== "release") {
    write(`--profile must be debug or release, got ${JSON.stringify(values.profile)}`);
    write(EMBEDDED_USAGE);
    return 2;
  }

  const application: EmbeddedApplication | undefined =
    values.manifest === undefined
      ? undefined
      : {
          manifest: resolve(context.cwd, values.manifest as string),
          package: values.package as string,
          features: (values.feature as string[] | undefined) ?? [],
          main: resolve(context.cwd, values.main as string),
        };
  const main =
    application !== undefined || values.main === undefined ? undefined : resolve(context.cwd, values.main as string);

  const sdkRoot = resolve(
    context.cwd,
    (values["sdk-root"] as string | undefined) ?? context.sdkRoot ?? (await discoverSdkRoot(import.meta.dirname)),
  );
  const options = {
    sdkRoot,
    target: values.target as string | undefined,
    profile: values.profile as EmbeddedProfile | undefined,
    application,
    main,
    workers: (values.worker as string[] | undefined)?.map((worker) => resolve(context.cwd, worker)),
    assets: (values.asset as string[] | undefined)?.map((asset) => resolve(context.cwd, asset)),
    baseExecutable: values["base-executable"] as string | undefined,
    cacheDir: values.cache === undefined ? undefined : resolve(context.cwd, values.cache as string),
    sourceCheckout: values.source === undefined ? undefined : resolve(context.cwd, values.source as string),
    ninja: values.ninja as string | undefined,
    macosSdk: values["macos-sdk"] as string | undefined,
    deploymentTarget: values["deployment-target"] as string | undefined,
    winsysroot: values.winsysroot as string | undefined,
    onCommand: (command: readonly string[]) =>
      write(`$ ${command.map((argument) => JSON.stringify(argument)).join(" ")}`),
  };
  const report = await packageEmbeddedApplication(
    prepareOnly
      ? { ...options, prepareOnly: true }
      : {
          ...options,
          prepareOnly: false,
          entry: resolve(context.cwd, values.entry as string),
          output: resolve(context.cwd, values.output as string),
          bun: resolve(context.cwd, values.bun as string),
        },
  );
  printEmbeddedReport(report, write);
  return 0;
}

function printEmbeddedReport(report: EmbeddedPackagingReport, write: (line: string) => void): void {
  if (report.prepareOnly) {
    write(report.artifacts.nativeManifest);
    return;
  }
  write(`sha256  ${report.sha256}  ${report.output}`);
  write(`Entry: ${report.entry?.identity}`);
  for (const worker of report.workers ?? []) write(`Worker: ${worker.identity} (${worker.source})`);
  write(`Graph target: ${report.graphTarget ?? "(none)"} (${report.qualification})`);
  write(`Static application: ${report.output}`);
}

if (import.meta.main) {
  try {
    const status = await runEmbeddedCommand(process.argv.slice(2), { cwd: process.cwd() });
    process.exitCode = status;
  } catch (error) {
    console.error(
      error instanceof EmbeddedPackagingError
        ? error.message
        : error instanceof Error
          ? (error.stack ?? error.message)
          : String(error),
    );
    process.exitCode = 1;
  }
}
