# Word and line selection on TextInput multi-click

Status: Implemented

## Scope and semantics

- A left mouse down with `click_count == 2` selects the UAX #29 word containing the mapped caret. `unicode_word_indices` supplies alphanumeric/number word ranges. At whitespace or punctuation with no containing word, selection prefers the next word on the same logical line and otherwise the preceding word (the forward-adjacent editor behavior).
- A left mouse down with `click_count == 3` selects the clicked logical line's content, excluding its terminating newline. A single-line TextInput's logical line is the entire text.
- Multiline selection uses paragraph/logical lines, not wrapped visual rows. Word lookup is restricted to the clicked logical line.
- Dragging after a double-click expands/contracts whole words. Dragging after a triple-click expands/contracts whole logical lines while preserving the anchor/head orientation, including reversed selections. A normal single click retains character-wise drag behavior.
- Selection remains UTF-16 externally and UTF-8 internally; existing highlighting, Shift navigation, and Cmd/Ctrl-C copy paths remain the observation surfaces. No protocol or TypeScript wire change is required.

## Evidence gathered before implementation

- `crates/react-gpui/src/renderer/input.rs:20-29` defines `NativeInputState` with text and UTF-16 selection/`selection_reversed`; `:67-143` maps shaped text positions and multiline layout; `:644-659` resolves a mouse point to a UTF-16 index through the cached layout; `:662-702` handled focus, anchor setup, selection, and selection-event emission on mouse down; `:704-747` handled character-wise drag updates and cleanup.
- `crates/react-gpui/src/renderer/paint.rs:620-650` owns the TextInput mouse subscriptions. The `MouseDownEvent` is available there and now passes its `click_count` into the input module; mouse move/up remain the existing dispatch path.
- The pinned GPUI source has `MouseDownEvent.click_count: usize` (`references/zed/crates/gpui/src/interactive.rs:139-162`). `VisualTestContext::simulate_mouse_down` hard-codes `click_count: 1` (`references/zed/crates/gpui/src/app/visual_test_context.rs:274-292`), as does the host helper (`crates/react-gpui-host/src/test_support.rs:210-215`). The same visual context exposes real event dispatch (`:324-331`), so tests construct `MouseDownEvent` with count 2/3 and call `simulate_event`; they do not invoke handlers directly.
- Existing display-backed tests in `crates/react-gpui/src/renderer.rs:996-1046` exercise multiline TextInput layout, and `:1048-1141` exercises real mouse dispatch and Cmd-C clipboard copy for selectable text. The new TextInput tests follow this `#[gpui::test]`/`VisualTestContext` pattern.
- `unicode-segmentation` is already in `Cargo.lock` as version `1.13.3` (lock entries around `:5631-5637`) and is now a direct workspace/react-gpui dependency, allowing `UnicodeSegmentation::unicode_word_indices` without relying on a transitive import.

## Deferrals

None. The pinned adapter helper cannot express click counts, but the real `simulate_event(MouseDownEvent { click_count: ... })` seam can, so no interaction behavior is deferred and no test helper/wire surface is added.

## Verification

- `cargo test -p react-gpui --lib renderer::input --locked`: 30 tests passed.
- `cargo test -p react-gpui --lib text_input_multi_click_dispatch --locked`: 1 real-dispatch word/line/drag/copy test passed.
- `cargo test -p react-gpui --lib text_input_triple_click_dispatch_selects_entire_single_line --locked`: 1 real-dispatch single-line test passed.
- `cargo test -p react-gpui --lib --locked`: 87 tests passed.
