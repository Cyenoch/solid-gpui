# @solid-gpui/core

SolidJS universal renderer for native GPUI surfaces.

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

Launch a direct JS application with `solid-gpui-host bun --conditions=browser app.js`
so the host owns protocol stdio and Solid resolves its client reactive runtime.
Bun APIs remain available. See [Vite integration](../../docs/vite.md) for both
authoring paths and application-owned native modules.

## Runtime selection

External Bun is the rapid development runtime. Embedded Bun is the intended
production packaging runtime for Bun-based applications; its current embedding
is macOS-only and its release pipeline remains separate work. QuickJS targets
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

```sh
bun --bun vite build
solid-gpui-host --runtime quickjs dist/app.js
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

## Protocol

The renderer speaks the lockstep framed Bebop v5 protocol. Each frame begins
with a four-byte little-endian payload length and carries a bounded Envelope.
The canonical schema is
`src/protocol/protocol.bop`; checked TypeScript bindings and schema metadata
are generated from it, and the native host consumes matching generated Rust
bindings. A schema-derived guard rejects malformed fields, unions, enums,
strings, and repeated values before generated decoding.

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
With Vite's `native` option, component imports and Motion use bindings exported
by the selected host. Rust edits automatically rebuild and replace its development
session; compilation and application failures keep Vite watching for a corrected
source edit. See [managed development sessions](../../docs/hot-reload.md#managed-development-sessions).
Component icon props use registered Iconify names or explicit SVG data; GPUI Kit's
internal assets do not extend the application icon catalog. See the repository's
`docs/gpui-components.md` and `docs/iconify.md` guides for the generated contracts.
