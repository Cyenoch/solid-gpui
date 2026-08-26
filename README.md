# React GPUI

> [!WARNING]
> **Early-stage WIP.** React GPUI is under active development. APIs, protocol
> details, and implementation behavior may change without notice; it is not
> production-ready.

React GPUI renders React trees into one or more native GPUI surfaces. React and
Bun own Fiber, hooks, context, fragments, and JavaScript closures; Rust and GPUI
own validated native tree state and drawing.

![React GPUI Gallery](docs/images/gallery.png)

_The gallery example exercises native controls, scrolling, overlays, drag
reordering, and text rendering._

## Status

The V3 path is working end to end: the React custom renderer emits an immutable Snapshot bootstrap followed by incremental Patches, the Rust host validates and applies them, native press/TextInput/VirtualList/keyboard/animation events return to JavaScript listener callbacks, and the host can select either `ProcessAdapter` or the in-process `EmbeddedBunAdapter`. Embedded Bun builds the pinned Bun/JSC graph from `crates/react-gpui-bun/bun_embed.patch`; Fast Refresh lives in `packages/react-gpui-dev`.

TextInput supports muted visual-only `placeholder` guidance when native text is
empty; selection notifications carry UTF-16 ranges plus a `reversed` head
orientation bit, while ordered `setSelection(start, end)` remains an explicit
range command.

Single-line TextInput supports click-to-place, drag selection, UTF-16-safe
Shift+arrow/Home/End extension, and a visible selection highlight. The caret
remains always visible rather than blinking so keyboard focus and the insertion
point stay available to users who benefit from reduced visual timing demands.
Multiline TextInput now uses GPUI wrapped-line shaping for painting,
point-to-character mapping, and IME candidate bounds, including empty and
trailing-newline lines. Ctrl/Cmd word-boundary movement and double-/triple-click
selection remain outside this renderer's minimal interaction contract.

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
call that root's `openSurface({ title?, width?, height?, kind?, resizable?, minSize? })`.
Await the returned surface ID, register it with `host.createRoot({ surfaceId, onClose })`,
and render the new tree:

```tsx
const host = createSurfaceHost(transport);
const root = host.createRoot({ surfaceId: 1 });
root.render(<Main />);
const surfaceId = await root.openSurface({
  title: "Inspector",
  width: 640,
  height: 480,
  kind: "floating",
  resizable: false,
  minSize: [320, 240],
});
const inspector = host.createRoot({ surfaceId, onClose: () => console.log("closed") });
inspector.render(<Inspector />);
```

Creation options map to GPUI's `WindowKind`, creation-time resizable flag, and
minimum size. `"floating"` is above-parent where supported, not a portable
global always-on-top guarantee; popup, max-size, runtime option setters, and a
center toggle remain unsupported. The host's initial window is a centered
`800×600` surface created before JavaScript starts.

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

System notifications and static menus are root-scoped integrations:

```tsx
await root.showNotification({ title: "Build finished", body: "Artifacts are ready." });
await root.setMenus([
  {
    title: "File",
    items: [
      { type: "action", name: "open", disabled: !canOpen, checked: isOpen },
      { type: "separator" },
      { type: "submenu", title: "More", items: [{ type: "action", name: "other" }] },
    ],
  },
]);
```

Optional notification actions use bounded `{ id, label }` pairs (at most three).
Register `onNotificationResponse: ({ tag, actionId }) => ...` in root options;
`actionId` is `null` for body activation, and responses for closed surfaces are
dropped. Notification delivery and response support are platform best effort.

Pass `onAction: (action) => ...` in `createRoot` options to receive the
selected string action. Menu state is static and state-driven: changing
`disabled` or `checked` re-sends the complete `setMenus` definition; omitted
flags default to `false`, and there is no incremental menu-state command.
`root.setKeybindings` uses full-replacement bindings such as
`{ keystrokes: "cmd-shift-p", actionName: "palette.open" }`; multiple chords
are separated by ASCII whitespace (for example, `"ctrl-k ctrl-1"`). The host
retains each surface's set, installs their union in the process-global GPUI
keymap, and routes a match to the active surface through the same `onAction`
callback. Context predicates and menu shortcut fields are not part of this
first API; invalid chords reject without replacing the previous set.
Disabled actions are unavailable to native activation, while checked actions
use GPUI's toggled indicator. Notifications are fire-and-forget platform
submissions: delivery and authorization are not guaranteed, Web/test menu
implementations may be no-ops, and Windows AppUserModel identity remains a
host packaging concern.

