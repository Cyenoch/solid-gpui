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
- **Outbound file drag — implemented as a constrained project feature.** The
  pinned GPUI `ExternalDragPayload` has only the `Files(FileDragPaths)`
  variant; text/string dragging is therefore not claimed. The renderer accepts
  up to eight validated host-local paths, probes directory metadata lazily when
  the drag leaves the viewport, and offers the native payload. Platform start
  success has no JavaScript completion/error event: macOS and Wayland Linux
  implement native starts, while X11 and Windows retain the default decline.
- **Keybinding registration — feasible-bounded, tracked for a dedicated
  round.** The pinned GPUI `App::bind_keys`, `clear_key_bindings`, and
  `KeyBinding`/`KeyContext` APIs make host-held registration possible, and the
  existing `EVENT_ACTION` channel can carry a matched action name. The open
  design is semantic rather than an upstream absence: root bindings versus
  menu shortcuts, action registration/building, context predicates, conflict
  precedence, and per-surface cleanup must be specified before exposing
  `Root.bindKey`. It is therefore a bounded candidate, not a promise to
  forward global keymap mutation directly.
- **`pointerEvents` — disposition: do not add a declarative prop.** GPUI's
  `Interactivity::should_insert_hitbox` only creates a hitbox when there is
  interactivity (`references/zed/crates/gpui/src/elements/div.rs:2292-2316`);
  with no listener, an overlay has no hitbox and basic pass-through is already
  native behavior. The default `HitboxBehavior::Normal` also does not occlude
  underlying hitboxes. A proposed `none` would only suppress this node's
  callbacks; the single wire `listenerId` means unrelated callbacks such as
  `onLayout` can still leave native pointer registration ambiguous. It cannot
  express “listener present but pass through” or partial overlay occlusion, so
  presenting it as CSS-like coverage would be a semantic false positive.
  Keep the boundary explicit and leave the richer policy tracked.

## Test-platform limitations

- **`zoom` — disposition: keep D-level test-platform coverage.** The pinned
  GPUI `TestWindow::zoom` is unimplemented, while the production command path
  and protocol handling are already covered at B level. A host trait/call-count
  seam would prove only that `zoom_window()` was invoked, not display-backed
  native zoom behavior. Keep the honest limitation: verify the toggle on a
  real desktop adapter rather than adding a low-value seam.

## Current review outcome

`boxShadow`, `fontFamily`, image `fallbackSource`, and the scale-factor
observation are now implemented and documented. Image `onError`, the true
upstream gaps, and the remaining pointer-events policy stay explicitly listed
so future work does not silently turn a policy choice into a claimed GPUI
limitation.
