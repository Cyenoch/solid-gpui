# 01 — Scroll event storm

**Status: resolved-engine-bound** (10,000-node stress); **resolved-no-gap** (143-node gallery-sized tree).

## Evidence

The final post-cleanup TypeScript harness drove one second of synchronous `MemoryTransport.push` calls at each native-rate target:

| Tree | 60 Hz | 120 Hz | 240 Hz |
| ---: | ---: | ---: | ---: |
| 143 nodes, p50 / p99 | 0.727 / 2.107 ms | 0.489 / 1.933 ms | 0.407 / 0.721 ms |
| 10,000 nodes, p50 / p99 | 8.779 / 12.709 ms | 8.618 / 11.455 ms | 8.744 / 12.412 ms |

The 143-node case stays below the 8.33 ms 120-Hz frame interval. The 10,000-node stateful case exceeds it at the median and p99, and every normal input produced a commit (60/60, 120/120, and 240/240).

A no-op 10,000-node control at 240 Hz measured **0.001 ms p50 / 0.004 ms p99**, with 0 commits. Temporary stage profiling measured one-operation scroll patch build **0.014 / 0.055 ms**, encode **0.014 / 0.087 ms**, submit **0.002 / 0.011 ms**, and all patch stages **0.029 / 0.143 ms** p50/p99. More than 99% of the stateful cost is therefore in the React reconciler/host commit-diff engine boundary, not event lookup or wire work.

An unpaced burst remained serial: scroll at a 240-Hz target measured **8.618 / 11.364 ms** p50/p99 and processed **111.9 events/s**, retaining 240/240 commits. There is no hidden queue or silent drop in `MemoryTransport`.

## Resolution

No production coalescing fix is justified by the measured split. The realistic 143-node gallery path has no frame-budget gap; the synthetic 10,000-node path is a resolved engine-bound limitation with evidence and a committed regression harness.
