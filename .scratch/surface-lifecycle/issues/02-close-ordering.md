# Close ordering, teardown, and reopen identity

Status: resolved

## Evidence

- `crates/react-gpui-host/src/main.rs:486-507` emits `EVENT_SURFACE_CLOSED`, removes only the matching window/surface/keybindings, and reports whether the registry is empty.
- `crates/react-gpui-host/src/main.rs:508-525` tears down all windows only after shared runtime termination.
- `packages/react-gpui/src/renderer/root-container.ts:814-818` disposes only the matching root on `EVENT_SURFACE_CLOSED`.
- `packages/react-gpui/src/surface-host.ts:93-181` allocates host IDs monotonically and now records retired IDs.

## Decision

The host's Rust allocator is monotonic and does not reuse IDs. The TypeScript host now matches that contract: explicit registration of a closed or explicitly unmounted ID throws `SurfaceIdReusedError`, even with a new epoch. Fresh host allocation remains available for a new surface.
