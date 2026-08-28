# React keyboard focus traversal

Status: resolved

## Evidence

- Predecessor audit: `history://focus-order` records that GPUI `.focusable()`
  does not imply a tab stop and that `FocusHandle.tab_stop(true)` is required.
- `crates/react-gpui/src/renderer/paint/mod.rs` applies explicit tab-stop
  semantics to focusable View and Pressable nodes.
- `crates/react-gpui/src/renderer/input.rs` synchronizes `FocusHandle` tab-stop
  metadata for View, Pressable, TextInput, and selectable Text.
- `crates/react-gpui-host/tests/command_roundtrip.rs` exercises real native
  dispatch for tree-order traversal, wrapping, disabled Pressable skipping,
  and conditional insertion.

## Resolution

Native traversal now reaches React focus nodes and follows GPUI tab-group path
plus insertion order. Disabled controls are skipped, traversal wraps, and each
native window has an independent focus graph.

Resolved-with-commit: `fix(renderer): make keyboard focus traversal reach react nodes`.
