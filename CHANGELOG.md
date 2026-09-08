# Changelog

## Unreleased

### Changed

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

- The September 7 hardening and runtime work passes 78 Bun tests and 287 Rust tests (one documentation example ignored), strict Clippy, protocol goldens, 23-route native Gallery layout/scroll checks, both application bundle targets, and the host archive check. QuickJS tests execute compiled Solid JSX, routing, cancellation, redirects, and every Gallery page in the real VM. See `.scratch/production-hardening/report.md` for scope and remaining limits.
- TypeScript package tests, workspace Rust tests, `bun run check`, and `bun run format` were run for the September 5 changes; see `docs/scroll-performance.md` for measured performance and native acceptance limits.
- `bun run audit` passed on September 7, including dependency advisories under the existing `deny.toml` policy and third-party notice verification. The September 5 maintenance-advisory report retains its historical evidence and a current recheck in `.scratch/supply-chain/issues/04-gpui-component-maintenance.md`.
- Full release qualification includes `bun run ci`, `bun run task embedded-check`, and `bun run task host-candidate-smoke`; the focused results above are not a claim that all release gates passed.
