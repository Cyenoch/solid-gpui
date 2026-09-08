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
