# gpui-performance

A reusable native GPUI performance monitor with no SolidJS or gpui-component
dependency. Its translucent overlay shows live FPS and a 15-second history
at the top left or top right of a window.

## Integration

Use the same GPUI version as the application. This workspace uses a path dependency:

```toml
gpui-performance = { path = "../gpui-performance" }
```

Create the entity once during window initialization, then mount it in a
relatively positioned container that fills the window:

```rust,ignore
use gpui_performance::{MonitorCorner, PerformanceMonitor};

// cx: &mut App
let monitor = cx.new(|cx| {
    PerformanceMonitor::new(MonitorCorner::TopRight, cx)
});
// Store the Entity in the window view and compose it in Render:
div().relative().size_full()
    .child(content.clone())
    .child(monitor.clone())
```

Use `MonitorCorner::TopLeft` for the other corner. The monitor does not create
a sampling Task; do not recreate its entity on every render. It has no click
listeners and does not change content layout. A 56 px right inset leaves room
for common window toolbar controls.

The solid-gpui gpui-component host mounts the monitor by default. Launch that
host with `SOLID_GPUI_PERF_MONITOR=0` to compare performance without it. The
host interprets this environment variable; the reusable component does not.

## Metric definitions

- Sampling follows native draws passively. During activity, the reading and graph update after at least 250 ms without requesting additional redraws.
- FPS equals consecutive draw intervals divided by actual elapsed time. It measures draw-to-draw cadence, not the reciprocal of CPU draw duration.
- A gap of at least 500 ms between draws starts a new activity segment. Idle time neither adds low readings nor connects graph segments; the last reading remains visible. This threshold cannot distinguish a main-thread stall of 500 ms or more from true idle time. Investigate severe stalls with external call stacks, native draw duration, and input latency. A 250 ms slow frame still contributes to active FPS; low-FPS samples are not filtered out.
- The graph retains active samples from the last 15 seconds at their actual timestamps. Its vertical scale starts at 120 FPS and expands in increments of 60.
- This is window **draw cadence**, not GPU completion or display scanout. The overlay shows `FPS —` until it has enough samples after mounting or resuming activity. A retained reading on a static window is not a live refresh-rate guarantee.
- Mount the entity directly in a window overlay that participates in every draw, outside cached subtrees that can skip rendering.

The [performance analysis guide](../../docs/performance-analysis.md) explains
how to control the monitor's observation overhead.
