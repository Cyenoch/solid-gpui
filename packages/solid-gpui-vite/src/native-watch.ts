import { dirname, join, resolve, sep } from "node:path";
import { runNativeCommand } from "./native-process.ts";
import type { NativeExportOptions } from "./native-export.ts";

export interface NativeWatchInputs {
  readonly roots: readonly string[];
  readonly configuration: readonly string[];
  readonly targetDirectory?: string;
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
  return {
    targetDirectory: resolve(metadata.target_directory),
    roots: metadata.packages
      .filter((entry) => !entry.source && reachable.has(entry.id))
      .map((entry) => resolve(dirname(entry.manifest_path))),
    configuration: [
      ...["Cargo.toml", "Cargo.lock", ".cargo/config", ".cargo/config.toml"].map((file) =>
        join(metadata.workspace_root, file),
      ),
      join(cwd, ".cargo/config"),
      join(cwd, ".cargo/config.toml"),
    ],
  };
}

export function isNativeInput(file: string, inputs: NativeWatchInputs): boolean {
  return (
    inputs.configuration.includes(file) ||
    (inputs.roots.some((root) => file.startsWith(root + sep)) &&
      (/\.rs$/.test(file) ||
        /(?:^|[/\\])Cargo\.(?:toml|lock)$/.test(file) ||
        /[/\\]\.cargo[/\\]config(?:\.toml)?$/.test(file)))
  );
}
