# Native Image rendering diagnosis

Date: 2026-09-07. User symptom: Image does not display. Work is not time-limited.

The packaged native Gallery Image page showed only the viewport border/background.
The configured Unsplash URL returned HTTP 200, image/jpeg, 13,199 bytes via curl.

Executed failing loop:

`cargo test --locked -p solid-gpui --lib remote_image_source_reaches_http_loader_and_decodes_pixels`

The production source mapping attempted `fs::read` on an HTTPS URL and failed
with `No such file or directory`. The host additionally retained GPUI's
NullHttpClient because no HTTP provider was installed. Both have been corrected.

A real local HTTP request through the selected native client then exposed a
second integration failure: reqwest's idle-read timer was polled outside Tokio,
panicking with `there is no reactor running`. The pinned reqwest adapter now
retains its runtime handle in response readers and enters it for each poll.
The guard never spans an await; timeouts remain enabled.

`cargo test --locked -p solid-gpui --lib image` now passes seven cases, including
real HTTP outside Tokio, decoded BGRA pixel checks, file URL percent-decoding,
and a failed primary whose inline fallback reaches the native sprite atlas.
TypeScript verifies inline sources larger than the old path limit, a source
update patch, and rejection above the 1 MiB source budget.

The fallback no longer doubles as a loading placeholder. The Gallery has remote,
inline, and forced-failure controls for direct native acceptance. Packaged native-window verification passed on macOS ARM64: the remote JPEG
rendered its gradient, inline SVG rendered its circle/triangle, and a deliberately
invalid PNG rendered the SVG fallback. All five object-fit modes were inspected;
height changes updated the viewport. The page was restored to Remote JPEG,
cover, 280 × 180, radius 8. These checks inspect actual rendered pixels, not only
accessibility nodes. Windows/Linux desktop acceptance remains deferred.

Final regression: `cargo test --workspace --locked --features solid-gpui/quickjs`
passed 306 tests. The integrated TypeScript run passed the package suites but
exposed a release-prep version-field mismatch after Shiki was added; the script
now reads and validates all six fields, and all three release-prep tests pass.
Application lifecycle tests also pass after preserving the narrowed definition
in generation callbacks. Concurrent runtime/Web changes remain in progress.

Final package build passed, including signature verification and the extracted
application's `--check-bundle`. Latest package type checking passed. Notices were
regenerated to include all three project-owned Bun packages. Distribution:
`dist/native-production-validation/solid-gpui-gallery-0.2.0-aarch64-apple-darwin.zip`.
