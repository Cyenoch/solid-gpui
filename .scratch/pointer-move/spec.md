# Pointer-move event-stream performance audit

## Status

**resolved-no-gap** for the registered 143-node stream at 240 Hz. The opt-in
capability does not add work to nodes without `onPointerMove`; the measured
stream stays well below the 4.17 ms 240-Hz interval.

## Scope and method

The TypeScript harness uses `MemoryTransport`, a fresh root, and a registered
`onPointerMove` callback on each of 143 nodes. It sends one valid logical
coordinate event per tick for one second and measures synchronous
`transport.push` latency. The Rust smoke test encodes 10,000 pointer-move
events and checks each frame against the maximum frame length. Focused
commands:

```text
bun test packages/react-gpui/tests/perf-event-storm.test.tsx
cargo test -p react-gpui --test perf_event_storm -- --nocapture
```

## Results

The final TypeScript run passed 1 test with 78 expectations. The registered
pointer-move stream produced 240 events and 240 commits at 240 Hz, with input
6,946 bytes and output 6,599 bytes; `transport.push` latency was p50
**0.660 ms** and p99 **1.040 ms**, below the 4.17 ms frame interval.

The Rust encoder smoke test passed and retained the event-frame size budget;
pointer-move frames are bounded by the same protocol maximum as the existing
event family. These measurements cover opt-in registered delivery only and do
not claim active-drag behavior, which remains a separate GPUI path.

## Resolution

- Registered pointer-move delivery: **resolved-no-gap** at 143 nodes and
  240 Hz.
- Unregistered nodes: no native move listener or pointer-move frame is
  registered by the renderer; this is enforced by the node capability and
  paint registration tests.
- No event coalescing or special-case fallback was added.
