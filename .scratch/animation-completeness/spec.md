# Animation completeness audit

## Scope

This round keeps the existing transition wire tuple and the four supported
properties (`opacity`, `backgroundColor`, `width`, and `height`). It fixes
observable transition lifecycle semantics without introducing a new per-frame
style mutation surface for properties that GPUI does not expose as a native
transition primitive.

## Evidence audit

| Candidate | Supported by a pinned GPUI transition primitive? | Decision and evidence |
| --- | --- | --- |
| `opacity` | Yes, as a scalar style value; GPUI's generic `AnimationExt::with_animation` accepts an arbitrary animator callback (`references/zed/crates/gpui/src/elements/animation.rs:75-99`), and GPUI's style model has `opacity` (`references/zed/crates/gpui/src/style.rs:298-303`). | Keep the existing wire bit (`crates/react-gpui/src/protocol.rs:82-85`) and native frame sampling. |
| `backgroundColor` | Yes for scalar color interpolation, but no dedicated transition path. Pinned GPUI's `Interpolate` supports both straight-channel `Rgba` and hue-aware `Hsla` (`references/zed/crates/gpui/src/spring.rs:407-452`); no `Hsla::lerp` or `TransitionPath` symbol exists in the pinned `gpui` source. | Keep the existing RGBA wire value and document that this renderer uses straight RGBA/sRGB-channel interpolation (`crates/react-gpui/src/renderer/animation.rs:225-234`). Do not grow the wire for a color-space selector. |
| `width` / `height` | GPUI's generic animation wrapper can animate any callback, and its layout engine accepts definite sizes (`references/zed/crates/gpui/src/elements/animation.rs:82-99`, `references/zed/crates/gpui/src/style.rs:229-237`), but there is no style-transition primitive. | Keep the existing bits. Native frame styles intentionally rewrite dimensions and reflow layout (`crates/react-gpui/src/renderer/animation.rs:352-379`). Numeric-to-numeric transitions are supported; an unset (`auto`) endpoint has no numeric target in the protocol and therefore remains an immediate change. |
| `borderRadius` | No dedicated GPUI transition primitive. GPUI stores corner radii in `StyleRefinement` (`references/zed/crates/gpui/src/style.rs:278-290`) and resolves/paints them as part of each style (`references/zed/crates/gpui/src/style.rs:705-754`). The only available route here would be per-frame style mutation, which is the cost this renderer deliberately avoids. | Keep unsupported. Do not add a transition bit or fake a `TransitionPath`. |
| `transform: scale/translate` | No generic layout-element transform primitive. Pinned `Transformation` exists under the SVG element and explicitly affects rendering only, not layout/hitbox (`references/zed/crates/gpui/src/elements/svg.rs:64-67`, `212-282`). Generic `AnimationExt` is only a callback wrapper, not a transform contract (`references/zed/crates/gpui/src/elements/animation.rs:82-99`). | Keep unsupported. A generic transform needs a new layout, hit-test, and painting contract; no wire growth this round. |

## Existing lifecycle findings

- The public TS surface already encodes `durationMs`, optional `delayMs`, optional
  `easing`, and an optional property list (`packages/react-gpui/src/style.ts:41-50`,
  `400-433`, `508-519`). The tuple is `[durationMs, delayMs, easing, propertyMask]`
  (`packages/react-gpui/src/style.ts:591-610`), so no wire growth is required.
- Easing is applied only after delay. `progress` returns zero while delayed and
  applies the selected curve to normalized active time (`crates/react-gpui/src/renderer/animation.rs:123-137`); the four curves are the standard
  linear/quadratic ease-in/quadratic ease-out/quadratic ease-in-out forms
  (`crates/react-gpui/src/renderer/animation.rs:210-223`).
- Retargeting already samples `values(now, false)` before replacing the target
  (`crates/react-gpui/src/renderer/animation.rs:58-120`), so a value change during
  flight starts from the sampled presentation value rather than the old origin.
  A fresh generation is assigned on every retarget (`:115-120`). Regression tests
  will lock this down for all existing value categories through the focused
  animation matrix.
- Property removal has an inconsistent lifecycle. Opacity removal maps the
  missing value to its default `1.0` (`:65-76`), and background removal maps a
  missing color through a transparent variant (`:150-163`, `:235-237`). In
  contrast, width/height require `from.zip(target)` (`:196-207`) and therefore
  snap when an endpoint is unset. More importantly, a commit that removes the
  `transition` object takes the `style.transition.is_none()` fast path and resets
  to target (`:263-270`); a common style composition that removes a hover/drag
  style can therefore snap even for opacity/background. This round preserves the
  prior transition configuration for a target change when the new style omits
  `transition`, allowing supported scalar properties to animate back. Numeric
  width/height-to-unset remains immediate because there is no numeric auto target.
- Delay is restartable per retarget: a new transition starts its declared delay
  from the retarget timestamp (`:115-118`). During that delay the sampled current
  value is held (`:130-136`), then easing begins from that value. This is explicit
  and tested rather than accidentally inheriting the old transition's clock.
- Completion is one event per generation (`crates/react-gpui/src/renderer/animation.rs:296-332`), and deleted nodes are pruned (`:240-245`).

## Bounded implementation

1. Preserve the previous transition metadata as the reverse/exit transition
   when a changed style omits `transition`; do not add a wire slot.
2. Keep retarget sampling and generation behavior, and add focused tests for
   supported property removal, retarget continuity, delay/easing, and the
   existing width/height path.
3. Update the protocol and package docs with exact supported properties,
   removal/retarget/delay semantics, straight RGBA interpolation, and the
   evidence-backed transform/border-radius exclusions.
4. Add a changelog entry under Fixed. The gallery remains unchanged.
