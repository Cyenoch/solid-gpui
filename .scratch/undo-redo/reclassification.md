# Undo/redo reclassification

Undo/redo was previously classified as an upstream gap in
`.scratch/text-input-completeness/spec.md` and
`.scratch/text-input-completeness/issues/02-undo.md`, because pinned GPUI has
no `InputHandler` undo method and macOS has no NSTextView-backed action.

That classification is corrected: this renderer owns `NativeInputState` and
routes every TextInput edit through its handlers, so a bounded host-owned
history is a legitimate project feature, the same as host-owned selectable
Text. No GPUI primitive or wire field is required. The implemented contract is
recorded in `spec.md` in this directory.
