# Ticket 07: recorded paint and CPU live frames

Branch: `adopt/07-paint-media`.
Worktree: `/private/var/folders/sl/42r5kc756mj96rlxkglrpr900000gn/T/opencode/solid-gpui-adoption-20261002/07-paint-media`.
Required `e879b66` ancestry verified before implementation.

## Public capability

Both components are registered by `solid_gpui::components::native_module()` and
generated into `@solid-gpui/core/components` and the website native bindings.
No wire schema, host kind, pointer handle, or competing paint protocol was added.

- `RecordedPaint`: `recording: PaintRecording`, optional `viewportHeight` (default
  240), optional `onViewport({width,height,scaleFactor})`. Recording contains
  `width`, `height`, `commands`, optional `fit: "contain" | "none"`, optional
  `transform: {scale,translateX,translateY}`, optional `clip: PaintRect | null`.
- Commands: quad bounds/color; straight path points/closed/stroke/strokeWidth/fill;
  single-line text origin/text/fontSize/color. Fill polygons must be strictly
  convex with one winding. Colors use solid validated hex or native theme tokens.
  Explicit uniform scale and translation affect retained geometry, strokes, and
  text; clipping intersects the transformed clip, viewport, and ancestor mask.
- `LiveFrame`: optional `viewportHeight`, optional `fit: "contain" | "cover" |
  "fill"`. `LiveFrameRef` is the generated type (no `LiveFrameHandle` alias).
- Ref commands: `replaceFrame(CpuFrame)`, `beginFrame(FrameUpload)`,
  `writeFrameChunk(FrameChunk)`, `presentFrame(sequence)`, `cancelFrame()`, `clear()`,
  `dispose()`, `getState()`. Each returns `FrameState` with `disposed`, last
  presented `sequence`, visible width/height, and owner `retainedBytes` including
  staging reservations. Upload sequences are reserved on admission, preventing
  delayed chunks from entering canceled/restarted uploads.
- Pixel input is tightly packed straight-alpha sRGB RGBA8 in validated `number[]`
  through the existing bounded JSON native-call bytes. Native code converts to
  GPUI BGRA once on presentation. No encoded images/data URLs are generated.

## Ownership and bounds

Paint admission is atomic before tree publication. Preparation is linear in path
points and occurs when recording content changes; replay reads current viewport,
theme/font, clip, and device scale. No JS callback runs in native paint. Maximums:
1,024 commands, 4,096 aggregate path points, 32,768 prepared triangle vertices,
2,048 UTF-8 bytes per text command, 16,384 aggregate text bytes, coordinates within
16,384, text fontSize × explicit scale at most 256. Fit shrinks without upscaling.
The NativeView mount owns recording/geometry and releases them at removal/epoch.

LiveFrame's native instance owns the capability through existing
surface/epoch/node/module/catalog checks. It holds one visible frame and one
staging upload. Dimensions 1..4,096, at most 16 MiB per decoded frame; inline and
chunk payloads at most 240 KiB. A 64 MiB host budget covers staging reservations
and pixel owners still retained by native scenes. Replacement reserves before
mutation; partial/stale/oversized frames preserve the visible content. Clear
releases pixels and keeps sequence history; dispose permanently closes the owner.
Unmount/epoch teardown invokes disposal. Pixel owner release drops GPUI atlas
images. GPU-driver reclamation is not synchronous, and replacement may temporarily
need old plus new frame capacity. Producers await acknowledgements; no native
unbounded frame queue exists.

## Concrete consumers and synchronization

Website catalog source `component-recipes/paint-media.ts` contains an interactive
workflow diagram (node movement, zoom, theme-aware native drawing, viewport
feedback) and a real acknowledged CPU test-pattern stream (single in-flight
upload, pause, clear, error state, timer cleanup). Both compile against generated
public APIs and appear in Low-level drawing navigation. Ticket 09 was sent the
final API for its `ReferenceStudioProps.preview` consumer: bounded 64×36 frames
advanced by user action, with native owner teardown.

