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
| Observed FPS             | Native presentation event cadence in the sample window  | Full-redraw capacity, physical scanout, or idle baseline with the HUD active |
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

The host uses [gpui-fps](../vendor/gpui-kit/crates/fps/README.md) from the pinned
GPUI Kit tree. A window retains its monitor entity and drops it on close.
`ComponentHost::with_performance_monitor(true)` explicitly enables it; the default
is disabled in every build. The website enables it in its native host.

Our headline starts in observed **FPS**, counted from native presentation events.
Right-click switches to **MAX FPS**, the reciprocal of sampled CPU draw cost,
capped by the display refresh rate. This estimates full-redraw capacity; it is
not observed cadence. **FRAME/P95** describe CPU draw timing, **DROP** counts draws
that exceeded the configured time budget (not compositor-dropped frames), and
**INV** describes native invalidations. CPU/GPU/memory are platform-dependent
process samples. Do not infer physical input latency from any of these readings.

The monitor skips its first eight draw samples and excludes marked HUD-only
readout draws from draw-cost statistics. The visible readout refreshes every
500 ms, including when content is idle, so the HUD itself can cause presentation
events. The clock stops after the HUD is no longer rendered. An inactive but still
rendered window can continue refreshing. This replaces the
old passive-only HUD; idle cadence with the HUD visible is not a baseline.

Compare the same workload with the overlay disabled and enabled. Change only
`with_performance_monitor`, rebuild, retain both binary identities and measurements,
and keep window size, input and data fixed. No environment variable overrides
this application policy. `frame-profile` and `bun run task website-native-profile`
retain separate CPU draw, input-to-present and invalidation-to-present interval
logging. Physical input-to-display acceptance still requires real platform input.


For a serial comparison using one diagnostic binary, build it first and run the
same 360-update, 500-row workload twice. This fixture alone accepts
`SOLID_GPUI_PROFILE_HUD`; it does not change the application's host policy.

```sh
cargo build -p solid-gpui --bin solid-gpui-profile --features frame-profile
SOLID_GPUI_PROFILE_HUD=0 target/debug/solid-gpui-profile bun --conditions=browser scripts/native-commit-profile.ts --native
SOLID_GPUI_PROFILE_HUD=1 target/debug/solid-gpui-profile bun --conditions=browser scripts/native-commit-profile.ts --native
```

Each run ends after approximately 12 seconds and checks every incremental update,
including narrow/wide resizes, and rejects a changed final content viewport.
It exercises native commits and presentation, not
physical trackpad latency. Keep compilation and other measurements out of the runs.

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

The website navigation regression check is:

```sh
bun run task website-navigation-check
```

It mounts the actual website through the native Vite pipeline and checks that
component route changes preserve the sidebar and send incremental updates. This
is a lifecycle correctness check, not a geometry or CPU benchmark. Use the native
measurement workflow above for Showcase scrolling and resize acceptance.

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

## Native commit attribution

Use the production diagnostic host to measure the actual foreground handoff,
decoder, transaction, native reconciliation, and GPUI rendering. It disables the
FPS overlay by default and uses the same `CommitPump` and `SolidRoot` as applications:

```sh
cargo build --locked -p solid-gpui --bin solid-gpui-profile --features frame-profile
cargo tree --locked -p solid-gpui --features frame-profile -e normal,build,features
target/debug/solid-gpui-profile bun --conditions=browser scripts/native-commit-profile.ts --native
```

Finish compilation first and verify that the feature graph excludes `test-support`.
The default native run performs 360 count updates at approximately 30 updates/s,
resizes 800 → 560 → 1280 → 800 pixels wide, and exits. Add `--hold` to keep the
window for manual interaction after the automatic run; close it within five
minutes. Confirm the final counter, button, editable text, actual scroll
movement, and narrow/wide geometry. Programmatic resizing does not establish
physical trackpad or OS live-resize acceptance.

`solid_commit_stages` aggregates elapsed milliseconds and invocation counts on
the foreground thread. Counts follow the printed stage order. `queue` includes
channel backpressure and foreground handoff wait; it excludes time before the
runtime reader receives the frame. `decode` includes the wire guard. `tree`
includes mutation and dependency collection; `dependencies` is its nested
subset. `validate` measures Extension contract validation. `commit` includes
transaction work, validation, native state reconciliation, and notification,
but excludes decode and subsequent window-owned Extension instance updates.
Its count also includes command admission; command execution is outside this
span. Filter the workload when comparing Snapshot/Patch commit costs.
`extensions` measures those instance updates. `render` measures SolidRoot's
preparation and GPUI element construction, not the later layout, paint, GPU, or
presentation stages. Nested stages must not be added to their containing stage.
The intervals are aggregates, not per-commit traces or end-to-end latency.

The same host emits `[solid-gpui-frame-profile]` intervals with CPU draw and
invalidation/input-to-presentation histograms. Use those and a bounded native
CPU sample to distinguish element construction, Taffy layout, text shaping,
prepaint, and scene building. Report inactive/idle intervals separately. The
logger does not create redraws and has no clock-reading cost without the
`frame-profile` feature (or test instrumentation).

The JavaScript fixture measures synchronous signal propagation and signal-to-
encoded-frame delivery separately. The latter includes commit construction,
encoding, and microtask scheduling, and stops before transport submission; it
is not IPC latency or pure Solid runtime cost. Every measured update must emit
one text operation with the expected value. Its memory-only scaling mode is:

```sh
bun --conditions=browser scripts/native-commit-profile.ts
```

For deterministic native CPU attribution, run the opt-in experiment serially:

```sh
cargo test --locked -p solid-gpui --lib snapshot_apply_and_first_draw_scaling_guard -- --ignored --nocapture
```

Both Snapshot and Patch stage measurements call the actual host entrypoints.
Each sample releases its test window so earlier scenes do not accumulate across
later workloads. Commit timing stops inside the GPUI update callback, before TestAppContext can
flush effects and automatically draw. The separately timed draw is a forced CPU
draw with executor draining; it is not a production frame interval. Ordinary
regressions enforce transaction correctness and dependency work bounds instead
of machine-specific millisecond thresholds.

### Native update and cache boundaries

A Patch journals changed nodes and derives original/final ancestors and typed
child ownership dependencies. Successful validation publishes the revision and
one change set; native routes, instances, focus observers, and cache invalidation
consume that set. A text update in a large unrelated tree must not clone that
tree or validate unrelated Extension contracts. Structural edits may still
visit shifted siblings, changed child lists, and dependent compositions.

A smaller commit does not guarantee a cheaper layout. Cached GPUI Entities need
explicit invalidation by their native state owner. In the shared-root pane
experiment, merely wrapping panes in `.cached(style)` suppressed both the static
pane and the changed counter: the native store reached `Count: 12` while the
rendered counter stayed at `Count: 0`. That experiment was rejected. Any future
region design must connect commits **and** native-only input, scrolling,
animations, and asynchronous content to region invalidation, and verify actual
rendered content before accepting timing improvements. Bounds, clipping, and
inherited text styles are additional cache dependencies.
