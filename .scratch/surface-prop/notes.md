# Surface lifecycle property test

## Invariants

- Every live root retains its `(surface_id, epoch)` identity across commands and draws.
- A command with the live surface ID and epoch produces exactly one command result for its request ID and does not silently disappear.
- A command with a mismatched epoch is rejected with `success = false` and `error = "surface or epoch mismatch"`; it must not panic or apply.
- A command routed with a different surface ID is rejected with the same mismatch result at the root seam.
- Reopening a retired surface ID with a new epoch leaves old-epoch commands rejected.
- Focus and activation commands are accepted only for live surface roots.
- Renderer side maps contain only node IDs present in their owning live root's NodeStore.
- Retired surface IDs are absent from the live state model unless explicitly reopened with a generation transition.

## Generator

The hand-rolled xorshift64* generator follows the tree property-test template exactly. Each of 16 seeds (`0..16`) runs 48 operations. Selection is approximately 14% fresh open, 10% retired-ID reopen when available, 10% close (while keeping one live root), 20% valid command, 15% invalid epoch, 11% wrong surface, 11% focus, and 9% activation. Candidate surface IDs are sorted before random selection for reproducibility.

## Findings

No renderer epoch/reuse violation was observed. Rust's surface seam exposes `TreeError::SurfaceMismatch` and command-result error text; typed `SurfaceClosedError`/`SurfaceIdReusedError` are TypeScript-layer errors already covered by the existing surface-host tests. Therefore this Rust property test asserts the Rust command-result mismatch contract and live-root model without duplicating the TS typed-error mapping.

## Runtime

Focused command: `cargo test -p react-gpui --lib surface_property`
Result: 2 property tests passed (seed-zero regression and 16-seed sequence), 130 tests filtered, 2 warnings, 0.14s test execution (cargo wall time 2.62s). Rust formatting completed with `cargo fmt --all`. Prettier was run against the requested files; it formatted `CHANGELOG.md` and correctly rejected the Rust file because no parser is configured for `.rs`.
