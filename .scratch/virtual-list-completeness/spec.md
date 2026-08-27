# VirtualList completeness

This workstream audits the six consumer-visible VirtualList edges against the
current GPUI `ListState` integration and closes only behavior that is genuinely
broken. The wire remains unchanged.

## Contract decisions

1. `onEndReached` is a range notification contract, not an exact viewport-bottom
   signal. Native GPUI emits a visible/required range; the renderer expands
   scroll-handler ranges by `overscan`, coalesces placeholder requests for the
   next frame, and deduplicates identical ranges. JavaScript fires the callback
   when the resulting range end reaches `data.length`, once until a later range
   ends before the data length. It can therefore fire before the final row is
   visually at the viewport bottom and can fire during initial layout when the
   required range already reaches the end. A real dispatch test locks this
   behavior and README documents it.
2. A persistent native list state must not jump to the top when its item count
   changes. Reconciliation preserves the prior logical item/offset through a
   uniform-hint reset, clamps an anchor beyond a shrink to the new end, and
   keeps an explicit old end anchor at the new end when data grows. Committed
   rows are remeasured when descendant host content changes so an `itemKey`
   identity shift cannot leave stale heights. Because keys are JavaScript-only,
   native restoration is index/offset based rather than key based. TypeScript's
   range state retains the prior committed range through filter/unfilter when
   no native range update supersedes it.
3. `scrollToIndex` accepts only an existing index (`0 <= index < data.length`);
   `index === data.length` is rejected synchronously/as a rejected Promise and
   sends no command. Valid index and `scrollToEnd` commands are honored by
   `ListState` even while rows carry only uniform estimates; layout then
   replaces estimates as rows are committed and measured.
4. The boundary wrapper remains an input-propagation fix, not a virtualization
   semantic change: GPUI's list listener runs before the wrapper's bubble
   `stop_propagation`, so the list consumes its wheel while an outer View does
   not. Empty-to-non-empty and placeholder flicker remain display-backed
   limitations; headless tests cover no regression in retained tree/range
   behavior, while visual flicker is documented rather than hidden.
5. `estimatedItemSize` is an initial uniform hint for every unmeasured item. A
   committed/visible row is remeasured at natural height, including after a
   range or content update; an item that is never committed legitimately keeps
   the estimate, so scrollbar/content drift converges only as rows are visited.
6. React rows are host subtrees in the committed child range. A row retained in
   the overlapping keyed range keeps its React state; a row removed from the
   committed range is deleted from the React tree and remounts when it returns.
   Consumers must keep durable state outside a virtualized row when it must
   survive eviction.

## Evidence

- Native range emission and placeholder coalescing:
  `crates/react-gpui/src/renderer/paint/virtual_list.rs:39-89`.
- Native range deduplication:
  `crates/react-gpui/src/renderer.rs:374-400`.
- Current reconciliation/reset and descendant remeasurement:
  `crates/react-gpui/src/renderer.rs:293-370`.
- GPUI reset, uniform hints, remeasurement, clamping, and end anchor:
  `references/zed/crates/gpui/src/elements/list.rs:341-375,405-475,501-549,559-674`.
- GPUI layout measures visible rows and retains hints for unrendered rows:
  `references/zed/crates/gpui/src/elements/list.rs:1060-1124,1178-1207`.
- GPUI list bubble ordering and upstream child-stop test:
  `references/zed/crates/gpui/src/elements/list.rs:1591-1620,1873-1925`.
- JavaScript end gate/range state and committed keyed children:
  `packages/react-gpui/src/index.ts:138-192`.
- JavaScript command bounds and command wiring:
  `packages/react-gpui/src/renderer/root-container.ts:224-284` and
  `packages/react-gpui/src/renderer/nodes.ts:150-156`.

## Validation

Focused Rust GPUI tests cover count-change restoration, command behavior with
estimated rows, measured-height convergence, and the wrapper's wheel boundary.
Focused TypeScript renderer tests cover range-triggered end semantics,
filter/unfilter range restoration, index bounds, and row eviction/remount.
README and CHANGELOG record the final consumer contract and the display-backed
boundary.
- One full-CI attempt timed out in the unrelated
  `tests::transport::process_event_writer_kills_closed_stdin_child_for_reader_eof`
  test; two immediate isolated reruns passed. No transport files changed.
