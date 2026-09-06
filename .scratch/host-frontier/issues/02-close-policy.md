# Asynchronous close policy

Status: resolved
Type: feature

`Root.setClosePolicy("allow" | "require-confirmation")` stores host policy. `allow` leaves GPUI's native close callback permissive. `require-confirmation` returns `false` synchronously from `Window::on_window_should_close`, emits `EVENT_CLOSE_REQUESTED` with `[9,requestId]`, and waits for `Root.resolveCloseRequest(requestId, allow)`. Deny leaves the native window open. Allow marks the request resolved and calls `window.remove_window()` through a direct GPUI update, bypassing the callback's pending veto so the close cannot re-enter.

Only one in-flight request is tracked per surface; repeated native requests while awaiting a decision are vetoed without duplicate events. Unknown, stale, or already-resolved request IDs are ignored. Transport shutdown and registry teardown clear policy state. Wayland layer-shell close and app-level quit remain outside this per-window callback boundary.

Evidence: `references/zed/crates/gpui/src/window.rs:5952-5963`, `references/zed/crates/gpui/src/platform.rs:857-861`, and `references/zed/crates/gpui/src/app/test_context.rs:939-965`.
