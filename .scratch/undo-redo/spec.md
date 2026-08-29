# Host-owned TextInput undo and redo

## Goal

Provide editor-grade undo/redo for every TextInput-shaped consumer without a
protocol or GPUI API change. `NativeInputState` already owns the authoritative
text, UTF-16 selection, and marked range, so the history stays in the host
alongside that model.

## Contract

Each input has a bounded history of 100 pre-edit snapshots. This cap keeps
per-input memory bounded while retaining a useful window for ordinary editing
sessions. A snapshot stores text, ordered UTF-16 selection, and selection
orientation; marked text is never stored in history. The oldest snapshot is
evicted when the cap is reached. Undo transfers the current snapshot to redo;
redo transfers the current snapshot to undo. Any new edit clears redo. Empty
undo/redo are no-ops.

Typing coalesces deterministically: consecutive ordinary insertions merge only
when both the prior and current operation have a collapsed caret and the
current operation starts at the prior operation's resulting caret. The state
stores no wall-clock timer. Cursor movement and selection-only changes never
push a history entry, but they break typing coalescing; thus the next edit
captures the current selection as its pre-state. Paste, cut, deletion, and
edits replacing a word/line selection are explicit boundaries.

Cmd/Ctrl-Z invokes undo. Shift-Cmd-Z and Ctrl-Y invoke redo. The key path
stops propagation only when an operation changed state. Undo and redo emit the
same change and selection events as ordinary native edits, so controlled inputs
observe reverted values through the normal event and acknowledgement pipeline;
there is no second state channel.

When an IME marked range is active, its first marked replacement captures a
composition baseline. Subsequent marked replacements update the composition
without adding entries. `unmark_text` commits that baseline as one `ImeCommit`
entry. If undo is pressed while marked text is active, the host commits the
composition first and then pops it as one entry. This is intentionally simple
and avoids discarding committed IME text.

## Implementation

- `crates/react-gpui/src/renderer/input.rs` contains snapshots, bounded stacks,
  coalescing, composition grouping, restore, and root event helpers.
- `crates/react-gpui/src/renderer/paint/text_input.rs` handles undo/redo key
  equivalents beside the existing host-owned clipboard keys.
- The wire protocol is unchanged.

## Verification

Focused Rust tests cover typing coalescing and selection boundaries,
paste/cut/IME boundaries, undo/redo and redo invalidation, cap eviction, and a
real `VisualTestContext` dispatch path that observes the reverted change event.
