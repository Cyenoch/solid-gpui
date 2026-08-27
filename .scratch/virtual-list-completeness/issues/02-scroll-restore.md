# Scroll restoration across data changes

Status: resolved
Type: task

Question: Does a persistent `ListState` keep a sane position across shrink,
grow, and key/filter changes?

Evidence: Renderer state is keyed by the VirtualList node id
(`crates/react-gpui/src/renderer.rs:293-370`). GPUI's
`reset_with_uniform_height` clears `logical_scroll_top`
(`references/zed/crates/gpui/src/elements/list.rs:352-375`), while
`ListState::scroll_to` clamps an anchor at/after the item count to the new end
(`references/zed/crates/gpui/src/elements/list.rs:659-674`). JavaScript retains
its prior range state and clamps only the derived values
(`packages/react-gpui/src/index.ts:138-149`).

Decision: Preserve the old logical item/offset around a uniform-hint reset;
clamp shrunk anchors using `scroll_to`, keep an explicit old-end anchor at the
new end when data grows, and remeasure the current committed range when changed
host descendants indicate identity/content replacement. Native restoration is
index/offset based because `itemKey` is not on the wire. TypeScript retains its
prior committed range through filter/unfilter when no native range update
supersedes it.

## Comments
- Implemented anchor preservation around GPUI's uniform-hint reset. Shrinks
  clamp through `scroll_to`, explicit old-end anchors stay at the new end on
  growth, and affected committed descendants trigger remeasurement.
- Verified by `cargo test -p react-gpui virtual_list --locked` (7 passed) and
  the filter/unfilter case in `bun test tests/renderer.test.tsx` (65 passed).
