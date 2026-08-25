# React GPUI

React GPUI is a first vertical slice that renders React trees into one or more GPUI surfaces. React and Bun own Fiber, hooks, context, fragments, and JavaScript closures; Rust and GPUI own validated native tree state and drawing.

## Status

The V3 path is working end to end: the React custom renderer emits an immutable Snapshot bootstrap followed by incremental Patches, the Rust host validates and applies them, native press/TextInput/VirtualList/keyboard/animation events return to JavaScript listener callbacks, and the host can select either `ProcessAdapter` or the in-process `EmbeddedBunAdapter`. Embedded Bun builds the pinned Bun/JSC graph from `crates/react-gpui-bun/bun_embed.patch`; Fast Refresh lives in `packages/react-gpui-dev`.

## Architecture

```text
React components and hooks
          │
          ▼
@react-gpui/core (Bun/TypeScript)
  Fiber commit → one framed MessagePack Commit Batch per surface
          │ stdout commits / stdin events
RuntimeAdapter (`ProcessAdapter` or `EmbeddedBunAdapter`)
          │
          ▼
Surface registry + one commit reader
  ├── surface 1 → ReactRoot → GPUI window
  ├── surface N → ReactRoot → GPUI window
  └── events/CommandResults demultiplexed by surfaceId
          │
          └── native events → framed events → matching JS root callback
```

A host starts with surface `1`. For multiple native windows, construct a
shared `createSurfaceHost(transport)`, render from one registered root, then
call that root's `openSurface({ title?, width?, height? })`. Await the returned
surface ID, register it with `host.createRoot({ surfaceId, onClose })`, and
render the new tree:

```tsx
const host = createSurfaceHost(transport);
const root = host.createRoot({ surfaceId: 1 });
root.render(<Main />);
const surfaceId = await root.openSurface({ title: "Inspector", width: 640, height: 480 });
const inspector = host.createRoot({ surfaceId, onClose: () => console.log("closed") });
inspector.render(<Inspector />);
```

`OpenSurface` is always a command from its requesting, already registered root
(`nodeId=1`); the host rejects unknown surface IDs and never implicitly opens a
window. A native close emits `EVENT_SURFACE_CLOSED` before teardown, routes only
to that root, and invokes its `onClose`. Closing the last native window shuts
down the runtime and process. Real Quartz multi-window display behavior is
validated separately on a macOS display-backed host; headless tests cover the
shared-reader/demultiplexing contract.

File dialogs are root-scoped asynchronous commands:

```tsx
const paths = await root.pickFiles({ title: "Choose files", multiple: true });
const savePath = await root.pickSavePath({ defaultName: "report.json" });
```

`pickFiles` chooses files or directories exclusively (`directories` selects
directories; `multiple` controls multiplicity) and resolves to a non-empty
path list or `null` on cancellation. `pickSavePath` returns a selected path or
`null`; an empty default name leaves the native suggestion unset. Save dialog
titles are not exposed because GPUI's raw save picker has no title/prompt
parameter. Picker failures reject the JavaScript promise. The file dialog
commands remain asynchronous so the GPUI event loop and other root commands
continue while the native modal is open.
Headless tests cover command validation, asynchronous completion, cancellation,
and value routing. Actual NSOpenPanel/NSSavePanel interaction requires a
display-backed macOS Quartz host run and is not exercised in headless CI.

A commit reader performs blocking process I/O away from the GPUI foreground executor, then applies each complete Commit Batch on the GPUI side. GPUI rebuilds ephemeral elements from the retained `NodeStore`; native callbacks send events through the same adapter. ProcessAdapter outbound events are drained by a named writer thread with an ordered queue bounded to 32 payloads and 16 MiB of queued payload bytes; full bounds fail immediately, while writer I/O failures are retained, request child stop, and on confirmed child death wake the commit reader for the host fatal path. Shutdown joins the writer only after child exit is confirmed; kill/wait errors return without blocking. StdioTransport input/output end, close, and error signals notify createRoot termination callbacks, and process examples exit nonzero through the injectable termination handler. Unexpected runtime EOF, framing, commit-validation, or outbound Native Event/CommandResult send errors are logged with context, stop the runtime, close the application, and return a nonzero CLI status; explicit application shutdown remains clean.

## Quick start

