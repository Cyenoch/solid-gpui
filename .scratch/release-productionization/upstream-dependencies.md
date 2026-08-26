# Upstream dependency audit

This is an internal release-productionization tracking note. It records the
boundary evidence used for the current renderer review; it is not a roadmap or
an upstream commitment.

## True upstream gaps

These items require an upstream GPUI API/semantic change rather than another
wire slot in this repository:

- **RTL explicit base-direction API.** The pinned GPUI `TextStyle`, `TextRun`,
  and `shape_line`/`shape_text` surfaces carry no paragraph/base-direction
  field or parameter. `unicode-bidi` is invoked with `None`, so P2
  auto-detects the first strong L/R/AL character instead of accepting a
  caller-selected base direction.
  Evidence: `references/zed/crates/gpui/src/style.rs:435-483`,
  `references/zed/crates/gpui/src/text_system.rs:985-1000`,
  `references/zed/crates/gpui/src/text_system.rs:391-403`,
  `references/zed/crates/gpui/src/text_system.rs:506-516`,
  `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cosmic-text-0.19.0/src/shape.rs:1348-1359`,
  `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/unicode-bidi-0.3.18/src/lib.rs:377-399`.
- **Bidi-aware hit/caret geometry.** GPUI's `LineLayout` carries no `rtl`,
  bidi `level`, glyph width, or caret affinity; `index_for_x` and
  `x_for_index` scan raw positions and indices, and `WrappedLineLayout`
  delegates position mapping to those scans. cosmic-text itself supplies
  RTL edge-aware cursor geometry and mixed-BiDi handling, but the GPUI
  backend conversion keeps only glyph ID, position, and start index.
  Evidence: `references/zed/crates/gpui/src/text_system/line_layout.rs:14-54`,
  `references/zed/crates/gpui/src/text_system/line_layout.rs:56-114`,
  `references/zed/crates/gpui/src/text_system/line_layout.rs:342-447`,
  `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cosmic-text-0.19.0/src/buffer.rs:115-197`,
  `references/zed/crates/gpui_wgpu/src/cosmic_text_system.rs:658-711`.
- **`letterSpacing`.** The pinned GPUI text style and shaping seam do not expose
  a letter-spacing field or shaping adjustment that the renderer can safely
  apply to every text run. Adding a tuple value without a GPUI shaping API
  would be a visual approximation, not the requested semantic.
- **Secure-input semantics.** The desktop GPUI input surface has no password
  obscuring/secure-entry primitive or keyboard-layout semantic. A renderer
  flag would not provide a secure editing contract, so `secureTextEntry` and
  `keyboardType` remain unsupported.

The generic `Text` selection model is not supplied by GPUI, so it is not a
ready-made upstream feature; however, unlike the remaining hard gaps, the
renderer can build a bounded host-owned model from public primitives.

## Re-reviewed upstream capabilities

- **RTL rendering — not an upstream gap.** The pinned WGPU text backend uses
  `cosmic-text` 0.19.0; its `ShapeLine` invokes `unicode_bidi::BidiInfo`,
  builds bidi-level spans, applies visual reordering, and shapes each span
  with HarfRust RTL/LTR directions. It emits positioned glyphs for painting,
  so RTL rendering is supplied by the shaping backend rather than missing
  from GPUI.
  Evidence: `references/zed/crates/gpui_wgpu/Cargo.toml:18-34`,
  `references/zed/crates/gpui_wgpu/src/cosmic_text_system.rs:627-645`,
  `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cosmic-text-0.19.0/src/shape.rs:135-201`,
  `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cosmic-text-0.19.0/src/shape.rs:1303-1404`,
  `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cosmic-text-0.19.0/src/shape.rs:1460-1568`,
  `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cosmic-text-0.19.0/src/shape.rs:2763-3013`.

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
- **Selectable `Text` — implemented as a bounded project feature.** The pinned
  GPUI `TextRun` has `background_color`, `TextLayout` exposes
  `index_for_position`, `position_for_index`, and bounds/line-layout accessors,
  and Zed Markdown owns a `RenderedText` selection/copy model. The renderer now
  shapes selectable Text with `shape_text`/`WrappedLine`, keeps UTF-8 anchor/head
  ranges in Rust, paints per-visual-row selection quads, uses the I-beam cursor,
  and copies directly through the host clipboard. Selection is visual-only:
  there are no new JS events or selection wire messages. Text changes clamp
  ranges to UTF-8 boundaries and unmounts discard layouts, ranges, and focus
  handles. The display-backed behavior is covered by the GPUI test adapter;
  platform-specific native cursor/clipboard verification remains a desktop
  concern.
- **Window creation options — implemented as a bounded surface feature.**
  GPUI's creation-only `WindowOptions`/`WindowParams` expose `kind`,
  `is_resizable`, and `window_min_size`; `WindowBounds::centered` is already
  used by this host. The renderer preserves the old
  `[title,[width,height]]` OpenSurface payload and appends
  `[kind,resizable,minWidth,minHeight]` only when options are requested.
  `normal`, `floating`, and `dialog` map to GPUI `WindowKind`; `floating` is
  above-parent where supported, not a portable global always-on-top level.
  `resizable` and `minSize` are creation-time only. GPUI has no portable
  `maxSize`, generic window-level, or runtime setter, so those remain
  intentionally unsupported. The first host window stays host-owned centered
  `800×600` and has no JavaScript startup negotiation.
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

`boxShadow`, `fontFamily`, image `fallbackSource`, scale-factor observation,
and selectable `Text` are implemented and documented. Selectable `Text` uses
the bounded host-owned design rather than requiring an upstream selection API;
its display-backed interaction is covered, while platform-specific native
cursor/clipboard behavior remains a desktop-adapter concern. Image `onError`,
the true upstream gaps, and the remaining pointer-events policy stay
explicitly listed so future work does not silently turn a policy choice into a
claimed GPUI limitation. RTL rendering is not an upstream gap after the bidi
re-review; explicit base-direction control and bidi-aware hit/caret geometry
remain separate true gaps, with geometry the deeper interaction boundary.
