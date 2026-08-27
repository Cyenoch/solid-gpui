# TextInput clipboard keyboard semantics

Status: implemented

## Question

Does the native `TextInput` support Cmd/Ctrl-C, Cmd/Ctrl-X, and Cmd/Ctrl-V
against its UTF-16 selection, including multiline content?

## Evidence before implementation

- `crates/react-gpui/src/renderer/paint/text_input.rs:418-429` handled only
  copy: it read `selected_text_for_copy`, wrote a `ClipboardItem`, and stopped
  propagation. There was no cut or paste key path.
- `crates/react-gpui/src/renderer/input.rs:1107-1115` already converted the
  ordered UTF-16 selection back to a UTF-8 substring.
- `crates/react-gpui/src/renderer/paint/text_input.rs:419-456` now handles
  copy, cut, paste, and Select All. C copies the UTF-16 selection; X copies
  then clears it through native replacement; V inserts text from the host
  clipboard.
- `crates/react-gpui/src/renderer/input.rs:1185-1192` converts the ordered
  UTF-16 selection back to a UTF-8 substring, while
  `:1222-1249` centralizes replacement and cut event emission.
- Pinned GPUI's default `InputHandler::paste` inserts a platform-delivered
  clipboard item (`references/zed/crates/gpui/src/platform.rs:1734-1743`), but
  its macOS key-equivalent path dispatches Cmd-key events through the view
  callback (`references/zed/crates/gpui_macos/src/window.rs:2429-2508`).

## Decision

Implemented host-owned C/X/V handling in the TextInput paint element. Cut copies
then replaces the selected range with empty text; paste reads the native
clipboard and replaces the current selection. Both routes use the existing
native edit event emission and `maxLength` enforcement. No wire change or JS
clipboard callback was added.

## Verification

The renderer test dispatches a real mouse selection followed by real `cmd-c`,
`cmd-x`, and `cmd-v` key events and asserts clipboard/text/selection state,
including a multiline cross-paragraph copy.
