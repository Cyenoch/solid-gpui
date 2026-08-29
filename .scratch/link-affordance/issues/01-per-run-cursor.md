# 01 — Per-run pointing cursor for interactive Text

Status: resolved
Type: task

Interactive nested `Text` runs now use pinned GPUI `InteractiveText`'s native
`clickable_ranges` hit testing for pointing-hand feedback. When a paragraph has
listener-bearing runs, the renderer skips only the paragraph node's cursor style
so the native range decision owns the cursor; paragraphs without interactive
runs retain the existing node-level cursor behavior. Listener identity, mouse
range activation, keyboard focus/Enter activation, and painted focus affordances
remain unchanged.

Evidence: `crates/react-gpui/src/renderer/paint/text_input.rs` selects
`apply_style_without_cursor` only when `clickable_ranges` is non-empty;
`crates/react-gpui/src/renderer/paint/style.rs` preserves all other style fields.
Focused tests cover the cursor decision and real headless frame mouse press
dispatch (`interactive_text_run_mouse_press_still_dispatches`), alongside the
existing keyboard/focus and affordance regressions.

## Acceptance

- Establish a reliable pointer-dispatch/display-backed test that distinguishes
  pointer movement over a link run, a non-link run, whitespace, and wrapped
  lines within the same paragraph.
- When the pointer is over a listener-bearing run, the native cursor is the
  pointing hand; leaving that run restores the applicable surrounding cursor.
  Non-link text and unrelated nodes must not inherit the link cursor.
- Preserve byte-range hit testing, mouse press identity, keyboard activation,
  focus/blur events, and the existing per-line focus affordance across wrapped
  paragraphs and rerenders.
- Use a pinned GPUI-supported range/hitbox mechanism or document a reviewed
  upstream API requirement; do not claim per-glyph cursor precision from a
  parent-level hitbox or ship a branch without real dispatch evidence.

## Comments

The current parent-hitbox behavior is documented honestly in the link keyboard
spec and consumer README. This issue is separate from the completed keyboard
focus visibility affordance and does not request a new wire event or callback.
