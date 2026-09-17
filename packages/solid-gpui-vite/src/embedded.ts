/**
 * Embedded application packaging: one Vite entry, one executable.
 *
 * This is the maintained entry point for building an application that carries
 * its JavaScript and the pinned Bun runtime inside a single native image. It
 * owns the whole chain — pinned Bun source preparation, the standalone
 * module-graph serializer, the generated Cargo crate, the final link and image
 * verification — so a consumer with its own Rust host does not fork the
 * packager, copy its cache validation, or reconstruct graph keys from file
 * names.
 *
 * The backend builds pinned Bun and Rust sources, so it always works against an
 * explicit SDK checkout:
 *
 * ```ts
 * import { packageEmbeddedApplication } from "@solid-gpui/vite/embedded";
 *
 * const report = await packageEmbeddedApplication({
 *   sdkRoot: "/path/to/solid-gpui",
 *   entry: "/path/to/app/dist/app.js",
 *   bun: "/path/to/pinned/bun",
 *   output: "/path/to/app/build/MyApp",
 *   profile: "release",
 *   application: {
 *     manifest: "/path/to/app/Cargo.toml",
 *     package: "my-app-host",
 *     features: ["quickjs"],
 *     main: "/path/to/app/crates/host/src/embedded.rs",
 *   },
 *   workers: ["/path/to/app/dist/worker.js"],
 * });
 * ```
 *
 * Or from a shell, through the package CLI:
 *
 * ```sh
 * solid-gpui embedded package --sdk-root /path/to/solid-gpui --entry app/dist/app.js \
 *   --bun /path/to/pinned/bun --output build/MyApp --manifest app/Cargo.toml \
 *   --package my-app-host --main crates/host/src/embedded.rs
 * ```
 *
 * Packaging is experimental: see `docs/distribution.md` for the target matrix,
 * the evidence behind each entry, and what remains unqualified. Nothing here is
 * a support statement.
 */

export { EmbeddedPackagingError } from "./embedded/errors.ts";
export {
  EMBEDDED_GRAPH_TARGETS,
  EMBEDDED_TARGETS,
  embeddedEntryPointIdentity,
  embeddedSourceRoot,
  embeddedWorkerIdentity,
  graphTargetSpec,
  resolveEmbeddedTarget,
} from "./embedded/targets.ts";
export type {
  EmbeddedGraphTarget,
  EmbeddedGraphTargetSpec,
  EmbeddedQualification,
  EmbeddedTargetSpec,
} from "./embedded/targets.ts";
export {
  extractGraphPayload,
  packageEmbeddedGraph,
  parseStandaloneGraph,
  writeEmbeddedGraphArtifacts,
} from "./embedded/graph.ts";
export type {
  EmbeddedEntryIdentity,
  EmbeddedEntryRole,
  EmbeddedGraph,
  EmbeddedGraphFile,
  PackageEmbeddedGraphOptions,
  PackagedEmbeddedGraph,
  WriteEmbeddedGraphOptions,
} from "./embedded/graph.ts";
export {
  assertAbortPanicStrategy,
  mergePatchTables,
  mergeProfileTables,
  resolveEmbeddedApplication,
  writeGeneratedApplication,
} from "./embedded/manifest.ts";
export type {
  EmbeddedApplication,
  GeneratedApplication,
  ResolvedApplication,
  WriteGeneratedApplicationOptions,
} from "./embedded/manifest.ts";
export { packageEmbeddedApplication } from "./embedded/package.ts";
export type {
  EmbeddedPackagingArtifacts,
  EmbeddedPackagingOptions,
  EmbeddedPackagingReport,
  EmbeddedProfile,
} from "./embedded/package.ts";
export { readPinnedBunBuild } from "./embedded/sdk.ts";
export type { PinnedBunBuild } from "./embedded/sdk.ts";
export { EMBEDDED_USAGE, runEmbeddedCommand } from "./embedded/command.ts";
export type { EmbeddedCommandContext } from "./embedded/command.ts";
