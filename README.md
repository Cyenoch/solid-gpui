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

## Build an application

The SDK is not on a public registry yet. Build matching tarballs from one pinned
checkout (this repository), then install them into your application. Use
[Bun](.bun-version) 1.4.2 or newer, and add the pinned
[Rust toolchain](rust-toolchain.toml) plus your platform's
[native build dependencies](docs/distribution.md#build-environment) when the
application builds a Rust host.

```sh
# In the SDK checkout
bun install --frozen-lockfile
bun run task sdk-pack ../sdk-tarballs   # solid-gpui-core/-vite/-router/-shiki.tgz

# In your application
bun init
bun add ../sdk-tarballs/solid-gpui-core.tgz solid-js
bun add -d ../sdk-tarballs/solid-gpui-vite.tgz vite
```

Point `vite.config.ts` at your entry and native host, extend the generated
TypeScript project, and run one sequence:

```sh
bun run generate     # solid-gpui prepare: build the host, export bindings, write .solid-gpui/tsconfig.json
bun run typecheck    # tsc --noEmit
bun run dev          # bun --bun vite, with Rust rebuilds and JS HMR
bun run test         # solid-gpui test, through the application's own Vite config
bun run build        # bun --bun vite build: the production bundle
bun run preview      # solid-gpui preview: the built host against the built bundle
```

Do not name a script `prepare`: package installation must not compile a native
host. [`docs/getting-started.md`](docs/getting-started.md) is the authoritative
version of this sequence. It also states the `[patch.crates-io]` and profile
requirements your Cargo workspace root must carry, and which step produces which
artifact — bindings, bundle, native executable, or distributable.

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
keeps Bun APIs available. The runtime is chosen explicitly at build time and is
never converted: a QuickJS build rejects Bun/Node imports instead of substituting
an engine. Both paths support Rust native modules.

## Run this repository

```sh
git clone https://github.com/Cyenoch/solid-gpui.git
cd solid-gpui
bun install --frozen-lockfile
bun run website:native
```

That builds and launches the shared website on the desktop. For native hot reload
run `bun run website:native:dev`; for the browser version follow the
[Web setup guide](docs/web.md) and run `bun run website`. Platform build
dependencies are in [`docs/distribution.md`](docs/distribution.md#build-environment).

The [desktop application example](examples/desktop-app/README.md) is a smaller
application to copy: one window, two routes, an application icon, and a Rust
command. Workspace development uses `bun run check` and `bun run test`;
`bun run task --help` lists every task.

## Go further

| Guide                                                         | What you will learn                                                          |
| ------------------------------------------------------------- | ---------------------------------------------------------------------------- |
| [Getting started](docs/getting-started.md)                    | The one external-consumer sequence, from install to production preview.       |
| [Choose a runtime](docs/runtimes.md)                          | Develop with external Bun, embed Bun, or use QuickJS with Rust services.      |
| [Rust integration](docs/rust-bridge.md)                       | Configure a desktop host, windows, titlebars, and typed native services.      |
| [Vite integration](docs/vite.md)                              | Plugin options, artifact lookup, testing, and source consumption.             |
| [Desktop application example](examples/desktop-app/README.md) | Run a complete application with routing, themes, scrolling, and Rust services. |
| [Desktop distribution](docs/distribution.md)                  | Bundle versus executable versus distributable, and per-target status.        |
| [Architecture and protocol](CONTEXT.md)                       | Understand ownership, rendering, and the Rust–TypeScript boundary.            |

See the [documentation index](docs/README.md) for the full guide list.

## License

[MIT](LICENSE) for project-owned code.
[Third-party notices](THIRD-PARTY-NOTICES.md) cover vendored code, fonts, and icons.

Native controls come from the pinned [GPUI Kit integration](docs/gpui-components.md).
Kit supplies styled controls, Base motion/custom controls and the desktop FPS HUD.
Its internal control assets are separate from application [Iconify icons](docs/iconify.md).
GPUI Shell is not part of the host.
