# 07 — React reconciler/host commit-diff boundary

**Status: resolved-engine-bound.**

## Evidence

The final 10,000-node stateful measurements at 240 Hz were **8.744 ms p50 / 12.412 ms p99** for scroll and **9.506 ms p50 / 13.084 ms p99** for drag-over. The same-size no-op control was **0.001 ms p50 / 0.004 ms p99** with 0 commits. Temporary stage profiling bounded scroll patch build/encode/submit at **0.029 ms p50 / 0.143 ms p99** and drag-over at **0.013 / 0.056 ms**.

Consequently, more than 99% of the stateful event latency is in the React reconciler/host commit-diff engine boundary. Tight-loop bursts did not reveal a hidden queue or event drop: scroll at a 240-Hz target processed 240/240 commits at 111.9 events/s with 8.618 / 11.364 ms p50/p99; drag-over processed 240/240 at 111.9 events/s with 8.696 / 11.284 ms.

The 143-node gallery-sized controls remain below the 8.33 ms 120-Hz frame interval: scroll at 240 Hz was **0.407 / 0.721 ms**, and drag-over was **0.487 / 1.178 ms** p50/p99.

## Resolution

This is a measured engine-bound limitation, not a protocol or listener hotspot. No speculative production coalescing or reconciler replacement was attempted. The committed perf harness preserves the stress case and the no-op control so a future renderer-engine change can be evaluated against the same evidence.
