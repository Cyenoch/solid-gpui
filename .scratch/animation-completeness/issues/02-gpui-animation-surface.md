# GPUI animation primitive feasibility

Status: resolved

## Question

Can this round add `borderRadius` or generic scale/translate transitions without
an unsupported fallback or a new wire contract?

## Answer

No. Pinned GPUI's `AnimationExt::with_animation` is a generic callback wrapper,
not a property transition path. Its `Interpolate` implementations cover scalar
`f32`/`Pixels`, straight-channel `Rgba`, and hue-aware `Hsla`, but the renderer's
wire stores colors as RGBA u32 and currently uses straight RGBA-channel lerp.
There is no pinned `TransitionPath` or `Hsla::lerp` API.

`borderRadius` is resolved from `StyleRefinement.corner_radii` during paint; no
native transition primitive exists, so animating it would require per-frame style
mutation. Generic transforms are only available through the SVG element's
`Transformation`, which explicitly does not affect layout or hit testing. Both
remain unsupported with no new property-mask bits.

Evidence: `references/zed/crates/gpui/src/elements/animation.rs`,
`references/zed/crates/gpui/src/elements/svg.rs`,
`references/zed/crates/gpui/src/style.rs`, and
`references/zed/crates/gpui/src/spring.rs`.
