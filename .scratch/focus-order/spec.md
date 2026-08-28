# Focus order and lifecycle

Status: resolved with commit.

## Scope

React GPUI keyboard traversal must expose every eligible React focus node to
GPUI's native tab graph, preserve tree/commit order for ordinary siblings, skip
disabled controls, wrap within one native window, and preserve per-window
isolation. Focused-node removal must synchronously notify JavaScript `onBlur`
and restore focus to the nearest live ancestor/restore target or the first
eligible stop.

Dynamic `UPDATE_FOCUSABLE` patches must be accepted for both `View` and
`Pressable` nodes. The wire protocol remains unchanged.

## Evidence

The implementation follows the predecessor audit at `history://focus-order`:
GPUI `.focusable()` does not imply a tab stop, `FocusHandle.tab_stop(true)` is
required, tab ordering is based on tab-group path plus insertion index, and
`Context::on_focus_lost` exposes the live ancestor restore target. These are
pinned-GPUI facts used by the fix rather than re-researched assumptions.

## Resolved behavior

- View and Pressable focus handles are explicit native tab stops.
- Disabled Pressables and disabled TextInputs are excluded; selectable Text is
  included.
- Traversal follows GPUI's tab-group path and insertion index, wrapping and
  remaining isolated per window.
- Focus loss emits the old node's blur before restoration, including when the
  node has already detached from the React graph.
- Detached focus/blur delivery is terminal and one-shot; other detached events
  remain rejected.
- Pressable focusable updates pass tree validation.

Resolved-with-commit: `fix(renderer): make keyboard focus traversal reach react nodes`;
`fix(renderer): restore focus and emit blur on focused-node unmount`;
`fix(tree): accept pressable focusable updates`.
