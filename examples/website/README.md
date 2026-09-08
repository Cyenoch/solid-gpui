# Solid GPUI website

Pages URL: [cyenoch.github.io/solid-gpui](https://cyenoch.github.io/solid-gpui/).
The site is awaiting its first deployment; the commands below run it locally.
Once the website source and [Pages workflow](../../.github/workflows/pages.yml)
reach `main`, each push builds, tests, and deploys the site automatically.
Pull requests only build and test. See the
[deployment guide](../../docs/web.md#github-pages) for the build environment,
required source files, and manual redeployment.

A GitHub Pages application rendered by Solid GPUI's Rust WebAssembly host.
Includes a bilingual landing page, interactive guides, a complete component usage catalog,
Markdown tables, and GPUI-rendered code highlighting. Page actions copy Markdown,
open its source, and navigate between entries.

The browser startup screen is defined in `index.html` and `src/base.css`. Its
stylesheet loads directly from the document head, so the branded loading state
appears before the application modules and WASM finish loading. The indeterminate
indicator respects reduced motion; startup errors replace the loading content
with an accessible alert. `src/bootstrap.ts` checks required browser APIs before
importing the application. If WebAssembly is missing, both graphics APIs are absent,
or the host cannot initialize either WebGPU or WebGL2, the shell shows a branded
“Browser not supported” card with recovery guidance, retry, a direct documentation
link, and expandable technical details. Other loading errors use a distinct message.
Keep this shell independent of the GPUI runtime.

The README, browser favicon, startup screen, shared Web/native header, and desktop
packages use the [approved project icon](../../assets/branding/README.md).
The header imports the 64-pixel PNG as an inline data URL so native and embedded
builds need no external image file or network request. Startup and favicon images
are processed by Vite with the GitHub Pages base path.

The component catalog is derived at build time from the generated SDK in
`packages/solid-gpui/src/components.ts`. Examples live in `component-examples.ts`;
translations live in `component-examples.zh-CN.ts`. The catalog check requires an
example for every export and type-checks the examples against the SDK. Component
previews listed in `component-previews.ts` run through the same Rust component
implementations as the desktop host. Vite compiles their documented TSX at build
time; the site mounts only the active example. Unverified examples keep their
usage documentation and an explicit availability note.

Code highlighting uses `@solid-gpui/shiki`: the Vite catalog loader starts its Bun
Worker, generates Shiki/Oniguruma results for examples, excerpts, type declarations
and Markdown fences, then disposes the service. The browser renders those results
with the package's `HighlightedCode` component and the existing GPUI text host.
No tokenizer or Oniguruma WASM is sent to the browser. Vite must run under Bun
(the `dev` and `build` scripts enforce this). New static snippets belong in
`src/snippets.ts` and `build-highlights.ts`; missing results are explicit errors.
Markdown fences retain their declared language, including plain text.

Run the website checks with:

```sh
bun run --cwd examples/website typecheck
bun --conditions=browser test examples/website/tests
```

Start the website with the matching SDK bindings and WASM host:

```sh
bun install --frozen-lockfile
bun run website
```

Browser `dev` and `build` both prepare the Rust WASM host and generated SDK
bindings before invoking Vite. Restart development after changing Rust contracts.
Direct Vite invocations bypass this synchronization.

See [Web host setup and capabilities](../../docs/web.md) for the toolchain,
browser requirements, architecture, tests, and deployment instructions.

## Native website and Showcase

The website is also the desktop application. Showcase uses the same navigation
layout as Components, with Workspace, Account, and Collections examples. The
preview and displayed source come from the same modules in `src/showcase`.

```sh
bun run website:native
bun run website:native:dev
bun run --cwd examples/website dev:native
bun run --cwd examples/website build:native
bun run --cwd examples/website build:embedded
bun run website:native:package
```

The native Rust package is `website-host`; the packaged executable is
`solid-gpui-website`, and verified archives are written to `dist/website/`.
Use `bun run task website-native-profile` for native performance measurements.

The shared `@solid-gpui/router` file-based route tree owns navigation on both platforms.
Route modules live in `src/routes`; `solidGpuiRouter()` in the shared Vite config
generates `src/routeTree.gen.ts` for browser, native, and embedded builds. Commit
the generated tree and exclude it from formatting. `bun run routes:generate`
regenerates it before standalone type checking. The Guides sidebar includes
[Router](../../docs/router.md) at `/docs/reference/router`, covering native setup
and linking to TanStack Router for general usage.
The root header remains mounted; each section keeps its sidebar across entry
changes. Web history synchronizes with hash URLs for GitHub Pages. Desktop uses
memory history and restores its current URL during hot reload. The browser-only
vgpu artwork layer is not included in the desktop build.

The Rust host and
application-specific native module live in `native`; generated bindings live in
`src/generated`. Runtime tests cover documented previews and retained navigation;
`tests/runtime.fixture.tsx` runs the real website through Vite without DOM globals.

## Native application migration

The Guides sidebar links to [native application migration](../../docs/native-migration.md)
at `/docs/reference/native-migration`. Reference pages and their highlighted code
are loaded from `docs/*.md`. English documents are authoritative; `.zh-CN.md`
copies are optional. The Chinese locale displays the English source when a
translation is unavailable, so adding a guide never blocks website startup.
The TitleBar/WindowBorder catalog example demonstrates
edge padding and a bottom border; the native migration fixture also exercises
application themes, custom window options, percentage sizes, gradients, and
embedded icons. Its native window and process services run on desktop.

Keep TypeScript's SDK paths aligned with Vite's source aliases, including JSX and
native subpaths, so application icon types come from one module. Build the WASM
host before checking or bundling the browser entry.