A commit reader performs blocking process I/O away from the GPUI foreground executor, then applies each complete Commit Batch on the GPUI side. GPUI rebuilds ephemeral elements from the retained `NodeStore`; native callbacks send events through the same adapter. ProcessAdapter outbound events are drained by a named writer thread with an ordered queue bounded to 32 payloads and 16 MiB of queued payload bytes; full bounds fail immediately, while writer I/O failures are retained, request child stop, and on confirmed child death wake the commit reader for the host fatal path. Shutdown joins the writer only after child exit is confirmed; kill/wait errors return without blocking. StdioTransport input/output end, close, and error signals notify createRoot termination callbacks, and process examples exit nonzero through the injectable termination handler. Unexpected runtime EOF, framing, commit-validation, or outbound Native Event/CommandResult send errors are logged with context, stop the runtime, close the application, and return a nonzero CLI status; explicit application shutdown remains clean.

## Quick start

From the repository root. The gallery is the recommended first run:

### Gallery (embedded Bun/JSC)

```sh
cargo run -p react-gpui-host --features embedded-bun -- \
  --runtime embedded \
  packages/react-gpui/examples/gallery.tsx
```

The first embedded build compiles the pinned Bun/JSC source graph under
`target/`.

### Counter (process runtime)

```sh
cargo run -p react-gpui-host -- \
  --runtime process \
  bun run packages/react-gpui/examples/counter.tsx
```

For opt-in Fast Refresh while developing an embedded entry, add `--watch`
before the entry path:

```sh
cargo run -p react-gpui-host --features embedded-bun -- \
  --runtime embedded \
  --watch \
  packages/react-gpui/examples/gallery.tsx
```

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

The core and development package public export names are locked by
`fixtures/api-surface.core.txt` and `fixtures/api-surface.dev.txt`; the locks
cover value/type names, not internal type structure. If an API surface test
fails, review the change and explicitly regenerate both snapshots with:

```sh
make api-surface-generate
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

For symptom → diagnosis → repair workflows, start with the
[Troubleshooting guide](docs/troubleshooting.md). This section keeps the
environment-variable quick reference:

- `REACT_GPUI_LOG=off|error|info|debug` controls host diagnostics (`error` is
  the default; invalid values fall back to `error` with one warning).
- `REACT_GPUI_TAP=/path/to/file.jsonl` enables process-local protocol metadata;
  use a distinct path for each process and summarize it with
  `python3 scripts/protocol-tap-report.py`.
- `REACT_GPUI_CRASH_DIR=/path/to/directory` chooses where the host panic hook
  writes `react-gpui-host-<pid>-<timestamp>.log`; it defaults to the system
  temporary directory.

The tap records frame metadata rather than payload contents and is not a
GPU/layout profiler. The guide explains the separate host/renderer tap setup,
the `make soak-smoke` bounded leak smoke, crash/stderr correlation, and the
known platform boundaries.

## Crash diagnostics reference

The host installs a standard-library panic hook before CLI/runtime startup.
Crash reports are written to
`${REACT_GPUI_CRASH_DIR:-the system temporary directory}` as
`react-gpui-host-<pid>-<timestamp>.log`; the original panic remains on stderr.
The report includes the host version/platform, panic location, and a
`Backtrace::capture()` result. Reproduce with:

```sh
RUST_BACKTRACE=full REACT_GPUI_CRASH_DIR=/tmp/react-gpui-crashes \
  cargo run -p react-gpui-host -- --runtime process bun run path/to/entry.tsx
```

For an optional APM integration, initialize the provider before the host's
standard hook and preserve the existing hook when adding the provider. This
illustrative snippet uses the optional Sentry Rust SDK but adds no dependency
to this repository:

```rust
let _sentry = sentry::init(("https://example.invalid/project", sentry::ClientOptions::default()));
let previous = std::panic::take_hook();
std::panic::set_hook(Box::new(move |info| {
    sentry::capture_message(&info.to_string(), sentry::Level::Error);
    previous(info);
}));
```

Use a distinct `REACT_GPUI_TAP` path for each process and run
`scripts/protocol-tap-report.py` beside the crash report. The tap records frame
metadata only, while the crash file records panic context; their timestamps,
direction, and message/subtype sequence provide the non-payload correlation
needed to localize a failure without persisting payload contents.

V3 supports the documented native host kinds, protocol-v3 press/TextInput/VirtualList/keyboard/pointer/hover/scroll/animation notifications, transitions, and accessibility fields. It does not provide synchronous native cancellation, arbitrary native widgets, or a browser/DOM compatibility layer. `ProcessAdapter` remains the default for fast iteration; `EmbeddedBunAdapter` is available with `--features embedded-bun` and is built from the pinned Bun source graph. Fast Refresh failures keep the last-good native tree visible and print an actionable stderr diagnostic.
