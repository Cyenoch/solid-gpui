# Interaction completeness

This work closes two user-facing gaps without adding style variants or scrims:

- Focusable View/Pressable nodes receive host focus/blur notifications using the existing event codes and nullable payload form. TextInput keeps its existing tag-1 payload and callback behavior. The Solid renderer maps the notifications to `onFocus`/`onBlur`; application state composes focus, hover, and active styles.
- Overlay nodes with `onPointerDownOutside` register a capture-phase GPUI mouse observer. The observer compares the pointer position with the rendered overlay bounds and the direct anchor subtree's rendered bounds; only a down outside both emits Event 22 with `[8,x,y]`. The normal target still receives its own pointer/press event. Escape remains JavaScript-owned.

Evidence: GPUI exposes `Context::on_focus`/`on_blur` subscriptions (`references/zed/crates/gpui/src/app/context.rs:545-617`) and `FocusHandle::is_focused` (`references/zed/crates/gpui/src/window.rs:496-607`). `Div::prepaint` registers focus handles and reports focus state (`references/zed/crates/gpui/src/elements/div.rs:2216-2234`), while `Window::on_mouse_event` is available during paint and runs capture before bubble (`references/zed/crates/gpui/src/window.rs:4847-4858,5181-5218`). Existing overlay placement is `deferred(anchored().position_mode(Local))` (`crates/solid-gpui/src/renderer/paint.rs:1127-1141`); its child receives final offset bounds during deferred prepaint (`references/zed/crates/gpui/src/elements/anchored.rs:122-215`).

Rejected alternatives: a `focusStyle` wire variant would duplicate the 42-slot style contract and make focus styling less composable than the existing hover state merge; faking focus with hover is incorrect for keyboard modality. A full-window invisible scrim Pressable would alter hit testing, steal pointer semantics, and fail to model nested overlays, so outside dismissal is a bounded host event instead.

Implementation note: an orphaned, malformed protocol.rs/event.rs hunk was found in the worktree before implementation. Its diff was confirmed and reverted to the clean baseline; this workstream now owns and rebuilds the event-22 and nullable focus wire changes. Focus observer and overlay tests cover protocol round trips and TypeScript dispatch guards; display-backed outside-click geometry remains host smoke coverage.
