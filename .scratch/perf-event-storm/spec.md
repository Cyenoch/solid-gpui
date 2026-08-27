# Native event-storm performance audit

## Status

**resolved-engine-bound** for the 10,000-node stress case, with **resolved-no-gap** for the 143-node gallery-sized case. The measurement does not justify a production event-coalescing change: the dominant cost is inside the React reconciler/host commit-diff engine boundary, not protocol encoding, listener fan-out, or native layout callbacks.

## Scope and method

The audit covers the TypeScript renderer event path, the Rust event wire encoder, native layout callback delivery, and the process writer queue. The TypeScript harness uses the existing synchronous `MemoryTransport`, creates a fresh root per scenario, and measures the synchronous `transport.push` call. It exercises a one-second run at 60, 120, and 240 Hz with 143-node and 10,000-node trees for scroll and drag-over events. It also runs tight-loop (unpaced) 10,000-node bursts, a 10,000-node no-op listener control, a 143-node layout stream, and a 143-node VirtualList visible-range stream.

Focused commands used for the committed harness were:

```text
bun test packages/react-gpui/tests/perf-event-storm.test.tsx
cargo test -p react-gpui --test perf_event_storm -- --nocapture
cargo test -p react-gpui --lib layout_next_frame_callbacks_dedupe_repeated_draws -- --nocapture
```

The temporary `REACT_GPUI_PROFILE` probe in `root-container.ts` was removed before the final TypeScript run and is not part of the committed runtime. The final TypeScript run passed 1 test, 0 failures, and 74 expectations.

## Stage decomposition

The no-op control isolates event decoding, listener lookup, and callback fan-out from a stateful React update:

| Stage | Measurement |
| --- | ---: |
| 10,000-node no-op, 240 Hz | p50 **0.001 ms**, p99 **0.004 ms**, 240 events, 0 commits |
| Scroll patch build (one operation) | p50 **0.014 ms**, p99 **0.055 ms** |
| Scroll patch encode | p50 **0.014 ms**, p99 **0.087 ms** |
| Scroll submit | p50 **0.002 ms**, p99 **0.011 ms** |
| Scroll patch stages total | p50 **0.029 ms**, p99 **0.143 ms** |
| Drag-over patch build (one operation) | p50 **0.007 ms**, p99 **0.020 ms** |
| Drag-over patch encode | p50 **0.004 ms**, p99 **0.037 ms** |
| Drag-over submit | p50 **0.001 ms**, p99 **0.004 ms** |
| Drag-over patch stages total | p50 **0.013 ms**, p99 **0.056 ms** |

The stateful 10,000-node runs cost about 9 ms at the median. The no-op control is four microseconds at p99, while the complete measured patch stages are at most 0.143 ms at p99. Therefore more than 99% of the stateful event cost is the React reconciler/host commit-diff engine boundary. The listener map is an O(1) lookup and is not a viable dominant hotspot.

## TypeScript event measurements

Values are `transport.push` latency in milliseconds (`p50 / p99`) from the final post-cleanup run. Each normal scenario sent one event per native-rate tick for one second.

| Tree | Event | 60 Hz | 120 Hz | 240 Hz |
| ---: | --- | ---: | ---: | ---: |
| 143 | scroll | **0.727 / 2.107** | **0.489 / 1.933** | **0.407 / 0.721** |
| 143 | drag-over | **0.513 / 1.468** | **0.444 / 0.756** | **0.487 / 1.178** |
| 10,000 | scroll | **8.779 / 12.709** | **8.618 / 11.455** | **8.744 / 12.412** |
| 10,000 | drag-over | **8.697 / 14.232** | **8.729 / 12.312** | **9.506 / 13.084** |

The 143-node tree remains below the 8.33 ms 120-Hz frame interval at every measured rate. The 10,000-node stateful tree exceeds that interval at the median and p99, so it is a real stress gap rather than a measurement artifact.

