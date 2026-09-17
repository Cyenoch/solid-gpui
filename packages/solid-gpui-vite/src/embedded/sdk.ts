/**
 * The SDK checkout owns the pinned backend: the Bun revision, toolchain, ninja
 * requirement, the embedding patch, the overlays compiled into the pinned Bun
 * source, and the SDK crates the generated application depends on.
 *
 * Nothing here is resolved from the environment or from `PATH`; the packager
 * reads these values from the checkout the caller named, so a product is always
 * built against the revision its own pin file declares.
 */

import { readFile, stat } from "node:fs/promises";
import { join, resolve } from "node:path";
import { fail } from "./errors.ts";

/** Canonical pinned Bun build inputs inside an SDK checkout. */
export type PinnedBunBuild = {
  readonly repository: string;
  /** Full 40-character commit the embedded runtime and serializer must share. */
  readonly revision: string;
  /** Rust toolchain the native embedding library and the host crate build with. */
  readonly toolchain: string;
  readonly ninjaVersion: string;
};

export function pinnedBunConfigPath(sdkRoot: string): string {
  return join(resolve(sdkRoot), "crates/solid-gpui-bun-sys/bun-build.json");
}

export async function readPinnedBunBuild(sdkRoot: string): Promise<PinnedBunBuild> {
  const config = pinnedBunConfigPath(sdkRoot);
  let parsed: unknown;
  try {
    parsed = JSON.parse(await readFile(config, "utf8"));
  } catch (error) {
    fail(
      `${config} could not be read: ${error instanceof Error ? error.message : String(error)}; ` +
        `pass the SDK checkout that owns crates/solid-gpui-bun-sys`,
    );
  }
  const fields = parsed as Record<string, unknown>;
  for (const key of ["repository", "revision", "toolchain", "ninjaVersion"] as const) {
    if (typeof fields[key] !== "string" || (fields[key] as string).length === 0) {
      fail(`${config} does not declare a ${key}`);
    }
  }
  if (!/^[0-9a-f]{40}$/.test(fields.revision as string)) {
    fail(`${config} does not declare a 40-character hexadecimal bun revision`);
  }
  return {
    repository: fields.repository as string,
    revision: fields.revision as string,
    toolchain: fields.toolchain as string,
    ninjaVersion: fields.ninjaVersion as string,
  };
}

/** Fails early when the SDK checkout is not a usable backend. */
export async function assertSdkCheckout(sdkRoot: string): Promise<void> {
  const required = [
    "crates/solid-gpui/Cargo.toml",
    "crates/solid-gpui-bun-sys/bun_embed.patch",
    "crates/solid-gpui-bun-sys/embedded/runtime.rs",
  ];
  for (const relative of required) {
    const path = join(resolve(sdkRoot), relative);
    try {
      await stat(path);
    } catch {
      fail(`SDK checkout ${JSON.stringify(resolve(sdkRoot))} is missing ${relative}`);
    }
  }
}
