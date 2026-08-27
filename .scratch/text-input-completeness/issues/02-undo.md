# TextInput undo and redo

Status: upstream-gap

## Question

Does pinned GPUI provide a native undo/redo primitive for this custom
`ElementInputHandler` path?

## Evidence

- `crates/react-gpui/src/renderer/input.rs:30-42` stores text, selection,
  marked range, edit sequence, and max length, but no edit history.
- The pinned `InputHandler` contract is an NSTextInputClient-shaped interface
  for selection, marked text, replacement, paste, geometry, and text length;
  `references/zed/crates/gpui/src/platform.rs:1669-1797` has no undo or redo
  method.
- Pinned macOS GPUI maps `OsAction::Undo` and `OsAction::Redo` to
  `handleGPUIMenuItem:` and comments that they are always disabled without an
  NSTextView/NSTextField (`references/zed/crates/gpui_macos/src/platform.rs:349-358`).
- GPUI's key-dispatch documentation demonstrates that applications must bind
  their own `Undo`/`Redo` actions (`references/zed/crates/gpui/src/key_dispatch.rs:8-40`);
  this is not an input-handler primitive.

## Decision

Document as an upstream gap. Do not add fake app-level history or intercept
Cmd/Ctrl-Z without a real GPUI primitive. Shift-Cmd-Z redo is likewise
unsupported.
