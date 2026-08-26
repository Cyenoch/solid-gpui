# Upstream dependency audit

This is an internal release-productionization tracking note. It records the
boundary evidence used for the current renderer review; it is not a roadmap or
an upstream commitment.

## True upstream gaps

These items require an upstream GPUI API/semantic change rather than another
wire slot in this repository:

- **RTL base direction and bidi caret/IME semantics.** The pinned GPUI surface
  has physical flex direction and physical text alignment, but no generic
  container/text base-direction contract. Unicode bidi shaping and caret/IME
  behavior therefore cannot be made coherent by a renderer-local mirror.
  Evidence: `references/zed/crates/gpui/src/styled.rs` (physical flex and text
  alignment methods), `references/zed/crates/gpui/src/style.rs` (style fields),
  and the current boundary statement in `docs/protocol.md`.
- **`letterSpacing`.** The pinned GPUI text style and shaping seam do not expose
  a letter-spacing field or shaping adjustment that the renderer can safely
  apply to every text run. Adding a tuple value without a GPUI shaping API
  would be a visual approximation, not the requested semantic.
- **Selectable `Text`.** GPUI's text element can shape and paint text, but the
  pinned public element/input contracts do not provide a selection model and
  selection event stream for a generic `Text` node. The renderer's selection
  command is intentionally scoped to native `TextInput` state.
- **Secure-input semantics.** The desktop GPUI input surface has no password
  obscuring/secure-entry primitive or keyboard-layout semantic. A renderer
  flag would not provide a secure editing contract, so `secureTextEntry` and
  `keyboardType` remain unsupported.

## Project limits that can be re-reviewed

These are not upstream blockers. They are renderer protocol and API choices,
so each can be revisited with a bounded implementation and bilateral tests:

- **`boxShadow` — implemented in this review.** GPUI already exposes
  `Style::box_shadow`, `BoxShadow`, the `box_shadow_style_methods!` macro, and
  `Window::paint_drop_shadows`/`paint_inset_shadows`. Zed uses the same native
  `.shadow(vec![BoxShadow::new(...)])` path. The renderer now carries one
  tagged tail slot (`tag=1` for one shadow, `tag=2` for two), validates finite
  offsets/non-negative blur and spread, maps RGBA/inset, and exercises single,
  double, and invalid vectors.
- **`fontFamily` — implemented in this review.** GPUI's `Styled::font_family`
  accepts `SharedString`; `TextSystem::resolve_font` first attempts the
  requested family and then walks the configured fallback stack. The renderer
  now carries a validated string tail slot (non-empty, at most 64 Unicode
  characters, no controls) and maps it to `font_family(SharedString)`. The
  previous statement that fallback was not guaranteed was too conservative for
  this pinned GPUI contract; the renderer relies on GPUI's documented fallback
  behavior.
- **`scaleFactor` — implemented in this review as an observation.** GPUI
  exposes `Window::scale_factor()` and the platform resize callback carries
  the scale. The renderer now batches that value with size/activation/
  appearance observation, emits a three-number resize payload, reports
  scale-only changes, and keeps legacy two-number payloads at `1`. The
  TypeScript callback and `WindowSize` store expose the optional/normalized
  value without changing logical-pixel dimensions.
- **Image fallback — implemented in this review; `onError` remains upstream.**
  GPUI's `StyledImage::with_loading` and `with_fallback` APIs accept native
  fallback elements for both loading and error states. The renderer now adds a
  validated optional `fallbackSource` to the Image host tuple and wires both
  states to that path. There is still no loader notification hook, so
  JavaScript `onError` remains a true upstream gap; this visual fallback covers
  the user-visible degradation path without inventing an event.
- **`pointerEvents`.** This is a hit-testing policy, not a CSS string that can
  be forwarded blindly. GPUI's interactive elements decide hit testing from
  their event handlers and element state; the current renderer only installs
  handlers for declared callbacks and preserves subtree ordering. A future
  policy can define `auto`/`none` (and ancestor/descendant precedence), but it
  must specify whether layout, hover, drag, and accessibility participation
  change together. No upstream API gap is implied.

## Test-platform limitations

- **`zoom`.** GPUI exposes the command/window path, but stock headless and
  `TestAppContext` runs do not provide a desktop compositor or reliable visual
  scale verification. Zoom command round-trips and host handling are covered;
  final visual scale remains an adapter/desktop verification concern rather
  than evidence of a renderer or upstream gap.

## Current review outcome

`boxShadow`, `fontFamily`, image `fallbackSource`, and the scale-factor
observation are now implemented and documented. Image `onError`, the true
upstream gaps, and the remaining pointer-events policy stay explicitly listed
so future work does not silently turn a policy choice into a claimed GPUI
limitation.
