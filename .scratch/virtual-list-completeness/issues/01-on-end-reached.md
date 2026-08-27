# `onEndReached` range semantics

Status: resolved
Type: audit

Question: Is `onEndReached` exact-end, near-end, scroll-only, and once-only?

Evidence: Native `ListState` calls the renderer scroll handler with the current
visible range (`references/zed/crates/gpui/src/elements/list.rs:941-959`). The
VirtualList handler expands that range by `overscan` and emits it
(`crates/react-gpui/src/renderer/paint/virtual_list.rs:39-46`); placeholder/row
render requests also coalesce and emit a next-frame range
(`crates/react-gpui/src/renderer/paint/virtual_list.rs:48-67`). Rust suppresses
identical range events (`crates/react-gpui/src/renderer.rs:374-400`). JavaScript
fires when the received range end is `>= data.length` and gates repeats until a
later range ends below the data length
(`packages/react-gpui/src/index.ts:145-159`).

Decision: Keep the zero-wire implementation. The callback is deterministic with
respect to the emitted committed/required range, not the exact visual bottom:
it can fire near the end (including initial layout), once per range reach, and
can fire again after the range retreats. Add a real framed-dispatch regression
test for threshold, dedupe, and re-entry, and document the contract.

## Comments

The visible range event is intentionally also the JavaScript committed-range
request, so adding a second raw-viewport event would be an unnecessary wire
change.
