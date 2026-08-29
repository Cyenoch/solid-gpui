# Global test-order independence sweep

Date: 2026-08-29

This sweep was run from the repository at HEAD `b61d3a3`, without starting or touching live gallery processes. The Bun CLI was checked first; Bun supports `--randomize` and `--seed=<val>` for whole-directory runs, so no file-pair fallback was needed.

## Results

| Suite | Runs | Pass | Fail | Result |
| --- | ---: | ---: | ---: | --- |
| `packages/react-gpui`: `bun test --randomize --seed=N --reporter dots`, N=1..10 | 10 (128 tests / 16 files each) | 10 | 0 | PASS |
| `packages/react-gpui-dev`: `bun test --randomize --seed=20260829 --reporter dots` | 1 (20 tests / 4 files) | 1 | 0 | PASS |
| `cargo test -p react-gpui --lib --locked` (default parallel) | 3 (114 tests each) | 3 | 0 | PASS |
| `cargo test -p react-gpui --lib --locked -- --test-threads=1` | 1 (114 tests) | 1 | 0 | PASS |
| `cargo test -p react-gpui-host --locked` (all targets, default parallel) | 3 (65 tests / 5 suites each) | 3 | 0 | PASS |
| `cargo test -p react-gpui-host --locked -- --test-threads=1` (all targets) | 1 (65 tests / 5 suites) | 1 | 0 | PASS |
| `cargo test -p react-gpui-host --test examples_render --locked` | 2 (25 tests each) | 2 | 0 | PASS |

### Bun per-seed record

| Seed | Tests | Pass | Fail |
| ---: | ---: | ---: | ---: |
| 1 | 128 | 128 | 0 |
| 2 | 128 | 128 | 0 |
| 3 | 128 | 128 | 0 |
| 4 | 128 | 128 | 0 |
| 5 | 128 | 128 | 0 |
| 6 | 128 | 128 | 0 |
| 7 | 128 | 128 | 0 |
| 8 | 128 | 128 | 0 |
| 9 | 128 | 128 | 0 |
| 10 | 128 | 128 | 0 |

The dev suite's randomized run used seed `20260829` and reported 20 pass / 0 fail across 4 files.

## Analysis

No failures or flakes were observed. Every requested randomized Bun seed passed; both Rust package modes (default parallel and single test thread) passed; and the real-process `examples_render` smoke passed twice. Therefore there was no failing order to minimize, no shared-state leak to root-cause, and no source or test fix was warranted.

The expected negative-path diagnostics printed by existing tests (validation error output, Fast Refresh parser-error output, protocol-tap missing-file diagnostics, and transport termination notices) did not cause test failures; each run completed with its reported pass count and zero failures.

No `CHANGELOG.md` entry was added because no consumer-visible fix landed.

## Verification commands

```text
cd packages/react-gpui && bun test --randomize --seed=1..10 --reporter dots
cd packages/react-gpui-dev && bun test --randomize --seed=20260829 --reporter dots
cargo test -p react-gpui --lib --locked                         # x3
cargo test -p react-gpui --lib --locked -- --test-threads=1
cargo test -p react-gpui-host --locked                           # x3, all targets
cargo test -p react-gpui-host --locked -- --test-threads=1      # all targets
cargo test -p react-gpui-host --test examples_render --locked  # x2
```
