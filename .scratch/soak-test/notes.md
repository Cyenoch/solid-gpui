# Renderer soak verification

## Method

A normal `#[gpui::test]` runs in the headless GPUI test context. It creates one fixed 500-node tree containing a rich-text paragraph (raw text plus a styled nested run), a text input, a virtual list, and 38 shallow view branches. The test applies 2,000 deterministic one-node style patches, redraws after every patch, calls `run_until_parked`, and simulates a text-input event every 20 iterations. It samples every 200 iterations and reports side-map sizes, input undo/redo totals, the event sequence counter, elapsed time, and macOS RSS from `ps -o rss= -p $(std::process::id())`.

The test is intentionally bounded and runs in CI (not ignored). Side-map entries are checked against live node IDs and each map is bounded by `tree nodes + 16`; aggregate undo and redo history are checked against the implementation's 100-entry cap. Sequence growth is checked against a generous 1 + 2,000 × 256 bound. A four-sample monotonic-growth guard catches a map that grows while the tree is fixed.

## Measured run

Command: `cargo test -p react-gpui --lib surface_soak -- --nocapture`

| iteration | elapsed (ms) | RSS (KiB) | sequence | undo | redo |
|---:|---:|---:|---:|---:|---:|
| 0 | 0 | 21,664 | 3 | 1 | 0 |
| 200 | 1,636 | 22,160 | 53 | 1 | 0 |
| 400 | 3,246 | 22,176 | 103 | 1 | 0 |
| 600 | 4,861 | 22,192 | 153 | 1 | 0 |
| 800 | 6,475 | 22,192 | 203 | 1 | 0 |
| 1,000 | 8,085 | 22,208 | 253 | 1 | 0 |
| 1,200 | 9,703 | 22,208 | 303 | 1 | 0 |
| 1,400 | 11,313 | 22,240 | 353 | 1 | 0 |
| 1,600 | 12,923 | 22,240 | 403 | 1 | 0 |
| 1,800 | 14,540 | 22,256 | 453 | 1 | 0 |
| 2,000 | 16,152 | 22,272 | 503 | 1 | 0 |

Side-map metrics stayed constant across samples: input states 1, text-input layouts 1, focus handles 2, focus observers 1, virtual-list maps 1 each, rich-text cache 1, animation maps 41 each, and all other maps at 0 or 1. Every side-map ID remained in the live 500-node tree.

## Verdict

Deterministic state assertions passed for all 2,000 patch/draw iterations. No map grew monotonically and no sequence/history bound was approached. RSS rose 608 KiB from the initial sample to the final sample (peak also 22,272 KiB), with a 496 KiB warm-up increment followed by 0–32 KiB sampling deltas. The shape is **plateau/flat after allocator and GPUI warm-up**, not an unbounded leak; it is far below the 50 MiB finding threshold. The run took 16.15 seconds (well below the 60-second budget).

No renderer leak was found or fixed. The soak adds test-only sequence and input-history metrics accessors; production behavior is unchanged.
