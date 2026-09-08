# Troubleshooting

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

## A signal changes but no Patch is emitted

Import reactive primitives from `@solid-gpui/core/runtime`, not a server-targeted Solid entry. Create the application inside the producer passed to `root.render`:

```ts
root.render(() => createComponent(App, {}));
```

Creating host nodes before entering the root has no surface owner and is rejected.

## `host operation is not associated with a root`

A component or host node was created outside a root producer or a retained Solid owner. Move creation into `root.render(() => ...)`. Do not share Host Nodes between roots.

## Raw text validation fails

Raw string/number children are valid only directly under `Text`. Wrap labels in `Text` rather than placing strings under `View` or `Pressable`.

## Protocol version mismatch

The TypeScript package and native host must use the same protocol version. Rebuild both from the same checkout; do not add compatibility decoding.

## Surface is closed

A closed or unmounted surface ID is permanently retired. Create a new surface and root instead of reusing the ID with another generation.

## Embedded Bun build fails

The embedded Bun/JSC adapter is macOS-only. Use `bun run website:native` for process
mode. An embedded host built with `embedded-bun` accepts an explicit application entry and
launches the embedded path; if that task fails, verify the pinned Bun source can
be fetched and that the generated native graph matches the checked-in patch.

## QuickJS cannot resolve a service or transport

Use `EmbeddedTransport` from `@solid-gpui/core/embedded` with the QuickJS host.
`StdioTransport` belongs to Bun's process or embedded stdio environment and
requires `process.stdin` and `process.stdout`.

Build with `solid-gpui-build --runtime quickjs <entry.tsx> <output.js>` so
dependencies are included in one ESM module. Node/Bun imports are rejected;
ambient `process`, `Bun`, filesystem, and network APIs such as `fetch` are
unavailable. Move those services into Rust Native Modules and call the
generated clients. The bundler's browser target selects portable dependencies;
it does not create a browser environment in QuickJS.

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

## File, clipboard, or dialog command rejects

Commands are bounded and validate arguments before crossing the wire. Use absolute non-empty file paths, stay within documented frame/resource limits, and treat platform-unsupported image clipboard operations as explicit errors.

## TypeScript resolves the wrong JSX types

Use:

```json
{
  "compilerOptions": {
    "jsx": "preserve",
    "jsxImportSource": "@solid-gpui/core"
  }
}
```

Use the shared Solid/Oxc universal transform through the Vite plugin or
`solid-gpui-build`. A generic React-style JSX transform cannot generate this
renderer's reactive host operations.

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
maps in both the Bun preload and Vite paths.


## Correlating native command failures

Import `NativeCommandError` from `@solid-gpui/core` or its `/native` entry.
Native rejection and malformed-result errors include `error.identity` with the
Surface, epoch, request, target node, command kind, and native function ID when
applicable. Record these fields alongside the error message to correlate with a
protocol trace. The identity retains no argument bytes or result payloads.
Application error strings can contain application data; choose their contents
accordingly. Signal cancellation preserves the original abort reason, and
transport shutdown retains its separate `TransportTerminatedError` type.
