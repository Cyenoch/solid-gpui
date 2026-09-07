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

For JSX, use `@solid-gpui/core/vite` or `solid-gpui-build`. Both use the pinned official Oxc-based Solid universal compiler; Babel is not required. TypeScript `jsxImportSource: "@solid-gpui/core"` supplies host element types only; it is not an automatic JSX runtime.

Launch direct Bun entrypoints with `bun --conditions=browser run app.ts` so
`solid-js` resolves the client reactive runtime.

## Runtime selection

Bun supports Rust-led applications and applications whose business logic and host
services run in Bun. QuickJS provides a lightweight embedded UI runtime for
Rust-led applications. Rust owns GPUI rendering in every mode.

`mountApplication` requires an explicit `transport` factory and owns its lifetime:

```tsx
import { mountApplication, Text } from "@solid-gpui/core";
import { EmbeddedTransport } from "@solid-gpui/core/embedded";

mountApplication({
  transport: () => new EmbeddedTransport(),
  setup: () => ({ render: () => <Text>Hello from QuickJS</Text> }),
});
```

Use `StdioTransport` from `@solid-gpui/core/stdio` for Bun, including embedded Bun.
Use `EmbeddedTransport` for the QuickJS host bridge. All surfaces in an application
share its connection.

```sh
solid-gpui-build --runtime quickjs app.tsx dist/app.js
solid-gpui-host --runtime quickjs dist/app.js
```

The host must be compiled with Cargo feature `quickjs`. The bundle includes the
Solid client runtime and must be a self-contained ES module. Node/Bun service
imports and unresolved dynamic imports are rejected; expose Rust services through
generated native commands. For Bun applications, build with `--runtime bun` and
launch the result through the Bun host mode.

## Protocol

The renderer speaks the lockstep framed Bebop v5 protocol. Each frame begins
with a four-byte little-endian payload length and carries a bounded Envelope.
The canonical schema is
`src/protocol/protocol.bop`; checked TypeScript bindings and schema metadata
are generated from it, and the native host consumes matching generated Rust
bindings. A schema-derived guard rejects malformed fields, unions, enums,
strings, and repeated values before generated decoding.

## Repository Gallery

Install the workspace with `bun install --frozen-lockfile`, then run `bun run gallery`
for the direct Bun project or `bun run gallery:vite` for Vite + Bun hot reload.
Shared application code lives in `examples/gallery`; the Vite project imports it.
