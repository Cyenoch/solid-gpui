# Cross-surface focus and activation routing

Status: resolved

## Evidence

- `references/zed/crates/gpui/src/window.rs:1713-1733` registers activation callbacks per native window.
- `references/zed/crates/gpui/src/window.rs:2924-2963` compares per-window active state and dispatches focus listeners with an empty path while inactive, producing blur for previously focused nodes.
- `crates/react-gpui/src/renderer.rs:430-479` emits observation events using the owning root's surface id and epoch.
- `packages/react-gpui/src/renderer/root-container.ts:819-831` rejects events for another surface or epoch before callback dispatch.
- `crates/react-gpui-host/src/test_support.rs:669-744` performs real headless dispatch across two native windows: surface 1 receives focus, activating surface 2 produces a blur for surface 1, and surface 2 receives its own focus event.
- `crates/react-gpui-host/tests/command_roundtrip.rs:32-36` runs that roundtrip against `TestAppContext`.

## Decision

No cross-surface routing gap was found. Activation/focus notifications remain scoped to each native window/root; the headless regression proves real dispatch preserves surface identity.
