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
import { createRoot } from "@solid-gpui/core";
import { createComponent } from "@solid-gpui/core/runtime";
import { StdioTransport } from "@solid-gpui/core/stdio";
import { Counter } from "./counter";

const root = createRoot(new StdioTransport(), { surfaceId: 1, epoch: 1 });
root.render(() => createComponent(Counter, {}));
```

Pass a producer to `root.render`. This ensures component creation occurs under the root's Solid owner and binds later reactive work to the correct native surface.

## JSX

JavaScript runs directly without a bundler. JSX/TSX uses Vite with
`@solid-gpui/vite`, the official Oxc-based Solid universal transform for
`@solid-gpui/core/runtime`. Set TypeScript's `jsx` to `preserve` and
`jsxImportSource` to `@solid-gpui/core`; the latter selects host element types.
The compiler is pinned to `@solidjs/compiler` 2.0.0-rc.6 while the application
runtime remains Solid 1.9.15.

For a Rust-led application using QuickJS, import `EmbeddedTransport` from
`@solid-gpui/core/embedded` and build a self-contained ESM entry. See the
[runtime selection](runtimes.md) and
[Vite integration](vite.md).

## Run the website on desktop

From the workspace root, run `bun run website:native` to build and launch the website
with Bun, or `bun run website:native:dev` for native hot reload. Inside
`examples/website`, `bun run dev` starts the Web version and `bun run dev:native`
starts the native version. Both share the same application and
`@solid-gpui/router` route tree; see [hot reload](hot-reload.md).

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
