# ADR-0013: Model interactive text as one shaped paragraph with clickable runs

- **Status:** Accepted
- **Date:** 2026-08-30

## Context

`Text` needs to support mixed raw and styled content, selectable paragraphs, and
links without turning every inline child into a separate native layout object.
The public tree already has `Text`/`RawText` nodes, listener IDs,
accessibility metadata, focus handles, and the existing press event. The pinned
GPUI APIs shape a paragraph from one string and an ordered `TextRun` list; they
do not provide a per-run font size or a per-glyph native hitbox. The resulting
boundary must therefore keep paragraph shaping, byte ranges, focus, selection,
and cursor behavior aligned.

## Decision

### Paragraph and run model

A rich-text paragraph is one `Text` node whose direct children may be raw
strings (`RawText`) and at most one level of nested `Text` runs. A nested run
may contain bare strings only; a `Text` child of a nested run is rejected, so
there are no grandchildren `Text` runs. The retained tree recomputes one
flattened UTF-8 `text_content` in child order. These rules are enforced by
`validate_nested_text_edge` and `validate_parent_child_kinds`
(`crates/react-gpui/src/tree/validation.rs:274-296,369-404`) and are part of
the current README contract (`README.md:49-63`).

A run may override exactly this run-level style set:
`color`, `fontWeight`, `fontStyle`, `textDecoration`, and `fontFamily`. Layout
and container properties are rejected for nested runs, including `fontSize` and
`lineHeight` (`crates/react-gpui/src/tree/validation.rs:298-368`). The pinned
GPUI `TextRun` contains a byte length, font, color, background, underline, and
strikethrough, but no size or line-height field
(`references/zed/crates/gpui/src/text_system.rs:985-1000`). Both
`shape_line` and `shape_text` receive one paragraph-level `font_size`
(`references/zed/crates/gpui/src/text_system.rs:391-403,506-516`). Therefore
per-run `fontSize`/`lineHeight` is rejected by design rather than ignored or
approximated; the outer paragraph supplies those values.

The nested `Text` remains the ordinary `kind=Text` node and the wire shape is
unchanged. Existing listener IDs and accessibility tuples carry run identity;
no inline node kind or protocol variant is added.

### Interactive runs and keyboard activation

A nested `Text` with `onPress` receives the existing listener allocation and
keeps its own node/listener identity. Only non-empty flattened child content
contributes a clickable range. The painter concatenates children into one
string, records each listener-bearing run as a UTF-8 byte range, and passes
those ranges to pinned `InteractiveText::on_click`
(`crates/react-gpui/src/renderer/paint/text_input.rs:29-65,893-918`; listener
allocation is in `packages/react-gpui/src/renderer/nodes.ts:282-307`). The
parent `InteractiveText` maps its mouse down/up character indexes back to the
same range and dispatches the existing `Event::press`; it does not create a
new wire event (`references/zed/crates/gpui/src/elements/text.rs:1020-1037,1138-1168`).
A link run uses the existing `accessibilityRole="link"` metadata and AccessKit
role mapping.

Each listener-bearing run is also a real native focus target. The host retains
one `FocusHandle` per eligible run and mirrors the ordinary interactive-node
path with `.focusable()`, `.tab_stop(true)`, and `.track_focus(...)`; focus and
blur use the existing events and listener identity
(`crates/react-gpui/src/renderer/paint/text_input.rs:938-959`,
`crates/react-gpui/src/renderer/paint/mod.rs:229-250`). This is required by the
pinned GPUI focus API: tracking a handle makes an element focusable, but
`.tab_stop(true)` is what makes it keyboard-reachable, and rendered tracked
handles are inserted into the native tab-stop graph
(`references/zed/crates/gpui/src/elements/div.rs:749-777,2216-2235,2417-2439`).

Keyboard activation is a bounded accessibility-equivalent press path: only
when the run's handle is focused, the key is an unmodified `Enter`, and the
keydown is not held/repeated, the host emits that run's existing `Event::press`
(`crates/react-gpui/src/renderer/paint/text_input.rs:960-976`). `Space` is not
synthesized; it retains normal scrolling behavior for links. Modified Enter,
held/repeated Enter, an unfocused handle, a non-interactive `Text`, and every
other key are outside this activation path. This is keyboard activation of a
focused link, not fabrication of a pointer action, and is the scoped exception
to ADR-0005's Pressable rule.

### Cursor precedence

The paragraph's node-level cursor style is applied normally when it has no
clickable ranges. If and only if clickable ranges exist, the renderer omits
that node-level cursor (`rich_text_uses_run_cursor` and
`apply_style_without_cursor`, `crates/react-gpui/src/renderer/paint/text_input.rs:864-882`).
Pinned `InteractiveText` then owns the pointing-hand decision for the current
range inside its parent hitbox (`references/zed/crates/gpui/src/elements/text.rs:1138-1147`).
This deliberately does not claim separate per-glyph hitboxes: range hit
identification and cursor choice remain the pinned parent `InteractiveText`
behavior. Paragraphs without clickable ranges retain their configured cursor.

### Selection and focus affordance geometry

Selectable rich text uses the same flattened paragraph string and ordered runs
as ordinary rich text. The host keeps selection and drag state locally under
the host-owned input model; it does not add JavaScript selection state or wire
selection events. `shape_text` produces the shared wrapped layout, and
`TextInputTextLayout::selection_bounds_per_line`/`bounds_for_range` calculate
geometry from global UTF-8 offsets without inspecting which run supplied those
bytes (`crates/react-gpui/src/renderer/paint/text_input.rs:983-1014`,
`crates/react-gpui/src/renderer/input.rs:323-427`). This extends the selectable
Text ownership decision in ADR-0012: selection/copy remains visual and native,
while run styles and link ranges share the paragraph geometry.

