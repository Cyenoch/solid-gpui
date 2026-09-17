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
- Rendering is offline: the cover image is inlined into the bundle by Vite
  (`?inline`), and the brand icon is compiled into the executable.

## Running the example

Use the repository's pinned Bun and Rust toolchains. Platform build prerequisites
are in the [build environment](../../docs/distribution.md#build-environment).

The shared SDK must be installed and built once at the repository root —
`@solid-gpui/vite` resolves to its built `dist/`, and the workspace packages link
by name:

```sh
bun install --frozen-lockfile
bun run build                 # workspace packages and binaries
bun run --cwd examples/desktop-app dev
```

`dev` starts Vite, builds the host, exports its bindings, and launches the window;
saving a TSX file reloads the page in place.

The same commands from this directory:

```sh
bun run generate      # solid-gpui prepare: build the host and export src/native.ts
bun run check:generated   # fail when src/native.ts or .solid-gpui/ is stale
bun run doctor        # environment and Cargo profile/patch report
bun run typecheck     # tsc --noEmit
bun run test          # solid-gpui test, through this example's own Vite config
bun run dev           # Vite builds and launches the host, then serves the app to it
bun run dev:rust      # host first: it starts Vite itself
bun run build         # writes dist/main.js and prepares the host
bun run preview       # solid-gpui preview: built host against the built bundle
```

`tsconfig.json` extends the generated `.solid-gpui/tsconfig.json`, which carries
the `#native` and `@solid-gpui/core/components` mappings, so the example does not
hand-maintain `paths`; `vite.config.ts` uses `solidGpuiSource()` for the
workspace-source aliases. Do not name the generator `prepare`: Bun runs a root
package's `prepare` script during install, which must not compile a host.

`dev` and `dev:rust` reach the same development setup from opposite ends: either
Vite spawns the host, or the host spawns Vite. Preview and packaged runs start the
host from this directory, which is also the working directory the host uses for any
application-owned relative path.

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
as `desktop:brand`; the generated `applicationIcons` record makes that name
available to the titlebar `<Icon>` as `applicationIcons["desktop:brand"]`, without
depending on registration order. Registration happens before the window opens,
so the icon needs no file at runtime. See
[Add application icons](../../docs/iconify.md#add-application-icons).

## File roles

| Path                            | Role                                                                                                              |
| ------------------------------- | ----------------------------------------------------------------------------------------------------------------- |
| `src/main.tsx`                  | Application entry: theme, router, titlebar shell, Home and Settings routes, `mountApplication` setup.             |
| `src/native.ts`                 | Generated from the built host. Do not edit; run `bun run generate` instead.                                        |
| `src/integration.test.tsx`      | Renders the real application tree under `bun run test`, including the inlined cover asset and the registered icon. |
| `native/src/main.rs`            | Rust host: window options and titlebar, embedded icon, `desktop` native module, and `solid_gpui::runtime::vite::Vite` startup for both `dev:rust` and `preview`. |
| `native/Cargo.toml`             | The `desktop-app-host` crate manifest, a member of the workspace.                                                 |
| `vite.config.ts`                | Vite root, the `solidGpui` plugin options, and `solidGpuiSource()` for workspace-source aliases.                  |
| `.solid-gpui/`                  | Generated TypeScript project and artifact record. Not committed; recreated by `generate` and `build`.             |
| `assets/cover.png`              | Cover image imported with `?inline`, so Vite embeds it in the bundle; no runtime file.                            |
| `assets/brand.svg`              | Brand icon compiled into the executable.                                                                          |
| `tsconfig.json`, `package.json` | Type checking, the generated-project `extends`, and the scripts above.                                            |

`src/native.ts` is exported whenever a Vite session prepares the host. Regenerate
it alone with `bun run generate`, and use `bun run check:generated` to fail on a
stale file instead of writing when nothing changed.

## Packaging

Ship `desktop-app-host` and the built bundle, using this directory as the host's
working directory. Both images are embedded — the cover through `?inline`, the brand
icon in the executable — so neither needs a runtime file or a network request. `bun run
preview` runs the built host against the built bundle recorded in
`.solid-gpui/artifacts.json`, which is also where a packaging script should read
the executable and bundle paths instead of reconstructing them. This
example starts an external Bun process, so `bun` must be available on `PATH`.
See
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
