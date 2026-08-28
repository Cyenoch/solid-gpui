# Pointer coordinates

Status: in progress

## Scope

Pointer down/up events carry the native pointer position so cursor-anchored
in-window overlays, cursor-following popovers, and custom press/release
gestures can use the same logical coordinate space as layout frames and
`onPointerDownOutside`.

The protocol remains v3. The pointer payload is now exactly
`[6,button,modifiers,action,clickCount,x,y]`; both TypeScript and Rust require
seven fields and validate finite, non-negative `x`/`y` values. Coordinates are
logical window pixels (not display/device pixels), matching GPUI's
`MouseDownEvent.position`, `MouseUpEvent.position`, `Window::viewport_size`,
layout frames, and outside-pointer events.

## Evidence

Pinned GPUI's `MouseDownEvent` and `MouseUpEvent` both expose
`position: Point<Pixels>` (`references/zed/crates/gpui/src/interactive.rs`). The
React host handlers in `crates/react-gpui/src/renderer/paint/mod.rs` already
receive those events; `onPointerDownOutside` and scroll emission prove the
same logical-window coordinate space is available at the renderer seam.

## Wire decision

Coordinates are required, not an optional tail. GPUI always supplies a position
for each press/release, and this protocol line has no historical pointer
payload package to preserve. A legacy five-field shape with a default `(0, 0)`
would erase a real host value and keep a second design alive, contrary to the
repository's clean-cutover rule. Therefore there is no old-arities decoder or
fallback: missing coordinates are invalid.

The host clamps each coordinate to the current viewport before encoding, so
negative and out-of-window native positions cannot cross the wire. Non-finite
values are defensively mapped to zero before the same clamp. Wire decoders on
both sides still reject non-finite or negative values.

Coordinates are emitted for down and up only. Per-move/hover coordinate
streaming is deliberately not exposed in this round: it would turn sparse
semantic notifications into a high-rate event stream and add JS/transport work
without a committed consumer contract. A future pointer-move feature should be
specified separately.

## Verification

- Rust protocol tests cover seven-field round trips and reject NaN, negative,
  and malformed coordinate payloads.
- Rust emitter tests cover viewport clamping and up-event coordinate delivery.
- TypeScript decoder and dispatch tests cover required coordinates and handler
  `x`/`y` values.
- TestApp pointer helpers accept coordinates and round-trip through the real
  MemoryTransport dispatch path.
- The macOS host display-backed test clicks a known painted control and asserts
  both emitted pointer actions carry the matching click coordinates.
