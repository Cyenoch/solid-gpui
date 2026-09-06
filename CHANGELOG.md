# Changelog

## Unreleased

### Changed

- Updated the runtime to the unified gpui-pre/platform 0.3.3 family and gpui-component 0.6.0, retaining Zed only as reference source.
- Redesigned the 18-route Gallery with independent responsive navigation/content panes and generated native controls. Fixed the router flex parent and removed expensive content-derived flex sizing from the window-filling work area.
- Preserved routed layout lifetime and sidebar scroll position across sibling navigation; expanded native scroll regression to live resize patches and Drag & Drop.
- Corrected light-theme surfaces, icon/label button colors, fixed-color swatch contrast, and input placeholder readability.
- Added reusable Rust component schema macros and typed ordinary Rust function exports with generated Promise clients, bounded InvokeNative commands, exact contract lookup and background execution.
- Migrated API-surface inspection to the TypeScript 7 asynchronous compiler API and Babel preset declarations to Babel 8 types.

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
- Unified Rust component/command authoring in `solid-gpui` with host-generated TypeScript; `@solid-gpui/core/components` includes Button, Checkbox, Switch, Progress and retained native Input. Vite tools are included in `@solid-gpui/core/vite`.
- Added the reusable gpui-component provider host: each surface keeps its `Entity<SolidRoot>` renderer target inside a provider-owned `gpui_component::Root`, and `ShowNotification` is rejected because gpui-component owns the global system notification callback.
- Added schema-derived bounded decoding guards for frame/message/repeated-field budgets, strict booleans and enums, UTF-8, field order, unions, terminators, and resource limits.
- Added the owned Rust Event queue with exact bounded reservation and a reusable writer buffer for direct framed output.

### Removed

- Removed the former reconciler, refresh runtime, type packages, test renderer, examples, generated output, and documentation tied to the previous frontend design.
- Removed obsolete host example suites that exercised deleted application examples.

### Verification

- TypeScript package tests, workspace Rust tests, `bun run check`, and `bun run format` were run for the September 5 changes; see `docs/scroll-performance.md` for measured performance and native acceptance limits.
- `bun run audit` remains blocked by three transitive maintenance advisories tracked in `.scratch/supply-chain/issues/04-gpui-component-maintenance.md`.
- Full release qualification still includes `bun run ci`, `cargo check -p solid-gpui-host --features embedded-bun`, and `bun run example:smoke`; the focused results above are not a claim that all release gates passed.
