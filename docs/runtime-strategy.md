# Runtime strategy

The three runtime modes have distinct product roles. These roles describe the
intended development and delivery workflow; packaging commands, prerequisites,
per-target evidence, and signing have one authoritative source in
[distribution](distribution.md#embedded-bun-static-applications). This guide
covers how each mode is built and owned, and the restrictions that follow from
that architecture.

JavaScript can run directly; JSX/TSX uses Vite, the sole application bundler.
Bun retains Bun APIs in development and built applications. Runtime choice does
not select a second compiler. See [Vite integration](vite.md).

## Positioning

| Runtime          | Primary role                                                              | Application responsibilities                                                                                                  |
| ---------------- | ------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------- |
| External Bun     | Rapid iteration and development, including Vite hot reload                 | Bun may own domain logic and services, or call Rust Native Modules.                                                           |
| Embedded Bun     | Production packaging for Bun applications as one self-contained executable | Keep the Bun application model while running Bun/JSC inside the native host and evaluating the application from the image.     |
| Embedded QuickJS | UI runtime for applications whose main capabilities live in Rust          | JSX/TSX owns UI composition, interaction, and reactive presentation state; Rust owns domain capabilities and system services.   |

UI state is still real application code: signals, event handlers, routing, local
validation, and presentation calculations belong in JSX/TSX. The QuickJS role
does not mean static markup; it means that files, networking, long-running domain
work, and other principal capabilities are implemented in Rust and exposed
through Native Modules instead of recreating Bun/Node services inside QuickJS.

All three modes use the same Solid universal renderer and native contracts. GPUI
owns native windows, input, layout, and painting in every mode. Application
ownership and process topology are separate decisions, as recorded in
[ADR-0017](adr/0017-runtime-engines.md).

## Embedded Bun static product

`bun scripts/bun-static-package.ts` — exposed as `bun run embedded:package` —
produces one application executable per target: the GPUI host, the Bun/JSC
runtime, and the serialized application with its declared assets and Worker
entries, in a single native image. It needs no separate Bun or Node executable,
JavaScript tree, `node_modules`, or Bun/JSC shared library. The pinned serializer
is a **build-time** input, not a runtime dependency. OS libraries and any other
target/profile-specific native dependencies still need qualification. Windows
packaging is experimental and Linux stops at native preparation — see
[platform status](distribution.md#platform-status-and-current-evidence) for what
each target has proven, and
[distribution](distribution.md#embedded-bun-static-applications) for commands,
prerequisites, and release steps.

### One native image, one dependency graph

The packager prepares the pinned Bun source (revision from
`crates/solid-gpui-bun-sys/bun-build.json`, with `bun_embed.patch` and the
embedding overlays applied), builds Bun's native graph in `embed-native` mode
against **prebuilt** WebKit/JSC archives, and generates a small Cargo workspace
whose `src/main.rs` includes the emitted application graph and the host entry.
That workspace depends on the GPUI host crate with `embedded-bun` enabled and on
Bun's Rust entry layer as an `rlib` (`bun_bin` with `solid-gpui-embed`), so Bun
and the application are compiled as one dependency graph under Bun's pinned
toolchain, with one panic policy and one global allocator inherited from Bun. The
application must not add a second `#[global_allocator]`. A separate Bun Rust
`staticlib` is rejected before the link, as is a native manifest whose schema or
target does not match. The native build emits `embed-native.json` (objects,
archives, link arguments, Rust flags, environment, cargo profile), which the
packager consumes instead of reconstructing a compiler response file; CLI inputs,
startup code, and GPU/window ownership stay with the application, not with Bun's
CLI main.

### Application delivery: an embedded module graph

Vite still compiles the Solid application. The packager then serializes the
Vite-built entry (plus `--workers` entries and `--assets`) with the pinned Bun
serializer under browser conditions, extracts the standalone module graph,
validates it against the pinned format, and discards the intermediate. Validation
rejects precompiled JSC bytecode and native addons outright, and requires the
graph's entry identity to be exactly the virtual key derived from the application
entry (`/$bunfs/root/<name>` on macOS, `B:/~BUN/root/<name>` on Windows).

The validated payload is embedded in the executable's own graph section: a
read-only static in `__BUN,__bun` on Mach-O (defining the `BUN_COMPILED` symbol at
the section's 16 KiB alignment) or in `.bun` on PE (an 8-byte little-endian
length prefix at section offset 0). The runtime maps that payload straight from
the image. Nothing is unpacked to disk at build or run time, and no mutable
payload copy is needed, because the graph is source-only.

Consequences worth stating plainly:

- The graph is a process-scoped singleton: every session in one process evaluates
  the same embedded graph.
- Graph-backed virtual paths work through Bun's graph-aware file APIs. Resources
  consumed directly by native GPUI code are not remapped automatically; pass
  their bytes through the host's resource API deliberately.
- This is not a sandbox. Paths that are not graph-backed still address the real
  filesystem.
- A computed import that the serializer could not discover is not silently
  resolved from a developer checkout; declare it as an entry or expect a visible
  missing-module failure.

### Startup API and session lifecycle

Applications start a packaged session with
`EmbeddedBunAdapter::start_packaged(entry)`, where `entry` is the graph key the
packager emitted. It is deliberately separate from `start(path)`, which
canonicalizes a filesystem entry and keeps the development path unchanged. The
failing session cannot silently fall back to another file: if the executable has
no usable graph, or no such entry, the session fails closed with a negative
`packaged_graph_status` (`UNAVAILABLE`, `MALFORMED`, `NOT_VIRTUAL`, `MISSING`,
`BYTECODE`, `NATIVE_LIBRARY`) reported through the commit/status path, and
nothing is evaluated. A packaged identity travels through the unchanged
five-function C ABI (`bun_embedded_create`/`run`/`wake`/`terminate`/`destroy`) as
the existing entry pointer-plus-length parameter with a tag byte distinguishing a
graph key from a filesystem path; no second ABI function, graph-install call, or
transport was added.

Lifecycle is the embedded lifecycle documented in the
[communication baseline](#communication-baseline): one persistent Bun owner
thread per process, one active session at a time, a fresh VM per session, and
`shutdown` that waits for complete teardown before the next session is admitted.
The graph singleton is adopted before VM initialization and survives teardown;
per-session state does not.

### Prebuilt WebKit/JSC policy

The packager always configures Bun's native build with `--webkit=prebuilt`, so
WTF, JavaScriptCore, and bmalloc come from the release archives matching Bun's
pinned WebKit revision for the target OS, architecture, libc ABI, and
debug/release/ASAN variant. There is no JavaScriptCore source rebuild, and the
recipe is a platform matrix rather than one archive usable everywhere. Variants
are not interchangeable: ASAN changes `WTF::Vector` layout, so mixing a sanitized
and a non-sanitized input can corrupt memory. A missing variant must fail the
build rather than silently compile or substitute an unrelated engine.

### Deployment restrictions

- **No bundled native-library extraction.** The graph validator rejects JSC
  bytecode and module-info blobs, N-API addons, and any `.node`, `.dll`, `.so`,
  or `.dylib` entry, because those could only be served by writing them to disk
  at run time. Embedded builds also disable the shared native extraction helper,
  including Worker VMs, and reject FFI graph paths; this does not sandbox
  external filesystem or FFI access.
- **No general fork/cluster JavaScript interpreter.** Embedded `fork()` rejects
  its default interpreter and an explicit `execPath` equal to
  `process.execPath`, and `cluster.fork()` uses the same guard; the host is not a
  Bun CLI, and ordinary external process spawning is unchanged. Use declared
  Worker entries for in-image concurrency, resolved against
  `import.meta.dirname` rather than the process working directory.
- **Resource closure is the application's responsibility.** Declare application
  Workers and resources with `--workers` and `--assets`. Arbitrary paths opened
  on the destination filesystem are not automatically embedded; include those
  application resources in the graph or provision them explicitly.
- **The packager emits a bare executable** — no application bundle, icon,
  metadata, entitlements, notices, or archive — so producing a macOS `.app`, an
  archive, or a checksum file remains a release step.
- **Linux packaging is preparation only**, and public Windows support stays
  experimental until the
  [qualification gates](distribution.md#windows-qualification-gates) pass.

The full authoritative list of packaging limits is in
[distribution](distribution.md#runtime-and-packaging-limits).

## Build-time inputs and destination requirements

| Input or requirement        | Build machine                                                                                                       | Destination machine                                                                                  |
| --------------------------- | ------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------- |
| Pinned Bun source           | Needed: shallow checkout at the pinned revision with `bun_embed.patch` and the embedding overlays applied.            | Never needed.                                                                                        |
| Pinned Bun executable       | Needed as the serializer, and it fails closed unless it reports the pinned revision.                                  | Never needed; not embedded.                                                                          |
| Toolchain                   | rustup toolchain from the Bun pin, C/C++ and LLVM for Bun's graph, Ninja 1.13.0, plus the target sysroot/SDK.        | Not needed.                                                                                          |
| Prebuilt WebKit/JSC         | Downloaded as build input for the target variant; never compiled from source.                                        | Not shipped as a sidecar; its code is inside the executable.                                          |
| Application JavaScript      | Vite output serialized into the graph.                                                                               | Not needed beside the executable.                                                                    |
| Runtime files               | —                                                                                                                    | None from Bun or Node. The executable is the application.                                             |
| OS libraries                | —                                                                                                                    | Normal system dependencies of the native image (window system, GPU driver, OS-provided ICU/DirectX modules). |

The `--source`, `--macos-sdk`, `--deployment-target`, `--winsysroot`,
`--base-executable`, `--main`, `--assets`, and `--workers` options exist so a
pinned checkout, a cross-target sysroot, an application-specific Rust host entry,
or a target-platform Bun base executable can be supplied explicitly instead of
being inferred. Cross-target serialization refuses to proceed without
`--base-executable`, because the pin does not cover a silent download of a
different base.

## Intended workflow

For a Bun-based application, develop with external Bun and Vite, then package
with Embedded Bun. Embedding must be qualified using the actual embedded runtime
and the actual packaged artifact; passing tests under the external Bun executable
does not qualify a differently pinned embedded Bun build, and a build under
another profile does not qualify the shipped one.

For a Rust-led application, ship the UI through QuickJS. External Bun and Vite
give rapid UI iteration, but the shared UI must also run in the real QuickJS VM to
catch unsupported dependencies and platform assumptions. QuickJS reload
(`bun run quickjs:dev`, or `vite` with `runtime: "quickjs"`) replaces the VM while
keeping the native host alive; production evaluates one bundle per runtime.

## Communication baseline

| Mode         | Current transport                                                                | Ownership boundary                                                            |
| ------------ | -------------------------------------------------------------------------------- | ----------------------------------------------------------------------------- |
| External Bun | Length-prefixed binary protocol over child stdin/stdout through `StdioTransport` | Owned bytes cross process boundaries.                                         |
| Embedded Bun | In-process bounded byte queues through `EmbeddedTransport`                       | Owned bytes cross the runtime and GPUI threads; this is not an OS stdio pipe. |
| QuickJS      | In-process bounded byte queues through `EmbeddedTransport`                       | Owned bytes cross the QuickJS worker and GPUI threads.                        |

Embedding the application in the image does not change this boundary: JavaScript
still exchanges owned frames with Rust, and a packaged session uses the same
adapter, queues, and pressure rules as a development session. Pressure must reach
the renderer scheduler; changing only an adapter or adding another queue would not
make embedded production bounded. No mode shares live JS objects, Solid owners,
closures, or GPUI handles between JavaScript and the GPUI thread; engine-local
Rust bindings are a different concept from sharing VM values across threads.

The authoritative transport invariants remain in [the protocol guide](protocol.md)
and [ADR-0017](adr/0017-runtime-engines.md). Research may recommend changes, but
an unimplemented recommendation does not silently replace these contracts.

## Performance interpretation

Embedded Bun removes the separate Bun process, but it still initializes and runs
Bun/JSC, and in-process transport still has queueing, synchronization, encoding,
and decoding costs. It does not imply zero-copy object access or a higher native
FPS. Inlining the runtime adds no JIT warm-up advantage and no startup shortcut by
itself: the graph is still parsed and evaluated per session. QuickJS avoids
shipping Bun services, but its interpreter must still execute the UI's
JavaScript, and native layout and painting remain shared GPUI work.

Embedded Bun's first build compiles Bun's native graph from pinned source but
reuses prebuilt WebKit/JSC archives instead of rebuilding JavaScriptCore; a
supported build cache and the extracted native manifest avoid repeating that
work. Build cost is not application startup time and does not validate the pin:
version, patch, and target-variant checks still run. See
[ADR-0002](adr/0002-embedded-bun-runtime.md).

No controlled three-runtime performance comparison is established by these roles.
Compare the same UI, workload, release profile, and native host with the
[performance workflow](performance-analysis.md), recording startup-to-content,
total process memory, JS work, transport cost, and native input-to-present
separately.

## Design research

The research records separate implementation facts from proposals and cite
primary sources:

- [Communication and ownership](../.scratch/runtime-strategy/transport-research.md).
- [QuickJS hot reload and Vite integration](../.scratch/runtime-strategy/quickjs-hot-reload.md).
- [Portable single-executable Embedded Bun](../.scratch/runtime-strategy/windows-embedded-bun.md) —
  static-link, graph-embedding, and Windows portability analysis. Its stage table
  is proposed work unless this guide states that a stage is implemented.

The embedded frame bridge, whole-bundle QuickJS reload, the Vite-owned QuickJS
build policy, and the static Embedded Bun packager are implemented. Bulk buffer
ownership transfer and a custom ModuleRunner remain measurement-gated
alternatives, and directly sharing live JS/GPUI objects is not the recommendation.
See the maintained [reload workflow](hot-reload.md#quickjs-application-reload).
