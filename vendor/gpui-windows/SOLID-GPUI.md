# GPUI windows provider

Vendored from gpui-pre-windows 0.3.5, Zed snapshot
`d89e9c2`.

Local changes implement owner-attached popup windows and live anchor positioning.
Keep the upstream license and package metadata when updating this provider.

Window-movement and recovery corrections:

- Repeated minimization preserves the parked frame callback. Restoration
  reinstalls it, checks renderer sizing, and requests one forced recovery frame
  even when logical bounds are unchanged.
- Consume the pending recovery flag even when another source already forces the
  current frame; do not leave a second forced redraw latched by short-circuiting.
- Refresh display metadata and notify bounds observers on `WM_DISPLAYCHANGE`,
  including work-area changes that retain the same monitor identity.

Native Windows qualification is separate from the shared tests and macOS checks;
see [`docs/performance-analysis.md`](../../docs/performance-analysis.md).
