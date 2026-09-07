# Application performance analysis

This workflow is for developers and agents. First establish that the user's
action performs the intended work, then measure its performance. General GPUI
development rules live in the [gpui-performance skill](../.agents/skills/gpui-performance/SKILL.md);
Solid application conventions live in the [solid-gpui skill](../.agents/skills/solid-gpui/SKILL.md).
[scroll-performance.md](scroll-performance.md) records past incidents and measurements.

## 1. Identify the measurement

Create a local artifact directory for the run and record these fields. Do not
combine samples after changes to machine load, versions, or window geometry.

```text
revision / dirty diff identity:
executable path + hash:
Cargo profile / features / actual GPUI source:
runtime (Bun process, embedded Bun, or QuickJS) / host profile / route / dataset:
viewport logical size / scale / theme / target display rate:
monitor on/off / diagnostic logging on/off:
trigger / duration / warmup / cache state:
background load:
correctness signal:
artifact paths:
```

Use `git rev-parse HEAD`, `git diff --stat`, `Cargo.lock`,
`cargo tree -e features`, and `shasum -a 256 <binary>` to establish the build
identity. HEAD alone does not identify a dirty workspace: retain the relevant
diff or its hash while preserving the user's existing changes. Finish building
before collecting measurements.

## 2. Choose metrics with explicit boundaries

| Metric                   | What it measures                                         | What it cannot establish                                     |
| ------------------------ | -------------------------------------------------------- | ------------------------------------------------------------ |
| Active FPS               | Draw intervals divided by elapsed active time            | Reciprocal CPU cost, display refresh rate, or scanout FPS    |
| CPU draw p50/p95/p99/max | CPU cost of native construction, layout, and painting    | All queueing or input latency, or smoothness by itself       |
| Input-to-present         | GPUI input to the platform presentation boundary         | Wheel-only latency or the time photons reach the display     |
| Dirty-to-present         | Invalidation request to the presentation boundary        | Direct comparisons across idle, startup, or resize intervals |
| Throughput/completion    | Completed work and total elapsed time                    | Whether a long poll starved the UI foreground                |
| TestAppContext timing    | CPU attribution and geometry on a deterministic platform | Real GPU submission, vsync, or trackpad dispatch             |

A frame interval is approximately 8.33 ms at 120 Hz and 16.67 ms at 60 Hz;
choose the target for the actual platform and workload. Brief high FPS can hide
long-tail input latency. Low idle FPS usually means the content has not changed.
A range of interval p95 values is not an overall p95: merge the underlying
distributions before reporting a pooled percentile.

## 3. Compare with and without the live monitor

See [gpui-performance](../crates/gpui-performance/README.md) for the reusable
component and integration. It samples draws passively and freezes its reading
and graph during idle periods. Its README defines the sample window, 500 ms
activity threshold, and inability to distinguish long stalls from idle time.
Use external profiling and input latency to investigate severe stalls.

Debug builds display the monitor by default; release builds hide it. Set
`SOLID_GPUI_PERF_MONITOR=1` or `0` to explicitly enable or disable it in either
build. Confirm the current task names before collecting a comparison:

```sh
# Native interval logging and the live overlay.
SOLID_GPUI_PERF_MONITOR=1 bun run task gallery-profile

# The same workload without the overlay's construction and painting cost.
SOLID_GPUI_PERF_MONITOR=0 bun run task gallery-profile
```

The host's `frame-profile` feature enables interval logging. The FPS overlay
does not start a timer or notification loop. Use the same binary, window,
input, and dataset in both runs, and retain both measurements.

## 4. Reproduction matrix and correctness

Change one factor at a time. For scrolling, cover first entry, sustained
scrolling, scrolling after idle, narrow → wide → narrow → wide resizing, both
sides of breakpoints, nested panes, and navigation. Record the exact route and
target pane. For long lists, also cover the first and last rows, recovery from
empty data, filtering to a small result near the end, and reordering.

