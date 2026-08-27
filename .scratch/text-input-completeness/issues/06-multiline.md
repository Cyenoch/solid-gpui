# TextInput multiline Enter and cross-line selection

Status: implemented

## Evidence before implementation

- The TextInput key handler emits submit only when `input.multiline` is false
  (`crates/react-gpui/src/renderer/paint/text_input.rs:394-403`), leaving
  multiline Enter to the platform text-input path.
- Multiline shaping stores paragraph starts and wrapped lines
  (`crates/react-gpui/src/renderer/paint/text_input.rs:118-136`), and the
  cached layout maps UTF-16 positions through the wrapped text
  (`crates/react-gpui/src/renderer/input.rs:92-126,246-297`).
- Copy reads the native ordered UTF-16 selection directly rather than relying
  on a single visual row (`crates/react-gpui/src/renderer/input.rs:1185-1192`);
  the real-dispatch renderer test copies a selection spanning both paragraphs
  (`crates/react-gpui/src/renderer.rs:1302-1438`).

## Decision

Keep Enter-to-newline separate from single-line submit and verify that a
selection spanning a wrapped/multiline boundary copies the exact UTF-8 text,
including its internal newline, through the existing native clipboard path.
No protocol or multiline-specific copy wire was added.
