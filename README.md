# Solid GPUI

Solid GPUI renders SolidJS owner trees into native GPUI surfaces. SolidJS owns
reactive UI state and composition; Rust and GPUI own the validated retained
tree, native interaction state, layout, and painting. Application logic can
live primarily in Rust or Bun.

> Early-stage work. Public APIs and protocol details may change without compatibility wrappers.

![Solid GPUI's native component workbench on macOS, with English navigation, editable workspace controls, and a live Rust-backed preview.](docs/images/gallery.png)

The Gallery is a native GPUI window. It demonstrates reactive controls, text
editing, layout, virtual lists, routing, and application-owned Rust commands.

## Try the Gallery

Use the Bun version in [`.bun-version`](.bun-version) and the Rust toolchain in
[`rust-toolchain.toml`](rust-toolchain.toml). Install your platform's native tools
listed in [Build environment](docs/distribution.md#build-environment), then run
from the workspace root:

```sh
bun install --frozen-lockfile
bun run gallery
```

Use `bun run gallery:vite` for native hot reload. Start with the
[getting started guide](docs/getting-started.md), then follow
[native keyboard shortcuts and menus](docs/keyboard-and-menus.md) and
[desktop distribution](docs/distribution.md). The [documentation index](docs/README.md)
also covers native modules, protocol ownership, and performance measurement.
See the [Gallery guide](examples/gallery/README.md) for screenshot provenance and
the English-language presentation policy.

## Architecture

```text
Solid signals and components
          │
          ▼
@solid-gpui/core
  universal renderer → transactional host mutations
          │
          ▼
Snapshot bootstrap / incremental Patch frames
          │
          ▼
solid-gpui Rust host
  validated NodeStore → native GPUI windows
```

The TypeScript renderer is built with `solid-js`'s client reactive runtime and a custom universal host. A root update is atomic at the wire boundary: the first update emits a Snapshot and later updates emit deterministic Patches. Native events run inside the matching Solid owner transaction; signal-driven mutations outside an event are coalesced into one microtask commit.

The host model follows the useful boundary demonstrated by
`references/gpui-component/crates/shell`: script code composes the UI while
the Rust host owns native rendering, input, and system capabilities. Bun and
QuickJS use the same lockstep framed Bebop v5 protocol and generated native
command contracts.

## Runtime choice

Use Bun for applications with either Rust-owned or Bun-owned domain logic.
Use QuickJS as a lightweight embedded UI runtime when Rust owns application
services. This choice does not move GPUI rendering out of Rust.

| Runtime          | Application role                              | Transport           | Deployment                                            |
| ---------------- | --------------------------------------------- | ------------------- | ----------------------------------------------------- |
| Bun process      | Rust-led or Bun-led, with Bun services        | `StdioTransport`    | Default host mode; separate Bun process               |
| Embedded Bun/JSC | Rust-led or Bun-led, with Bun services        | `StdioTransport`    | Optional `embedded-bun` feature; macOS embedding      |
| Embedded QuickJS | Rust-led UI; services through native commands | `EmbeddedTransport` | Optional `quickjs` feature; self-contained ESM bundle |

QuickJS supplies UI scheduling and the native bridge. It does not expose
`process`, `Bun`, Node modules, or browser application APIs such as `fetch`.
Put file/network/domain operations in Rust Native Modules and call their
generated Promise clients. The QuickJS integration does not establish release
qualification on every target platform. See [ADR-0017](docs/adr/0017-runtime-engines.md)
for the ownership decision and [the build guide](docs/hot-reload.md#production-bundles)
for runtime-specific entrypoints.

## Packages

- `packages/solid-gpui` — `@solid-gpui/core`, the Solid universal renderer, protocol encoder, transports, and native component functions.
- `packages/solid-gpui-router` — `@solid-gpui/router`, the DOM-free TanStack Router Core adapter, native links, and per-surface memory history.
- `crates/solid-gpui` — Rust SDK, host, native components, and optional Bun/QuickJS runtimes; `gpui-component` is an opt-in feature.
- `crates/solid-gpui-macros` — internal Rust authoring macros, re-exported by the SDK.
- `crates/solid-gpui-bun-sys` — optional internal Bun FFI/build integration.
- `examples/gallery/native` — an application's own Rust module, exporting its components and commands through its actual host.

- `references/gpui-component/` — checked-in GPUI Component reference source, including GPUI Shell.

## Component model

Components are ordinary functions returning host nodes. Use the renderer's `createComponent` so component execution remains attached to the correct Solid owner:

```ts
import { Pressable, Text, View, createRoot } from "@solid-gpui/core";
import { createComponent, createSignal } from "@solid-gpui/core/runtime";
import { StdioTransport } from "@solid-gpui/core/stdio";

function Counter() {
  const [count, setCount] = createSignal(0);
  return createComponent(View, {
    get children() {
      return [
        createComponent(Text, { children: () => `Count: ${count()}` }),
        createComponent(Pressable, {
          accessibilityRole: "button",
          onPress: () => setCount((value) => value + 1),
          children: createComponent(Text, { children: "Increment" }),
        }),
      ];
    },
  });
}

const root = createRoot(new StdioTransport());
root.render(() => createComponent(Counter, {}));
```

For JSX, use `@solid-gpui/core/vite` or `solid-gpui-build`. Both use the official
Oxc-based Solid compiler in universal mode with `@solid-gpui/core/runtime`.
The compiler is pinned to `@solidjs/compiler` 2.0.0-rc.6; the application runtime
remains Solid 1.9.15. TypeScript's `jsxImportSource: "@solid-gpui/core"` selects
host element types only. The non-JSX example above needs no JSX compiler.
Direct Bun entrypoints must use `bun --conditions=browser run app.ts` so
`solid-js` resolves its client reactive runtime. The repository task commands
already apply this condition.

Application entrypoints choose their connection explicitly. Import
`StdioTransport` from `@solid-gpui/core/stdio` for Bun, or `EmbeddedTransport`
from `@solid-gpui/core/embedded` for QuickJS. Pass a transport factory to
`mountApplication`; it owns that connection and retains it during hot reload.

## Routing and shared application state

Create one router per native surface. Each router owns its location, history,
params, search state, and route lifecycle. Routers may reuse the same finalized
route tree; do not mutate route options or children after creating the first
router.

Application-global data is separate from routing. Create stores, query clients,
and services once in the application's JavaScript runtime, then pass the same object
references through each router context. Add per-window dependencies such as
`windowId` beside that shared object:

```ts
import { Outlet, createRootRouteWithContext, createRoute, createRouter } from "@solid-gpui/router";

interface RouterContext {
  app: {
    session: { userId: string | undefined };
  };
  windowId: string;
}

const app: RouterContext["app"] = {
  session: { userId: undefined },
};
const rootRoute = createRootRouteWithContext<RouterContext>()({ component: Outlet });
const homeRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/",
});
const routeTree = rootRoute.addChildren([homeRoute]);

export function createWindowRouter(windowId: string) {
  return createRouter({
    routeTree,
    context: { app, windowId },
  });
}
```

Solid contexts belong to one owner root and therefore do not cross windows.
The shared `app` reference above is the explicit application scope; each
router context remains a distinct window scope. TanStack Query, if used, stays
a separate data-cache concern and can be placed in `app` rather than coupled to
navigation.

## Rust exports

Declare application logic and native components in Rust with `#[native_module]`
and export their TypeScript bindings from the application's host. Generated
components, instance refs, and Promise clients share that Rust contract;
gpui-component controls are available through the optional integration.
See [Rust authoring](docs/rust-bridge.md). The generated
[gpui-component API](docs/gpui-components.md) includes controls, data views,
overlays, settings, docking, charts, and plot computations.

## Development

The Commander CLI in `scripts/tasks.ts` is the single task entrypoint:

| Command           | Purpose                                                                         |
| ----------------- | ------------------------------------------------------------------------------- |
| `bun run build`   | Build JavaScript and declaration artifacts.                                     |
| `bun run format`  | Check Rust, TypeScript, and JSON formatting.                                    |
| `bun run check`   | Build, typecheck, and lint the workspace.                                       |
| `bun run test`    | Run the Rust and Solid renderer tests.                                          |
| `bun run ci`      | Run the macOS CI gates, including protocol goldens and the host release bundle. |
| `bun run audit`   | Audit dependencies and verify third-party notices.                              |
| `bun run gallery` | Launch the native component workbench and Rust API demo.                        |

The runtime uses the unified `gpui-pre`/platform 0.3.3 family and gpui-component 0.6.0. Zed in `references/` is implementation reference source, not a patched runtime dependency. Native layout dependencies are optimized in development; see [scroll diagnosis and regression](docs/scroll-performance.md).

Run `bun run task --help` for protocol generation, API surface, release,
embedded-adapter, and host-candidate commands. Core task dispatch uses
Bun-native process APIs and argument arrays. Embedded Bun/JSC remains macOS-only;
release and stress helpers may additionally require Bash and platform tools.

`bun run ci` covers the macOS Actions checks locally. The separate
[cross-platform workflow](.github/workflows/cross-platform.yml) runs Linux and
Windows host checks; native interaction and release qualification require each
target's own runner and display. `bun run task host-candidate-smoke` builds and verifies
the archive before launching its extracted host. Release preparation and package
packing accept explicit operands: `release-prep <version>`, `package-pack <output>`,
and `router-package-pack <output>`. Candidate workflows upload artifacts; they do
not publish a release.

## Protocol and ownership

Every frame is a four-byte little-endian payload length followed by a bounded
Bebop v5 Envelope. The canonical schema lives in
`packages/solid-gpui/src/protocol/protocol.bop`; checked TypeScript and Rust
bindings plus schema metadata are generated from it. Both sides apply a
schema-derived guard before generated decoding, while the Rust process adapter
uses an owned Event queue and reusable writer buffer for native events. Surface
identity, generation, revision, node identity, command results, and event
sequence checks remain framework-independent. See [`docs/protocol.md`](docs/protocol.md)
for the wire contract and [`CONTEXT.md`](CONTEXT.md) for domain vocabulary.

## License

Project-owned code is licensed under [MIT](LICENSE). Third-party dependencies,
vendored code, fonts, and icons retain their own licenses and notices; see
[`THIRD-PARTY-NOTICES.md`](THIRD-PARTY-NOTICES.md).

## Native hot reload

The independent projects are `examples/gallery` (Bun) and `examples/gallery-vite` (Vite + Bun).
Run `bun run gallery` or `bun run gallery:vite` from the workspace root. The Vite project imports the shared application from the Bun project.
See [Vite + Bun guide](docs/hot-reload.md) for application integration and explicit state preservation.
