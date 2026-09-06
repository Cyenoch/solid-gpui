# Solid GPUI

Solid GPUI renders SolidJS owner trees into native GPUI surfaces. SolidJS owns reactive application state and composition; Rust and GPUI own the validated retained tree, native interaction state, layout, and painting.

> Early-stage work. Public APIs and protocol details may change without compatibility wrappers.

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
solid-gpui-host
  validated NodeStore → native GPUI windows
```

The TypeScript renderer is built with `solid-js`'s client reactive runtime and a custom universal host. A root update is atomic at the wire boundary: the first update emits a Snapshot and later updates emit deterministic Patches. Native events run inside the matching Solid owner transaction; signal-driven mutations outside an event are coalesced into one microtask commit.

The host model follows the useful boundary demonstrated by `references/gpui-component/crates/shell`: script code owns composition and business state, while the Rust host owns rendering, layout, native input, and system capabilities. Solid GPUI uses a lockstep framed Bebop v5 protocol for the external Bun process and embedded adapter.

## Packages

- `packages/solid-gpui` — `@solid-gpui/core`, the Solid universal renderer, protocol encoder, transports, and native component functions.
- `packages/solid-gpui-router` — `@solid-gpui/router`, the DOM-free TanStack Router Core adapter, native links, and per-surface memory history.
- `crates/solid-gpui` — Rust SDK, host, native components and optional embedded runtime; `gpui-component` is an opt-in feature.
- `crates/solid-gpui-macros` — internal Rust authoring macros, re-exported by the SDK.
- `crates/solid-gpui-bun-sys` — optional internal Bun FFI/build integration.
- `examples/gallery/native` — an application's own Rust module, exporting its components and commands through its actual host.

- `references/gpui-component/` — checked-in GPUI Component reference source, including GPUI Shell.

## Component model

Components are ordinary functions returning host nodes. Use the renderer's `createComponent` so component execution remains attached to the correct Solid owner:

```ts
import { Pressable, Text, View, createRoot, StdioTransport } from "@solid-gpui/core";
import { createComponent, createSignal } from "@solid-gpui/core/runtime";

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

For JSX, compile with the Solid Babel transform in universal mode and set `moduleName` to `@solid-gpui/core/runtime`. `jsxImportSource: "@solid-gpui/core"` selects host element types only; it is not an automatic JSX runtime. The non-JSX form above has no compiler dependency and is the repository's executable example.
Direct Bun entrypoints must use `bun --conditions=browser run app.ts` so
`solid-js` resolves its client reactive runtime. The repository task commands
already apply this condition.

## Routing and shared application state

Create one router per native surface. Each router owns its location, history,
params, search state, and route lifecycle. Routers may reuse the same finalized
route tree; do not mutate route options or children after creating the first
router.

Application-global data is separate from routing. Create stores, query clients,
and services once in the Bun/JSC application runtime, then pass the same object
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

Write ordinary logic in Rust with `native_module!` and call it through generated typed Promise clients. Native component providers use the reusable schema macros for properties and events; gpui-component is one provider. See [Rust authoring](docs/rust-bridge.md).

## Development

Install the root Bun workspace once:

```sh
bun install --frozen-lockfile
```

The Commander CLI in `scripts/tasks.ts` is the single task entrypoint:

| Command                    | Purpose                                                |
| -------------------------- | ------------------------------------------------------ |
| `bun run build`            | Build JavaScript and declaration artifacts.            |
| `bun run format`           | Check Rust, TypeScript, and JSON formatting.           |
| `bun run check`            | Build, typecheck, and lint the workspace.              |
| `bun run test`             | Run the Rust and Solid renderer tests.                 |
| `bun run ci`               | Run formatting, checks, tests, pack smoke, and audits. |
| `bun run audit`            | Audit dependencies and verify third-party notices.     |
| `bun run gallery`         | Launch the native component workbench and Rust API demo. |

The runtime uses the unified `gpui-pre`/platform 0.3.3 family and gpui-component 0.6.0. Zed in `references/` is implementation reference source, not a patched runtime dependency. Native layout dependencies are optimized in development; see [scroll diagnosis and regression](docs/scroll-performance.md).

Run `bun run task --help` for protocol generation, API surface, release,
embedded-adapter, and host-candidate commands. Core task dispatch uses
Bun-native process APIs and argument arrays. Embedded Bun/JSC remains macOS-only;
release and stress helpers may additionally require Bash and platform tools.

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

Apache-2.0. Dependency attribution is in [`THIRD-PARTY-NOTICES.md`](THIRD-PARTY-NOTICES.md).

## Native hot reload

The independent projects are `examples/gallery` (Bun) and `examples/gallery-vite` (Vite + Bun).
Run `bun run gallery` or `bun run gallery:vite` from the workspace root. The Vite project imports the shared application from the Bun project.
See [Vite + Bun guide](docs/hot-reload.md) for application integration and explicit state preservation.
