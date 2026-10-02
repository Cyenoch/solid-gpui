# Native region qualification artifacts

`abba-final-policy/` is the accepted A/B/B/A construction attribution run.
`final-summary.json` separates root and region counters and costs. It establishes
no resize speed gain or display performance claim; see `../../delivery-05.md`.

Earlier `abba`, `abba-bundled`, `abba-resize` and `abba-final` runs are rejected
(no sustained presentation, or failed viewport query) and must not be pooled.
`abba-qualified` and `construction-summary.json` are preliminary mixed-span
attribution superseded by the root/region-specific final counters. Their totals
record the counterexample that led to changed-viewport bypass, not root averages.

`.txt` files preserve raw logs because repository rules ignore `.log` files.
`control.patch`, `features.json`, `environment.json` and binary identity files
record exactly what ran. `correctness.json`/`region-correctness.png` establish
shared-workload native Metal paint/input behavior separately from production
construction. `style-storage.jsonl` records current v7 allocation ownership,
not an inline baseline or GPU memory/presentation result.
