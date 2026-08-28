# Clipboard image interchange

Status: resolved
Verdict: future

## Question

Does pinned GPUI expose enough clipboard image support for a bounded React
`setClipboardImage`/`getClipboardImage` surface?

## Evidence

- `references/zed/crates/gpui/src/platform.rs:310-321` exposes synchronous
  clipboard read/write plus an async read fallback.
- `references/zed/crates/gpui/src/platform.rs:2345-2354,2556-2565,2699-2707`
  defines `ClipboardEntry::Image` carrying an `ImageFormat` and encoded bytes.
- macOS reads/writes image pasteboard bytes at
  `references/zed/crates/gpui_macos/src/pasteboard.rs:52-107,159-172,236-248`.
- Windows reads/writes native image formats at
  `references/zed/crates/gpui_windows/src/clipboard.rs:70-128,194-258`.
- X11 has image MIME helpers at
  `references/zed/crates/gpui_linux/src/linux/x11/clipboard.rs:980-1068`,
  but its current adapter writes through `set_text` at
  `references/zed/crates/gpui_linux/src/linux/x11/client.rs:1759-1771`.
- Wayland reads image offers at
  `references/zed/crates/gpui_linux/src/linux/wayland/clipboard.rs:123-137,199-214`,
  while its write path advertises/sends text only at
  `references/zed/crates/gpui_linux/src/linux/wayland/client.rs:1233-1256` and
  `references/zed/crates/gpui_linux/src/linux/wayland/clipboard.rs:183-187`.

## Decision and action

The native seam is real but platform-uneven. Do not implement this round: the
current command result values have no bytes tag, and adding one collides with
the concurrent file-I/O protocol ownership. The parent spec contains the
future payload draft: MessagePack binary bytes, a closed image-format enum, an
explicit byte cap below the frame ceiling, and platform-specific failure
semantics. No portable RGBA promise is allowed.
