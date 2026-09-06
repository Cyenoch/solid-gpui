# Host capability frontier

This bounded work adds two host capabilities at the existing SolidJS/GPUI seam:

- `tooltip?: string` on `View` and `Pressable`, encoded as host-property tag 5 and rendered with pinned GPUI's `Interactivity::tooltip` path. Tooltip display uses a minimal in-host `Render` view and inherits GPUI's default 500 ms show delay.
- An asynchronous close policy. JavaScript selects `allow` (native close proceeds) or `require-confirmation` (the host vetoes the synchronous native close, emits `EVENT_CLOSE_REQUESTED`, and waits for `resolveCloseRequest(requestId, allow)`). An allowed resolution closes through the GPUI window removal path without re-entering the veto callback.

The protocol stays lockstep v3. Host-property tag 5 is `[5, text]`. `COMMAND_SET_CLOSE_POLICY=23` uses the root command payload string (`allow` or `require-confirmation`), and `COMMAND_RESOLVE_CLOSE_REQUEST=24` uses `[requestId, allowCode]` where `allowCode` is `1` or `0`. `EVENT_CLOSE_REQUESTED=23` uses root event fields and payload `[9, requestId]`.

Pinned evidence: GPUI's `Div::tooltip`/`Interactivity::tooltip` (`references/zed/crates/gpui/src/elements/div.rs:662-675,1603-1611`), `AnyTooltip` (`references/zed/crates/gpui/src/app.rs:2990-3004`), tooltip placement/default lifecycle (`references/zed/crates/gpui/src/elements/div.rs:3516-3571`; `references/zed/crates/gpui/src/window.rs:3202-3267`), and vetoable `Window::on_window_should_close` (`references/zed/crates/gpui/src/window.rs:5952-5963`).

Context menus intentionally remain an application-composed in-window overlay: pointer-right events plus a positioned overlay and `onPointerDownOutside`, equivalent to the existing dropdown pattern. GPUI's generic `AnchoredPopup` is rejected on macOS and exposes no contextual `NSMenu` seam, so this work adds no context-menu API.

Boundaries: JavaScript confirmation is asynchronous by design; no synchronous `preventDefault` claim is made. Layer-shell Wayland close notifications and app-level quit are outside this per-window veto. Headless `simulate_close` proves callback cycles and event emission; native close behavior remains display-backed platform smoke coverage.
