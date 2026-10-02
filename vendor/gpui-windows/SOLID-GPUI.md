# GPUI windows provider

Vendored from gpui-pre-windows 0.3.7, Zed snapshot
`1a28cff4b409169bac058bca40dfbfeb7621d19b` as declared by the published manifest.
The [source inventory](../GPUI-SOURCES.md) records the actual 0.3.7 archive digest
and changed paths, compared on 2026-10-02.

Local changes implement owner-attached popup windows and live anchor positioning.
Keep the upstream license and package metadata when updating this provider.

Shader/build and text corrections:

- Release shader generation uses Cargo's target and target debug-assertion state,
  tracks HLSL includes and compiler environment, and diagnoses an unusable compiler.
- Debug shaders compile embedded HLSL and bounded static includes in memory; the
  executable does not reopen shader files in the build-machine checkout. The
  `embedded_shaders_compile_without_source_files` regression requires Windows.
- `.SystemUIFont` resolves the locale's message UI font through
  `SPI_GETNONCLIENTMETRICS`, rather than the desktop icon-title font.

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
