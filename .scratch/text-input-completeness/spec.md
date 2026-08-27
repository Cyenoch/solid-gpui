# TextInput completeness

## Goal

Close the bounded native editing gaps in `TextInput` without changing the
protocol wire: host-owned keyboard semantics use the existing Rust selection
state and GPUI clipboard APIs. Preserve UTF-16 positions externally and UTF-8
text storage internally.

## Audit and decisions

| Item | Current state and evidence | Decision |
| --- | --- | --- |
| Clipboard | `TextInput` now paints host-owned C/X/V handling at `crates/react-gpui/src/renderer/paint/text_input.rs:419-456`; C copies the UTF-16 selection, X copies then replaces it, and V inserts native clipboard text. GPUI's `InputHandler::paste` default only handles a platform-delivered paste (`references/zed/crates/gpui/src/platform.rs:1734-1743`), while macOS key-equivalent handling dispatches Cmd-key callbacks through the view (`references/zed/crates/gpui_macos/src/window.rs:2429-2508`). | **implemented**: host-owned Cmd/Ctrl-C/X/V; cut replaces the current UTF-16 selection and paste uses the native clipboard. |
| Undo/redo | The input state has no history, and GPUI's `InputHandler` contract has no undo/redo operation (`references/zed/crates/gpui/src/platform.rs:1669-1797`). Pinned macOS GPUI explicitly routes `OsAction::Undo`/`Redo` to a disabled fallback because no NSTextView/NSTextField exists (`references/zed/crates/gpui_macos/src/platform.rs:349-358`). | **upstream-gap**: document the missing primitive; do not invent app-level undo. |
| Select All and word navigation | TextInput paint dispatches bare arrows plus Alt/Option word mode (`crates/react-gpui/src/renderer/paint/text_input.rs:404-418`), host-owned Select All is in the same key handler (`crates/react-gpui/src/renderer/paint/text_input.rs:419-456`), and `move_word_selection` reuses the UAX helper (`crates/react-gpui/src/renderer/input.rs:996-1096`). | **implemented**: Cmd/Ctrl-A and Alt/Option-Left/Right (including Shift extension), reusing the existing word-range helper. |
| Password/secure input | `TextInputProperties` has no secure flag (`crates/react-gpui/src/protocol.rs:337-350`); the package contract already records `secureTextEntry` as unsupported (`packages/react-gpui/README.md:623-625`). Pinned GPUI exposes no password-obscuring input primitive. | **upstream-gap**: retain documentation and skip. |
| `maxLength` with IME marked text | Both ordinary and marked replacement paths calculate available UTF-16 units and truncate through the same helper (`crates/react-gpui/src/renderer/input.rs:325-403`). State and controlled reconciliation also clamp to UTF-16 units (`crates/react-gpui/src/renderer/input.rs:343-358,420-433`). | **resolved-no-gap**: add focused marked-text coverage; no separate IME workaround. |
| Multiline Enter and copy | Submit is gated by `input.multiline` (`crates/react-gpui/src/renderer/paint/text_input.rs:394-403`); multiline layout caches paragraph starts and wrapped lines (`crates/react-gpui/src/renderer/paint/text_input.rs:118-136`), while copy reads the UTF-16 selection directly from native text (`crates/react-gpui/src/renderer/input.rs:1185-1192`). | **implemented**: keep Enter-to-newline platform input separate from single-line submit and verify cross-line selection copy through the multiline layout. |

## Scope and invariants

- No TypeScript or protocol tuple change is required.
- Clipboard operations are host-owned and do not emit a JavaScript copy/cut/paste callback.
- Selection and edit events retain their existing `EVENT_CHANGE` and
  `EVENT_SELECTION` payloads, including `selection_reversed`.
- Cut and paste use `NativeInputState::replace`, so `maxLength`, marked-text
  clearing, edit sequencing, UTF-16 offsets, and controlled acknowledgement
  behavior remain centralized.
- Secure text and undo/redo remain explicit upstream gaps rather than fake
  fallbacks.

## Verification plan

The focused Rust tests use the existing GPUI `VisualTestContext` dispatch seam:
real mouse events establish selection, real key events drive C/X/V, Select All,
word-wise arrows, and Home/End, and the multiline case copies a selection that
crosses a paragraph boundary. Unit coverage only protects the UTF-16 boundary
and marked-text max-length invariants.