Tight-loop burst measurements confirm synchronous serialization rather than hidden queueing or dropping:

| Burst | p50 / p99 | Events/s | Commits |
| --- | ---: | ---: | ---: |
| Scroll, 10,000 nodes, 120 Hz target | **8.600 / 11.883 ms** | **112.5** | 120 / 120 |
| Drag-over, 10,000 nodes, 120 Hz target | **8.731 / 11.749 ms** | **111.6** | 120 / 120 |
| Scroll, 10,000 nodes, 240 Hz target | **8.618 / 11.364 ms** | **111.9** | 240 / 240 |
| Drag-over, 10,000 nodes, 240 Hz target | **8.696 / 11.284 ms** | **111.9** | 240 / 240 |

`MemoryTransport.push` invokes listeners synchronously and does not queue or drop these frames. The burst therefore spends about 2.14 seconds processing 240 events instead of pretending to sustain 240 commits per second.

The layout stream at 143 nodes produced 240 input events, 239 output commits, p50 **0.366 ms**, and p99 **1.590 ms**. The visible-range stream produced 240 input events, 239 commits, p50 **0.263 ms**, and p99 **1.215 ms**; repeated ranges are already idempotent at the VirtualList boundary.

## Native layout deduplication

The headless GPUI regression test renders **128 measured nodes** twice. Each draw schedules 128 `on_next_frame` callbacks, for **256 callbacks total**, while the measured-node dedupe emits exactly **128 layout events**. The focused test passed in **4.384 ms**. This validates callback delivery deduplication without adding a second production scheduling layer.

## Rust wire and writer measurements

The Rust encoder smoke test encoded 10,000 events of each kind and verified every frame against `MAX_FRAME_LENGTH`:

| Event | Payload | Framed bytes | 10,000 encodes |
| --- | ---: | ---: | ---: |
| Scroll | 34 B | **38 B** | **20.715 ms** |
| Drag-over | 20 B | **24 B** | **11.654 ms** |
| Visible range | 14 B | **18 B** | **8.330 ms** |
| Layout | 31 B | **35 B** | **10.193 ms** |

The process writer has a **32-frame** queue cap and a **16 MiB** queued-byte cap. At the measured event rates, a completely stalled writer reaches the frame cap in:

- `32 / 240 Hz = 0.133 s`;
- `32 / 120 Hz = 0.267 s`;
- `32 / ~107 Hz = 0.299 s`.

This is the reported **0.13–0.30 s** stall approach. At 18–38 bytes per event frame, 32 frames consume less than 1.3 KiB, so the frame cap—not the 16 MiB byte cap—is first for these event classes. The queue is bounded and ordered; it does not silently drop events. A `WouldBlock` while the runtime is still running is retained as a fatal writer/runtime failure, while explicit shutdown is handled as a nonfatal close.

## Resolution

- **Scroll and drag-over:** resolved-engine-bound for 10,000-node stress; resolved-no-gap at 143 nodes. The existing one-event/one-commit behavior is observable under separate synchronous pushes, but the measured dominant cost is React reconciliation and host diff/commit.
- **Listener fan-out:** resolved-no-gap. The no-op control is 0.001 ms p50 and 0.004 ms p99.
- **Layout callback storm:** resolved-no-gap. Native measured-node dedupe reduces 256 callbacks to 128 emitted events across two draws.
- **Protocol encode/submit:** resolved-no-gap. One-operation patch stages are at most 0.143 ms p99; Rust 10,000-event encoding stays below 21 ms total per event family.
- **Writer queue:** resolved-no-gap for loss behavior, with an explicit bounded-stall failure at the 0.13–0.30 s frame-cap arithmetic above.
- **Host paint count:** blocked-with-evidence for an exact compositor paint counter. GPUI invalidation uses dirty-view coalescing and the test platform suppresses duplicate frame scheduling, but react-gpui exposes no public paint-call counter. No speculative host paint change was made.
