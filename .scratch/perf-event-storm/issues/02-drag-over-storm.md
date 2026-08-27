# 02 — Drag-over event storm

**Status: resolved-engine-bound** (10,000-node stress); **resolved-no-gap** (143-node gallery-sized tree).

## Evidence

The final post-cleanup TypeScript harness measured synchronous `MemoryTransport.push` latency for one second:

| Tree | 60 Hz | 120 Hz | 240 Hz |
| ---: | ---: | ---: | ---: |
| 143 nodes, p50 / p99 | 0.513 / 1.468 ms | 0.444 / 0.756 ms | 0.487 / 1.178 ms |
| 10,000 nodes, p50 / p99 | 8.697 / 14.232 ms | 8.729 / 12.312 ms | 9.506 / 13.084 ms |

Every normal input produced a commit (60/60, 120/120, and 240/240). The 143-node tree remains below the 8.33 ms 120-Hz interval, while the 10,000-node tree exceeds it at every rate.

The 10,000-node no-op listener control at 240 Hz measured **0.001 ms p50 / 0.004 ms p99**, with 0 commits. One-operation drag patch profiling measured build **0.007 / 0.020 ms**, encode **0.004 / 0.037 ms**, submit **0.001 / 0.004 ms**, and total protocol stages **0.013 / 0.056 ms** p50/p99. The remaining roughly 9 ms is the React reconciler/host commit-diff engine boundary.

The unpaced 240-Hz-target burst measured **8.696 / 11.284 ms** p50/p99 at **111.9 events/s**, with 240/240 commits. The synchronous transport serializes work instead of dropping events.

## Resolution

No event-specific production fix is justified: fan-out and wire stages are below 1% of stateful latency, and the 143-node path has no budget gap. The 10,000-node result is a resolved engine-bound stress finding guarded by the committed perf test.
