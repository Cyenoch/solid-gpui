# Option 3′ landing record

Date: 2026-08-31
Starting HEAD: `5c29636` (`docs: option 3 stub patch proof`)
Pinned Zed revision: `6805d952f9f3d702f760aa11b1547df8a625fa16`

## Authorization rationale

Option 3′ is the conservative variant authorized for landing. It removes
GPL-licensed code from the distributed host binaries and distributes zero Zed
tracing code in the replacement: the local implementation is independently
authored Apache-2.0 code reproducing upstream's own documented no-op
`instrument` semantics. It keeps the replacement/distribution posture
Apache-only while preserving the existing consumer-facing attribute path.
This does **not** select Option 4, clear the two no-license-field crates, or
constitute legal review. The standing no-compromise directive favors landing
this proven fix over leaving it dormant. The change is fully reversible by
deleting the local patch and stub, restoring the upstream dependency.

## Landed implementation

- `third_party/ztracing-stub/Cargo.toml`: exact nine-line `ztracing` v0.1.0
  proc-macro package, `React GPUI contributors`, Apache-2.0, no dependencies.
- `third_party/ztracing-stub/src/lib.rs`: exact eight-line independently
  authored identity `instrument` attribute macro.
- `third_party/ztracing-stub/README.md`: replacement rationale, blocker-note
  pointer, upstream no-op citation, host-graph/runtime boundary, and
  reversibility (11 lines).
- Root `Cargo.toml`: `[patch."https://github.com/zed-industries/zed"]` maps
  `ztracing` to the local stub and cites the blocker issue.
- `CHANGELOG.md`: added the requested entry under `Unreleased` → `Changed`.
- The audit issue records `Option 3′ EXECUTED`, narrows the STOP to
  `gpui_shared_string`/`gpui_util` plus `self_cell` Apache-option confirmation,
  and remains `ready-for-human`.

No upstream Zed source was copied or modified. No live gallery process was
touched.

## Lockfile

`cargo update -p ztracing` regenerated `Cargo.lock`. The diff removes the
upstream `zlog`, `ztracing_macro`, and source-backed `ztracing` records, removes
tracing-subscriber support records no longer reachable (`nu-ansi-term`,
`sharded-slab`, `thread_local`, `tracing-log`, `tracing-subscriber`, and
`valuable`), and adds a sourceless local `ztracing v0.1.0` record with no
dependencies. Cargo also refreshed already-resolved Windows support references
from `windows 0.61.3`/`windows-core 0.61.2` to `0.62.2`; no source manifest
changed those packages.

## Target graph proof

Every command used `--no-dev-dependencies`; Cargo emitted its compatibility
warning that this flag is now spelled `-e=no-dev`, then produced the results
below.

| Target | `-i zlog` | `-i ztracing_macro` | `-i ztracing` |
| --- | --- | --- | --- |
| `aarch64-apple-darwin` | **not found** (exit 101; package ID did not match) | **not found** (exit 101; package ID did not match) | **local path** `/Users/jgbingzi/workspace/sp/vue-gpui/third_party/ztracing-stub` |
| `x86_64-unknown-linux-gnu` | **not found** (exit 101; package ID did not match) | **not found** (exit 101; package ID did not match) | **local path** `/Users/jgbingzi/workspace/sp/vue-gpui/third_party/ztracing-stub` |
| `x86_64-pc-windows-msvc` | **not found** (exit 101; package ID did not match) | **not found** (exit 101; package ID did not match) | **local path** `/Users/jgbingzi/workspace/sp/vue-gpui/third_party/ztracing-stub` |

The three local inverse traces show the stub consumed by GPUI, `sum_tree`, and
the host paths for each target. Linux and Windows were proved with Cargo tree
resolution only; no cross-build was claimed.

## License checks

The informational `cargo deny check licenses` check remains intentionally RED
under the existing strict empty allow-list policy. Before this patch it
reported 670 rejected records, three GPL package manifest rejection blocks, and
two `no-license-field` warnings. After this patch it reported 663 rejected
records, zero GPL package manifest rejection blocks, one Apache-2.0 local stub
block, and the same two `no-license-field` warnings. The remaining STOP is not
silently allow-listed or removed.

`cargo deny check advisories` passed (`advisories ok`).

## Verification

- `cargo test -p react-gpui --lib --locked`: **135 passed**, one suite.
- `make ci`: **passed** — Rust formatter, workspace check/clippy/tests,
  Bun formatter/typecheck/tests/package smoke, and advisory audit all green.
  The workspace test portion passed all suites.
- `make embedded-bun`: **passed** — host check, two embedded examples tests,
  and embedded counter test.
- `make examples-smoke`: **passed** — 14/14 examples, 0 failures; each
  startup and bounded teardown passed.
- `make kill-resilience`: **passed** — 6/6 host/renderer/group kill cases,
  0 failures.
- `cargo fmt --all -- --check`: **passed** (also run by `make ci`).
- The spike proof already ran the full React GPUI library suite through the
  identity macro path; it recorded 135 passing tests and exercised renderer,
  rich-text, and SVG paths.

Commit: `build(deps): replace ztracing with local no-op stub (option 3′)`
