# GPUI dependency patch

Source: crates.io [`gpui-pre` 0.3.7](https://crates.io/crates/gpui-pre/0.3.7),
Zed snapshot `1a28cff4b409169bac058bca40dfbfeb7621d19b` as declared by
`Cargo.toml.orig` and the published crate manifest. Compared against the actual
0.3.7 archive on 2026-10-02; its SHA-256 is
`0e87a42bb37c7cb4e76dd1ac0ce88851e46e976e0373a47ab3e0757abffee54d`.
The original Apache-2.0 license and upstream provenance are retained.
See the [source and file inventory](../GPUI-SOURCES.md) for all patched GPUI crates.

Local changes:

- Bundle the two upstream SVG-test font fixtures with their original licenses,
  and use crate-relative test paths; the published crate refers outside its package.

- Include the line limit in wrapped text cache identity, so changing a line clamp
  cannot reuse incompatible geometry within or across frames. The regression
  constructs the cache with the font-generation counter required by GPUI 0.3.7.
  A [local upstream patch](../../.scratch/comparison-adoption/upstream/gpui-line-clamp-cache.patch)
  contains this isolated fix and regression; nothing has been submitted upstream.

- Preserve virtual List height hints on first layout and width changes. Width
  changes invalidate measured rows while retaining estimates for unmeasured
  scroll geometry; visible layout replaces them with actual sizes. This avoids
  shrinking a large list's scrollbar to the measured subset or measuring all
  rows. The solid-gpui ScrollShadow composition regression covers the 100,000-row
  extent, bounded committed ranges, native commands, resizing, and filtering.
  `splice_with_size_hint` preserves measured neighbors and scroll anchors while
  seeding replacement rows; the in-crate regressions cover replace/prepend/append.

- Clamp Div wheel offsets before notification, let local listeners run first,
  and stop bubbling only when Div or List actually consumes displacement.
  Parent scrolling remains reachable at an inner boundary (`elements/div.rs`,
  `elements/list.rs`).

- Separate bounds synchronization and observer delivery from full-tree refresh.
  Viewport, scale, display identity, visual window state, and content-relevant
  pointer changes invalidate rendering; passive origin changes and repeated
  same-size callbacks do not. Observer notifications and forced recovery frames
  retain their own invalidation semantics. Test-platform move and state callbacks
  cover observer delivery, hover painting, cursor resolution, and recovery.

- Use the executor clock for spring animation updates, matching the scheduler
  that advances deterministic animation tests instead of mixing simulated time
  with wall-clock elapsed time.

- Keep managed-image frame leases through redraw and select failure fallbacks from
  the current provider request. Decode failures lay out and prepaint the fallback
  within the resolved image bounds; replacing a source or resizing to a pending
  source-set candidate does not inherit a previous request's failure.
  `ManagedImageSource` supplies intrinsic size and physical frame context with
  an entity lease. `ParsedSvg::size` exposes dimensions before raster allocation;
  `ObjectFit` derives the value traits used by managed requests.

- Share popup geometry/display selection and add native popup repositioning to
  `PlatformWindow` and `Window` (`platform/popup.rs`, `platform.rs`, `window.rs`).
  The test platform records anchor requests; native provider implementations are
  inventoried separately. Unsupported platforms return an explicit capability
  error.

The native bidirectional geometry refactor is tracked in
`../../.scratch/native-production-completion/bidi-implementation.md`.
