# Desktop application example

A complete Solid GPUI desktop application: SolidJS pages and layout, a Rust host
that owns the window, and Vite as the bundler. Use it as a starting point for an
ordinary desktop app. The [guides](../../docs/README.md) describe the individual
APIs; this example shows them composed into one application.

[中文说明](README.zh-CN.md)

## Overview

- One native window containing a router-driven SolidJS tree: a titlebar shell plus
  the Home and Settings routes.
- `desktop-app-host` (Rust) configures the native window and titlebar options, registers the
  embedded brand icon before the window opens, and exposes the `desktop` native
  module with the `serviceCount` command.
- `@solid-gpui/vite` builds that host and exports its bindings to `src/native.ts`,
  which is how TypeScript learns the component catalog and the `serviceCount` signature.
- Rendering is offline: the cover image is read from disk at runtime, the brand
  icon is compiled into the executable.

## Running the example

Use the repository's pinned Bun and Rust toolchains. Platform build prerequisites
are listed in [Getting started](../../docs/getting-started.md).

The shared SDK must be installed and built once at the repository root —
`@solid-gpui/vite` resolves to its built `dist/`, and the workspace packages link
by name:

```sh
bun install --frozen-lockfile
bun run build                 # workspace packages and binaries
bun run --cwd examples/desktop-app dev
```

`dev` starts Vite, builds the host, regenerates `src/native.ts`, and launches the
window; saving a TSX file reloads the page in place.

The same commands from this directory:

```sh
bun run dev        # Vite builds and launches the host, then serves the app to it
bun run dev:rust   # host first: it starts Vite itself
bun run build      # writes dist/main.js
bun run start      # runs the host against dist/main.js with --production
```

`dev` and `dev:rust` reach the same development setup from opposite ends: either
Vite spawns the host, or the host spawns Vite. Production runs the host in this
directory so `bun dist/main.js` resolves `assets/cover.png`.

## What the example demonstrates

### Home route layout and paint

A cover image stretched to fill a fixed-height hero block with `widthPercent` and
`heightPercent`, a gradient overlay drawn with `linearGradient` stops that fade
into the page background, and a row of cards that reflows because each carries
`minWidth` with `flexGrow`. A single button calls `serviceCount` and renders the
returned value, so the Rust round trip is visible in the UI. See
[Layout and paint](../../docs/native-composition.md#layout-and-paint).

### Bounded settings page

The Settings route is a fixed header, a scrolling 14-row form, and a fixed
footer. The scroll viewport uses `height: 0` with `flexGrow: 1` and `minHeight: 0`
inside a column that is itself bounded by the shell, so only the middle section
scrolls and the final row stays reachable. The page also shows Solid `For`/`Show`
and a controlled `Select` whose choices can be loaded and unloaded while the
application-owned value survives. See
[bounded page scrolling](../../docs/scroll-performance.md#bounded-page-scrolling).

### Application icon

`native/src/main.rs` compiles `assets/brand.svg` into the binary and registers it
as `desktop:brand`; the generated `applicationIcons` export makes that name
available to the titlebar `<Icon>`. Registration happens before the window opens,
so the icon needs no file at runtime. See
[Add application icons](../../docs/iconify.md#add-application-icons).

## File roles

| Path                            | Role                                                                                                              |
| ------------------------------- | ----------------------------------------------------------------------------------------------------------------- |
| `src/main.tsx`                  | Application entry: theme, router, titlebar shell, Home and Settings routes, `mountApplication` setup.             |
| `src/native.ts`                 | Generated from the built host. Do not edit; regenerate instead.                                                   |
| `native/src/main.rs`            | Rust host: window options and titlebar, embedded icon, `desktop` native module, dev/production runtime selection. |
| `native/Cargo.toml`             | The `desktop-app-host` crate manifest, a member of the workspace.                                                 |
| `vite.config.ts`                | Vite root, the `solidGpui` plugin options, and absolute aliases to the SDK sources.                               |
| `assets/cover.png`              | Cover image read from disk at runtime; must ship next to the app.                                                 |
| `assets/brand.svg`              | Brand icon compiled into the executable.                                                                          |
| `tsconfig.json`, `package.json` | Type checking and the scripts above.                                                                              |

`src/native.ts` is regenerated whenever Vite prepares a session, and can be
regenerated alone with:

```sh
cargo run --manifest-path native/Cargo.toml -- --export-native > src/native.ts
```

(`--export-native` prints the bindings; the plugin additionally formats them.)

## Packaging

Ship `desktop-app-host`, `dist/main.js`, and `assets/cover.png`, retaining the
relative paths and using this directory as the host's working directory. This
example starts an external Bun process, so `bun` must be available on `PATH`.
The brand icon is embedded; neither image needs a network request. See
[Desktop distribution](../../docs/distribution.md) for native dependencies,
application bundles, and signing.

## Further reading

- [Desktop host configuration](../../docs/rust-bridge.md#desktop-host-configuration)
  and [window options and titlebar](../../docs/rust-bridge.md#window-options-and-titlebar)
  for the host and window setup used by `native/src/main.rs`.
- [Application theme overrides](../../docs/gpui-components.md#application-theme-overrides)
  for how `theme` is applied after mount.
- [Native grid](../../docs/native-composition.md#native-grid) for the grid layout
  fields.
- [Rust bridge](../../docs/rust-bridge.md) for native modules, commands, and DTOs.
