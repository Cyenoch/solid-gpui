# Focus visibility for keyboard users

Status: resolved
Type: feature

Focusable View/Pressable nodes currently register GPUI focus handles for key input but expose no focus/blur callback, so JavaScript cannot merge a visible focus treatment. GPUI's `Context::on_focus`/`on_blur` and `FocusHandle::is_focused` provide the native transition seam (`references/zed/crates/gpui/src/app/context.rs:545-617`, `references/zed/crates/gpui/src/window.rs:496-607`).

Design: subscribe once per mounted focusable node that has a listener, emit existing Event 4/5 with a null payload, and dispatch `onFocus`/`onBlur` for View/Pressable. TextInput remains on its existing tag-1 text payload path. Application state composes focus with hover/active style rather than adding a `focusStyle` wire field.

Rejected: focusStyle wire variants duplicate the style tuple and do not compose with hover; using hover as a focus proxy is wrong for keyboard modality.

## Comments

The pre-existing protocol hunk was an orphaned partial change. It was confirmed by `git diff` and reverted before this implementation rebuilt the nullable focus path.
Implementation: Rust installs `Context::on_focus`/`on_blur` subscriptions for retained focusable View/Pressable listeners and emits Event 4/5 with a null payload. TypeScript validates and dispatches the generic payload to the callback while preserving TextInput's tagged payload. Renderer and protocol tests cover callback routing and round trips.
