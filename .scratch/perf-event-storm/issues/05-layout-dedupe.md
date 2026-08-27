# 05 — Layout callback deduplication

**Status: resolved-no-gap.**

## Evidence

The focused headless GPUI regression test renders **128 measured nodes** for two draws. Each draw schedules 128 `on_next_frame` callbacks, producing **256 callbacks** in total. Per-node measured-layout deduplication emits exactly **128 layout events**. The test passed in **4.384 ms**.

The final TypeScript layout stream at 143 nodes and 240 Hz also stayed within the small-tree budget: 240 input events, 239 commits, **0.366 ms p50 / 1.590 ms p99**. The visible-range stream was **0.263 ms p50 / 1.215 ms p99** with 240 events and 239 commits.

## Resolution

The native measured-node dedupe is sufficient. No additional React-side layout event queue or paint-time coalescing was added. The 128-to-128 result records that repeated draw callbacks do not become duplicate wire events.
