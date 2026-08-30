# Last-surface process lifecycle verification

## Verdict

**Last-surface exit is implemented and verified at the closest honest headless seam.** The native host's window-close observer removes the surface from `SurfaceRegistry`, checks whether the registry is empty, and calls `App::quit()` only for the final surface. The two-surface headless integration test passes: the first close leaves one surface and does not request quit; the final close empties the registry and requests quit.

The process-level macOS observation remains display-backed only. The TS API has no programmatic native-window close operation (`Root` exposes `unmount()`, which retires the renderer root but does not remove the native window). Native close is initiated by the window chrome and enters GPUI's display/window callback path. Therefore this change does not claim a headless process exit code or timing for a user window close.

## Code path

- `crates/react-gpui-host/src/main.rs:283-309`: `SurfaceRegistry::should_close` permits a clean close for `ClosePolicy::Allow`; for `RequireConfirmation` it emits one `CloseRequested` event and returns `false` while pending. A denied resolution leaves the surface open; an allowed resolution defers `window.remove_window()` (`:376-434`).
- `crates/react-gpui-host/src/main.rs:681-702`: `SurfaceRegistry::window_closed` emits `SurfaceClosed` before deleting the window, surface, and keybindings entries, then returns `self.surfaces.is_empty()`.
- `crates/react-gpui-host/src/main.rs:869-877`: the host's `on_window_closed` observer calls `window_closed`; when it returns `true` for the final surface, it calls `cx.quit()`.
- `crates/react-gpui-host/src/main.rs:884-895`: `on_app_quit` shuts down the runtime. This is the clean application shutdown path, distinct from renderer transport failure, which uses `close_all` and a fatal nonzero exit.
- GPUI's pinned `references/zed/crates/gpui/src/app.rs:1900-1912` removes the window, invokes `on_window_closed`, and also applies GPUI's own quit mode. The host explicitly calls `cx.quit()` for its final-surface condition, so macOS's default explicit quit mode does not leave the host running after the last host surface closes.

## API and display limitation

`packages/react-gpui/src/renderer.ts:143-173` exposes `Root.unmount()` and surface commands such as `openSurface`, but no `closeSurface`/`closeWindow` operation. `Root.unmount()` at `packages/react-gpui/src/renderer.ts:342-353` clears the renderer tree and disposes its transport; it does not send a native close command. Native close is therefore user/display initiated through the window chrome. No scratch process app was created: a programmatic close seam does not exist, and simulating a display close would violate the verification requirement.

## Headless evidence

Added `crates/react-gpui-host/tests/command_roundtrip.rs:90-94`, backed by `crates/react-gpui-host/src/test_support.rs:2164-2226`, which drives the same `on_window_closed` callback shape used by `main`:

- Open initial surface 1 and protocol-open auxiliary surface 2.
- Remove surface 1: registry length becomes 1 and `quit_requested == false`.
- Remove surface 2: registry is empty and `quit_requested == true`.
- The callback calls `cx.quit()` exactly when `window_closed` reports the final surface.

Command run:

```text
cargo test -p react-gpui-host --test command_roundtrip closing_one_surface_keeps_host_alive_and_last_surface_requests_quit --locked
```

Result: `1 passed; 0 failed; 27 filtered out; finished in 0.01s` (test process wall time approximately 2.95s including compilation). No host process exit code/timing was recorded because the actual user-close trigger is display-backed and the headless GPUI platform's `quit` is intentionally a no-op.

Existing close-policy coverage remains in `crates/react-gpui-host/tests/command_roundtrip.rs:84-88`: it verifies clean close is allowed, confirmation close is vetoed while pending, denial keeps the surface, and a subsequent allowed resolution removes the last surface. The new test covers the process-lifecycle inverse that prior per-surface tests did not cover.