Verify that content moves, fixed window elements stay still, independent panes
do not move together, final content remains reachable, and buttons and text
remain present. Wheel logs without content displacement do not reproduce a
scrolling workload. Synthetic wheel input has previously failed to move this
application. In that situation, collect real trackpad input in a dedicated
diagnostic window and record the capture interval; do not keep treating
ineffective synthetic input as a valid test.

The repository's geometry and CPU diagnosis entrypoint is:

```sh
bun run task gallery-scroll-audit
```

It derives routes from `PAGES` and covers narrow windows and retained-window
resizing. This test-platform audit finds problems and checks correctness; it
does not qualify as a production benchmark. Keep correctness assertions active,
and enable timing thresholds explicitly only in a controlled environment.
Do not remove interactions, explanatory text, or meaningful validation to
stabilize timing.

## 5. Attribute cost along the actual path

```text
native input → native state / optional JS event → Solid reactive update
→ commit bytes → native foreground application → invalidation
→ request_layout → layout/measure → prepaint → paint → present
```

Scrolling does not always pass through JavaScript. Inspect the actual path and
measure protocol commits separately from purely native scrolling. See the
[source map](../.agents/skills/gpui-performance/references/sources.md) for source
locations and pinned versions.

On macOS, start with a 15-second main-thread capture using `sample`. Confirm
that the PID belongs to the measured native application, not its Bun child or
another user window. Replace both placeholders with the verified PID and a
new local artifact path:

```sh
sample <native-app-pid> 15 1 -file <new-local-output.sample.txt>
```

Use locally available Instruments/Time Profiler or `xctrace` for more precise
attribution. Keep captures to 15–30 seconds, with an outer timeout of at most
five minutes. Match the binary, dSYM, and architecture. Inspect aggregates and
the target main thread without loading a whole trace into agent context.
If symbol folding or deduplication prevents reliable attribution, build a
separate profiling binary and recapture. State that its compiler options
exclude it from normal timing comparisons. Keep traces local.

Distinguish layout, text shaping, scene construction, foreground polling,
locks/I/O, and GPU work. Make a falsifiable prediction: for example, isolating
an intrinsic-measurement hotspot should reduce layout cost. Minimize the page
only to locate the cause; restore all application behavior in the final fix.
Revert experiments that show no improvement instead of accumulating assumptions.

## 6. Qualify a production benchmark

Before reporting production throughput, scheduling, or rendering performance,
read Zed's `gpui-bench` and check the `bench-support` and `BenchAppContext` APIs
provided by the linked version. Do not invent substitutes for unavailable
upstream profiles or macro parameters.

- Drive real constructors, queues, executors, and rendering through production input boundaries.
- Exclude `test-support` from the full feature graph. Separate test and benchmark packages if needed to avoid Cargo feature unification.
- Run a bounded smoke check before measurement. Run serially and limit each execution to five minutes.
- Separate preparation, measurement, and correctness checks. Fix the dataset size and warm/cold state; retain completion counts and ordering.
- Observe an independent UI signal as well as completion time during foreground-heavy work. Exercise a queue beyond capacity with a sustained producer.
- Treat headless budget overruns without vsync as a proxy metric, not actual display dropped frames.

## 7. Acceptance and documentation

Replay the same workload against the baseline and candidate. Report raw values,
regressions, machine noise, and untested cases. After fixing a reduced case,
return to the original page, native input, and resize sequence. Retain only
key correctness tests, and distinguish user confirmation, native profiles,
and test-platform results.

The conclusion should identify the trigger, the expensive work, why the change
reduces that work, the verified behavior invariants, native reproduction and
acceptance status, reviewable artifacts, and remaining uncertainty.

Put reusable rules in the relevant skill reference and concrete window sizes
and measurements in the incident record. Include applicability conditions and
counterexamples. Replace outdated conclusions so unresolved and subsequently
confirmed claims do not remain side by side without context.
