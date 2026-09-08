# Web host and GitHub Pages

The website uses Solid GPUI components and the universal renderer. Rust GPUI
renders the Landing Page, documentation and examples into a browser
canvas. It does not translate the component tree into DOM elements.

## Build and run

From the repository root:

```sh
bun install --frozen-lockfile
rustup toolchain install nightly-2026-07-28 --profile minimal --target wasm32-unknown-unknown
cargo +nightly-2026-07-28 install wasm-bindgen-cli --version 0.2.121 --locked
bun run website
```

Open `http://127.0.0.1:5173/solid-gpui/`. The build writes generated WASM bindings
into `examples/website/src/wasm/` and the deployable site into
`examples/website/dist/`. Neither directory is committed. Browser `dev` and `build` commands first run
`scripts/build-web-host.sh` to regenerate SDK bindings and rebuild the WASM host
from the same Rust sources. Restart `dev` after Rust or protocol changes; Vite
handles TypeScript/TSX changes. Calling Vite directly bypasses this preparation
and can pair new component bindings with an obsolete host catalog.

Set `WASM_BINDGEN` to a matching executable if your default wasm-bindgen version
differs. The CLI must match the `wasm-bindgen` version in `Cargo.lock` exactly;
a newer CLI is not interchangeable. To replace an already installed version, run:

```sh
cargo +nightly-2026-07-28 install wasm-bindgen-cli --version 0.2.121 --locked --force
wasm-bindgen --version
bun run website:build
```

This replaces the CLI in Cargo's installation directory. If another project
needs a different version, install with `--root <directory>` and point
`WASM_BINDGEN` at `<directory>/bin/wasm-bindgen` for this project's build.

Set `PAGES_BASE_PATH=/` for hosting at a domain root. The default is
`/solid-gpui/`; all local assets respect that prefix. Hash routes keep
Docs deep links refreshable on static hosting.

## Rendering and ownership

`crates/solid-gpui-web` uses the pinned gpui-pre 0.3.3 Web platform in
single-threaded mode. It needs no SharedArrayBuffer or cross-origin isolation
headers. The upstream wasm_thread dependency still requires the pinned nightly
at compile time. The host embeds licensed Inter, IBM Plex Sans, Lilex, and Noto Sans SC fonts;
browser system fonts are not available to GPUI's text shaper.

`start()` asynchronously initializes graphics and opens a window. `submit()`
accepts one bounded, framed protocol message and applies it to the existing
`SolidRoot` on foreground. `drain_events()` returns framed native events outside
Rust update callbacks. `WebTransport` owns event polling and host disposal.
Only bytes cross this boundary; Solid state remains in JavaScript.

The platform uses its WebGPU/WebGL backend selection. Local patches in
`vendor/gpui-web` and `vendor/gpui-wgpu` enable transparent browser surfaces
so the native Hero text can appear above GPU artwork. Actual availability depends
on the browser and GPU. The bootstrap reports startup failures. This Web host is
a component-capable target with explicit platform limits:

- View, Text, Pressable, TextInput, Image, Icon, and core VirtualList use the
  existing native renderer. Iconify and component SVG icons are embedded locally.
- The host registers the shared gpui-component Native Module and initializes its
  theme, focus infrastructure, and dialog/sheet/notification layers. The
  `component-runtime` Cargo feature excludes the desktop launcher and transport.
  Application Native Modules still require explicit registration.
- System motion preferences use a browser media-query subscription with cleanup.
  Native and Web hosts share the override policy.
- The website compiles approved examples from `component-examples.ts` into actual
  Solid previews; `component-previews.ts` defines the enabled set. The displayed
  code and executable preview share one source. Editor syntax highlighting, OS
  integration, and complex unverified examples are not advertised as Web previews.
- Blocking native commands are rejected. Async native futures are polled by
  GPUI and cancelled with their owning request.
- Desktop filesystem, process, notification, menu, and multi-window services
  are not part of the browser host contract.
- GPUI Web's canvas accessibility and mobile IME support are upstream platform
  limitations. The site provides metadata, but canvas documentation is not
  equivalent to server-rendered searchable HTML.

## Website

