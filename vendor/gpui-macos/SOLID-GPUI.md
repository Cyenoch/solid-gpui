# GPUI macos provider

Vendored from gpui-pre-macos 0.3.5, Zed snapshot
`d89e9c2`.

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
