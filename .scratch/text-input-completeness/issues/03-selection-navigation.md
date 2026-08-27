# TextInput Select All and word-wise navigation

Status: implemented

## Question

Are Select All and Option/Alt word-wise arrow movement available through the
existing host keymap seam?

## Evidence

- `crates/react-gpui/src/renderer/paint/text_input.rs:404-418` now forwards
  unmodified arrows/Home/End/Up/Down and allows Alt/Option word mode; the
  adjacent handler at `:419-456` owns Cmd/Ctrl-A.
- `crates/react-gpui/src/renderer/input.rs:1088-1156` preserves the
  character-wise movement helper and adds word-boundary movement, while the
  existing UAX word helper remains at `:996-1021`.
- The key handler's movement and selection methods use the same UTF-16 ranges
  and reversed-head orientation as the existing mouse and IME paths.

## Decision

Implemented Cmd/Ctrl-A as host-owned selection of the full UTF-16 text and
Option/Alt-Left/Right as word-boundary movement. Shift extends from the current
head while preserving reversed orientation. Word movement reuses
`word_selection_range`; no second segmentation implementation or wire field
was added. Existing bare and Shift arrow/Home/End behavior remains unchanged.

## Verification

The renderer test establishes a real selection and drives Select All, bare
Home/End, word-wise arrows, and Shift+word movement through
`VisualTestContext::simulate_keystrokes`.
