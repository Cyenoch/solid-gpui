# Getting started

## Install

```sh
bun install --frozen-lockfile
bun run task --help
bun run check
```

## Build a component

Solid GPUI's runtime module provides the client Solid primitives and the universal renderer's `createComponent` function.

```ts
import { Pressable, Text, View } from "@solid-gpui/core";
import { createComponent, createSignal } from "@solid-gpui/core/runtime";

export function Counter() {
  const [count, setCount] = createSignal(0);
  return createComponent(View, {
    style: { padding: 24, gap: 12 },
    get children() {
      return [
        createComponent(Text, { children: () => `Count: ${count()}` }),
        createComponent(Pressable, {
          accessibilityRole: "button",
          accessibilityLabel: "Increment",
          onPress: () => setCount((value) => value + 1),
          children: createComponent(Text, { children: "Increment" }),
        }),
      ];
    },
  });
}
```

The getter and function children are reactive reads. When `count` changes, the Solid render effect updates the raw-text Host Node and the root emits one incremental Patch.

## Mount a root

```ts
import { StdioTransport, createRoot } from "@solid-gpui/core";
import { createComponent } from "@solid-gpui/core/runtime";
import { Counter } from "./counter";

const root = createRoot(new StdioTransport(), { surfaceId: 1, epoch: 1 });
root.render(() => createComponent(Counter, {}));
```

Pass a producer to `root.render`. This ensures component creation occurs under the root's Solid owner and binds later reactive work to the correct native surface.

## JSX

JSX is optional. To use it, configure the Solid Babel transform for universal output and set its runtime module to `@solid-gpui/core/runtime`. Set TypeScript's `jsx` to `preserve` and `jsxImportSource` to `@solid-gpui/core`; the latter selects host element types and does not provide an automatic JSX runtime. The direct Bun Gallery uses the repository universal JSX preload; the Vite Gallery uses `@solid-gpui/core/vite`.

## Run Gallery

From the workspace root, run `bun run gallery` for direct Bun execution or
`bun run gallery:vite` for Vite bundling and native hot reload. Each project also
supports `bun run dev` from its own directory. Both use the shared application in
`examples/gallery`; see [hot reload](hot-reload.md).

## Test without a display

```ts
import { MemoryTransport, View, createRoot } from "@solid-gpui/core";
import { createComponent } from "@solid-gpui/core/runtime";

const transport = new MemoryTransport();
const root = createRoot(transport);
root.render(() => createComponent(View, {}));
console.log(transport.submitted.length); // Snapshot frame
root.unmount();
```

Use `MemoryTransport` for framed renderer output and root command contracts. Native layout, painting, dialogs, and platform window behavior require a display-backed host.
