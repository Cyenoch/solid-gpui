# List boundary and empty/placeholder regression

Status: resolved
Type: audit

Question: Did the new boundary wrapper or drag-capability changes alter list
scrolling, empty transitions, or placeholder behavior?

Evidence: VirtualList wraps the GPUI list in a bubble `on_scroll_wheel` boundary
that calls `stop_propagation` (`crates/react-gpui/src/renderer/paint/virtual_list.rs:91-106`).
GPUI registers the list listener during paint before painting children, and its
upstream test proves a child stop prevents list scrolling
(`references/zed/crates/gpui/src/elements/list.rs:1591-1620,1873-1925`). Empty
state is a JavaScript branch that emits no VirtualList node
(`packages/react-gpui/src/index.ts:161-164`), while committed rows and
placeholders are selected in the native row closure
(`crates/react-gpui/src/renderer/paint/virtual_list.rs:70-89`).

Decision: Keep the wrapper. Add a display-backed GPUI test proving list wheel
consumption still occurs while an outer View does not receive the same wheel;
retain the existing empty/range headless coverage. Placeholder flicker remains
a documented display-boundary limitation, not a renderer workaround.

## Comments
- Kept the wrapper's propagation boundary and added a display-backed wheel
  regression test. Empty-state and placeholder behavior remain the documented
  JavaScript/display boundary rather than gaining a fallback.
- Verified by `cargo test -p react-gpui virtual_list --locked` (7 passed) and
  `bun test tests/renderer.test.tsx` (65 passed).
