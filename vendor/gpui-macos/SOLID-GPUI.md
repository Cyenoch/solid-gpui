# GPUI macos provider

Vendored from gpui-pre-macos 0.3.7, Zed snapshot
`1a28cff`.

Local changes implement owner-attached popup windows and live anchor positioning.
Keep the upstream license and package metadata when updating this provider.

Window-movement corrections:

- Queue popup `setFrame:display:` on the foreground executor, guarded by window
  lifetime. AppKit move/resize callbacks must run after GPUI releases its App and
  window borrows; popup anchor options are published before the queued change.
- Treat a window without an AppKit screen as not maximized. Newly created panels
  can be sampled before they acquire a screen; never dereference a nil NSScreen.

Shared bounds invalidation and native qualification guidance live in
[`docs/performance-analysis.md`](../../docs/performance-analysis.md).

Blurred backgrounds (issue #3):

- Use an ordinary `NSVisualEffectView` with Sidebar material and explicit
  BehindWindow blending. Preserve Active state, content autoresizing and the
  existing remove/recreate lifecycle behind the Metal view.
- Remove the BlurredView subclass and its updateLayer override. AppKit owns
  material layers, desktop tinting, saturation and accessibility adaptation;
  no private layer classes or filter descriptions are inspected or mutated.
- The desktop-only `host_background_platform` test covers the real host's native
  material and view lifecycle. See the [background guide](../../docs/rust-bridge.md#macos-blurred-window-backgrounds)
  for visual qualification and application alpha requirements.