When a listener-bearing run is focused, the host paints one opaque, high-
contrast 1px quad for each wrapped-line segment of that run's range. The quads
are derived from the same selection geometry during prepaint and painted
without mutating the public style or wire contract
(`crates/react-gpui/src/renderer/paint/text_input.rs:187-219`). They are focus
visibility affordances, not selection state and not pointer hitboxes.

### Assembly cache and shaping boundary

The renderer caches assembled rich parts (`text`, ordered `TextRun`s, clickable
ranges, and targets) by `Text` node ID. A replacement Snapshot clears the
cache. A successful Patch invalidates touched nodes, their post-patch content
ancestors, pre-patch ancestors affected by moves/deletes, and deleted entries;
unrelated rich paragraphs remain cached
(`crates/react-gpui/src/renderer.rs:248-360`). This is the targeted invalidation
contract established by the rich-run assembly work (`598448a`, `704beff`) and
measured in `.scratch/rich-perf/measurement.md:26-42` and
`.scratch/rich-perf/issues/01-targeted-invalidation.md:30-44`.

We do not add a ReactRoot-level shaped-layout cache. The split benchmark found
that the residual `shape_text` request was about 11–13% of the frame, below the
30% savings threshold, while pinned GPUI already reuses platform-shaped lines
through `LineLayoutCache` keyed by text, font size, font runs, wrap width, and
force width. The final wontfix decision is recorded by `d885f32` and in
`.scratch/rich-perf/measurement.md:44-61`,
`.scratch/rich-perf/issues/02-shaped-text-cache.md:31-70`, and
`references/zed/crates/gpui/src/text_system/line_layout.rs:574-636`. A future
shaped cache would have to account for width, scale, font registration, run
identity, line height, wrapping, hit testing, selection, activation, and
affordance geometry before it could be correct; the current measured boundary
does not justify that new ownership and invalidation surface.

## Pinned API constraints

The workspace pins GPUI 0.2.2 to Zed revision
`6805d952f9f3d702f760aa11b1547df8a625fa16` (the evidence is recorded in
`.scratch/link-keyboard/spec.md:17-21`). The following are implementation
constraints until that pin changes:

- `TextRun` has no per-run size; `shape_line`/`shape_text` take one global
  paragraph `font_size` (`references/zed/crates/gpui/src/text_system.rs:391-403,506-516,985-1000`).
- `InteractiveText::on_click` accepts byte-index ranges and dispatches through
  one parent hitbox; its cursor code sets `PointingHand` when the shaped index
  is in a clickable range (`references/zed/crates/gpui/src/elements/text.rs:1020-1037,1138-1168`).
- Focus requires a retained `FocusHandle`, tracked focus, and explicit tab-stop
  registration; `.focusable()` alone does not add a traversal stop
  (`references/zed/crates/gpui/src/elements/div.rs:749-777,2216-2235,2417-2439`).
- Range geometry must use the shared shaped layout and global UTF-8 offsets;
  `selection_bounds_per_line` is the common multiline geometry path
  (`crates/react-gpui/src/renderer/input.rs:323-427`).

Changing any of these constraints, or introducing a new wire-visible text
shape, requires source evidence and a new decision rather than a compatibility
branch in the current model.

## Alternatives rejected

- **Represent every run as a separate native element:** rejected because it
  would split one paragraph's shaping and wrapping, require separate hitboxes
  and geometry reconciliation, and make selection/copy across run boundaries
  a second model. The pinned contiguous `shape_text` path already accepts
  ordered runs without that split.
- **Permit `fontSize` or `lineHeight` on a nested run:** rejected because the
  pinned `TextRun` cannot carry either value and the shaping APIs use one global
  size; silently dropping a requested style would make the API dishonest.
  Splitting the paragraph solely to simulate sizes would violate the paragraph,
  range, and shared-geometry contract.
- **Mirror rich selection in JavaScript or add selection wire events:** rejected
  by ADR-0012 because asynchronous duplicate state would race native geometry
  and controlled rendering, while selectable rich text has no consumer-facing
  transient selection event.
- **Give each run an independent cursor hitbox or cursor callback:** rejected
  because pinned `InteractiveText` already provides the bounded range decision
  and does not expose per-glyph cursor regions; extra elements would claim a
  precision the API cannot guarantee. The parent cursor is therefore suppressed
  only when ranges exist, allowing the pinned range decision to own it.
- **Cache shaped layouts by text or run array alone:** rejected because width,
  scale, font resolution, line height, wrapping, and range geometry are part of
  correctness, and the pinned `LineLayoutCache` already covers the expensive
  platform layout boundary. The measured residual is below the agreed savings
  threshold.

## Consequences

- Rich text remains a single ordinary Text node on the wire. Existing listener,
  accessibility, focus, press, and selectable fields are reused; no new event,
  node kind, style slot, or selection protocol is introduced.
- A nested interactive run is both a mouse range target and a native tab stop.
  Its focus/blur and Enter press retain the run's node/listener identity, while
  Space remains non-activating.
- Selection, link activation, cursor choice, and focus affordances depend on
  one flattened UTF-8 paragraph and one shaped layout. Changes to run content or
  style must invalidate the paragraph assembly and preserve the shared geometry
  boundary.
- The host owns transient focus affordance and selectable-range state; React
  continues to own content, callbacks, and controlled rendering. Unsupported
  deeper nesting and layout-affecting nested styles remain explicit validation
  errors rather than degraded rendering.
