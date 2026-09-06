# Live-region semantics

Status: resolved
Type: research

Determine whether the pinned AccessKit/GPUI seam can carry live-region announcements for asynchronous status changes.

## Answer

Upstream gap. AccessKit 0.24.1 defines `Live::{Off,Polite,Assertive}` and the `Live` node property (`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/accesskit-0.24.1/src/lib.rs:569-588,2117-2142`). Pinned GPUI's `AriaProperties` has no live field (`references/zed/crates/gpui/src/elements/div.rs:1997-2020`), `StatefulInteractiveElement` has no public `aria_live`/live-region builder (`references/zed/crates/gpui/src/elements/div.rs:1246-1452`), and `Interactivity::write_a11y_info` has no `Node::set_live` call (`references/zed/crates/gpui/src/elements/div.rs:3392-3455`). Do not add a renderer wire field or provide a false fallback. Asynchronous application status remains invisible to screen readers until GPUI exposes this seam.

## Comments

The stock headless TestPlatform also cannot activate or inspect the AccessKit tree; a future upstream implementation needs a desktop-adapter verification pass.