From the repository root:

```sh
cargo run -p react-gpui-host -- --runtime process bun run packages/react-gpui/examples/counter.tsx

# Embedded Bun/JSC (builds the pinned source graph under target/)
cargo run -p react-gpui-host --features embedded-bun -- --runtime embedded packages/react-gpui/examples/counter.tsx

```

For opt-in Fast Refresh while developing an embedded entry, add `--watch`.

## Workspace map

- `crates/react-gpui-bun/` — bounded in-process Bun/JSC adapter and pinned source patch/build.
- `packages/react-gpui-dev/` — Babel React Refresh transform, runtime globals, family refresh, and last-good failure handling.
- `crates/react-gpui-host/` — executable host with explicit ProcessAdapter/EmbeddedBunAdapter selection.
- `packages/react-gpui/` — TypeScript package `@react-gpui/core`, custom React renderer, transports, tests, and counter example.
- `crates/react-gpui/` — Rust protocol, runtime adapter seam, snapshot validation, retained node store, and GPUI rendering entity.
- `references/zed/` — checked-in GPUI reference source used by the workspace.

## V3 protocol and ownership invariants

- The authoritative wire reference is [`docs/protocol.md`](docs/protocol.md). It defines framing, Snapshot/Patch/Event/Command tuples, node and style slots, validation, fixtures, and evolution rules.
- Every message is a four-byte little-endian payload length followed by MessagePack bytes; payloads are bounded at 16 MiB. A completed React commit is one atomic Commit Batch: Snapshot bootstrap, then Patch revisions.
- React/Bun own Fiber, hooks, closures, and callback state. Rust/GPUI owns validation, the retained tree, native input/focus/window state, and native rendering. Runtime adapters carry only the versioned wire exchanges.
- Native events and surface commands are semantic boundaries; their complete code directories, ownership, payloads, and CommandResult value tags are maintained in `docs/protocol.md` rather than duplicated here.
- Protocol fixtures and golden tests lock producer bytes and cross-language meaning; use `make protocol-golden-generate` when changing the contract.

## Development commands

The local one-shot verification entry point is the same command used by the
ordinary CI jobs:

```sh
make ci
```

`make ci` runs Rust formatting, a locked Rust workspace check, strict Clippy
with warnings denied, and tests, then formatting, typechecking, tests, clean
package builds, tarball content checks, and an external tarball consumer smoke
test for both Bun packages. Each package uses its checked-in `bun.lock` with
`bun install --frozen-lockfile`; the formatter is the pinned Prettier `3.6.2`
dependency and the Bun runtime is pinned to `1.4.0` by `.bun-version`. The Rust
toolchain is pinned in `rust-toolchain.toml`.
The ordinary CI workflow also runs `make host-release-check` as an independent
macOS ARM release-bundle job, so CLI parsing and archive checks run on pull
requests rather than only in the manual candidate workflow.

The individual local gates are also available when iterating:

```sh
make rust-format
make rust-check
make bun-build
make bun-pack-smoke
make bun-ci
```

The embedded Bun build is intentionally not part of `make ci` because it
clones and compiles the pinned Bun/JSC source graph. Run its locked host
feature check and embedded adapter tests explicitly with:

```sh
make embedded-bun
```

The ordinary CI workflow runs on the GitHub-hosted `macos-15` ARM runner.
The independent embedded-Bun workflow is manually dispatchable and only
automatically considered for pull requests touching its inputs; it uses the
`macos-26` ARM runner because the pinned Bun/JSC build requires the macOS 26
SDK/toolchain.

For direct package work, the equivalent commands are:

```sh
cd packages/react-gpui
bun install --frozen-lockfile
bun run format
bun run typecheck
bun run test
bun run build
bun pm pack --dry-run

cd ../react-gpui-dev
bun install --frozen-lockfile
bun run format
bun run typecheck
bun run test
bun run build
bun pm pack --dry-run
```

### Host release candidate

The release candidate is the default process-runtime host for macOS ARM. It
does not bundle Bun or a renderer entry: users still need Bun and a renderer
command/entry such as `bun run packages/react-gpui/examples/counter.tsx`.

Build and verify the staged candidate locally:

```sh
make host-release-bundle
make host-release-check
```

To rehearse a real user consuming the extracted binary, run:

