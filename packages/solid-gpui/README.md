# @solid-gpui/core

SolidJS universal renderer for native GPUI surfaces.

```sh
bun add --exact @solid-gpui/core solid-js
```

Install matching SDK releases from npm. The native host is supplied separately;
see [Getting started](https://github.com/Cyenoch/solid-gpui/blob/main/docs/getting-started.md).

```ts
import { Text, View, createRoot } from "@solid-gpui/core";
import { StdioTransport } from "@solid-gpui/core/stdio";
import { createComponent, createSignal } from "@solid-gpui/core/runtime";

function App() {
  const [message] = createSignal("Hello");
  return createComponent(View, {
    children: createComponent(Text, { children: message }),
  });
}

createRoot(new StdioTransport()).render(() => createComponent(App, {}));
```

Host rendering is transactional: the first update emits a Snapshot and later signal updates emit incremental Patches. GPUI owns native layout, input, focus, and painting.

Write JavaScript directly without a bundler, or use the separate `@solid-gpui/vite` package to compile JSX/TSX with the pinned official Oxc-based Solid universal compiler. TypeScript `jsxImportSource: "@solid-gpui/core"` supplies host element types only; it is not an automatic JSX runtime.

Import `For`, `Index`, `Show`, `Switch`, and `Match` from
`@solid-gpui/core/runtime` for native JSX children. They use Solid's existing
runtime implementations with native element types; direct `solid-js` control-flow
declarations describe DOM children instead. Native JSX does not accept DOM nodes.

JSX callback, forwarded, and assignment refs share the runtime's `applyRef` ABI.
Callbacks preserve the Solid owner without tracking signal reads; nested callback
arrays compose in order. See [JSX refs](../../docs/vite.md#jsx-refs) for handle
types and cleanup semantics.

`@solid-gpui/core/runtime` is the client entry for every reactive helper, taken
from the one Solid instance the renderer uses:

- primitives and control flow: `createSignal`, `createMemo`, `createEffect`,
  `createComponent`, `For`, `Index`, `Show`, `Switch`, `Match`, `createRoot`,
  `batch`, `untrack`;
- owner and lifecycle: `onMount`, `onCleanup`, `onError`, `catchError`, `getOwner`,
  `runWithOwner`, `getListener`, `createComputed`, `createReaction`,
  `createDeferred`, `createUniqueId`;
- context and children: `createContext`, `useContext`, `children`;
- arrays and utilities: `mapArray`, `indexArray`, `splitProps`, `createSelector`,
  `on`, `observable`, `from`;
- store: `createStore`, `createMutable`, `modifyMutable`, `produce`, `reconcile`,
  `unwrap`, plus the `Store`, `SetStoreFunction`, and `StoreSetter` types.

Importing the renderer or this entry under a non-client resolution throws with an
actionable message instead of rendering nothing: Solid resolved to its server build,
or two copies of `solid-js` are loaded and signals cannot notify the renderer. See
[troubleshooting](../../docs/troubleshooting.md#tests-or-scripts-render-nothing).

Launch a direct JS application with `solid-gpui-host bun --conditions=browser app.js`
so the host owns protocol stdio and Solid resolves its client reactive runtime.
Bun APIs remain available. For a JSX/TSX application, `@solid-gpui/vite` compiles
the bundle, exports native bindings, provides the `solid-gpui` CLI (`prepare`,
`preview`, `doctor`, `test`), and runs application tests through the same Vite
configuration; see [Getting started](../../docs/getting-started.md) for the full
sequence and [Vite integration](../../docs/vite.md) for both
authoring paths and application-owned native modules.

Packages resolve to their built `dist` by default. Debugging SDK sources requires
the explicit `solid-gpui-source` condition (`bun --conditions=solid-gpui-source`)
plus the `solidGpuiSource()` plugin or the generated TypeScript project; Bun ignores
custom conditions in `bunfig.toml`.

## Runtime selection

External Bun is the rapid development runtime. Embedded Bun is the intended
production packaging runtime for Bun-based applications. The direct Rust
`embedded-bun` library feature supports macOS only; the separate static packager
also reaches experimental Windows targets. Its build and runtime evidence does
not establish supported Windows delivery. The authoritative
[distribution matrix](../../docs/distribution.md#platform-status-and-current-evidence)
records target qualification. QuickJS targets
applications whose main capabilities live in Rust, with JSX/TSX responsible for
UI composition and reactive presentation state. Rust owns GPUI rendering in
every mode. See [runtime strategy](../../docs/runtime-strategy.md).

`mountApplication` requires an explicit `transport` factory and owns its lifetime:

```tsx
import { mountApplication, Text } from "@solid-gpui/core";
import { EmbeddedTransport } from "@solid-gpui/core/embedded";

mountApplication({
  transport: () => new EmbeddedTransport(),
  setup: () => ({ render: () => <Text>Hello from QuickJS</Text> }),
});
```

Use `StdioTransport` from `@solid-gpui/core/stdio` for external Bun.
Use `EmbeddedTransport` for both Embedded Bun and QuickJS. All surfaces in an application
share its connection.

A DOM-free QuickJS application type-checks with the published ambient types instead
of its own declarations: set `"lib": ["ES2024"]` and
`"types": ["@solid-gpui/core/quickjs"]`. The entry declares exactly what the engine
provides (timers, `performance.now()`, `console`, UTF-8 text codecs, `self`, the
router's URL/event/abort/headers/bodyless-`Response` primitives, `import.meta.url`,
and `declare module "*?inline"` for inlined assets) and deliberately omits `fetch`,
`requestAnimationFrame`, `import.meta.hot`, filesystem/socket access, and every
Node or Bun API, so an unavailable capability fails at compile time. The prepared
`.solid-gpui/tsconfig.json` already includes it for a QuickJS project.

The default `StdioTransport` reads the host-owned process stdin. When that pipe
closes, it notifies termination listeners and exits the renderer, even if
application timers are still active. Clean EOF exits with status 0; I/O failure
exits with status 1 and a diagnostic. Application-owned detached services are
not terminated. `dispose()` only releases the connection. Embedders can pass
`{ input, output }` streams, or explicitly set `{ exitOnHostClose: false }` when
their process must outlive its host connection.

```sh
bun --bun vite build
solid-gpui preview          # or: solid-gpui-host --runtime quickjs dist/app.js
```

Configure `solidGpui({ entry: "app.tsx", runtime: "quickjs" })` in Vite.
The host must be compiled with Cargo feature `quickjs`. The bundle includes the
Solid client runtime and must be a self-contained ES module. Node/Bun service
imports and unresolved dynamic imports are rejected; expose Rust services through
generated native commands. For Bun applications, configure Vite with `runtime: "bun"` and
launch the result through the Bun host mode.

## System popovers

`SystemPopover` from `@solid-gpui/core` creates an owned native popup Surface that
can cross its parent window boundary. Its content factory shares Solid context
and signals while keeping native input and disposal local to the child Surface.
See [System popovers](../../docs/system-popover.md) for controlled forms,
platform support, multi-display placement, and native verification limits.
Use the generated `Popover` for in-window content, including browser UI.

## Headless application tests

Use `TestHost` from `@solid-gpui/core/testing` with `createRoot(host.transport)`.
It can also wrap an existing `MemoryTransport`, including already-submitted
frames. `surface(id)` exposes an ordered committed tree after replaying Snapshot
and Patch updates; `dispatch` delivers test events, `nativeProps` decodes native
component DTOs, and `nativeCalls` plus `reply`/`reject` inspect and settle native
requests without importing the generated protocol. Previous tree views retain
their revision and epoch for stale-event tests. Unmount roots after each test.

Inspect submitted styles, accessibility/disabled state and virtual-list ranges.
Input selections use UTF-16 code units and default to `text.length`; `inputState`
exposes the committed edit acknowledgement and selection. Input dispatch emits
change followed by selection, matching native editing order.
Inject explicit visible-range, layout and pointer observations through `dispatch`.
`scrollCommands` and `replyScroll` expose offset queries and scroll actions;
unanswered requests stay pending. These observations do not simulate GPUI layout.

Use the opt-in `NativeAcceptance` export for real native paint/geometry and
hit-tested input. It owns an explicitly launched native executable, supports
click/type/drag/wheel and stable captured targets, and provides Metal PNG
screenshots on macOS. Deterministic mode uses GPUI's native test platform; GPU
mode and physical input have distinct qualification limits. See the
[native acceptance guide](../../docs/native-acceptance.md) for build commands,
custom production HostProfiles, cleanup, and supported platforms.

Run TSX tests through `solid-gpui test` so they share the application's Vite
configuration and one Solid runtime. The test process keeps Bun's environment
even for QuickJS applications; this does not relax production UI capabilities.
The [testing guide](../../docs/vite.md#testing) documents the complete interface,
scheduling and cleanup rules. Native layout/painting and platform services still
require a real host.

## Protocol

The renderer speaks the lockstep framed Bebop v7 protocol. Each frame begins
with a four-byte little-endian payload length and carries a bounded Envelope.
The canonical schema is
`src/protocol/protocol.bop`; checked TypeScript bindings and schema metadata
are generated from it, and the native host consumes matching generated Rust
bindings. A schema-derived guard rejects malformed fields, unions, enums,
strings, and repeated values before generated decoding.
Version 7 adds explicit native length units, flex basis, axis overflow and native
hover/active/focus-visible paint refinements. Rebuild JavaScript and native hosts
together; older frames are rejected. See [layout and paint](../../docs/native-composition.md#layout-and-paint)
for spacing precedence, supported units and state-style constraints.

## Repository website

Install the workspace with `bun install --frozen-lockfile`, then run `bun run website:native`
for the native website or `bun run website:native:dev` for Vite + Bun hot reload.
Web and desktop share the application and router in `examples/website`.

## Images

`Image.source` and `fallbackSource` accept HTTP(S) URLs directly, local paths,
native `file:` URLs, and `data:image/...` URLs. GPUI owns loading and caching;
no JavaScript fetch is required. See [Images](../../docs/native-composition.md#images)
for an example, fallback behavior, and custom host setup.

Use `@solid-gpui/core/motion` for native Motion and lifetime-aware Presence.
With Vite's `native` or explicit `host` option, component imports and Motion use
bindings exported by the selected host. `native` additionally watches Rust edits
and automatically rebuilds and replaces its development
session; compilation and application failures keep Vite watching for a corrected
source edit. See [managed development sessions](../../docs/hot-reload.md#managed-development-sessions).
Component icon props use registered Iconify names or explicit SVG data; GPUI Kit's
internal assets do not extend the application icon catalog. See the repository's
`docs/gpui-components.md` and `docs/iconify.md` guides for the generated contracts.
