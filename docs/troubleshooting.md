# Troubleshooting

## `@solid-gpui/*` does not resolve from a registry

The SDK packages are not published to npm yet, so `bun add @solid-gpui/core` has
nothing to resolve. Pack matching tarballs from one pinned checkout and install
those:

```sh
bun run task sdk-pack ../sdk-tarballs     # in the SDK checkout
bun add ../sdk-tarballs/solid-gpui-core.tgz solid-js
bun add -d ../sdk-tarballs/solid-gpui-vite.tgz vite
```

Keep every `@solid-gpui/*` package from one `sdk-pack` run so the SDK is a single
revision. The `solid-gpui` Rust crate is likewise consumed from that checkout as a
path or git dependency rather than from a registry. Installing a package by name
becomes correct only once publication happens; the rest of the sequence does not
change. To run against SDK sources directly instead, use the explicit
[solid-gpui-source condition](vite.md#consume-packages-from-source).

## The task CLI cannot resolve a package

Run commands from the repository root after installing the committed workspace
lock:

```sh
bun install --frozen-lockfile
bun run task --help
```

Do not install from `packages/solid-gpui`; the repository has one root
`bun.lock`. Verify `bun --version` matches `.bun-version` before investigating
dependency resolution.

## Bindings are stale or missing

Run the generator and add its check to CI:

```sh
bun run generate          # solid-gpui prepare
bun run check:generated   # solid-gpui prepare --check
```

`prepare` resolves your `vite.config.ts`, builds the configured host when needed,
exports its bindings, and writes the bindings file plus `.solid-gpui/tsconfig.json`
and `.solid-gpui/artifacts.json`. `--check` writes no files and fails when either
generated file is stale — it still builds the host to compare against the real
catalog — which is what a CI gate or a pre-commit hook should run. Never hand-edit
`.solid-gpui/` or the generated bindings: the component catalog must be the running
host's own, and a renumbered catalog with unchanged test names is exactly the
failure a name-based test cannot catch. If the outputs disagree with the host you
launched, `bun run doctor` reports the active Cargo dependencies, profiles, and
patch requirements.

## Package installation tries to compile the host

A `package.json` script named `prepare` is an install lifecycle hook: Bun runs it
during `bun install` (verified on 1.4.2), so naming the generator `prepare` makes
installation build a native host. Name it `generate` (or anything else) and run it
explicitly after installing.

## The CLI refuses to run a cross-compiled host

`solid-gpui prepare` and development execute the host they export, so a
`native.target` naming another platform or architecture is rejected instead of
running a foreign binary. Drop `target` for the prepare/development host, or pass
`native.exporter` to name a host executable that can export bindings on this
machine. Cross-compiled artifacts are built by Cargo for packaging, and their paths
come from `.solid-gpui/artifacts.json` rather than a rebuilt command line.

## A signal changes but no Patch is emitted

Import reactive primitives from `@solid-gpui/core/runtime`, not a server-targeted Solid entry. Create the application inside the producer passed to `root.render`:

```ts
root.render(() => createComponent(App, {}));
```

Creating host nodes before entering the root has no surface owner and is rejected.

## Tests or scripts render nothing

Importing `@solid-gpui/core`, `@solid-gpui/core/runtime`, or the renderer under a
non-client resolution now throws instead of rendering an empty tree:

- Server build: `@solid-gpui/core: solid-js resolved to its server build, so signals
  never notify the native renderer. Resolve Solid's client build (Bun:
  --conditions=browser, Vite: resolve.conditions) or run tests through
  @solid-gpui/vite/test, which sets it for you.`
- Duplicate Solid: `@solid-gpui/core: two copies of solid-js are loaded, so signals
  created by one copy never notify the other. Deduplicate solid-js (Vite:
  resolve.dedupe: ["solid-js"]) so the application and the SDK share one reactive
  graph.`

Use `bun run test` (`solid-gpui test`) for application tests: the runner adds the
`browser` condition and dedupes Solid itself. Vite development and builds set their
own conditions, and `solidGpui()` sets `resolve.dedupe: ["solid-js"]`, so a duplicate
usually means a dependency ships its own `solid-js` copy rather than a missing
dedupe entry. `--conditions=browser` is only needed for raw `bun test` or `bun run`
invocations that bypass both.

## `host operation is not associated with a root`

A component or host node was created outside a root producer or a retained Solid owner. Move creation into `root.render(() => ...)`. Do not share Host Nodes between roots.

## Raw text validation fails

Raw string/number children are valid only directly under `Text`. Wrap labels in `Text` rather than placing strings under `View` or `Pressable`.

## Protocol version mismatch

The TypeScript package and native host must use the same protocol version. Rebuild both from the same checkout; do not add compatibility decoding.

## Surface is closed

A closed or unmounted surface ID is permanently retired. Create a new surface and root instead of reusing the ID with another generation.

## Native contract mismatch

An Extension must match the host's provider, catalog digest, entry ID, and entry
version. A catalog mismatch reports the module name, renderer and host catalog
digests, and the host's name for that entry. Entry IDs can change between
catalogs, so the reported host entry is not necessarily the renderer's component.
A missing adapter instead points to an unregistered module or unsupported host.

Use the host path printed by Vite to check which executable is running. Enable
the required Cargo features and register the module in that host. With `native`
configured, saving a corrected Rust source or Cargo manifest rebuilds the host
and its generated bindings. Both `#native` and `@solid-gpui/core/components` use
those bindings. An explicit `host` command also exports its catalog with
`--export-native`; it never substitutes the SDK's checked-in component catalog.
Rebuild that executable yourself, then save application source to export fresh
bindings and start a new session. `native` is the Cargo-managed equivalent;
neither option requires application-specific native modules.

The rejected session terminates; Vite keeps watching for the next edit. It does
not keep retrying an unchanged failing executable or continue applying Patches
after a rejected commit. See [managed development sessions](hot-reload.md#managed-development-sessions).

## Windows host overflows its stack

GPUI layout and painting recurse through the element tree. The default Windows
executable stack reservation can be too small for deep debug layouts and animated
controls. Host entrypoints reserve a 16 MiB application-thread stack on Windows
when the calling thread has less. Profile-based entrypoints take a factory and
construct the profile on that thread; no application thread wrapper is required.
macOS continues running on its real main thread.

Use `SOLID_GPUI_LOG=info` to read the measured thread reservation, and
`SOLID_GPUI_APP_STACK_BYTES` to compare bounded runs of the same page at different
budgets. An explicit `/STACK` linker reservation is another host-owned option.
Keep UI content, motion settings, and Cargo profile identical during comparison;
check startup, route changes, and repeated interaction. A larger budget cannot
repair unbounded recursion. Disabling animation or substituting every Button with
a custom Pressable hides the trigger rather than establishing a safe launch.

## Windows debug startup fails while creating DirectWriteTextSystem

`Error creating DirectWriteTextSystem` with `os error 3` can indicate missing
debug shaders, not missing fonts. Rebuild with the current renderer: it embeds
the HLSL modules and `alpha_correction.hlsl` and compiles them from memory.
Changing the working directory or copying JavaScript cannot fix an older EXE.
Release builds require the SDK shader compiler at build time.

Diagnose the underlying initialization error; a CPU-feature warning or breakpoint
exit code alone does not identify the cause.

## Static Embedded Bun fails loading a debug builtin

If a debug executable searches the build machine's `build/.../js` directory for
`node:worker_threads`, rebuild through the current static packager. Its patch
disables disk reload and generates embedded builtin source with `--embed-modules`.
Both are required; disabling `BUN_DYNAMIC_JS_LOAD_PATH` alone leaves invalid
module spans. Do not copy builtin JS beside the executable or disable assertions.

Windows GUI-subsystem settings do not suppress native assertion dialogs. Stop
a failing candidate and inspect its error or debugger stack before relaunching.

## Static Embedded Bun loses environment or Worker paths on Windows

- **`os.tmpdir()` contains `undefined\temp`:** set the parent environment's
  `TEMP`/`TMP` and rebuild with the current embedding overlay. The VM must import
  inherited variables through `load_process()` before application execution;
  disabling `.env` loading is not a substitute. Do not hardcode a temporary path.
- **A declared Worker fails with `ENOENT`:** declare its entry with `--workers`
  and resolve it against `import.meta.dirname`, not the process working directory.
  Use the current patch, which passes the graph's canonical key to the loader
  instead of a native-separator spelling of that key.

## Embedded Bun build fails

Use the [static application packager](distribution.md#embedded-bun-static-applications)
for Windows. Direct builds of the `embedded-bun` Cargo feature remain macOS-only.
Start with the [packaging prerequisites](distribution.md#static-packaging-prerequisites)
and `bun install --frozen-lockfile`; the serializer, patched native source and
prebuilt WebKit must match the pinned revision and target.

| Symptom | Remedy |
| --- | --- |
| Windows release cannot find `fxc.exe` | Install the Windows SDK compiler or set `GPUI_FXC_PATH`. Release shaders require DXBC, not DXIL or debug shader substitution. |
| ARM64 debug cannot find `libcmtd.lib` or `libcpmtd.lib` | Include Microsoft's matching `Microsoft.VC.14.44.17.14.CRT.ARM64.Desktop.debug.base.vsix` in the SDK splat. Verify the official package checksum; keep debug/release CRT libraries separate. |
| `wasi.initialize is not a function` on Windows ARM64 | The pinned Solid compiler lacks a native ARM64 binding, and its WASM fallback needs a WASI API unavailable in the pinned driver Bun. Build Vite inputs on a supported compiler host, then package the resulting JS. Use the current lazy JSX preload for plain TypeScript commands. |
| `ENAMETOOLONG` during builtin generation | Rebuild with the current embedding patch, which passes relative module inputs from an explicit working directory. |
| Source extraction fails with Win32 error `1314` | The build account cannot create required symlinks. Have the build-host administrator provision that capability before retrying extraction; runtime users do not need it. |

## Windows ARM64 release exits with `0xC0000409`

Inspect the stack using the matching PDB; this code alone does not distinguish
an assertion, stack failure or missing dependency. For the JSC clock-comparison
assertion, use original MSVC 14.44 headers/libraries with `--winsysroot` to match
the prebuilt WebKit ABI. MSVC 14.51 changes the relevant `std::partial_ordering`
return convention. Do not disable assertions or change clock behavior to hide
the mismatch; diagnose other fail-fast stacks on their own evidence.

## QuickJS cannot resolve a service or transport

Use `EmbeddedTransport` from `@solid-gpui/core/embedded` with the QuickJS host.
`StdioTransport` belongs to Bun's process or embedded stdio environment and
requires `process.stdin` and `process.stdout`.

Configure `solidGpui({ entry, runtime: "quickjs" })` and run `bun --bun vite build` so
dependencies are included in one ESM module. Node/Bun imports are rejected;
ambient `process`, `Bun`, filesystem, and network APIs such as `fetch` are
unavailable. Move those services into Rust Native Modules and call the
generated clients. The bundler's browser target selects portable dependencies;
it does not create a browser environment in QuickJS.

## Vite is ready but the QuickJS window is blank

Vite's `ready` and bundle-size messages confirm compilation, not successful
application rendering. A native window showing only its background can be the
initial loading tree waiting for a later update.

Check the actual runtime exception before changing layout. In a startup chain
such as `native.windowChrome().then(value => setChrome(value)).catch(...)`,
the catch handler receives both command failures and synchronous rendering
errors triggered by `setChrome`. A generic “window configuration failed” message
can hide `Maximum call stack size exceeded` during main-component creation.
Preserve the exception message and stack in stderr diagnostics; avoid logging
native result payloads that may contain application data.

If Cargo reports `profile package spec ... did not match any packages`, compare
the profile names with the consuming workspace's resolved dependencies:

```sh
cargo tree --locked -i rquickjs-sys
```

The current QuickJS feature uses `rquickjs-sys`. Obsolete `quickjs-jit-sys` or
`quickjs-jit-core` profiles do not optimize it. Replace those stale entries with
the workspace-root profile in [Application build configuration](hot-reload.md#application-build-configuration),
then rebuild and restart the native host. Saving TSX cannot change native
compiler settings. See the stack diagnosis below before increasing stack limits.

For an application that initially renders a loading tree, an optional protocol
tap can distinguish successful native replies from a missing UI update:

```sh
SOLID_GPUI_TAP=target/solid-gpui-startup.jsonl ./target/debug/my-app --runtime quickjs dist/app.js
```

Substitute the application's actual executable and bundle paths. Use one host
without the reload supervisor for this capture, with the affected development
bundle, so runtime generations do not share the same tap output file.

Inspect `snapshot`, command-result events with `success: true`, and subsequent
`patch` records. Successful replies without a Patch can locate the failure after
the native command; they do not establish its cause. This is not a universal
startup assertion: an application that renders its complete UI in the first
Snapshot need not emit a Patch. Verify actual page content and an interaction.
The tap records metadata only; stop it after the bounded reproduction.

A consumer regression was reproduced with the same development bundle and
2 MiB JS stack limit: stale profiles produced a blank window, a caught stack
overflow, and no UI Patch; optimizing `rquickjs-sys` restored main-interface
rendering. Keep bundle, state, and stack limits fixed for this comparison.

## QuickJS overflows the stack on a nested route

First compare the consuming workspace's Cargo profile with this repository.
Set `[profile.dev.package.rquickjs-sys]` with `opt-level = 3` in the application's
workspace-root `Cargo.toml`, then rebuild and restart the host. Cargo reads
profiles from the workspace root and ignores dependency profiles.
See the [Cargo profiles reference](https://doc.rust-lang.org/cargo/reference/profiles.html).

Interpreter optimization affects native stack usage as well as execution speed.
With the same 2 MiB JS stack limit, an unoptimized debug build can overflow while
initializing a nested component tree that succeeds in an optimized build. A
successful test in this repository does not establish the consuming application's
build configuration.

Compare direct startup at the affected route, navigation from a simpler route,
and restoration after reload. Navigation may reuse an existing parent layout;
startup and reload recreate it. Keep the bundle, captured state, layout, and
stack limits fixed when comparing build profiles. If the optimized build still
fails, minimize the route and inspect recursive component creation and reactive
updates before increasing the stack budget.

## QuickJS rejects captured state

Check the field path in the error, such as `$.state[0].session.userId`.
`state[0]` contains the value returned by the application's `captureState`.
Return a dedicated JSON state object with explicit `null` values or omitted
optional properties. Nested `undefined`, sparse arrays, accessors, and live
runtime objects cannot cross the VM boundary. Returning `undefined` from
`captureState` itself means there is no captured value.

Capture runs in the old VM before the candidate is created. Correct its live
state, or restart the host after fixing a capture function that always returns
invalid data. A JSON stringify/parse round trip can silently discard data;
use the [captured-state contract](hot-reload.md#captured-state) to define the
handoff explicitly.

## QuickJS reports `applied`, then the page fails

`applied` confirms that the host activated a validated candidate. Route loading,
component initialization triggered by that loading, and asynchronous native
effects can still fail afterward. These failures are outside the rollback
boundary; the last published native tree can remain visible after its VM stops.

Inspect the subsequent runtime diagnostic and verify page-specific content plus
an interaction. A loading view or a persistent navigation label does not prove
the restored page is usable. See [application reload verification](hot-reload.md#verify-application-reload).

## Process host exits after renderer failure

Inspect stderr for the renderer error and the host crash-report path. Protocol decode/validation failures are fatal because continuing would lose revision agreement. Application render errors are thrown to the application; Solid GPUI does not invent fallback UI.

## A killed host leaves the Bun renderer running

Use the default `StdioTransport` for a process renderer. It owns renderer
lifetime when reading `process.stdin`: host pipe closure notifies termination
listeners, then exits, even when application timers keep the event loop alive.
Clean EOF exits with status 0; read/write failure exits with status 1 and a
diagnostic. There is no parent-PID polling and no process-group kill; detached
application services are unaffected.

Connections over supplied streams remain embedder-owned unless
`exitOnHostClose: true` is explicit. `dispose()` never exits the process.
`exitOnHostClose: false` deliberately opts out for a process that must outlive
the host. Do not use that option for an ordinary host-owned renderer child.

## File, clipboard, or dialog command rejects

Commands are bounded and validate arguments before crossing the wire. Use absolute non-empty file paths, stay within documented frame/resource limits, and treat platform-unsupported image clipboard operations as explicit errors.

## TypeScript resolves the wrong JSX types

Use:

```json
{
  "extends": "./.solid-gpui/tsconfig.json",
  "compilerOptions": {
    "jsx": "preserve",
    "jsxImportSource": "@solid-gpui/core"
  }
}
```

The generated project carries the `#native` and `@solid-gpui/core/components`
mappings that match the Vite aliases, so re-run `bun run generate` after changing
the host instead of editing `paths` by hand.

Use the Solid/Oxc universal transform through the [Vite plugin](vite.md). A generic React-style JSX transform cannot generate this
renderer's reactive host operations.

For `For`/`Show`/`Index`/`Switch`/`Match` return-type or children errors, import
those components from `@solid-gpui/core/runtime`, not `solid-js`. Solid's upstream
control-flow declarations use DOM elements; the runtime entry exposes the same
implementations with native types. See [native control flow](native-composition.md#solid-async-control-flow).

## A QuickJS application has no DOM or asset types

Use the published ambient types instead of hand-writing declarations:

```json
{
  "compilerOptions": {
    "lib": ["ES2024"],
    "types": ["@solid-gpui/core/quickjs"]
  }
}
```

They declare the timers, `performance.now()`, `console`, UTF-8 text codecs, `self`,
the router's URL/event/abort/header and bodyless `Response` primitives,
`import.meta.url`, and `declare module "*?inline"` with a default `string` export, so
an inline PNG import type-checks without a local `.d.ts`. They deliberately declare
no `fetch`/`Request`, filesystem or socket API, `requestAnimationFrame`,
`import.meta.hot`, or Node/Bun API: a compile error there is real, because the
engine does not provide the capability. The prepared `.solid-gpui/tsconfig.json`
adds the entry for a QuickJS project; re-run `bun run generate` after changing the
runtime.

## A page is blank, clipped, or cannot reach its final row

Check stderr first: a rejected commit is not a layout failure. Native data must
match its generated contract; raw labels belong in `Text`.

For layout failures, check the complete parent chain, including router shells.
`flexGrow` only participates in the parent's layout; it does not enable flex on
the node. Explicitly use columns where remaining **height** is distributed.
Bound the scroll viewport with `height: 0, flexGrow: 1, minHeight: 0` under a
bounded column, and keep its content naturally sized with `flexShrink: 0`.
Core `overflow: "scroll"` supports wheel scrolling, but does not create a visible
scrollbar. Use `Scrollable` when one is needed. See the runnable
[bounded page recipe](scroll-performance.md#bounded-page-scrolling).

## An Iconify name is rejected

The full Iconify library is not bundled. Import `ICON_NAMES` from
`@solid-gpui/core` to browse built-ins. For another icon, embed its SVG in the
host and use the generated `applicationIcons` name; do not cast arbitrary
strings to `IconName`. See [application icons](iconify.md#add-application-icons).

## Scrolling is unbounded or slow

Check the flex parent all the way through router wrappers, then check whether
the window-filling work area starts from an unnecessary content-derived flex
basis. See [native scroll performance](scroll-performance.md) for the actual
Gallery regression, calibrated CPU budget and native acceptance requirements.

## Dependency upgrade breaks tooling

TypeScript 7 exposes compiler services under `typescript/unstable/async`. The
root `typescript` export no longer supplies `createProgram`. Use the async API
under Bun; the synchronous client's private Node pipe handles are unavailable.
Await `api.close()` so snapshot disposal finishes before closing its connection.
Regenerate API fixtures and run the semantic re-export test as well as
`package-typecheck` after upgrades. Keep the official Solid compiler pinned:
its release-candidate version is separate from the Solid 1 runtime, and the
shared transform disables Solid 2 built-in auto-imports. Compiler upgrades
must preserve reactive updates, owner cleanup, import side effects, and source
maps in both Vite development and production builds.

## Correlating native command failures

Import `NativeCommandError` from `@solid-gpui/core` or its `/native` entry.
Native rejection and malformed-result errors include `error.identity` with the
Surface, epoch, request, target node, command kind, and native function ID when
applicable. Record these fields alongside the error message to correlate with a
protocol trace. The identity retains no argument bytes or result payloads.
Application error strings can contain application data; choose their contents
accordingly. Signal cancellation preserves the original abort reason, and
transport shutdown retains its separate `TransportTerminatedError` type.