Synchronized authoritative English/Chinese `docs/paint-media`, component family
tables, native composition guides, documentation index, website navigation label
and translation, catalog examples, website README, changelog, and generated SDK
and website bindings. Source comments and canonical documentation are English.

## Verification

Environment: macOS arm64 desktop session, linked vendored `gpui-pre 0.3.7`, GPUI Kit
0.7.0. Debug native correctness build, 320×240 logical raster acceptance window
at native 2× scale, monitor disabled. No FPS or input latency claim. TestAppContext
scene tests use a 500×400 window, 2× then 1× DPI, and a 300×200 resize.

Passed:

- Red-before-green admission and frame replacement tests; five focused final
  correctness/resource tests cover aggregate bounds/finite geometry, native
  transform/clip/theme/resize/DPI quads, exact RGBA byte conversion, atomic chunk
  presentation, host saturation/release, canceled sequence reuse rejection,
  clear/dispose, and real epoch replacement/old-owner command rejection.
- Real native `host_paint_media_platform`: raster pixel assertions for quad,
  convex path, shaped text, CPU frame, and clear/dispose. Capture:
  `paint-media-native.png` (640×480 device pixels). This uses the actual native
  platform/renderer rather than a TestAppContext timing or file-size assertion.
- Seven website tests, including generated catalog coverage, SDK type-checking
  executable examples, bilingual guide loading/highlighting, and retained runtime.
- Website typecheck and production frontend build after WASM host generation.
- WASM release host build with the pinned nightly and wasm-bindgen 0.2.121.
- Targeted rustfmt, formatter, and `git diff --check`.

Commands (set the requested `CARGO_BUILD_JOBS=2` and shared `CARGO_TARGET_DIR`):

```sh
export RUSTFLAGS=--cfg=ticket07_build
cargo test -p solid-gpui --features gpui-component,test-support --lib paint_media
cargo test -p solid-gpui --features gpui-component,test-support --lib recording_admission
cargo test -p solid-gpui --features gpui-component,test-support --lib frame_replacement
cargo test -p solid-gpui --features gpui-component,test-support --test host_paint_media_platform
bun scripts/native-codegen.ts
bun scripts/native-codegen.ts --check
bun --conditions=browser test examples/website/tests
cargo +nightly-2026-07-28 build --locked -p solid-gpui-web --target wasm32-unknown-unknown --release
wasm-bindgen "$CARGO_TARGET_DIR/wasm32-unknown-unknown/release/solid_gpui_web.wasm" --out-dir examples/website/src/wasm --target web
bun run --cwd examples/website build:frontend
```

The distinct Rust cfg is a build fingerprint only. Another ticket's incompatible
proc-macro artifact was observed in the shared target; the fingerprint prevents
cross-worktree reuse without changing the required target path. Do not run a
second branch concurrently with the same fingerprint.

## Limits and integration

CPU frame admission/presentation compiles for the portable GPUI path and WASM;
native pixel/window evidence here is macOS only. Windows/Linux/browser interactive
pixel acceptance remains an integration/platform qualification. No hardware
decoder, camera, audio, native shared-surface import, IOSurface pointer bytes,
zero-copy claim, or platform acceleration shim is exposed. Concave fills,
Bézier paths, arbitrary canvas state stacks/readback, rotation, skew, and multiline
paint text are outside this explicit contract; this is useful retained native
drawing and decoded CPU frames, not browser Canvas.

Integration must regenerate combined bindings after merging all native modules,
particularly ticket 03's contract identity changes. Catalog IDs/digests are
generated; consumers must use component names/refs, never hard-code entry IDs.
The native acceptance fixtures resolve their component IDs from the same module.
The requested integration branch will be merged into this branch before reporting;
this branch does not merge itself into integration and publishes nothing.
