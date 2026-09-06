# Rich-text interactions

## Scope

Lift selectable rich text now that GPUI shapes one contiguous UTF-8 paragraph from ordered `TextRun`s; add pressable nested `Text` runs without a protocol variant. A nested run owns its existing listener ID, pointer hit-testing maps the parent paragraph's byte index to that run, and the native event keeps the run node/listener identity. Link runs use the pointing-hand cursor at the containing Text hit region because pinned GPUI exposes node-level cursor/hitboxes, not per-glyph cursor regions.

## Selectable-lift experiment

The headless GPUI draw path will render one selectable Text paragraph with multiple nested styled runs, drag across a run boundary, inspect the retained `TextInputTextLayout` selection geometry, and copy the selected UTF-8 range. `shape_text` accepts ordered runs whose lengths partition the same string; `position_for_utf8`, `selection_bounds_per_line`, and copy use paragraph byte offsets, so the implementation must preserve the same global offsets while painting each run.

## Link model

`TextProps.onPress` is valid for Text nodes. A nested run with non-empty flattened text registers its listener. The parent Text's rich painter maps pointer-down/up positions to the containing byte range and emits the run's press event. Keyboard activation is delivered through the rendered Text focus stop and emits the focused run's press event for Enter/Space. Accessibility role `link` is encoded as the existing accessibility metadata role and mapped to pinned AccessKit `Role::Link`; unsupported role support is a hard validation error, not a fallback.

Deeper Text nesting remains rejected. A press handler on a run with no text is rejected with an actionable error. No wire shape changes are required: listener IDs and accessibility tuples already exist.
