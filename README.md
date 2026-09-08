<p align="center">
  <img src="assets/branding/solid-gpui.png" width="96" height="96" alt="Solid GPUI logo" />
</p>

<h1 align="center">Solid GPUI</h1>

<p align="center">
  <strong>Build native apps. Stay with Solid.</strong>
</p>

<p align="center">
  <a href="https://cyenoch.github.io/solid-gpui/">Website (Pages)</a> ·
  <a href="docs/getting-started.md">Get started</a> ·
  <a href="docs/gpui-components.md">Components</a> ·
  <a href="docs/README.md">Documentation</a>
</p>

Build desktop interfaces in JavaScript or JSX/TSX. **SolidJS manages reactive state;
Rust and GPUI handle native layout, rendering, and input.**

The shared website brings together interactive guides, a component catalog, and
Showcase apps. It runs on desktop and in the browser through an experimental
Rust WebAssembly host.

> Early-stage software: APIs may change. GitHub Pages is awaiting its first
> deployment; you can run the website locally with the commands below.

## What you get

- **Familiar Solid.** Signals, components, and reactive TSX with the
  [Solid GPUI renderer](packages/solid-gpui).
- **Native building blocks.** Text editing, virtual lists, themes, and optional
  [gpui-component controls](docs/gpui-components.md), from buttons to docking panels.
- **Typed Rust integration.** Export Rust functions and components with
  [generated TypeScript bindings](docs/rust-bridge.md).
- **Tools for complete apps.** [Routing](packages/solid-gpui-router),
  [code highlighting](docs/shiki.md), and [Vite development and builds](docs/vite.md).

## Try it locally

Install [Bun](.bun-version), the pinned [Rust toolchain](rust-toolchain.toml), and
your platform's [native build dependencies](docs/distribution.md#build-environment).

```sh
git clone https://github.com/Cyenoch/solid-gpui.git
cd solid-gpui
bun install --frozen-lockfile
bun run website:native
```

For native hot reload, run `bun run website:native:dev`. For the browser version,
follow the [Web setup guide](docs/web.md), then run `bun run website`.

## A small example

```tsx
import { Pressable, Text, View } from "@solid-gpui/core";
import { createSignal } from "@solid-gpui/core/runtime";

export function Counter() {
  const [count, setCount] = createSignal(0);

  return (
    <View style={{ padding: 24, gap: 12 }}>
      <Text>Count: {count()}</Text>
      <Pressable accessibilityRole="button" onPress={() => setCount((value) => value + 1)}>
        <Text>Increment</Text>
      </Pressable>
    </View>
  );
}
```

Write JS directly without a bundler, or use [@solid-gpui/vite](docs/vite.md) to
compile JSX/TSX. Bun and QuickJS execute the resulting JavaScript; selecting Bun
keeps Bun APIs available. Both paths support Rust native modules. Then
[mount your application](docs/getting-started.md#mount-a-root) in a GPUI host.

## Go further

| Guide                                                | What you will learn                                                      |
| ---------------------------------------------------- | ------------------------------------------------------------------------ |
| [Choose a runtime](docs/runtimes.md)                 | Develop with Bun, embed Bun on macOS, or use QuickJS with Rust services. |
| [Native application guide](docs/native-migration.md) | Configure windows, titlebars, themes, and application services.          |
| [Desktop distribution](docs/distribution.md)         | Build, verify, and sign application bundles.                             |
| [Architecture and protocol](CONTEXT.md)              | Understand ownership, rendering, and the Rust–TypeScript boundary.       |

For workspace development, use `bun run check` and `bun run test`.
Run `bun run task --help` for all tasks; see the
[documentation index](docs/README.md) for the full guides.

## License

[MIT](LICENSE) for project-owned code.
[Third-party notices](THIRD-PARTY-NOTICES.md) cover vendored code, fonts, and icons.
