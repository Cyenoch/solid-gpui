# Solid GPUI website

Pages URL: [cyenoch.github.io/solid-gpui](https://cyenoch.github.io/solid-gpui/).
The [Pages workflow](../../.github/workflows/pages.yml) builds, tests, and deploys
the site automatically when pushes to `main` change website, documentation,
branding, SDK, Rust, or build inputs. Matching pull requests
only build and test. See the
[deployment guide](../../docs/web.md#github-pages) for the build environment,
required source files, and manual redeployment.

A GitHub Pages application rendered by Solid GPUI's Rust WebAssembly host.
Includes a bilingual landing page, interactive guides, a complete component usage catalog,
Markdown tables, and GPUI-rendered code highlighting. Page actions copy Markdown,
open its source, and navigate between entries.

Markdown tables keep aligned headers and rows at every viewport width. Columns
share the available width equally, with a 220-pixel minimum per column to keep
prose readable. Wider tables scroll horizontally inside the article instead of
compressing the final column or switching to stacked records. This shared renderer
also displays component API tables on Web and desktop.
Tables use the shared `ScrollShadow` component in horizontal mode. Overflowing tables show an always-visible, draggable horizontal scrollbar with
space below the final row. Native gradient overlays mark only edges with more
content, shrinking and becoming transparent near the boundary; they disappear at the corresponding scroll boundary or when the table
fits. The overlays do not intercept input and paint below the scrollbar.

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
bun run website:build
bun --conditions=browser test examples/website/tests
```

The build generates the WASM module before type checking. Standalone
`bun run --cwd examples/website typecheck` requires those generated files.
Pages owns these browser checks; the SDK package CI gate runs independently of
website build products.

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

All three targets use `@solid-gpui/vite`: native development owns the host and
Bun ModuleRunner, embedded builds select QuickJS, and browser builds select the
universal web transform. Native bindings come from the configured Cargo host.
`build:embedded` writes the final self-contained `dist-embedded/app.js`; packaging
copies that artifact without another bundling pass.

The native Rust package is `website-host`; the packaged executable is
`solid-gpui-website`, and verified archives are written to `dist/website/`.
Use `bun run task website-native-profile` for native performance measurements.
The Website Packages workflow builds the three platform candidates manually for
release qualification. See [continuous integration](../../docs/ci.md) for the
automatic checks and cache policy.

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

## Runtime and development guides

The Guides sidebar publishes these references from the authoritative Markdown
sources and their `.zh-CN.md` copies:

- [Iconify](../../docs/iconify.md) at `/docs/reference/iconify` covers the built-in offline catalog, Solid usage, styling, and application icon registration.
- [Vite integration](../../docs/vite.md) at `/docs/reference/vite` covers direct JS, JSX/TSX builds, Bun APIs, native modules, and Rust-owned development.
- [Choose a runtime](../../docs/runtimes.md) at `/docs/reference/runtimes` introduces the runtime and transport choices.
- [Development workflow](../../docs/hot-reload.md) at `/docs/reference/hot-reload` covers the consuming workspace's QuickJS build profile, captured-state contract, generation lifecycle, and application reload verification.
- [System popovers](../../docs/system-popover.md) at `/docs/reference/system-popover` documents the core API, editable and nested native Surfaces, multi-display placement, and platform acceptance limits. This capability needs a desktop host; its example is published as source without a browser preview.
- [Troubleshooting](../../docs/troubleshooting.md) at `/docs/reference/troubleshooting` explains nested-route stack errors, rejected state, and failures after activation.

Keep runtime guidance in those sources. `src/documentation.ts` loads the pages
and `build-highlights.ts` generates code highlighting for both languages during
the website build. The existing navigation lists these pages; changes to their
content require no separate article or route-tree entry. Public guides describe
application-independent contracts and workflows; incident-specific evidence
belongs in investigation records.

## Native application migration

The Guides sidebar links to [native application migration](../../docs/native-migration.md)
at `/docs/reference/native-migration`. Reference pages and their highlighted code
are loaded from `docs/*.md`. English documents are authoritative; `.zh-CN.md`
copies are required for guides published on the website; a missing translation
fails website startup or build. Each copy must include a localized level-one
heading. Navigation uses that heading in the selected language unless `Docs.tsx`
defines a shorter label, which must also have a `locale.zh-CN.ts` translation.
Guide search includes both language versions. Keep headings, navigation labels,
and content synchronized when adding or updating a guide.
The TitleBar/WindowBorder catalog example demonstrates
edge padding and a bottom border; the native migration fixture also exercises
application themes, custom window options, percentage sizes, gradients, and
embedded icons. Its native window and process services run on desktop.

Keep TypeScript's SDK paths aligned with Vite's source aliases, including JSX and
native subpaths, so application icon types come from one module. Build the WASM
host before checking or bundling the browser entry.
