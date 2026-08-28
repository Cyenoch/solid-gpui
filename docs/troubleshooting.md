# Troubleshooting

This guide is the symptom-first entry point for a React GPUI application. It
sits between the application-level [getting started guide](getting-started.md)
and the protocol reference. Start with the symptom, run the smallest diagnostic
that confirms it, then apply the fix or follow the stated platform boundary.

## Quick symptom directory

| Symptom | Start here |
| --- | --- |
| Window is blank or never appears | [No window / white screen](#no-window-or-white-screen) |
| `Cannot render` or a render-time `TypeError` | [Cannot render](#cannot-render) |
| A press, key, pointer, scroll, or layout callback never runs | [Event does not fire](#event-does-not-fire) |
| A command Promise never settles | [Promise never resolves](#promise-never-resolves) |
| IME, Chinese input, or caret behavior looks wrong | [IME and text input](#ime-and-text-input) |
| Frames are slow or the app stutters | [Low frame rate or stutter](#low-frame-rate-or-stutter) |
| The host crashed or the crash file cannot be found | [Crash reports](#crash-reports) |
| A renderer/host upgrade changes behavior | [Upgrade and compatibility](#upgrade-and-compatibility) |

## No window or white screen

### Symptom

The host process starts but no native content appears, or a window is present
but remains blank. The first thing to establish is whether JavaScript submitted
the initial Snapshot. A renderer entry run directly without a host does not make
a native Surface.

### Most likely causes

- The renderer command or entry path is wrong, so no renderer starts.
- The renderer throws before the first Commit Batch (often a validation error).
- The transport terminates before the host receives the Snapshot.
- The host is running in process mode but the renderer is being run as a
  stand-alone Bun script instead of as the host's child process.

### Verify

First verify the host binary's early CLI path without starting GPUI:

```sh
cargo run -p react-gpui-host -- --version
```

Then run the known-good counter through the host with diagnostics enabled:

```sh
REACT_GPUI_LOG=info \
  cargo run -p react-gpui-host -- --runtime process \
  bun run packages/react-gpui/examples/counter.tsx
```

The `info` stream should show host startup and later runtime termination. Do not
use `bun run packages/react-gpui/examples/counter.tsx` by itself to test native
rendering: the host owns the pipe and Surface lifecycle.

Record the `--version` output with the reproduction. The `info` startup line
includes `protocol=v3` beside the runtime mode, entry, and process ID; the
version line includes the host package version and `protocol=v3`. These values
let support distinguish a version mismatch from a renderer or transport failure.

For a protocol-level check, give the host and its process renderer separate tap
files. Both processes read `REACT_GPUI_TAP`; the `env` wrapper overrides it only
for the child renderer, avoiding two processes truncating one JSONL file:

```sh
tap_dir="${TMPDIR:-/tmp}/react-gpui-troubleshoot-$$"
mkdir -p "$tap_dir"
host_tap="$tap_dir/host.jsonl"
renderer_tap="$tap_dir/renderer.jsonl"
REACT_GPUI_LOG=info REACT_GPUI_TAP="$host_tap" \
  cargo run -p react-gpui-host -- --runtime process \
  env "REACT_GPUI_TAP=$renderer_tap" bun run packages/react-gpui/examples/counter.tsx

python3 scripts/protocol-tap-report.py "$host_tap" "$renderer_tap"
```

Stop the running example after the reproduction with Ctrl-C, then run the
report command. The report records frame metadata, not payload contents. Its
JSON includes frames by kind, overall and patch byte rates, one-second
frame/byte timeline buckets, a byte-size histogram, event-type counts, and
malformed-frame counters. Look for renderer outbound Snapshot/Patch frames,
host inbound frames, and a termination or error record. No renderer Snapshot in
the renderer tap points to renderer startup or pre-submit failure; a renderer
Snapshot with no host progress points to process/host transport or host-side
rejection. The tap is not a paint/GPU profiler and cannot prove that a display
compositor painted a frame. It also does not record transport queue depth,
backpressure, or per-commit timing; `REACT_GPUI_LOG=debug` currently has no
per-commit timing line.

For the in-process runtime, use the same check without a child override:

```sh
REACT_GPUI_TAP="${TMPDIR:-/tmp}/react-gpui-embedded.jsonl" \
  cargo run -p react-gpui-host --features embedded-bun -- \
  --runtime embedded packages/react-gpui/examples/counter.tsx
python3 scripts/protocol-tap-report.py "${TMPDIR:-/tmp}/react-gpui-embedded.jsonl"
```

### Fix or boundary

Correct the host command/entry first. If the renderer throws, continue with
[Cannot render](#cannot-render). If the transport terminates, continue with
[Promise never resolves](#promise-never-resolves) and [Crash reports](#crash-reports).
A successful Snapshot submission is necessary but not sufficient for
compositor/display-backed behavior; Quartz and other visual checks require a
real desktop host.

## Cannot render

### Symptom

`root.render(...)` throws `TypeError`, the application reports `Cannot render`,
or an Error Boundary displays its fallback instead of the intended tree.

### Most likely causes

`validateStyle` and host-property validation intentionally fail before an
invalid Commit Batch is submitted. Common examples are negative dimensions,
non-finite values, zero `fontSize`, invalid enum strings, malformed colors,
unsupported transition properties, malformed `boxShadow`, invalid
`fontFamily`, invalid Image paths, and children under `Image`.

### Verify

Run the renderer validation regression group:

```sh
cd packages/react-gpui
bun test tests/renderer.test.tsx --test-name-pattern "validation errors"
```

For the complete style validator coverage, run:

```sh
bun test tests/renderer.test.tsx --test-name-pattern "validates, freezes"
```

The source of the TypeError family is
`packages/react-gpui/src/style.ts:258-437`; host-kind and Image validation are
in `packages/react-gpui/src/renderer/props.ts`. A stack pointing at
`validateStyle`, `validateProps`, or `createInstance` confirms a synchronous
consumer-input failure rather than a native paint failure.

### Fix or boundary

Fix the named property rather than catching and resubmitting the same tree.
Use an Error Boundary when the application has useful recovery UI; without one,
`root.render()` throws synchronously and does not submit an invalid frame.
The complete field constraints and unsupported fields are in the
[Style](protocol.md#style-tuple-all-42-slots), [host properties](protocol.md#hostproperties-variants),
and [error ownership](getting-started.md#error-and-recovery-boundaries) sections.

## Event does not fire

### Symptom

A callback never runs even though the component is visible, or it fires on one
component kind but not another.

### Most likely causes

1. **The callback is not supported by that kind.** `onPress` belongs to
   `Pressable`; `View` has key, pointer, hover, scroll, drag, and layout paths
   but no press callback. `Text` and `Image` expose layout; `TextInput` does
   not expose layout. `VirtualList` reports `VisibleRange` and has list scroll
   commands rather than `onScroll`. `onKeyDown` is supported on View,
   Pressable, and TextInput; pointer/hover/drag handlers are View/Pressable
   paths.
2. **The listener is absent or the node is not eligible.** A listener ID is
   installed only for the declared callback and supported kind. Disabled
   Pressables lose interaction and focus. A View key listener also requires
   `focusable` to opt into the native tab/focus path.
3. **The callback changed across commits.** While a node remains mounted, a
   function replacement updates the JavaScript callback table without changing
   the listener ID. Adding or removing the listener (the 0↔nonzero transition)
   changes the native listener field and is applied by the next commit. Events
   for an old revision, detached node, or mismatched listener are rejected.
4. **The event is a semantic notification, not a browser event.** There is no
   capture/bubble DOM contract or synchronous `preventDefault`; TextInput/IME
   preference is checked before keymap dispatch.

### Verify

Use the focused headless paths that inject real protocol events:

```sh
cd packages/react-gpui
bun test tests/renderer.test.tsx --test-name-pattern "dispatches focused View key"
bun test tests/renderer.test.tsx --test-name-pattern "dispatches pointer buttons"
bun test tests/renderer.test.tsx --test-name-pattern "dispatches View scroll"
bun test tests/renderer.test.tsx --test-name-pattern "dispatches internal drag"
```

For a source-level listener check, inspect the kind matrix in
`packages/react-gpui/src/renderer/props.ts` and callback/listener assignment in
`packages/react-gpui/src/renderer/nodes.ts`. For a live process, use the
separate-file tap command from [No window or white screen](#no-window-or-white-screen)
and inspect event subtype counts. A tap event still does not contain callback
payloads.

### Fix or boundary

Move the callback to a supported host kind, add the required `focusable` opt-in,
or wait for the commit that installs the listener before expecting a native
notification. Keep callback identity changes separate from listener presence
changes when diagnosing revision timing. For unsupported behavior, the support
matrix in [getting started](getting-started.md#components-and-common-props) and
the event directory in [protocol.md](protocol.md#3-event-directory) are the
contract. `pointerEvents` is intentionally not exposed; the boundary is
explained in the [package README](../packages/react-gpui/README.md#styles).

## Promise never resolves

### Symptom

A root command or node command returns a Promise that appears to remain pending.

### Most likely causes

- The host never received or never processed the command, so no
  `CommandResult` arrived.
- A native picker is still open. File dialogs are asynchronous and wait for
  user completion; cancellation is a successful result with a missing value.
- The surface closed or the Runtime Adapter terminated while the command was
  pending. Those paths reject pending Promises; they do not resolve them with a
  fake success.
- The command/node pair is unsupported or the node is detached. The TypeScript
  side rejects these before framing, and the host repeats ownership checks.

### Verify

Run the lifecycle regression paths:

```sh
cd packages/react-gpui
bun test tests/renderer.test.tsx --test-name-pattern "rejects pending commands"
bun test tests/renderer.test.tsx --test-name-pattern "surface close"
bun test tests/transport.test.ts --test-name-pattern "transport termination"
```

In a live process, run the tap report and inspect its `command_success` summary.
An unmatched command request means no matching result was observed; a matched
`success=false` result means the Promise did settle and the native contract
rejected the operation. Payload bytes are deliberately absent from the report,
so use the command ID and application logs to identify the call.

### Fix or boundary

Keep one root/surface alive until asynchronous commands complete, and always
handle both resolve and reject paths. Do not assume a picker cancellation is an
error: `null` is the documented cancellation value. Treat
`TransportTerminatedError` as terminal for that adapter; create a new root and
adapter rather than retrying onto a failed stream. The lifecycle ownership and
all root/node command restrictions are in [protocol.md](protocol.md#4-command-directory).

## IME and text input

### Symptom

Chinese/Japanese/Korean composition, marked text, candidate placement, caret
movement, or multiline selection appears wrong.

### Most likely causes

The renderer deliberately has a mixed-precision text contract:

- TextInput selection and edit positions are UTF-16 code units, not UTF-8 bytes
  or Unicode scalar counts. `maxLength` uses the same UTF-16 unit contract.
- `onSelectionChange` carries `start`, `end`, and `reversed`; TextInput event
  payloads always include the `reversed` boolean.
- TextInput/IME takes precedence before keymaps; cached GPUI shaped layouts
  cover single-line and multiline/newline UTF-16 positions and point lookup.
- IME candidate placement remains approximate and display-backed. Placeholder
  geometry still falls back to element bounds.
- There is no browser composition event or synchronous event cancellation
  surface, and `setSelection(start, end)` sets an ordered range but does not set
  selection orientation.

### Verify

Run the UTF-16/selection regression group:

```sh
cd packages/react-gpui
bun test tests/renderer.test.tsx --test-name-pattern "selection"
cargo test -p react-gpui multiline_utf16_positions_cover_emoji_empty_lines_and_trailing_newline --locked
```

Read the exact current boundaries in
[TextInput behavior](../packages/react-gpui/README.md#scroll) and the
[protocol event directory](protocol.md#3-event-directory). Confirm that
application state treats selection offsets as UTF-16 units and that marked
ranges are preserved until native composition changes them.

### Fix or boundary

Do not convert selection offsets using UTF-8 byte positions. Preserve the
`reversed` bit when displaying a selection direction, and use the native text
value delivered by `onChangeText`/submit rather than a stale closure. Multiline
caret and point-to-character mapping use cached wrapped GPUI geometry, including
empty/trailing-newline lines. IME candidate placement remains a known
display-backed approximation; verify it on a real desktop adapter.

## Low frame rate or stutter

### Symptom

The application stutters, emits too many commits/events, or appears to spend
unexpected time in the transport.

### Most likely causes

- A component is generating large Snapshot/Patch payloads instead of keeping
  updates local to the changed subtree.
- A native event source is producing an event storm (for example repeated
  pointer, scroll, layout, or key notifications).
- The observed cost is actually native layout/paint or a display compositor,
  which the protocol tap does not measure.
- Tap output is being written to a slow or shared path, or the bounded tap has
  reached its 64 MiB capacity.

### Verify

For a bounded leak/transport smoke, run:

```sh
make soak-smoke
```

For a reproduction with protocol metadata, use the separate host/renderer tap
command from [No window or white screen](#no-window-or-white-screen), then:

```sh
python3 scripts/protocol-tap-report.py /tmp/react-gpui-troubleshoot-*/host.jsonl \
  /tmp/react-gpui-troubleshoot-*/renderer.jsonl
```

Read `frame_rate_hz`, `bytes_by_kind`, event subtype counts,
`command_success`, and frame-interval `p50`/`p95`. Large Patch byte totals point
to commit breadth; high event counts/rates point to a notification storm;
unmatched commands point to a missing lifecycle receipt. The report records
metadata only and cannot attribute time to GPUI layout, paint, or GPU work.

### Fix or boundary

Reduce the React commit scope and avoid sending state updates for every native
notification unless the application needs them. Keep tap files per process and
remove or rotate a file after a reproduction. Treat `make soak-smoke` as a
bounded leak smoke, not multi-hour performance proof; for native paint or
compositor issues, use a display-backed profiler.

## Crash reports

### Symptom

The host exits, stderr contains a panic or fatal runtime message, or the crash
file is not where expected.

### Most likely causes

- A GPUI paint panic or host invariant failure terminated the host.
- A Snapshot/Patch validation failure or transport termination took the fatal
  path; these are not recovered by resynchronizing the shared stream.
- `REACT_GPUI_CRASH_DIR` points to a directory the host cannot create/write.
- The process ended normally (for example explicit shutdown), so no panic report
  was expected.

### Verify

Reproduce with an explicit crash directory and a full Rust backtrace:

```sh
crash_dir="${TMPDIR:-/tmp}/react-gpui-crashes"
mkdir -p "$crash_dir"
RUST_BACKTRACE=full REACT_GPUI_CRASH_DIR="$crash_dir" REACT_GPUI_LOG=info \
  cargo run -p react-gpui-host -- --runtime process \
  bun run packages/react-gpui/examples/counter.tsx
printf '%s\n' "$crash_dir"/react-gpui-host-*.log
```

The host's standard panic hook keeps the original panic on stderr and writes
`react-gpui-host-<pid>-<timestamp>.log`. The report contains version/platform,
panic location, and `Backtrace::capture()` output. If it cannot write, stderr
prints `react-gpui-host: unable to write crash report: ...`.

A process renderer's `TransportTerminatedError` also carries an integer host
exit code and the last 50 stderr lines when the host supplies them. Handle
`onTransportTermination` and log `error.exitCode` and `error.stderrTail`; do
not treat a retained failed transport as a recoverable command result.

### Fix or boundary

Use `REACT_GPUI_LOG=info` for startup/termination context and `debug` for the
same diagnostics alongside fatal context. Preserve the crash file and stderr
tail together when filing a report. A GPUI paint panic has no safe node-level
resume path; a shared Runtime Adapter failure closes every registered surface.
Display/compositor behavior still requires a real desktop runner.

## Upgrade and protocol cutover

### Symptom

After upgrading one side of the renderer/host, current frames are rejected, a
notification action disappears, selection direction changes, or resize callbacks
receive the wrong arity/value.

### Verify

Regenerate and run both protocol golden directions after an intentional wire
contract change:

```sh
make protocol-golden-generate
cargo test -p react-gpui --test protocol_golden --locked
cd packages/react-gpui
bun test tests/protocol-golden.test.ts
```

The current v3 contract uses one shape for each required positional payload:

- **TextInput:** eight slots, with `reversed` in the final slot.
- **CommandResult:** seven slots, with a nullable typed-value slot.
- **Submit:** a string payload, including the empty string.
- **Notifications:** `[title, body]` and action-extended command forms remain
  valid because optional action data is intentionally emitted by current
  encoders; action responses are a separate Event 21 path.
- **Window resize:** three slots `[width,height,scaleFactor]`; scale-only
  changes are reported.

Run focused protocol checks when diagnosing one of these cases:

```sh
cargo test -p react-gpui protocol_v3_host_properties_and_event_payload_tags_round_trip --locked
cargo test -p react-gpui notification_and_menu_commands_and_action_events_round_trip --locked
cargo test -p react-gpui window_resize_wire_accepts_scale_factor --locked
```

### Fix or boundary

Upgrade the renderer and host together after a protocol cutover. Do not add
decoder fallbacks or default-fill removed fields; update both current
encoders, decoders, tests, and golden fixtures as one reviewed change. The
authoritative slot, event, and command rules are in
[protocol.md](protocol.md).

## Known boundaries worth checking first

- AX disabled state is retained and validated, but the pinned GPUI public
  builder does not expose a disabled-state mapping. Verify the final AccessKit
tree on a display-backed host; this is a documented platform/API boundary, not
  a missing callback retry. See [package accessibility](../packages/react-gpui/README.md#accessibility).
- `pointerEvents` has no declarative prop. Native normal hitboxes already allow
  basic pass-through when no listener is installed; partial occlusion semantics
  are not represented by this protocol.
- `Root.zoom()` has headless command coverage, but its visual effect remains
  display-backed because the pinned TestWindow does not implement native zoom.
- Image `onError` remains a true upstream gap. `fallbackSource` is the visual
  loading/error degradation path; without it, a failed image remains blank.
- `VirtualList` requires a bounded viewport and uses native variable-height
  measurement; `estimatedItemSize` is an initial hint, not a fixed row height.

For a complete field-level contract, use [protocol.md](protocol.md). For
consumer recipes, use [getting-started.md](getting-started.md). For environment
variables and opt-in tap/crash diagnostics, this repository's root
[README](../README.md#debugging) remains the quick reference.
