# `estimatedItemSize` hint honesty

Status: resolved
Type: audit

Question: Does a wrong estimate persist or get replaced by measured rows?

Evidence: The renderer seeds every ListState item with a uniform estimate
(`crates/react-gpui/src/renderer.rs:322-328`), and GPUI documents that rendered
items replace hints while unrendered items retain them
(`references/zed/crates/gpui/src/elements/list.rs:341-349`). During layout,
visible/unknown items are rendered and measured
(`references/zed/crates/gpui/src/elements/list.rs:1060-1124`), while the
remaining tree keeps cached sizes (`references/zed/crates/gpui/src/elements/list.rs:1178-1207`).
Range changes and affected committed descendants explicitly invalidate the
committed range (`crates/react-gpui/src/renderer.rs:363-370`).

Decision: No semantic change. Add/strengthen a headless variable-height test to
show measured rows replace an intentionally wrong estimate and document that a
never-committed row legitimately remains estimated until visited.

## Comments
- Strengthened the variable-height coverage with a deliberately wrong estimate
  and a later committed row; measurement replaces the hint while an unvisited
  placeholder remains estimated.
- Verified by `cargo test -p react-gpui virtual_list --locked` (7 passed) and
  `bun test tests/renderer.test.tsx` (65 passed).
