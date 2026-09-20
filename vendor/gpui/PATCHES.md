# GPUI dependency patch

Source: crates.io `gpui-pre` 0.3.5, copied from the resolved registry source.
The original Apache-2.0 license and upstream provenance are retained.

Local changes:

- Bundle the two upstream SVG-test font fixtures with their original licenses,
  and use crate-relative test paths; the published crate refers outside its package.

- Include the line limit in wrapped text cache identity, so changing a line clamp
  cannot reuse incompatible geometry within or across frames.

- Preserve virtual List height hints on first layout and width changes. Width
  changes invalidate measured rows while retaining estimates for unmeasured
  scroll geometry; visible layout replaces them with actual sizes. This avoids
  shrinking a large list's scrollbar to the measured subset or measuring all
  rows. The solid-gpui ScrollShadow composition regression covers the 100,000-row
  extent, bounded committed ranges, native commands, resizing, and filtering.

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

The native bidirectional geometry refactor is tracked in
`../../.scratch/native-production-completion/bidi-implementation.md`.
