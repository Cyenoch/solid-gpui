# Changelog

## Unreleased

- Add strict X/Y spacing, explicit px/rem/percent/auto dimensions, flex basis,
  aspect ratio and independent overflow axes. Core View and Pressable support
  native hover, active and keyboard focus-visible paint refinements; hover
  callbacks subscribe explicitly. Protocol v7 requires rebuilding both peers.
  Replace widthPercent/heightPercent with explicit percent lengths.

## Unreleased

## [0.5.2]

### Fixed

- Fix native router blockers silently bypassed without a DOM (#8). SDK-owned memory history checks push, replace, back, forward, and go before mutation, and the RouterCore adapter waits for confirmation before loading routes. New requests supersede pending decisions; unregister and destroy cancel them; `ignoreBlocker` explicitly bypasses checks. Native history methods return promises and propagate confirmation failures. Verify mounted owner preservation, installed Bun packages, and the real QuickJS VM.

### Upgrade notes

- Keep all four npm packages and matching native sources on 0.5.2. Native navigation blockers return `true` to cancel and `false` to proceed. Await direct history methods to handle asynchronous confirmation errors; window-close confirmation remains a separate host lifecycle concern.

## [0.5.1]

### Fixed

- Export the JSX compiler's `applyRef` helper from the public runtime so component callback, forwarded, and assignment refs load and build correctly. Support ordered nested callback arrays and expose `Ref<T>` for wrapper props.
- Invoke host ref callbacks without reactive dependency tracking while preserving their Solid owner and `onCleanup` lifecycle. Verify real TSX from installed package archives through the public test runner and production Bun/QuickJS builds, including native VirtualList handle command replies.

### Upgrade notes

- Keep all four npm packages and matching native sources on 0.5.1. Ref callback return values are ignored; use `onCleanup` for cleanup. VirtualList and TextInput refs are invoked at mount and are not cleared at unmount.

## [0.5.0]

### Added

- Extend the public TestHost with committed semantic styles, accessibility and virtual-list descriptors, visible-range/layout/pointer events, and explicit typed scrolling command replies. Events and replies share revision-aware sequencing; unanswered requests remain pending.

### Fixed

- Verify Cargo Git GPUI patches against the resolved host repository and full revision, including workspace inheritance, without requiring a sibling SDK checkout. Detect missing, unused, registry and mismatched patch sources; retain path checks.
- Cancel blocked process commit reads independently of kill success, attempt cleanup after request errors, and preserve shutdown errors. Process status now distinguishes `ShutdownRequested`, failed cleanup, and confirmed `Shutdown`; callers can join their reader and retry cleanup with retained ownership.

### Upgrade notes

- Keep npm packages and native sources on 0.5.0 and rebuild the host. Exhaustive Rust `RuntimeStatus` matches must handle `ShutdownRequested`.

## [0.4.1]

### Added

- Release tags now build Gallery Desktop (the website's standalone QuickJS application) for macOS ARM64, Linux x86-64, and Windows x86-64, then attach verified archives and SHA-256 files to GitHub Releases. Manual runs can backfill an existing tag or produce candidates only.

### Fixed

- Use AppKit's Sidebar material for macOS blurred window backgrounds and remove the custom visual-effect layer/filter rewriting. Native material tint and accessibility behavior remain system-owned; application content must allow alpha through to reveal the effect.
- Restore the vendored GPUI line-clamp regression's font-generation setup so the GPUI test suite compiles against 0.3.7; reverified the existing window-movement invalidation fix.
- Preserve LF Rust source checkouts so Windows desktop packages retain the native contract identities in committed JavaScript bindings. Release backfills disable automatic CRLF conversion before checking out older tags.

## [0.4.0]

### Upgrade notes

- Keep all four `@solid-gpui` packages and the native sources on 0.4.0. Rebuild
  the native host and regenerate bindings when upgrading.
- `@solid-gpui/vite/embedded` now exports `packageEmbeddedApplication`,
  `EmbeddedPackagingError`, and input/report types. Build scripts that imported
  graph, manifest, target-resolution, or CLI helpers must use the packaging API.

### Changed

- Removed the unused protocol codec wrapper, narrowed `@solid-gpui/vite/embedded` to application packaging and its input/report types, and consolidated native export failure tests without repeated Cargo builds. Removed per-component documentation dates and automatic New badges; release changes live here. Condensed website/runtime guidance and corrected the protocol glossary and historical evidence links.
- Adopted npm installation as the application-consumer path, added public scoped-package metadata, and aligned Shiki with the 0.3.0 SDK. Removed custom SDK/single-package packing tasks in favor of standard package-manager commands, retained packed-consumer verification, and corrected doctor binding-recovery commands. Added tag-triggered GitHub Actions publication through npm OIDC, separate verification/publish jobs, committed-version checks, and exact-archive partial-release recovery. Synchronized npm onboarding and release setup guidance in English and Chinese.
- Released host child links on root teardown. Managed images now select failure fallbacks from the current prepaint request instead of carrying an earlier source's failure into a pending replacement or resized source-set candidate.
- Released unused NativePlot path slots on primitive replacement.
- Fixed strict image-test lint failures after integration: removed an unused import and initialized GIF regression frames directly, preserving their disposal, ordering, timing and pixel assertions.
- Separated exact-input Pages WASM and SDK-binding caches, with independent producers and unchanged full local builds. Removed nested Cargo builds from Rust-hosted QuickJS fixtures while preserving real Vite/VM checks. Split native CI into independent check/test lanes with isolated dependency caches and a fail-closed aggregate; checkout credentials are not retained in these build jobs.
- Updated `@solidjs/compiler` from `2.0.0-rc.6` to `2.0.0-rc.9`, retaining the latest stable `solid-js` runtime at `1.9.15`. Added a local Windows ARM64 native compiler build/package path using the exact upstream source and `SOLID_COMPILER_NATIVE`, without patching or publishing upstream. Synchronized English/Chinese installation, development and troubleshooting guides and regenerated dependency notices.
- Updated vendored GPUI Kit and its reference pin from `501c7392` to `0e63ea79` (0.6.4 plus subsequent changes), preserving local editing, overlay, virtual-list and geometry-cache seams. Added native InputGroup/Questionnaire bindings, atomic inline tokens, editor search and paste events, Motion sequences, TextView stream fades and interactive pie charts. Controlled metadata now declares alternative `valueProps` so token content and plain values share acknowledgements. Regenerated SDK, website and desktop host catalogs; synchronized English/Chinese guides, executable website examples, navigation, documentation dates and dependency notices.

- Cut over to lockstep protocol v6 with explicit layout subscriptions and identity-aware VirtualList revision spans. Batch sibling indexing and rich-text derivation per atomic commit, emit minimal same-parent moves, retain hinted list state across data edits, suspend delayed-animation frame demand, and retain correctly invalidated chart/plot tessellation. JS and native artifacts must be rebuilt together; no v5 decoder is retained.
- Added layout- and DPI-aware managed native images with bounded target buckets, native JPEG reduction, target-sized SVG rasterization, `sourceSet` candidate selection, weak sharing of identical sources/variants, streamed GIF/WebP frames, bounded fetch/decode admission, and release of inactive decoded pixels/atlas allocations. Preserved original `objectFit` geometry and the prior displayed frame until redraw, while pausing new animation work offscreen, under reduced motion, or in inactive windows.
- Compacted retired Surface IDs into exact coalesced ranges (with sparse growth proportional to gaps), cleared them when the JavaScript peer terminates, removed write-only focused-link bounds, and made full Snapshots refresh viewport capabilities.
- Preserved the real Bun environment in the public test runner without relaxing production QuickJS restrictions, forwarded Vite mode through preview and test CLI/API paths, and added `@solid-gpui/core/testing` for committed-tree inspection, event injection and native-call replies without exposing generated protocol bindings.
- Split native and SDK CI checks while preserving the required aggregate status, isolated complete Cargo build caches from Embedded Bun checks, and stopped caching failed build graphs. Pages restores exact-input WASM and generated SDK catalog artifacts together and keeps all frontend/headless tests without rebuilding native hosts; release preparation packs the SDK once.
- Corrected native DTO validation to distinguish documentation, literal text, and property names from unsupported `any`/`bigint` types, while checking nested template-literal type interpolations. Native module exports preserve DTO documentation and reject unsupported types through the same binding contract.
- Fixed strict workspace Clippy failures in component tests, removed obsolete JSX preload arguments from plain JavaScript scroll fixtures, and refreshed the generated dependency notices after tooling inputs changed. CI guidance now distinguishes package tests from the independent notice audit.
- Consolidated application tooling around one Vite configuration: `solid-gpui prepare`, read-only generated-file checks, `doctor`, Vite-backed Bun tests, and production `preview`. Prepared TypeScript paths and structured native/bundle artifacts replace duplicated consumer path tables; Cargo builds are locked by default and preserve profile, target, exporter, and runtime-host ownership.
- Published explicit SDK source conditions and `solidGpuiSource()`, plus matching tarball packing without native compilation during JS package builds. Migrated the desktop and website examples and synchronized the English/Chinese onboarding and advanced guides.
- Track Cargo-declared native inputs, including build-script resources and external directories, after the first build. Testing now shares the application's Vite transforms and Solid resolution, with clear errors for server-build and duplicate-Solid runtimes; the private JSX preload has been removed.
- Exposed application-owned Embedded Bun packaging through `@solid-gpui/vite/embedded`, preserving workspace dependencies, root patches/profiles, and serializer entry identities. Added single-assignment full-`u32` completion results independently of the process exit status; Embedded Bun target qualification remains experimental.
- Added `Row`/`Column`, DOM-free QuickJS ambient types, and `Dialog.showFooter` without changing custom-footer precedence. Application icon catalogs are now keyed records (`applicationIcons["prefix:name"]`), replacing positional access.
- Moved Windows application-thread stack ownership into host entrypoints. Host runners now take module/profile factories, so non-`Send` GPUI state is constructed on the correct thread without application wrappers; macOS remains on its main thread. Added `ComponentHost::with_initialize` for first-frame native configuration and resolved the Windows system-font alias from the OS UI font.
- Normalized optional `undefined` object fields recursively at native JSON boundaries while retaining strict array and non-JSON validation. Select and Combobox now retain controlled keys while asynchronous catalogs are absent, reconcile them without overwriting unacknowledged user edits, and report actual selection changes rather than redundant confirmations.
- Made explicit Vite host commands export their actual native catalog for components, Motion, and `#native`, failing rather than substituting SDK bindings. Added native-typed Solid control flow to `@solid-gpui/core/runtime` and type-only generated protocol imports for `verbatimModuleSyntax` source consumers.
- Made default stdio renderer lifetime follow the host pipe, including unexpected EOF and read errors, without killing detached application services. Managed reload consumes captured state once, and Surfaces opened after activation must prepare a valid first Snapshot.
- Expanded the desktop application example with bounded long-form scrolling, asynchronous controlled choices, optional native DTO fields, and actionable offline icon diagnostics. Renamed `examples/native-migration` to `examples/desktop-app` and consolidated its duplicated guide into Rust integration, component themes, native UI composition, scrolling, and Iconify documentation; dated verification records now live in `.scratch/desktop-app/`.
- Grouped the Components navigation by native family. `examples/website/component-groups.ts` assigns every catalog page to one group in the order of the family table in `docs/gpui-components.md`; the catalog refuses to build for a page without a group, a group member without a page, or a page listed twice, and the sidebar and each page's previous/next links follow that grouped order. Group captions use the foreground color in bold at the sidebar edge, component rows keep muted labels whose highlight hugs the label with even insets on all sides while the press target spans the full row width, the desktop sidebar drops its redundant `Components` caption, and search lists only the groups that still have matches.
- Moved the pinned Kit tree to upstream `501c7392` and the patched GPUI family to `gpui-pre 0.3.5` (Zed snapshot `d89e9c2`). Kit adopts the removal of the freeform tiles canvas: `DockArea` now has splits and tab groups only, so the dock SDK drops the `tiles` layout kind, its four ref commands, `tilesScrollbar`, and the `tiles` theme token, and gains the native `Panel::title_bar` hook. Kit's new `Empty` family is bound as six SDK parts. Scroll bounce and the inline text plugin remain unbound base surfaces. The same graph pulls `rustls 0.23.45`, which clears `RUSTSEC-2026-0285` in the committed lockfile.
- Aligned three vendored Kit surfaces with the content they draw: a dialog popup bounds its height to the room the window leaves above a 24 px bottom gap so an oversized body scrolls inside it, the notification card is one flex row with a uniform 16 px inset and no absolutely positioned child (its icon and action centre on the first line of wrapped copy), and a popup menu's leading icon draws at `Size::Medium` beside its `text_sm` label. The notification card draws no close control: a toast is dismissed by clicking it, by a middle-click, or by its own timer.
- Made ScrollShadow borrow a direct, matching-axis VirtualList viewport, sharing native scrolling, fades, commands, and one scrollbar without expanding windowed rows. Preserved unmeasured list height estimates through first layout and width changes.
- Added an optional embedded QuickJS UI runtime for Rust-led applications, with bounded transport queues, cancellable scheduling, and the existing native protocol/command contract. Bun continues to support Rust-led and Bun-led applications.
- Made Vite the application bundler through the separate `@solid-gpui/vite` package. Direct JS needs no bundler; JSX/TSX uses one Solid/Oxc transform for Bun and QuickJS builds. Native host startup, bindings, Vite HMR/watch, and Rust-owned development share the same project configuration. Bun retains its runtime APIs; the previous standalone bundler and TSX launchers have been removed.
- Replaced direct Prettier tooling with Oxfmt, preserving the existing formatting style and check commands; removed an unused generated-file formatter.
- Moved process I/O exports to `@solid-gpui/core/stdio`, added the QuickJS `/embedded` transport, and made application transport ownership explicit.
- Removed redundant renderer state, corrected callback and route reactivity, made native Patch rollback complete, and bounded foreground commit polling. Frame decoding is iterative and sibling reindexing avoids repeated vector copies.
- Made maintained documentation and agent guidance English-first while preserving intentional Unicode input cases. Release preparation now enforces gates even for synchronized versions and restores every release file on failure.
- Updated the runtime to the unified gpui-pre/platform 0.3.3 family and gpui-component 0.6.0, retaining Zed only as reference source.
- Redesigned the 18-route Gallery with independent responsive navigation/content panes and generated native controls. Fixed the router flex parent and removed expensive content-derived flex sizing from the window-filling work area.
- Preserved routed layout lifetime and sidebar scroll position across sibling navigation; expanded native scroll regression to live resize patches and Drag & Drop.
- Corrected light-theme surfaces, icon/label button colors, fixed-color swatch contrast, and input placeholder readability.
- Added reusable Rust component schema macros and typed ordinary Rust function exports with generated Promise clients, bounded InvokeNative commands, exact contract lookup and background execution.
- Migrated API-surface inspection to the TypeScript 7 asynchronous compiler API.

- Replaced the previous renderer with a SolidJS universal host renderer backed by the client reactive runtime.
- Renamed TypeScript packages, Rust crates, binaries, environment variables, scripts, workflows, and release artifacts to `solid-gpui`.
- Made Solid owner updates transactional: initial output is a Snapshot and subsequent signal updates are coalesced into incremental Patches.
- Removed the separate development facade; tests use the core `MemoryTransport` and `Root` directly.
- Reworked examples around a single executable Solid signal counter.
- Documented the host-owned rendering boundary using the checked-in GPUI Shell reference as an architectural guide.
- Cut over the renderer/host wire contract to lockstep Bebop v5 with one canonical `.bop` schema, checked generated TypeScript/Rust bindings, and no legacy runtime decoder.
- Added the provider-neutral v5 `Icon` and `Extension` Host Node surface,
  including exact Extension Catalog Identity and transactional adapter
  validation before publication.
- Unified Rust component/command authoring in `solid-gpui` with host-generated TypeScript; `@solid-gpui/core/components` includes Button, Checkbox, Switch, Progress and retained native Input. Vite tools are included in `@solid-gpui/vite`.
- Added the reusable gpui-component provider host: each surface keeps its `Entity<SolidRoot>` renderer target inside a provider-owned `gpui_component::Root`, and `ShowNotification` is rejected because gpui-component owns the global system notification callback.
- Added schema-derived bounded decoding guards for frame/message/repeated-field budgets, strict booleans and enums, UTF-8, field order, unions, terminators, and resource limits.
- Added the owned Rust Event queue with exact bounded reservation and a reusable writer buffer for direct framed output.

### Removed

- Removed the former reconciler, refresh runtime, type packages, test renderer, examples, generated output, and documentation tied to the previous frontend design.
- Removed obsolete host example suites that exercised deleted application examples.

### Verification

- Hosted CI efficiency validation preserves all gates and 314 passing core Rust tests (one existing ignored). Warm native critical path fell from 640 s to 334 s while summed native runner time stayed 640 s versus 638 s. Pages measured 181 s for an exporter-only change and 48 s for two exact artifact hits; first-cache/cold costs remain explicit in [CI measurements](docs/ci.md#september-20-hosted-measurements). Branch cancellation, mixed-cache behavior, isolated local native/package checks and full website builds/tests passed; no production deployment was triggered.
- The September 20 GPUI Kit update passes 1589 vendored library tests, 301 native library tests (one existing opt-in test ignored), two native integration tests, eight native-binding tests and eight website tests. Workspace compilation, strict native Clippy, package type checks, formatting, generated-catalog checks and reproducible dependency notices pass. The WASM website builds. Real macOS and browser interaction verifies input groups, questionnaire navigation/submission, token insertion, editor search/replacement and sequence replay; English/Chinese catalog and guide rendering were inspected. Windows/Linux, mobile touch behavior, image/file paste and quantitative performance were not requalified.

- Follow-up native acceptance found and fixed VirtualList recreation after empty data: Create now starts data revision zero rather than replaying the detached incarnation's edit. The regression failed before the fix and passes afterward; the SDK suite now has 118 passing tests. Real macOS screenshots verify restoration of 10,000 rows, end/filter/command scrolling, retained prepend position, rich-text updates, delayed opacity completion and repeated resizing. The user confirmed manual acceptance on September 20, closing the native acceptance task. Earlier automation limits remain documented separately; no additional per-scenario traces or quantitative presentation measurements are implied.
- The September 20 performance changes pass 287 native library tests (one opt-in experiment ignored), 66 vendored plot/chart tests, protocol goldens in both languages, deterministic protocol regeneration, workspace Cargo checking, strict core Clippy, TypeScript checks and repository formatting. The WASM website builds and its eight tests pass; English/Chinese performance evidence was visually checked in the browser. Serial CPU probes verify structure, identity, text, list count/anchor and geometry invariants; see [performance analysis](docs/performance-analysis.md#september-2026-cpu-comparison). User-confirmed native acceptance does not extend these measurements to release builds, low-end devices, Windows, GPU/FPS or physical-input latency.
- The September 17 DX changes pass the 95-test core suite through the public Vite-backed runner, 53 toolchain tests, package type checks and formatting, and repeated clean tarball install/prepare/check/typecheck/test flows. The desktop command chain and desktop/website production previews were exercised; the website build and English/Chinese guide rendering were checked. Native Dialog footer behavior and a real application-owned Embedded Bun executable returning `1223` were verified. Native screenshot/click acceptance was omitted at the user's request; other operating systems and the full release-qualification matrix were not requalified.

- The September 16 integration fixes pass 254 Rust library tests (one ignored), 86 core package tests, 12 Vite/export/build/API tests, four protocol tests, package type checks, and the website build plus eight website tests. A real Windows VM reproduced the 1 MiB native stack overflow; the same host promoted to 16 MiB completed native UI interaction. The Windows Bun desktop application example passed choice/scroll acceptance and its renderer exited 0.68 seconds after abrupt host termination. See the [integration verification record](.scratch/desktop-app/verification.md#integration-verification-2026-09-16) for the Windows toolchain, remaining fixture failures, and platform qualification limits.
- The September 7 hardening and runtime work passes 78 Bun tests and 287 Rust tests (one documentation example ignored), strict Clippy, protocol goldens, 23-route native Gallery layout/scroll checks, both application bundle targets, and the host archive check. QuickJS tests execute compiled Solid JSX, routing, cancellation, redirects, and every Gallery page in the real VM. See `.scratch/production-hardening/report.md` for scope and remaining limits.
- TypeScript package tests, workspace Rust tests, `bun run check`, and `bun run format` were run for the September 5 changes; see `docs/scroll-performance.md` for measured performance and native acceptance limits.
- `bun run audit` passed on September 7, including dependency advisories under the existing `deny.toml` policy and third-party notice verification. The September 5 maintenance-advisory report retains its historical evidence and a current recheck in `.scratch/supply-chain/issues/04-gpui-component-maintenance.md`.
- Full release qualification includes `bun run ci`, `bun run task embedded-check`, and `bun run task host-candidate-smoke`; the focused results above are not a claim that all release gates passed.
