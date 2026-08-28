# Root window controls

## Goal

Expose the pinned GPUI window operations that are useful to desktop React applications: minimize, read global bounds, read fullscreen/maximized state, and activate the window. Every operation is a root-only command and uses the existing request/receipt flow.

## Pinned evidence

The pinned GPUI revision is `6805d952f9f3d702f760aa11b1547df8a625fa16`.

- `references/zed/crates/gpui/src/window.rs:2128-2133` exposes `Window::is_maximized()`.
- `references/zed/crates/gpui/src/window.rs:2449-2452` exposes `Window::bounds()` as global `Bounds<Pixels>`.
- `references/zed/crates/gpui/src/window.rs:2477-2480` exposes `Window::is_fullscreen()`.
- `references/zed/crates/gpui/src/window.rs:5708-5711` exposes `Window::activate_window()`.
- `references/zed/crates/gpui/src/window.rs:5718-5721` exposes `Window::minimize_window()`.
- `references/zed/crates/gpui/src/platform/test/window.rs:215-217` reports maximized as `false`; `:286-288` reports `is_active()` as `false`; `:322-324` leaves `minimize()` as `unimplemented!()`.
- The test window still exposes readable bounds and updates size through its resize operation. Activation is callable but headless active-state observation is intentionally not a proof of foreground focus.
- The public Window API has no runtime position setter. `openSurface` can choose creation-time size and centered bounds, but cannot restore a saved position.

## Decisions

- `COMMAND_MINIMIZE_WINDOW = 30`: root-only, null payload, no value.
- `COMMAND_GET_WINDOW_BOUNDS = 31`: root-only, null payload, value tag 8 `[8,[x,y,width,height]]`. Values are finite logical/global bounds; on macOS the origin is screen-relative global top-left coordinates.
- `COMMAND_GET_WINDOW_STATE = 32`: root-only, null payload, value tag 9 `[9,[fullscreen,maximized]]`.
- `COMMAND_ACTIVATE_WINDOW = 33`: root-only, null payload, no value.
- The existing `EVENT_WINDOW_ACTIVATION` remains the activation-state channel; no duplicate `isActive` command is added.
- No title read or position setter is added: the former has no current consumer and the latter is upstream-blocked by the pinned public API.

## Persistence boundary

Applications can save `getWindowBounds()` and restore the saved size with `openSurface({ width, height })`; creation uses GPUI's centered bounds. Position restoration is not claimed because the pinned API has no runtime or creation-position setter in this protocol. The resulting recipe is “bounds read + centered + size restore,” with exact position restore explicitly upstream-blocked.

## Verification contract

- Rust protocol tests round-trip command 30–33 and value tags 8/9, including negative global coordinates and boolean state.
- Host headless tests assert readable bounds and the TestWindow fullscreen/maximized matrix, and route activation. Minimize is validated through wire/routing coverage without calling the pinned headless `unimplemented!()` method; display-backed testing is required for miniaturization.
- TypeScript renderer tests assert all four command tuples and decode tags 8/9.
- Golden vectors include every command 30–33 in both directions.
- Fuzz seeds include every command 30–33; the deterministic fuzz harness continues to validate malformed and truncated frames without panics.
