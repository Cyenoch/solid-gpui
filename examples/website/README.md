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

`component-introduced.ts` records when each component's documentation was created
and when it last changed; the catalog refuses to build without a record for every
export. Seed both dates from the commit that wrote the documentation, add the
current date when you document a new component, and bump `updated` when you change
an existing entry — including a change made through an example, a recipe or a
translation. A component documented on or after `newBadgeEpoch` keeps a localized
**New** badge in the component navigation and on its page for `newBadgeWindowDays`;
documentation written earlier is never marked, so the rule never relabels an
established catalog at once. Component pages show the page's earliest creation date
and latest update date, each API Reference entry shows its own pair, and both appear
in the Markdown copied from the page.

`component-groups.ts` assigns every catalog page to one navigation group, in the
order of the native family table in `docs/gpui-components.md`; the catalog refuses to
build for a page without a group, for a group member without a page, or for a page
listed twice. The sidebar prints those group captions in the same order, hides a
group while the component search leaves it empty, and keeps each group's pages
alphabetical, so the sidebar and each page's previous/next links follow one order.
Group captions sit at the sidebar edge in the foreground color; component rows keep
muted labels, and their highlight hugs the label with even insets on all sides while
the press target spans the full row width and height.
Compound parts stay on their owner's page through `component-families.ts`.

The ScrollShadow catalog includes a direct-child core `VirtualList` example with
10,000 stable data items. Its parent has bounded height and the child fills both
dimensions, so the preview demonstrates native scroll ownership and core
windowing without eagerly creating 10,000 JSX rows; the displayed source is the
same source used by the preview.

Code highlighting uses `@solid-gpui/shiki`: the Vite catalog loader starts its Bun
Worker, generates Shiki/Oniguruma results for examples, excerpts, type declarations
and Markdown fences, then disposes the service. The browser renders those results
with the package's `HighlightedCode` component and the existing GPUI text host.
No tokenizer or Oniguruma WASM is sent to the browser. Vite must run under Bun
(the `dev` and `build` scripts enforce this). New static snippets belong in
`src/snippets.ts` and `build-highlights.ts`; missing results are explicit errors.

The shared Vite config, its native/embedded entry points, and their transitive
configuration dependencies use explicit `.ts` extensions for relative imports
and re-exports, including type-only imports, so both bundled and native Vite
config loaders can resolve them. Keep this convention when adding build helpers.

Markdown fences retain their declared language, including plain text.
Expanded code blocks keep the complete source in one selectable paragraph, with
line numbers in a separate non-selectable gutter. Drag selection crosses tokens,
blank lines, and line breaks without including line numbers. Long lines scroll
horizontally; collapsed excerpts remain non-selectable with per-line fading.

Run the website checks with:

```sh
bun run --cwd examples/website build:host
bun run --cwd examples/website build:frontend
bun --conditions=browser test examples/website/tests
```

`bun run website:build` runs the same host and frontend steps locally. The host
step generates the WASM module and the SDK bindings before type checking.
Standalone `bun run --cwd examples/website typecheck` requires those generated
files. Pages owns these browser checks; the SDK package CI gate runs independently
of website build products.

