# Focused-node unmount lifecycle

Status: resolved

## Evidence

- Predecessor audit: `history://focus-order` identifies
  `Context::on_focus_lost` as the synchronous native boundary for a removed
  focused node and requires blur before restoration.
- `crates/react-gpui/src/renderer.rs` tracks the focused React node, emits its
  direct blur using the old listener identity, and restores the nearest live
  target or deterministic first focus stop.
- `packages/react-gpui/src/renderer/dispatch.ts` accepts only one terminal
  detached focus/blur notification; press, key, pointer, and other detached
  events remain rejected.
- `crates/react-gpui-host/tests/command_roundtrip.rs` verifies real dispatch
  emits blur for a focused deleted node and restores its live ancestor.

## Resolution

Focused-node unmount now synchronously emits JavaScript blur and restores
focus without relying on a detached node remaining in the ordinary listener
maps. Detached focus callbacks are released after the terminal notification.

Resolved-with-commit: `fix(renderer): restore focus and emit blur on focused-node unmount`.

## Integration note

The outside-click gate exposed an adjacent lifecycle regression during
integration: the in-flight `host-config.ts` detached-focus changes dropped the
existing `appendChild` `child.parent = parent` assignment. That omission was in
this working tree after the focus lifecycle edit, not in accessibility commit
`0e51899` (which only changes accessibility prop comparison in this file).
Restoring the parent link fixes overlay anchor bounds and keeps the existing
outside-click dispatch intact.
