# GPUI linux provider

Vendored from gpui-pre-linux 0.3.5, Zed snapshot
`d89e9c2`.

Local changes implement owner-attached popup windows and live anchor positioning.
Keep the upstream license and package metadata when updating this provider.

X11 property notifications compare fullscreen, maximization, and tiling state
before and after `_NET_WM_STATE` / `_GTK_EDGE_CONSTRAINTS` updates. A changed
visual state delivers the resize callback outside state/callback borrows even
when client dimensions are unchanged, so rendering does not depend on a later
`ConfigureNotify`. Wayland retains its existing forced-redraw configure path.

Native Linux qualification is separate from the shared tests and macOS checks;
see [`docs/performance-analysis.md`](../../docs/performance-analysis.md).