Pages caches both generated host outputs, `src/wasm` and
`packages/solid-gpui/src/components.ts`, under an exact-input key covering the
Rust sources, pinned toolchains, and generated SDK contract. Restoring both keeps
warm and cold frontend inputs identical. A cache
hit with unchanged Rust inputs skips the native toolchain setup and the host build,
but the type check, the bundle, and the tests above always run. Regenerate and
commit `packages/solid-gpui/src/components.ts` with any host change that alters it,
and bump the `pages-web-host-v2` namespace in
[the workflow](../../.github/workflows/pages.yml) when the host build gains an
input. See [the deployment guide](../../docs/web.md#pages-caching).

The reference documentation catalog loads `docs/*.md` and their `.zh-CN.md`
translations directly through `src/documentation.ts`. Update those sources for
both website languages. The Troubleshooting guide includes native QuickJS blank
startup diagnosis, stale Cargo profiles, and protocol-tap interpretation; these
native runtime checks are separate from the browser's WASM startup checks.
The Guides sidebar publishes [Preserve UI state](../../docs/capture-state.md) at
`/docs/reference/capture-state`, with a complete `captureState`/`setup(previous)`
example, QuickJS data limits, dirty-draft restoration, and restart boundaries.

Start the website with the matching SDK bindings and WASM host:

```sh
bun install --frozen-lockfile
bun run website
```

Browser `dev` and `build` both prepare the Rust WASM host and generated SDK
bindings before invoking Vite. `build` composes `build:host`, which regenerates
the SDK bindings and the WASM host, and `build:frontend`, which runs the route
generator, the type check, and the Vite bundle; `dev` prepares the host and then
starts Vite. Neither script reuses a previously generated host, so a local build
is always a full rebuild. Restart development after changing Rust contracts.
Direct Vite invocations bypass this synchronization.

Native component dependencies come from `vendor/gpui-kit`; the pinned
`references/gpui-kit` submodule is used for upstream comparison. The
[component guide](../../docs/gpui-components.md) describes which GPUI Kit layers
the host exposes, and is published in both website languages. The 0.6.4-based
pin includes InputGroup, Questionnaire, inline input tokens, editor search,
Motion sequences, streamed-text fades, and interactive pie charts. Their examples
and API tables use generated contracts. The host embeds only Kit's default control
icons; application icons use the separate offline Iconify catalog. Shell is not
part of the native or Web host. FPS is the upstream Kit HUD in the desktop host,
with definitions in the performance guide.

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
bun run --cwd examples/website generate:native   # solid-gpui prepare --config vite.native.config.ts
bun run --cwd examples/website build:native
bun run --cwd examples/website preview:native    # solid-gpui preview --config vite.native.config.ts
bun run --cwd examples/website build:embedded
bun run website:native:package
```

All three targets use `@solid-gpui/vite`: native development owns the host and
Bun ModuleRunner, embedded builds select QuickJS, and browser builds select the
universal web transform. Native bindings come from the configured Cargo host in
`vite.native.config.ts`, which `generate:native`, `dev:native`, `build:native`,
`preview:native`, and the root tasks all share. The shared config adds
`solidGpuiSource({ root })` in place of the removed hand-written SDK aliases; it
still carries the aliases the site needs itself, such as the native `HeroVisual`
swap and the browser `assert` shim.

The website keeps its own `tsconfig.json` with
`"customConditions": ["solid-gpui-source"]` rather than extending the generated
`.solid-gpui/tsconfig.json`: the browser typecheck and build must not require a
native host build, and the source condition selects the SDK sources directly.
Native development automatically rebuilds Rust source and Cargo configuration
changes, including local dependencies, and replaces the host session with matching
bindings. `#native`, component imports, and Motion use that host's generated catalog.
Build and application failures leave Vite watching for the next source edit;
Ctrl+C stops development. Host replacement reopens windows and resets state.
The [development guide](../../docs/hot-reload.md#managed-development-sessions)
describes watched inputs and the distinct browser and Rust-owned launch behavior.
`preview:native` runs the built host against the built bundle recorded in
`.solid-gpui/artifacts.json` without rebuilding.

`build:embedded` writes the QuickJS bundle to `dist-embedded/app.js`, and
`bun run website:native:package` embeds that file in the Rust executable with
`include_bytes!` through the `distribution` Cargo feature
(`SOLID_GPUI_WEBSITE_BUNDLE`, one bundling pass, no second compiler). That is the
website's release pipeline and it is separate from the experimental Embedded Bun
static packager described in
[distribution](../../docs/distribution.md#embedded-bun-static-applications), which
links a Bun/JSC runtime and an application module graph into one executable; the
website package does not use it.

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

- [Installation](../../docs/getting-started.md) at `/docs/reference/getting-started` is the authoritative external-consumer sequence: install, prepare, typecheck, develop, test, build, and production preview, with the Cargo requirements and the generated project.
- [Distribution](../../docs/distribution.md) at `/docs/reference/distribution` separates the bundle, the native executable, and the distributable, and records verified versus experimental target capability.
- [Iconify](../../docs/iconify.md) at `/docs/reference/iconify` covers the built-in offline catalog, Solid usage, styling, and application icon registration.
- [Vite integration](../../docs/vite.md) at `/docs/reference/vite` covers direct JS, JSX/TSX builds, Bun APIs, native modules, the published test runner, artifact lookup, and Rust-owned development.
- [Choose a runtime](../../docs/runtimes.md) at `/docs/reference/runtimes` introduces the runtime and transport choices, including the experimental Embedded Bun packager.
- [Development workflow](../../docs/hot-reload.md) at `/docs/reference/hot-reload` covers the consuming workspace's QuickJS build profile, captured-state contract, native rebuild watching, generation lifecycle, and application reload verification.
- [System popovers](../../docs/system-popover.md) at `/docs/reference/system-popover` documents the core API, editable and nested native Surfaces, multi-display placement, and platform acceptance limits. This capability needs a desktop host; its example is published as source without a browser preview.
- [Troubleshooting](../../docs/troubleshooting.md) at `/docs/reference/troubleshooting` explains stale bindings, install-hook mistakes, cross-target refusals, wrong Solid resolution, nested-route stack errors, rejected state, and failures after activation.

Keep runtime guidance in those sources. `src/documentation.ts` loads the pages
and `build-highlights.ts` generates code highlighting for both languages during
the website build. The existing navigation lists these pages; changes to their
content require no separate article or route-tree entry. Public guides describe
application-independent contracts and workflows; incident-specific evidence
belongs in investigation records.

## Reference guide ownership

Desktop host and titlebar configuration belongs in [Rust integration](../../docs/rust-bridge.md),
application themes in the [component guide](../../docs/gpui-components.md), layout
and painting in [native UI composition](../../docs/native-composition.md), scrolling
in the [scroll guide](../../docs/scroll-performance.md), and icon registration in
[Iconify](../../docs/iconify.md). Do not add another umbrella article that repeats
those contracts. The [desktop application example](../desktop-app/README.md)
contains running instructions and links back to these sources.

Reference pages and their highlighted code are loaded from `docs/*.md`.
English documents are authoritative; `.zh-CN.md`
copies are required for guides published on the website; a missing translation
fails website startup or build. Each copy must include a localized level-one
heading. Navigation uses that heading in the selected language unless `Docs.tsx`
defines a shorter label, which must also have a `locale.zh-CN.ts` translation.
Guide search includes both language versions. Keep headings, navigation labels,
and content synchronized when adding or updating a guide.
The TitleBar/WindowBorder catalog example demonstrates
edge padding and a bottom border; the desktop application example also exercises
application themes, custom window options, percentage sizes, gradients, and
embedded icons. Its native window and process services run on desktop.

Do not hand-maintain the SDK subpath mappings: `solidGpuiSource()` in the shared
Vite config and `customConditions: ["solid-gpui-source"]` in `tsconfig.json` both
derive from the same package exports, so JSX, the runtime
entry, native subpaths, and application icon types resolve to one module. Build the
WASM host before checking or bundling the browser entry.

## Reactive update and performance guidance

The Signals & updates chapter describes local binding updates and atomic native
commits without promising local-only layout or painting. The Performance analysis
and Protocol pages import their authoritative Markdown from `docs/`; English and
explicit `.zh-CN.md` copies are synchronized together.

For a bounded native commit workload with the monitor disabled, follow
[the commit profiling commands](../../docs/performance-analysis.md#native-commit-attribution).
The website's profile command additionally exercises the full component host.
Both use actual host instrumentation; test-platform CPU measurements are separate
from native presentation and physical input acceptance.
