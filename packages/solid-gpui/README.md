# @solid-gpui/core

SolidJS universal renderer for native GPUI surfaces.

```ts
import { Text, View, createRoot, StdioTransport } from "@solid-gpui/core";
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

For JSX, use the Solid universal transform with `moduleName: "@solid-gpui/core/runtime"`. TypeScript `jsxImportSource: "@solid-gpui/core"` supplies host element types only; it is not an automatic JSX runtime.

Launch direct Bun entrypoints with `bun --conditions=browser run app.ts` so
`solid-js` resolves the client reactive runtime.

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
