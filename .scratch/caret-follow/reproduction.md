# Caret visibility reproduction

Before this change, `TextInputElement` shaped all wrapped lines at the element origin and painted the caret directly from `position_for_utf8`. There was no native per-input visual offset or content mask. With six lines and a 40px host viewport, the layout content measured 156px and the end caret landed at y=130px, outside the viewport. The focused regression test `overflowing_multiline_text_input_keeps_caret_inside_element_bounds` captures this scenario and now verifies the computed offset keeps the caret in bounds.

The fix stores a host-owned `scroll_offset` beside selection in `NativeInputState`. Prepaint computes a bounded offset from the active caret or IME marked range, clamps it to content minus viewport, shifts text/caret/selection consistently, clips painting to element bounds, and maps mouse/IME geometry through the offset. No wire state or new events are involved.
