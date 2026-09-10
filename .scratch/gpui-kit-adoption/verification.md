# GPUI Kit adoption verification

Date: 2026-09-10. Upstream: `05433bd8e9e75af2f3aa508141b78ba21bfa6261`.
This records the final scope: Component, Base, Kit testing, FPS, and separated
asset ownership. Shell and its JIT migration were withdrawn at the user's request.

## Delivered contracts

- Reference and vendor roots are `gpui-kit`; crate names retain their upstream identity.
- Carousel retains keyed native selection, keyboard interaction, controls and pagination.
- Editor exposes directed multiple selections, native auto-close/smart indent and
  bounded language rules; invalid UTF-8/CRLF and stale selection updates are atomic.
- Markdown frontmatter and shared SVG icon sources are opt-in native capabilities.
- Native Motion, springs, keyframes, stagger and Presence retain native animation state.
  The Solid Presence helper releases its child owner only after the current exit.
- BaseButton/Checkbox/Switch/Toggle keep native interaction with application visuals.
- FPS is explicitly enabled and owned per window. The old local HUD crate is removed.
- Kit supplies only its 101 default control assets. The upstream source contains
  1830 SVGs; Web no longer embeds that full directory. Iconify and registered app
  icons have independent names and asset namespaces. Internal Kit paths are rejected
  by application component icon props; there is no cross-catalog fallback.

## Automated checks

- Workspace Rust tests with QuickJS passed, including 254 core tests (one existing
  opt-in test ignored), native Kit interaction tests, macros, and Iconify tests.
- Workspace Clippy with all targets, QuickJS, and warnings denied passed.
- Rust formatting, SDK generation verification and TypeScript package CI passed.
- SDK tests include Presence reversal and cleanup; native tests cover multiple
  selections/undo, keyed Carousel reorder, disabled Base activation, icon namespace
  isolation, and per-window FPS cleanup.
- Website build (including Rust WASM), all six website tests, and the bilingual
  catalog/example type check passed. Native migration bindings were regenerated.
- Third-party notices were regenerated; Cargo advisory checks passed.

## Actual UI

The built website initialized WebGPU without console errors. Carousel navigation
changed Overview to Details and updated its pagination. NativePresence visibly
faded and removed its child. Layout was inspected at reported canvas sizes
1683×1052 and 1052×1183; the preview tool's resize timed out and later reverted
its viewport, so this is inspection evidence rather than a stable device emulation.

A real macOS diagnostic window completed Count: 360. Clicking Increment produced
361, typing replaced the input with Native input verified, and right-clicking the
HUD switched observed FPS to MAX capacity. The window closed afterwards. Synthetic
scroll did not establish content displacement; physical trackpad acceptance is
not claimed. Windows/Linux real windows were not tested.

Browser screenshots are available in the session artifacts:

- Carousel after navigation: `/Users/jgbingzi/.t3/userdata/browser-artifacts/browser-screenshot-127-0-0-1-mtvig4w7-cb07341d.png`
- Presence after exit: `/Users/jgbingzi/.t3/userdata/browser-artifacts/browser-screenshot-127-0-0-1-mtviivp0-a45286c5.png`

## Bounded native measurement

See [raw completion and interval records](native-hud-comparison.json). Both runs
used one diagnostic binary, 500 rows and 360 verified incremental updates,
including width changes 800 → 560 → 1280 → 800. Both finished at 800×600, scale 2,
with one Solid App construction, 360 operations and 32652 frame bytes. Each run
completed in approximately 12.5 seconds with no concurrent compilation.

Signal-to-frame p95 was 0.239 ms without the HUD and 0.257 ms with it. These are JS
commit-production measurements, not native paint or input-to-display latency.
Window activation differed and some render intervals were suppressed, so the
samples do not establish a controlled CPU/presentation overhead comparison.
The fixture now checks the final content viewport; it does not confuse platform
window bounds, which include titlebar chrome, with the drawable area.

The visible HUD intentionally refreshes at 500 ms. Inactive but still rendered
windows can continue refreshing; it stops after no longer being rendered.
The default host remains HUD-disabled. No measured performance improvement is claimed.

## Documentation synchronization

Updated the documentation index, GPUI Kit guide and translation, Iconify guide
and translation, performance and scroll guides, package/project overviews, vendor
provenance and notices, website README, component families, examples, variants,
translations and generated SDK/website/migration bindings. Website content is
loaded from the authoritative Markdown sources, with generated APIs from Rust.
