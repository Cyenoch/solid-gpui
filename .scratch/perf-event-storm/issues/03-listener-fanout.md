# 03 — Listener fan-out and dispatch

**Status: resolved-no-gap.**

## Evidence

Listener dispatch looks up the registered listener by ID in the renderer's map and invokes it synchronously. The 10,000-node no-op control was designed to retain event decode, listener lookup, and callback fan-out while avoiding React state changes. At 240 Hz it processed 240 events with 0 commits at **0.001 ms p50 / 0.004 ms p99**.

For comparison, stateful 10,000-node events cost 8.744 ms p50 / 12.412 ms p99 for scroll and 9.506 ms p50 / 13.084 ms p99 for drag-over. The no-op control is below 0.05% of the stateful median, so listener fan-out cannot explain the storm gap.

## Resolution

No listener queue, broadcast abstraction, or coalescing layer was added. The existing O(1) listener map and synchronous dispatch are sufficient; adding machinery here would not address the measured React reconciler/host commit-diff cost.
