# Overlay outside-pointer dismissal

Status: resolved
Type: feature

The dropdown overlay currently closes only from its trigger or Escape. GPUI exposes a paint-time global `Window::on_mouse_event` observer with capture-before-bubble ordering (`references/zed/crates/gpui/src/window.rs:4847-4858,5181-5218`), and deferred anchored children receive offset bounds during prepaint (`references/zed/crates/gpui/src/elements/anchored.rs:122-215`).

Design: `position: "overlay"` View or Pressable nodes with `onPointerDownOutside` retain their rendered bounds in a small paint wrapper. Their direct anchor subtree records a rendered-bounds marker. A capture observer emits Event 22 (`[8,x,y]`) only when the pointer-down is outside both regions; inside overlay downs remain ordinary pointer events. TypeScript exposes the callback on both host kinds, and Escape remains keyboard-owned.

Rejected: a full-window invisible scrim Pressable is a hit-testing workaround that changes pointer target semantics and cannot model nested overlays.

## Comments

The pre-existing protocol hunk was an orphaned partial change. It was confirmed by `git diff` and reverted before this implementation rebuilt Event 22.
Implementation: overlay components use the callback to close only on true outside downs. Rust protocol coverage checks Event 22 round trips and malformed-target guards.