```sh
make host-candidate-smoke
```

This extracts the archive, launches the extracted host against the repository
counter as a user-supplied renderer entry from a fresh working directory, and
requires a Snapshot frame, the 5-second timeout exit `124`, startup info
diagnostics, `--help`, and `--version`. It does not claim to validate window
pixels on a display-less environment.

The embedded runtime has a separate, non-publishing rehearsal:

```sh
make host-embedded-candidate-smoke
```

It builds the embedded host release binary on the macOS 26 SDK/toolchain,
runs the extracted-style binary from a fresh directory with the counter entry,
and checks the in-process Snapshot commit count, info diagnostic, timeout
`124`, `--help`, and `--version`. It is an exercise rather than a release
archive: embedded package inclusion remains a ready-for-human release-matrix
decision tracked in `.scratch/release-productionization/issues/03-embedded-build-coverage.md`
and `06-cross-platform-host.md`.

Before a candidate release, synchronize the workspace and package versions
from one Cargo version with a matching `CHANGELOG.md` section:

```sh
make release-prep VERSION=0.1.1
```

This updates the workspace Cargo/package manifests, refreshes Cargo and Bun
locks, and verifies frozen installs. It is idempotent when all three manifests
already use the requested version; it never creates or edits a changelog
section. The manual `Release Prep` workflow runs this step, then `make ci`,
and packs tarballs as verification artifacts only. Actual npm/crates.io
publication still requires a human, an approved version/changelog, and real
registry credentials; this workflow contains no publish step or secrets.

Host diagnostics are controlled by `REACT_GPUI_LOG=off|error|info|debug`;
the default is `error`, and invalid values fall back to `error` with one
warning. `info` adds startup and runtime-termination status lines, while
`debug` preserves those diagnostics alongside existing fatal context.

The archive is written to `dist/` as
`react-gpui-host-<cargo-version>-<target>.tar.gz` and contains only the
release host binary, `README.md`, `LICENSE`, and `SHA256SUMS`. The check
extracts it into a new temporary directory, validates the allowlist and
checksums, verifies executable permissions, and runs `--help` and `--version`
without starting GPUI or requiring a display. The archive includes per-file
SHA-256 checksums for extracted-file consistency. `host-release-check` creates
two archives from the same staged payload and requires their SHA-256 values to
match; this empirically proves deterministic stage-to-archive output for that
candidate. It does not claim full build reproducibility across fresh compiler
runs or different archive tool versions. The archive is unsigned.

The manual `Host Release Candidate` workflow runs the ordinary `make ci` gate
before bundling and uploads this unsigned candidate as a short-retention
artifact; it does not publish a GitHub Release or package.

## Debugging

Set `REACT_GPUI_TAP` before constructing a transport or runtime adapter to
write process-local protocol metadata as JSONL:

```sh
REACT_GPUI_TAP="${TMPDIR:-/tmp}/react-gpui-tap-$$.jsonl" bun run packages/react-gpui/examples/counter.tsx
python3 scripts/protocol-tap-report.py "${TMPDIR:-/tmp}/react-gpui-tap-$$.jsonl"
```

Each process truncates its own path; use distinct paths rather than sharing a
file between processes. Records contain monotonic time, direction, peer,
message kind, complete framed byte count including the four-byte header, and
strictly increasing sequence. Optional event/command subtype numbers,
request IDs, and command-result success values are included. Payload contents
are never recorded. The tap stops at 64 MiB with a final
`tap_stopped`/`reason="capacity"` record. An unopenable path prints one stderr
warning and disables the tap without affecting transport operation. The
zero-dependency report script merges multiple files by time and reports frame
rate, kind/byte statistics, event subtype counts, correlated command success,
and frame interval p50/p95. `MemoryTransport` remains untapped.

V3 supports the documented native host kinds, protocol-v3 press/TextInput/VirtualList/keyboard/pointer/hover/scroll/animation notifications, transitions, and accessibility fields. It does not provide synchronous native cancellation, arbitrary native widgets, or a browser/DOM compatibility layer. `ProcessAdapter` remains the default for fast iteration; `EmbeddedBunAdapter` is available with `--features embedded-bun` and is built from the pinned Bun source graph. Fast Refresh failures keep the last-good native tree visible and print an actionable stderr diagnostic.
