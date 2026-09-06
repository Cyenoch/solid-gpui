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

## Embedded runtime build fails

The embedded Bun/JSC adapter is macOS-only. Use `bun run gallery` for process
mode. An embedded host built with `embedded-bun` accepts an explicit application entry and
launches the embedded path; if that task fails, verify the pinned Bun source can
be fetched and that the generated native graph matches the checked-in patch.

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

Compile JSX with the Solid universal transform and `moduleName: "@solid-gpui/core/runtime"`.

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
Babel presets use `PresetTarget`, not `PluginItem`. Regenerate API fixtures and
run the semantic re-export test as well as `package-typecheck` after upgrades.