Before the GPUI application mounts, a lightweight HTML startup screen displays the
Solid GPUI mark and an indeterminate loading indicator. Its stylesheet loads before
the application modules, and the indicator stays still when reduced motion is
preferred. The bootstrap checks for WebAssembly and graphics APIs before loading
the application. Missing required APIs or failure of both host graphics backends
shows a responsive “Browser not supported” card with browser update and graphics
acceleration guidance. It offers retry, an external Markdown documentation link,
and expandable technical details, all without the GPUI runtime. A missing WebGPU
API alone does not block WebGL2-capable browsers. Other startup failures show a
separate loading-error message in the same accessible shell;
JavaScript-disabled browsers receive an explicit enable-JavaScript message.

The favicon, startup screen, and shared navigation use the
[approved Solid GPUI icon](../assets/branding/README.md). Vite publishes startup
assets under the Pages base path. The navigation image is embedded as a data URL
for both browser and native builds; it does not fetch an image during navigation.

The site includes an English/Chinese language switch with a persisted preference.
The landing page, interactive guides, and reference documents follow the selected
language. English sources and explicitly named `.zh-CN.md` translations are maintained separately.

Code snippets use lexical highlighting rendered as native GPUI rich text, with
selection and a Copy action. This is presentation highlighting, not a language server.

The Hero embeds the official [vgpu Optimized Black Hole preview](https://vgpu.sh/preview/optimized-black-hole)
as a background behind the GPUI-rendered Hero title and actions. A gradient
keeps the foreground legible. It requires network access to vgpu and a compatible
WebGPU browser. Native layout bounds position and clip the background; it cannot intercept
pointer or keyboard input. Page navigation, scrolling out of view, and a hidden document release the iframe.
The Hero plays automatically and has no pause control.
The foreground interface and interactive examples remain GPUI WASM. Embedded
Iconify assets supply navigation and action icons.

## GitHub Pages

The deployment URL is [cyenoch.github.io/solid-gpui](https://cyenoch.github.io/solid-gpui/).
The repository's About website field and root README link to this address.
The repository's publishing source is **GitHub Actions**, and its `github-pages`
environment permits deployments from `main`. The first deployment still requires
the website source and workflow to be committed and pushed.

The [Pages workflow](../.github/workflows/pages.yml) handles the full deployment:

- Pushes to `main` changing website, documentation, branding, SDK, Rust, or build
  inputs build, test, and publish the website. Pull requests with those changes
  build and test without publishing. Manual runs publish only from `main`.
- Ubuntu installs the Linux native libraries needed to export SDK bindings,
  the repository's pinned Rust and Bun toolchains, and the matching Web nightly
  and a checksum-verified, prebuilt wasm-bindgen CLI. Cargo dependency caches
  include both pinned compilers and are reused across source commits.
  `bun run website:build` regenerates native bindings,
  builds the WASM host, type-checks the site, and bundles its assets.
- Assets use `/<repository-name>/`, which is `/solid-gpui/` for this site.
  Only `examples/website/dist` is uploaded. The deploy job receives Pages write
  and OIDC permissions; no personal access token or backend is required.
- New pull request runs cancel outdated checks. Production runs wait for an
  active deployment to finish. A final HTTP check verifies the published page.

See [continuous integration](ci.md) for path filters, caching, and the separate
manual native packaging workflows.

For the first publication, push the workflow together with `examples/website`,
`assets/branding`, the Web host, workspace dependencies, lockfiles, vendored patches, and referenced
documentation. The workflow cannot build from its YAML file alone. Do not commit
generated `dist`, `target`, or `src/wasm` directories.

After the workflow reaches `main`, a manual redeployment is available in
**Actions → GitHub Pages → Run workflow**, or with:

```sh
gh workflow run pages.yml --repo Cyenoch/solid-gpui --ref main
```

A local build does not publish the site. Check the GitHub Pages workflow and
its `github-pages` deployment for the published URL and deployment result.

## Key checks

```sh
bun --conditions=browser test examples/website/tests
bun run --cwd examples/website typecheck
bun run website:build
```

In an actual browser, verify the landing counter, documentation search and input,
code selection/copy, language persistence, Hero background rendering and route cleanup,
and narrow/wide layout. Verify the production preview as well as Vite development.
