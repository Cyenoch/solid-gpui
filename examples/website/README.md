# Solid GPUI website

[Live website](https://cyenoch.github.io/solid-gpui/) ·
[Documentation index](../../docs/README.md) ·
[Web host and Pages deployment](../../docs/web.md)

The website shares its guides, component catalog, Showcase apps, and file-based
routes across the Rust WebAssembly host and desktop host. Browser navigation uses
hash URLs; desktop navigation uses memory history with hot-reload restoration.

## Run and verify

```sh
bun install --frozen-lockfile
bun run website
```

Browser development and full builds prepare the WASM host and generated SDK
bindings before starting Vite. After Rust contract changes, restart development.
Direct Vite commands bypass that preparation.

```sh
bun run --cwd examples/website build:host
bun run --cwd examples/website build:frontend
bun --conditions=browser test examples/website/tests
```

`bun run website:build` composes the two build steps. Standalone `typecheck` and
`build:frontend` require the generated host files. Pages caches WASM and SDK
bindings independently using exact producer inputs; frontend checks always run.
See [Pages caching](../../docs/web.md#pages-caching) before changing build inputs.

Tests verify catalog coverage, executable TSX examples, highlighting, Markdown,
startup classification, and retained navigation without DOM globals. Follow the
[preview verification guide](tests/README.md) for visual and interaction checks.

## Content ownership

| Content                  | Source                                                                                    |
| ------------------------ | ----------------------------------------------------------------------------------------- |
| Reference guides         | `../../docs/*.md` and explicit `.zh-CN.md` translations, loaded by `src/documentation.ts` |
| Component API            | Generated `../../packages/solid-gpui/src/components.ts`, read by `component-catalog.ts`   |
| Component examples       | `component-examples.ts`, `component-variants.ts`, and `component-recipes/`                |
| Component translations   | `component-examples.zh-CN.ts`, recipe translations, and `src/locale.zh-CN.ts`             |
| Preview availability     | `component-previews.ts`; only the active compiled example mounts                          |
| Navigation               | `component-groups.ts` for page groups; `component-families.ts` for compound parts         |
| Showcase                 | `src/showcase`; displayed source and running previews share modules                       |
| Routes                   | `src/routes`; regenerate and commit `src/routeTree.gen.ts` with `bun run routes:generate` |
| Static code highlighting | `src/snippets.ts` and `build-highlights.ts`                                               |
| Branding                 | [Approved assets](../../assets/branding/README.md)                                        |

The catalog requires an example for every generated component and exactly one
navigation group per page. Tests type-check examples against the SDK. Release
changes belong in [CHANGELOG.md](../../CHANGELOG.md).

The Low-level drawing group includes an interactive RecordedPaint workflow diagram
and an acknowledged LiveFrame CPU stream. Their executable source lives in
`component-recipes/paint-media.ts`; both use the generated native contracts. See
[paint and frame ownership](../../docs/paint-media.md) for budgets and platform limits.

English guides are authoritative. Website guides require a `.zh-CN.md` copy with
a localized level-one heading. `src/Docs.tsx` owns shortened navigation labels;
translate those in `src/locale.zh-CN.ts`. Both languages load the authoritative
Markdown directly, including their highlighted code. Keep runtime instructions
in those guides and link to them from examples.

The Vite reference includes the public `@solid-gpui/vite/compiler` API in both
languages. Virtual component previews compile through that same canonical source
module; Vite project transforms and preview snippets must never carry separate
compiler implementations. `build-highlights.ts` reads guide code directly, so
compiler examples need no copied website snippet. TextView's retained native
Markdown examples remain in the component catalog; the documentation site's
section/table presentation is composed separately for navigation.

Topic ownership is indexed in [docs/README.md](../../docs/README.md): host/window
configuration belongs in Rust integration, themes in GPUI components, layout in
native composition, scrolling in scroll performance, and icon registration in
Iconify. Investigation records belong in `.scratch/`.

## Build conventions

- Run Vite under Bun. Build-time Shiki uses a Bun Worker and disposes it after
  highlighting; the browser receives highlighted runs rather than a tokenizer.
- Use explicit `.ts` extensions in shared Vite configuration imports and their
  transitive helpers, so bundled and native config loaders agree.
- `solidGpuiSource({ root })` and TypeScript's `solid-gpui-source` condition derive
  SDK mappings from package exports. Keep application-specific aliases in the
  shared config. Browser typechecking uses the website's own `tsconfig.json` so
  it does not need a native host build.
- Regenerate API content from Rust contracts with `build:bindings`; do not edit
  generated SDK declarations by hand. The selected native host supplies its own
  `#native`, component, and Motion bindings.
- Keep the startup shell (`index.html`, `src/base.css`, `src/bootstrap.ts`,
  `src/startup.ts`) independent of GPUI so loading and unsupported-browser errors
  remain visible before WASM starts.

## Desktop development and packaging

The website loads the authoritative English and zh-CN getting-started, Vite and
distribution guides directly. Those guides also cover the versioned standalone
template, explicit stock-host acquisition and generic consumer packaging; the
gallery's embedded executable remains its separate application-specific package.
The manual `Standalone Delivery Candidates` workflow creates paired SDK/host
artifacts and clean-consumer evidence without publishing releases.

Download **Gallery Desktop** from [GitHub Releases](https://github.com/Cyenoch/solid-gpui/releases):
macOS Apple Silicon (`.zip`), Linux x86-64 (`.tar.gz`, Ubuntu 24.04 baseline), and
Windows x86-64 (`.zip`). No Bun or Node installation is required. Release tags
build and attach all three archives plus SHA-256 files after extracted-app checks.
macOS builds are ad hoc signed without notarization; Windows builds are unsigned.
See [distribution](../../docs/distribution.md#build-and-verify) for launch
instructions and desktop verification limits.

Native contract digests describe exported interfaces and behavioral versions;
separate build envelopes lock generated bindings to the selected implementation
and exact SDK provenance. LF/CRLF source differences are normalized. Regenerate
bindings after implementation changes even when the contract digest stays equal.
The extracted bundle check validates both identities. See [native identities](../../docs/rust-bridge.md#native-contract-and-build-identities).

```sh
bun run website:native:dev
bun run --cwd examples/website generate:native
bun run --cwd examples/website build:native
bun run --cwd examples/website preview:native
bun run website:native:package
```

`vite.native.config.ts` selects the Cargo host and bindings used by prepare,
development, build, and preview. Managed development rebuilds Rust changes and
replaces the host session; see [hot reload](../../docs/hot-reload.md). Preview
uses the built artifacts recorded in `.solid-gpui/artifacts.json`.

The Rust package is `website-host`; the packaged executable is
`solid-gpui-website`. Packaging compiles the QuickJS entry with `build:embedded`
to `dist-embedded/app.js`, embeds it with `SOLID_GPUI_WEBSITE_BUNDLE` and the
`distribution` Cargo feature, then writes archives to `dist/website/`. This is
the website's QuickJS release path; Embedded Bun has its own
[packaging guide](../../docs/distribution.md#embedded-bun-static-applications).

Use `bun run task website-native-profile` with the
[performance measurement workflow](../../docs/performance-analysis.md).
For a serial production A/B/B/A region workload after compilation, use
`bun scripts/native-region-compare.ts BASELINE_BINARY CANDIDATE_BINARY NEW_DIRECTORY`
from the repository root. The guide documents visible-content checks and the
static definite-size region boundary; protocol checks alone do not prove a
painted update or native scroll displacement.
Workflow triggers and release qualification live in [CI](../../docs/ci.md).
