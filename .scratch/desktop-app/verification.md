# Desktop application verification history

Recorded before the example was renamed from `native-migration` to
`desktop-app`. Names and measurements below describe those dated runs, not
current acceptance or a migration workflow. Keep reusable API guidance in
`docs/`; run the current [desktop application example](../../examples/desktop-app/README.md)
using its own README.

## Integration verification (2026-09-16)

- macOS: 254 Rust library tests passed (one ignored), 86 core package tests,
  12 Vite/export/build/API tests, four cross-language protocol tests, and eight
  website tests passed. Package type checks, the migration production build,
  and the website WASM/TypeScript/production build passed. The consumer's
  TypeScript project also passed against current SDK sources with
  `verbatimModuleSyntax` enabled, without modifying that project.
- Windows: tested in a Windows 11 ARM64 Parallels VM with x86-64 GNU debug hosts
  cross-built using Zig, and portable ARM64 Bun 1.4.2. Both the QuickJS website
  and Bun migration host were linked with a 1 MiB initial stack. Their normal
  entrypoints reported a 16 MiB application-thread reservation and rendered
  successfully. The same website executable with
  `SOLID_GPUI_APP_STACK_BYTES=1048576` instead reported a native stack overflow
  and exited with `0xC000041D`; no animation policy or page content was changed.
- Windows pixels and interactions confirmed readable Chinese UI, native input,
  checkbox changes, ordinary animated buttons, and a Rust command round trip.
  The migration Select emitted zero changes on catalog load, menu opening, or
  confirming the current row; a different selection emitted exactly one change.
  Unloading its catalog retained that controlled key and event count. Wheel
  scrolling reached row 14 and the end marker while the header/footer stayed
  fixed. macOS also reached the final row with the bounded layout.
- Abruptly terminating the Windows migration host left no renderer orphan:
  its Bun child disappeared after 0.68 seconds. The macOS host-kill smoke also
  left no renderer child. Separate stdio regressions cover detached-service
  preservation and custom-stream ownership.
- Windows font probes resolved and rasterized `.SystemUIFont`; its sampled
  glyphs matched Microsoft YaHei UI on this zh-CN guest. Windows core tests
  recorded 196 passes, seven failures, and one ignored test: five failures
  require POSIX `sh`/`sleep`, and two require the unstaged Windows JSX toolchain.
  The two absolute-path fixture failures were corrected and passed separately.

This is not MSVC/native-ARM64 host qualification or a measurement of every
consumer application's required stack. Linux and the complete Windows release
matrix were not exercised. The original fresh-`hotKey` blank-window report was
not independently isolated; the demonstrated fixes address managed handoff
capture and first-Snapshot preparation for later-opened Surfaces, with
regressions and managed reload/rollback checks.

## Integration verification (2026-09-08)

The merged workspace passes 220 Rust library tests, four cross-language protocol
tests, 37 migration/renderer tests, workspace TypeScript checks, and all five
website tests. The migration fixture's host and generated bindings compile.
The WASM release host and website production bundle build successfully with
`wasm-bindgen 0.2.121`. Website checks cover component examples, documentation
highlighting, and retained navigation. The native window interactions recorded
below were not manually repeated during this integration.

## Verification record (2026-09-07)

Environment: macOS 26.6.2 on Apple Silicon, Bun 1.4.2, Rust 1.98.1.

- `bun run check`: passed, including generated protocol/native contract checks,
  workspace builds, TypeScript checks, and workspace Clippy with QuickJS enabled.
- Core/router/HMR tests: 72 passed. Rust library tests with `gpui-component`:
  205 passed. Bidirectional protocol golden checks: passed.
- The example's TypeScript check and Vite production build passed.
- Native UI smoke tests used the example host in a temporary macOS app bundle.
  Initial content was 1100 × 720; resizing below the minimum stopped at 960 × 640.
  Both sizes retained the two-column summary and bottom action border.
- A single 48 px titlebar was visible, with traffic lights at the configured left
  inset and vertical center. Header input typing, input double-click, navigation,
  and native button clicks worked without zooming the window. Blank-area
  double-click zoomed and restored through the macOS window behavior. Fullscreen
  and restoration retained the titlebar/content layout. A blank-area drag was
  exercised; absolute screen displacement was not measured by the UI capture.
- The native input, button, tag, text, background, custom SVG, cover image,
  gradient, segment seam, and edge borders were visually inspected. No FPS
  overlay appeared in the debug application.
- A TSX title edit painted without input to wake the window. A syntax error and
  a synchronous render exception retained the last successful UI; restoring the
  source recovered in the same host process (PID 31772 during the final run).
  The Rust service counter advanced from 1 before HMR to 2 after recovery.
- The production JavaScript bundle ran in the same native host without Vite,
  rendered the local image and embedded icon, and invoked the Rust service.
  Closing both development and production windows exited the host successfully
  and shut down their Bun children.

These checks exercise framework APIs. Applications need their own visual and
interaction acceptance tests. Select, dialog, menu, and TextView theme integration is
covered by the shared native theme implementation and automated checks, but
their entire state matrix was not visually exercised in this fixture. The
linked GPUI background primitive supports exactly two gradient stops; additional
stops are rejected explicitly. Linux/Windows window behavior and QuickJS HMR
were not manually verified in this delivery.
