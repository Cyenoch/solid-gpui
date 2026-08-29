# 01 — Per-run pointing cursor for interactive Text

Status: needs-triage
Type: task

Interactive nested `Text` runs currently use the parent paragraph's
`InteractiveText` hitbox for pointing-hand feedback. The link runs retain
correct listener identity, mouse range activation, keyboard focus, and painted
focus affordances, but the cursor cannot be scoped to only the link glyph range
with the pinned GPUI surface. An attempted run-scoped implementation did not
pass the real headless pointer-dispatch test and was removed rather than
shipping an unverified cursor state machine.

Evidence: `.scratch/link-affordance/summary.txt` and
`.scratch/link-keyboard/spec.md:92-105` (cursor boundary and attempted
implementation outcome).

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
