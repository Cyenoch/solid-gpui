# Keyboard-accessible interactive Text runs

Status: approved for implementation after the release-readiness gate (`0f1983d`).

## Scope

A nested `Text` run with `onPress` is a real keyboard focus target when it is
mounted at the supported one-level nesting depth. It participates in the same
native GPUI tab-stop graph as other React focus targets, emits the existing
focus/blur events for its own listener, and activates with Enter. This minimal
slice does not paint a per-run focus affordance; focus visibility remains a
documented boundary. Space is not an activation key for links: links
conventionally activate with Enter, while Space retains its normal scrolling
behavior. A Text without `onPress` does not acquire a focus handle or tab stop.
Existing rejection of Text nesting at depth 2 stays unchanged.

## Pinned API evidence

The workspace pins GPUI `0.2.2` to Zed revision
`6805d952f9f3d702f760aa11b1547df8a625fa16` (`Cargo.toml:16-24`; the same source
is vendored under `references/zed`).

- A `FocusHandle` stores `tab_index` and `tab_stop`, and
  `FocusHandle::tab_stop(bool)` updates both the handle and the shared focus
  map (`references/zed/crates/gpui/src/window.rs:525-587`). A handle is created
  from `App::focus_handle()` (`references/zed/crates/gpui/src/app.rs:2680-2683`).
  Therefore non-Input elements need an explicit handle retained by the host and
  `.tab_stop(true)`; `.focusable()` alone is not enough.
- GPUI's `InteractiveElement::track_focus` marks an element focusable and
  records the supplied handle; `.tab_stop` controls keyboard reachability
  (`references/zed/crates/gpui/src/elements/div.rs:749-777`). During request
  layout it creates a state-backed handle when needed
  (`.../div.rs:2141-2159`). During prepaint it calls
  `Window::set_focus_handle`, registers the AccessKit focus mapping, and sets
  the focused AX node (`.../div.rs:2216-2234`). During paint it inserts the
  handle into `Window::next_frame.tab_stops` (`.../div.rs:2417-2439`).
- The same pinned interactive path provides focus styling via
  `.focus(...)`/`.focus_visible(...)` (`references/zed/crates/gpui/src/elements/div.rs:1211-1241`),
  but this minimal host slice intentionally leaves a visible per-run affordance
  as a follow-up boundary.
- GPUI's tab graph orders entries by tab-group path and insertion index
  (`references/zed/crates/gpui/src/tab_stop.rs:77-90`); `next` and `prev` skip
  entries whose `tab_stop` is false and wrap when they reach either end
  (`.../tab_stop.rs:111-170`). `Window::focus_next` and `focus_prev` consume
  exactly that rendered-frame graph (`.../window.rs:2092-2112`).
- GPUI's generic click implementation records Enter/Space keydown and emits a
  keyboard click on the matching keyup only if focus generation is unchanged
  (`references/zed/crates/gpui/src/elements/div.rs:2887-2947`). This is the
  precedent for safe key synthesis, but `InteractiveText` itself is only a
  mouse range helper and has no focus/keyboard path
  (`references/zed/crates/gpui/src/elements/text.rs:980-1037,1122-1269`).
  The host adds a focused run key listener using the existing event/listener
  path; it synthesizes the existing `Event::press` for Enter keydown (not a new
  wire event) and guards the current focus handle.
- Key handlers are attached to the focused element's GPUI dispatch path through
  `Window::on_key_event` (`references/zed/crates/gpui/src/window.rs:4870-4881`),
  and `KeyDownEvent` exposes the normalized keystroke and held state
  (`references/zed/crates/gpui/src/interactive.rs:23-43`). The React host's
  existing View/Pressable key path forwards those events through
  `emit_key_event` (`crates/react-gpui/src/renderer/paint/mod.rs:227-280`);
  link activation uses the same native listener target but does not require a
  TypeScript key event.
- Focus observers are already installed with `Context::on_focus`/`on_blur` for
  eligible React handles (`crates/react-gpui/src/renderer.rs:450-483`). GPUI's
  AccessKit mapping requires `set_focusable` before `set_focus`
  (`references/zed/crates/gpui/src/window/a11y.rs:219-245`), which the tracked
  focus element performs in prepaint. The nested Text run must therefore be a
  real focus-tracked element with an ID/role, not merely a synthetic range.

## Focus model and wire behavior

The retained protocol already allows a listener on `Text` and carries the
existing `focusable` slot plus `listenerId`; `TextChild` already exposes
`onPress` (`packages/react-gpui/src/renderer/types.ts:237-244`). No TypeScript
API or new wire field is needed. The host derives a run's focus eligibility
from `kind=Text` + nested-under-Text + nonzero listener. The parent Text remains
an ordinary paragraph container; only listener-bearing nested runs get handles.

The Rust retained-tree validation changes the focusable kind predicate from
`View | Pressable` to `View | Pressable | Text`, but additionally requires a
nonzero listener for focused Text. TypeScript continues to encode the listener
for nested Text. The existing depth check in
`crates/react-gpui/src/tree/validation.rs:268-290` rejects a Text child under a
nested Text parent, so depth 2 remains invalid.

The renderer retains one `FocusHandle` per eligible run node ID and reconciles
its tab-stop state alongside other focus handles. Focus observers include these
Text IDs and emit existing null-payload `EVENT_FOCUS`/`EVENT_BLUR` notifications
with the run's listener ID. Tab order is GPUI's rendered tree order, not a
second host-managed order.

## Activation and known boundary

- **Activation:** Enter only, on an unmodified, non-held keydown while the run's
  handle is focused. It emits `Event::press` with that run's node/listener IDs,
  exactly the same event consumed by the existing TypeScript `onPress` path.
  Space is deliberately not synthesized for `accessibilityRole="link"`; this
  avoids hijacking the conventional scroll key and matches link keyboard
  behavior. Mouse activation remains `InteractiveText` range hit-testing.
- **Focus visibility boundary:** This minimal slice does not add a painted
  per-run focus outline or underline. A future affordance must use shaped
  per-line range geometry without changing the public style wire.
- **Hover cursor:** keep the existing node-level/InteractiveText per-range
  pointing-hand behavior. A per-run hover callback is not required for the
  keyboard contract and would add another range tracking state machine.

## Tests and acceptance

Focused proof must cover: Tab from a preceding View reaches the listener-bearing
nested Text and emits its focus event; Enter invokes only that run's press; Text
without `onPress` does not create a handle/tab stop; depth-2 nested Text remains
rejected. Focus visibility is the documented boundary above. Existing mouse
range activation, selectable Text behavior, and generic View/Pressable focus
behavior remain intact.
