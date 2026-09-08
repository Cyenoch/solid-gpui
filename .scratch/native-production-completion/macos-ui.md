# macOS native-window verification

Date: 2026-09-07. Local Apple Silicon. Cross-platform acceptance is deferred.

The release package was built, ad-hoc signature verified, extracted, and passed
`--version` and `--check-bundle`. Artifact moved to
`dist/native-production-validation/solid-gpui-gallery-0.2.0-aarch64-apple-darwin.zip`.

Actual native window checks through CUA:

- Finder launch of the extracted release app displays the workspace immediately,
  without resize. Direct executable launch also displays it correctly.
- Transitions page navigates and paints; enabling Reduce motion changes the native
  AX switch value to on. Expanding the surface reaches Width 420 / Height 160,
  reports Completed, and exposes the status value `Transition finished (gen: 1)`
  through the same native AX tree.
- Both normal and maximized window layouts were inspected.

The automation tool's managed background launch reproduced a black initial
window, with content becoming visible after zoom. This did not reproduce through
Finder or direct process startup of the identical release binary. Debug probes
showed commits and revision-2 renders in normal launch, with or without the debug
performance monitor. This is a launch-condition-specific observation, not proof
of a fixed rendering bug or universal launch qualification. No speculative
refresh loop was added. All diagnostic source instrumentation was removed.

Screen-reader speech, platform IME, full keyboard accessibility, and Windows/Linux
GPU/runtime acceptance are not established by these checks.

## Updated package inspection

The later package includes the three-state motion preference and real directory
scan. CUA selected the repository `fixtures` directory through NSOpenPanel; the
rendered result was `11 files · 2 directories` and `Scan completed`, also exposed
as the native accessibility status. This validates an actual worker/picker/UI
round trip, not just a generated progress fixture.

`Follow System`, `Reduce Motion`, and `Full Motion` are visible. Selecting reduced
motion then expanding the transition exposed `Transition finished (gen: 1)`.
The app preference was returned to Follow System. OS settings were not modified.

Grid inspection uncovered black child text despite a parent theme color. The
renderer was constructing explicit runs from default styles before native
inheritance existed. Run assembly now caches local descriptors and resolves them
against the current native text style during layout; plain text uses native
inherited styling directly. The shared path covers interactive and selectable
rich text. A real native-layout regression switches parent theme color on the
same cached parts and preserves explicit inline color/weight overrides.

## Image rendering acceptance — 2026-09-07

The packaged native Gallery now displays the remote JPEG, inline SVG, and the
SVG fallback after a deliberately invalid PNG. Inspected all five object-fit
modes and a height update in the actual window. Restored Remote JPEG / cover /
280 × 180 / radius 8. See image-diagnosis.md for root causes and regression tests.
