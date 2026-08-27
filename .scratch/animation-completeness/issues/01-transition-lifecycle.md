# Transition lifecycle semantics

Status: resolved

## Question

Do style removals and mid-flight target changes preserve the consumer-visible
transition behavior promised by the public surface?

## Answer

Yes for the existing scalar properties. A changed style that omits `transition`
now reuses the previous transition metadata, so opacity removal animates toward
its implicit `1.0` target and background removal animates toward the transparent
variant of its previous color. A retarget samples the current presentation value
before starting a new generation; a later retarget while the transition field is
still omitted reuses the active state's timing and property mask. Delay is held
from each retarget timestamp before easing begins. Numeric width/height remain
numeric-to-numeric only; an unset/`auto` endpoint changes immediately because
the wire has no numeric target.

Evidence: `crates/react-gpui/src/renderer/animation.rs` (`retarget_with_transition`,
`reconcile_animation_states`, and `retarget_existing_animation`), with behavior
coverage in `crates/react-gpui/src/renderer.rs`.
