# 06 — Process writer queue saturation

**Status: resolved-no-gap** for loss behavior, with an explicit bounded-stall failure mode.

## Evidence

The process writer queue is bounded at **32 frames** and **16 MiB** queued bytes. For the measured event-rate envelope, a fully stalled writer approaches the frame limit in:

- `32 / 240 Hz = 0.133 s`;
- `32 / 120 Hz = 0.267 s`;
- `32 / ~107 Hz = 0.299 s`.

Thus the practical stall window is **0.13–0.30 s** before the frame cap at the tested rates. Event frames are only 18–38 bytes in the Rust wire measurement (visible range, drag-over, layout, and scroll respectively), so 32 frames consume less than 1.3 KiB. The 32-frame limit is reached long before the 16 MiB byte limit for these events.

The queue preserves order and does not silently drop events. `WouldBlock` at a still-running runtime is retained as a fatal writer/runtime failure; explicit shutdown is treated as a nonfatal close. The TypeScript unpaced bursts also retained all commits (120/120 or 240/240), confirming no local drop path in `MemoryTransport`.

## Resolution

The queue limit is intentional backpressure/failure containment, not an event-loss gap. No larger queue or drop policy was added. The 0.13–0.30 s arithmetic is recorded so a future host integration can choose an explicit overload policy rather than accidentally turning a bounded failure into unbounded memory growth.
