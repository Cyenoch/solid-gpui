# Troubleshooting

This guide is the symptom-first entry point for a React GPUI application. It
turns the renderer's typed errors and platform boundaries into a short lookup:
find the symptom, check the smallest diagnostic, and apply the documented fix.
For the complete wire contract, see the [protocol reference](protocol.md). For
a first application, see [getting started](getting-started.md).

## Contents

- [Start with the host identity](#start-with-the-host-identity)
- [Transport terminated](#transport-terminated)
- [Protocol version mismatch](#protocol-version-mismatch)
- [Malformed or oversized frames](#malformed-or-oversized-frames)
- [Surface is closed or an ID cannot be reused](#surface-is-closed-or-an-id-cannot-be-reused)
- [A custom font is not used](#a-custom-font-is-not-used)
- [A file or clipboard image is too large](#a-file-or-clipboard-image-is-too-large)
- [Clipboard image is unsupported on this platform](#clipboard-image-is-unsupported-on-this-platform)
- [No GUI in headless macOS CI](#no-gui-in-headless-macos-ci)
- [Close confirmation appears to hang](#close-confirmation-appears-to-hang)
- [A command Promise rejects or remains pending](#a-command-promise-rejects-or-remains-pending)
- [The tree cannot render](#the-tree-cannot-render)
- [An event callback does not fire](#an-event-callback-does-not-fire)
- [Text input or IME behavior is wrong](#text-input-or-ime-behavior-is-wrong)
- [The app stutters](#the-app-stutters)
- [Crash reports](#crash-reports)

## Start with the host identity

### Symptom

The window does not appear, or support logs contain a renderer/host compatibility
question. Before inspecting a crash or a protocol trace, the host and renderer
versions need to be known.

### Cause

The host prints the protocol identity on its version path, and its `info`
startup diagnostic prints the same protocol identity with the runtime mode,
entry, and process ID. A renderer started directly without the host does not
create a native Surface.

### Fix

Run the host's early diagnostic (it does not start GPUI):

```sh
react-gpui-host --version
```

The output has this shape:

```text
react-gpui-host <host-package-version> protocol=v3
```

For a process run, add `REACT_GPUI_LOG=info` and retain the startup line too:

```sh
REACT_GPUI_LOG=info react-gpui-host --runtime process -- \
  bun run path/to/app.tsx
```

Include both lines in a support report. Run the renderer through the host, not
as a stand-alone `bun run` command.

**Where this is enforced:** `crates/react-gpui-host/src/main.rs`
(`version_line`, `startup_diagnostic`, and host argument handling), and
`README.md` (host diagnostics and process-mode quick start).

## Transport terminated

### Symptom

`onTransportTermination` receives a `TransportTerminatedError`; pending root
commands reject with that same error, and later commands cannot use the failed
transport. The error has a discriminated `cause` when the termination reason is
known. `exitCode`, `stderrTail`, and `crashReportPath` may also be available for
a process transport.

### Cause

`error.cause?.kind` is one of these five observable cases:

| Kind | Meaning and typical trigger | Application response |
| --- | --- | --- |
| `shutdown` | The adapter or shared surface host was intentionally disposed or shut down. | Do not restart unless the application intentionally wants a new session. |
| `eof` | The input or output stream ended or closed. | Treat the root as terminal; if the child was expected to live, inspect its process lifecycle and start a fresh session. |
| `exit` | A process wrapper reported an integer child exit code. | Record `error.exitCode` and the stderr tail, then decide whether the application should start a fresh child. |
| `protocol` | A frame decoder or event decoder rejected a frame, including a malformed, oversized, or incompatible frame. `detail` identifies the rejection. | Fix the protocol or version problem. Do not continue reading or retry on the same transport. |
| `io` | An input/output operation failed, or an untyped termination was normalized to an I/O detail. | Record `detail` and the stderr/process context, then replace the failed transport if recovery is appropriate. |

The variants are diagnostics, not a restart policy. All failed transports and
roots are terminal; a replacement consists of a new child (when applicable),
new `StdioTransport`, and new root. An intentional `shutdown` is the exception
to an automatic restart decision.

### Fix

Branch on the cause kind instead of matching `error.message`:

```tsx
onTransportTermination: (error) => {
  switch (error.cause?.kind) {
    case "shutdown":
      return;
    case "exit":
      console.error("host exit", error.exitCode, error.stderrTail);
      break;
    case "protocol":
    case "io":
    case "eof":
    default:
      console.error("transport terminated", error.cause, error.stderrTail);
  }
  // If recovery is desired, create a new child, transport, and root.
}
```

`createProcessTerminationHandler()` is suitable for examples that should log
the error and exit with code `1`; it is not a transport restart mechanism.

**Where this is enforced:** `packages/react-gpui/src/transport.ts`
(`TransportTerminationCause`, `TransportTerminatedError`, and
`StdioTransport`), `packages/react-gpui/src/renderer/root-container.ts`
(termination rejects pending commands), and the package README's
[transport troubleshooting](../packages/react-gpui/README.md#troubleshooting).

## Protocol version mismatch

### Symptom

A `ProtocolVersionMismatchError` is reported while decoding a host event. Its
message says that the host speaks one protocol version while the renderer
package speaks another.

### Cause

The current renderer package speaks protocol v3. When an event contains a
different integer protocol version, `decodeEvent` raises the typed error rather
than silently treating the frame as a current event.

### Fix

Update `@react-gpui/core` to a release matching the host's reported version, or
pin the host binary to a v3 release. The actionable error message is:

```text
protocol version mismatch: host binary speaks protocol v<N>; this renderer package speaks protocol v3 — update @react-gpui/core to a v<N> release / pin the host binary to a v3 release
```

Upgrade the host and renderer together. Do not add a decoder fallback or keep
using the terminated transport.

**Where this is enforced:** `packages/react-gpui/src/protocol.ts`
(`ProtocolVersionMismatchError` and `decodeEvent`),
`crates/react-gpui/src/protocol/wire/{event.rs,snapshot_patch.rs,command.rs}`
(`UnsupportedProtocol`), and `packages/react-gpui/tests/renderer.test.tsx`
(the actionable mismatch contract).

## Malformed or oversized frames

### Symptom

The host exits after rejecting a renderer commit, or the renderer reports a
`TransportTerminatedError` with `cause.kind === "protocol"`. Pending commands
reject and later input is ignored. A panic may additionally produce a crash
report path on stderr.

### Cause

The two directions fail fast at their own boundary:

- A malformed renderer-to-host Snapshot, Patch, or Command is reported as a
  rejected renderer commit. The host shuts down the runtime and exits nonzero;
  it does not skip the frame and continue with a potentially divergent tree.
- A malformed or oversized host-to-renderer event causes the TypeScript root or
  shared surface host to terminate with a `protocol` cause. With
  `createProcessTerminationHandler`, the renderer process logs the termination
  and exits with code `1`.
- A panic is a separate host failure path. The panic hook writes a report and
  prints this exact stderr marker:

  ```text
  react-gpui-host: crash report: <path>
  ```

  A malformed commit handled by `fatal_runtime_failure` is not a panic, so it
  has no crash-report path merely because the protocol was rejected.

### Fix

Preserve the complete stderr tail, the `cause.detail`, and any crash-report
path. Fix the producer/host mismatch or malformed payload, then start a fresh
process, transport, and root. Do not retry a command on the failed stream or
expect a resynchronization frame.

If a crash marker is present, open the path it names. The host uses
`REACT_GPUI_CRASH_DIR` when set and otherwise the system temporary directory.
The marker may be absent when the process was killed, exited normally, or the
host could not write the report; keep the exit code and stderr in those cases.

**Where this is enforced:** `crates/react-gpui/src/renderer/commit_reader.rs`
and `crates/react-gpui/src/transport.rs` (`fatal_runtime_failure`),
`crates/react-gpui-host/src/main.rs` (commit-reader fatal path and panic hook),
`packages/react-gpui/src/renderer/root-container.ts` and
`packages/react-gpui/src/surface-host.ts` (protocol termination), and
`packages/react-gpui/src/protocol.ts` (`FrameDecoder` size checks).

## Surface is closed or an ID cannot be reused

### Symptom

A command rejects with `SurfaceClosedError`, or a shared surface host throws
`SurfaceIdReusedError` when creating a root for an ID that was already closed.
A close callback may run for one surface while other roots on the same host
continue working.

### Cause

A native `EVENT_SURFACE_CLOSED` disposes only the matching root. Disposal
rejects its pending commands with an error whose message is
`surface <id> is closed`, clears its listeners, and ignores later input. An
explicit `root.unmount()` has the same closed-root behavior.

`createSurfaceHost` retires an ID after native close or unmount. Reusing that ID
with a new epoch is deliberately rejected with:

```text
surface <id> was already closed and cannot be reused
```

This prevents an old lifecycle generation from being confused with a new
surface. An active duplicate ID is a separate registration error.

### Fix

Stop issuing commands to the closed root. Handle `onClose` and remove the root
from application state. For a new window, let `createSurfaceHost` allocate the
next ID or choose an ID that has never been registered; do not revive a retired
ID by changing only `epoch`.

**Where this is enforced:** `packages/react-gpui/src/renderer/root-container.ts`
(`SurfaceClosedError`, `onSurfaceClosed`, and `dispose`),
`packages/react-gpui/src/surface-host.ts` (`retiredSurfaceIds` and
`SurfaceIdReusedError`), and `packages/react-gpui/tests/{renderer,surface-host}.test.ts*`.

## A custom font is not used

### Symptom

Text continues to use the fallback family even though `loadFont()` was called,
or the font Promise rejects with a file/format error.

### Cause

The host reads and validates a TrueType/OpenType file, extracts a usable family
name, registers it, and returns that metadata family. A missing/unreadable
file, directory, oversized file, malformed font, or font with no usable family
rejects the command. GPUI caches both successful and failed family resolution;
this API does not invalidate that cache. Loading after text has already been
laid out can therefore leave that text on the fallback family.

### Fix

For deterministic startup typography, await registration before the first
render and use the returned family:

```tsx
const family = await root.loadFont(fontPath);
root.render(<Text style={{ fontFamily: family }}>Custom typography</Text>);
```

If startup should not wait, deliberately render the first frame with the
fallback stack and switch state after registration. The examples fire the
Promise before rendering and set state when it settles:

```tsx
const fontReady = root.loadFont(FONT_PATH);
root.render(<TwoInputs fontReady={fontReady} />);

// In the component:
useEffect(() => {
  let mounted = true;
  void fontReady
    .then((family) => {
      if (mounted) setFontFamily(family);
    })
    .catch((error: unknown) => {
      if (mounted) setFontStatus(`Font load failed: ${String(error)}`);
    });
  return () => {
    mounted = false;
  };
}, [fontReady]);
```

Do not assume that a late retry will invalidate an earlier failed lookup. Fix
the path or font and use a fresh, intentional registration/render boundary.

**Where this is enforced:** `packages/react-gpui/src/renderer/root-container.ts`
(`loadFont` validation), `crates/react-gpui/src/renderer/commands.rs`
(`spawn_load_font` and `font_family`), `references/zed/crates/gpui/src/text_system.rs`
(`font_ids_by_font` caches both `Ok` and `Err` resolution results),
`packages/react-gpui/examples/text-input.tsx` (the fire-Promise-then-setState
pattern), and the package README's
[Runtime fonts](../packages/react-gpui/README.md#runtime-fonts) guidance.

## A file or clipboard image is too large

### Symptom

A file or encoded clipboard image command rejects with a size error, or a
manually framed payload throws a `RangeError`.

### Cause

The complete frame payload is capped at `MAX_FRAME_SIZE` (16 MiB). File and
clipboard-image payloads reserve 1 KiB for the MessagePack command/frame
envelope, so each operation's content cap is:

```text
MAX_FRAME_SIZE - 1 KiB
```

The affected operations are:

| Operation | Observable error |
| --- | --- |
| `readTextFile(path)` | The host rejects an oversized file with `file is too large`; invalid UTF-8 is a separate `file is not valid UTF-8` rejection. |
| `writeTextFile(path, content)` | The TypeScript guard rejects with `RangeError: file content exceeds the supported size`; the host-side result is `file content is too large`. |
| `setClipboardImage({ format, bytes })` | The TypeScript guard rejects with `RangeError: clipboard image bytes must be non-empty and within the supported size`; the host validates the same bound. |
| `getClipboardImage()` | An encoded native image over the bound rejects with `clipboard image is unsupported or too large`. |
| `framePayload(payload)` or inbound decoding | A frame over 16 MiB throws `frame exceeds maximum size`, or `frame length <N> exceeds maximum <N>` while decoding. |

### Fix

Keep text/file content below the operation cap. For generated visual assets,
persist the encoded image to a host-visible file and pass its path to `Image`
instead of putting image bytes in the retained tree. For clipboard images,
keep the original supported PNG, JPEG, GIF, or SVG encoding and split or
externalize application data before crossing this bounded seam. Do not raise
the limit on only one side of the process boundary.

**Where this is enforced:** `packages/react-gpui/src/protocol.ts`
(`MAX_FRAME_SIZE`, `MAX_*_BYTES`, `framePayload`, and `FrameDecoder`),
`packages/react-gpui/src/renderer/root-container.ts` (client guards),
`crates/react-gpui/src/protocol.rs` (Rust constants and framed I/O), and
`crates/react-gpui/src/renderer/commands.rs` (file and image command results).

## Clipboard image is unsupported on this platform

### Symptom

`setClipboardImage()` or `getClipboardImage()` rejects with an Error whose
message is exactly `platform-unsupported` on X11 or Wayland.

### Cause

The host's image clipboard branch is implemented for macOS and Windows. On
other targets it returns a failed `CommandResult` with the literal
`platform-unsupported`; the TypeScript command Promise rejects that result. It
does not silently convert image bytes to text, and no RGBA conversion or format
transcoding is promised.

### Fix

Handle this rejection as an expected capability result. Use clipboard text when
that is sufficient, or persist/share the asset through a host-visible file and
let the application decide how to present it. Do not retry the same image
command expecting a different result on the same X11/Wayland host.

**Where this is enforced:** `crates/react-gpui/src/renderer/commands.rs`
(`COMMAND_CLIPBOARD_WRITE_IMAGE` and `COMMAND_CLIPBOARD_READ_IMAGE`),
`packages/react-gpui/src/renderer/root-container.ts`, and the package README's
[Clipboard images](../packages/react-gpui/README.md#clipboard-images) section.

## No GUI in headless macOS CI

### Symptom

A headless CI job cannot run, or cannot re-test, display-dependent GPUI
renderer/host smoke paths. This symptom alone does not establish that the GUI
path is broken.

### Cause

The current Quartz validation environment can have no active display. In that
environment, display-dependent paths are not re-tested; the repository's
non-GUI checks remain available. The failure story for a particular compositor
or window-server setup is environment-specific, so this guide does not invent a
single error string.

### Fix

Run display-backed renderer/host smoke tests on a macOS environment with an
active display. In headless CI, use the checks that do not require Quartz,
including host `--help`/`--version`, process/runtime checks, CLI parsing, and
package smoke checks. Report the display environment separately from protocol
or renderer failures.

**Where this is enforced:** `.scratch/release-productionization/issues/04-quartz-no-display.md`
(the active-display boundary and available non-GUI coverage), and
`README.md` (display-backed validation status).

## Close confirmation appears to hang

### Symptom

Closing a window does nothing, repeated close attempts do not produce more
callbacks, or an asynchronous confirmation UI seems stuck.

### Cause

With `require-confirmation`, the host synchronously vetoes the native close,
stores one pending request ID, and emits one `EVENT_CLOSE_REQUESTED`. While that
request is pending, repeated native attempts are vetoed without duplicate
events. The window therefore remains open until JavaScript resolves that exact
request. A missing `onCloseRequested` handler, a forgotten Promise callback, a
rejected confirmation Promise with no fallback, or a root/transport that has
already become unusable can leave the application with no successful resolution
path. There is no automatic timeout in this contract.

Unknown, stale, or already-resolved IDs are acknowledged or ignored without
affecting the current request. Changing policy and tearing down the transport
clear pending close state; application quit and Wayland layer-shell teardown
are outside this per-window callback contract.

### Fix

Always resolve every request, including the rejection path of the confirmation
operation. Resolve `false` to keep the window open or `true` to allow the host's
close path:

```tsx
onCloseRequested: (requestId) => {
  void confirmDiscard()
    .then((allow) => root.resolveCloseRequest(requestId, allow))
    .catch(() => root.resolveCloseRequest(requestId, false));
},
```

Keep the request ID paired with the root that received it, and do not reuse an
old ID after the surface closes. If the application does not need a prompt, use
the default `allow` policy rather than installing a confirmation flow that has
no resolver.

**Where this is enforced:** `crates/react-gpui-host/src/main.rs`
(`should_close`, `resolve_close_request`, and per-surface pending state),
`docs/adr/0009-async-close-confirmation.md`, and
`packages/react-gpui/src/renderer/root-container.ts` (`resolveCloseRequest`).

## A command Promise rejects or remains pending

### Symptom

A root or node command rejects, or a Promise appears not to settle while a
native operation is in progress.

### Cause

File pickers are asynchronous: a user cancellation is a successful `null`
result, while a native failure rejects. A surface close or transport
termination rejects every pending command. Unsupported command/node pairs are
rejected before or at the host ownership check. A genuinely pending command
usually means the host has not produced its matching `CommandResult` yet; a tap
report can distinguish an unmatched request from a matched `success=false`
result.

### Fix

Handle both resolve and reject paths for every command, keep the root alive until
an asynchronous operation completes, and treat `TransportTerminatedError` and
`SurfaceClosedError` as terminal lifecycle signals. Do not convert a failed
transport into a fake command success or retry a command on the same stream.
For a picker, handle `null` as cancellation rather than as a failure.

**Where this is enforced:** `packages/react-gpui/src/renderer/root-container.ts`
(pending command map and rejection paths), `crates/react-gpui/src/renderer/commands.rs`
(asynchronous command acknowledgements), and `packages/react-gpui/README.md`
(command and transport lifecycle guidance).

## The tree cannot render

### Symptom

`root.render(...)` throws a `TypeError`, the application reports `Cannot render`,
or an Error Boundary shows its fallback instead of the intended tree.

### Cause

The TypeScript renderer validates styles, host properties, and children before
submitting a Commit Batch. Negative or non-finite values, zero `fontSize`,
invalid enum values/colors, malformed shadows, invalid font-family values,
invalid image paths, and children under `Image` are examples of synchronous
consumer-input failures. An invalid batch is not sent to the host.

### Fix

Fix the named property rather than catching and resubmitting the same tree.
Use an Error Boundary when the application has useful recovery UI. Consult the
[style tuple](protocol.md#style-tuple-all-42-slots) and
[host properties](protocol.md#hostproperties-variants) sections for field
constraints.

**Where this is enforced:** `packages/react-gpui/src/style.ts`,
`packages/react-gpui/src/renderer/props.ts`, and
`packages/react-gpui/src/renderer.ts` (synchronous render error ownership).

## An event callback does not fire

### Symptom

A callback never runs even though its component is visible, or it runs for one
component kind but not another.

### Cause

Callbacks are supported only on their documented host kinds. For example,
`onPress` belongs to `Pressable`; View keyboard delivery requires `focusable`;
disabled Pressables lose interaction and focus; `TextInput` and `VirtualList`
have their own event/command boundaries. Native events are semantic
notifications, not bubbling DOM events, and there is no synchronous
`preventDefault()`.

### Fix

Move the callback to a supported host kind, add the required `focusable` opt-in,
or wait for the commit that installs the listener. Check the component matrix in
[getting started](getting-started.md#components-and-common-props) and the
[event directory](protocol.md#3-event-directory). For unsupported behavior,
treat the documented boundary as a contract rather than a callback retry.

**Where this is enforced:** `packages/react-gpui/src/renderer/props.ts` and
`packages/react-gpui/src/renderer/nodes.ts` (listener eligibility),
`packages/react-gpui/src/renderer/dispatch.ts` (event dispatch), and
`docs/protocol.md` (event ownership).

## Text input or IME behavior is wrong

### Symptom

Chinese/Japanese/Korean composition, marked text, caret movement, or multiline
selection appears wrong.

### Cause

TextInput selection and edit positions use UTF-16 code units, not UTF-8 bytes
or Unicode scalar counts. `onSelectionChange` also carries the `reversed`
head-orientation bit. TextInput/IME handling takes precedence before keymaps;
display-backed candidate placement remains approximate, and this renderer does
not expose browser composition events or synchronous cancellation.

### Fix

Keep selection offsets in UTF-16 units, preserve `reversed` when displaying
selection direction, and use the native text delivered by `onChangeText` or
submit. Verify candidate placement and final text geometry on a real desktop
adapter rather than a headless backend.

**Where this is enforced:** `packages/react-gpui/src/renderer/props.ts`,
`packages/react-gpui/src/protocol.ts` (TextInput event validation),
`crates/react-gpui/src/renderer/paint/text_input.rs`, and the TextInput sections
of `docs/protocol.md` and `packages/react-gpui/README.md`.

## The app stutters

### Symptom

The application emits unexpectedly many commits/events, or the protocol tap
shows large frames and high event rates.

### Cause

Large Snapshot/Patch payloads, high-frequency pointer/scroll/layout/key
notifications, or a slow/shared tap path can be responsible. The protocol tap
records frame and event metadata; it does not measure GPUI layout, paint, GPU,
compositor, transport queue depth, or per-commit time.

### Fix

Reduce React commit scope, avoid state updates for notifications the application
does not need, and use one tap file per process. Compare frame sizes and event
counts with:

```sh
python3 scripts/protocol-tap-report.py /path/to/tap.jsonl
```

Use a display-backed profiler for native paint or compositor work. The tap is a
diagnostic aid, not proof of a GUI frame-rate problem.

**Where this is enforced:** `packages/react-gpui/src/protocol-tap.ts`,
`scripts/protocol-tap-report.py`, and `README.md` (tap limits and reported
fields).

## Crash reports

### Symptom

The host exits, stderr contains a panic or fatal runtime message, or a crash
file is not where expected.

### Cause

Host panics use the panic hook described in [Malformed or oversized
frames](#malformed-or-oversized-frames). Fatal protocol/transport failures call
the host's fail-fast exit path and are not recovered by resynchronizing the
stream. A report may be absent when the process ended normally, was killed
without the panic hook, or could not create/write its report directory.

### Fix

Set an explicit directory and preserve stderr while reproducing:

```sh
crash_dir="${TMPDIR:-/tmp}/react-gpui-crashes"
mkdir -p "$crash_dir"
RUST_BACKTRACE=full REACT_GPUI_CRASH_DIR="$crash_dir" \
  REACT_GPUI_LOG=info react-gpui-host --runtime process -- \
  bun run path/to/app.tsx
```

Keep the `react-gpui-host: crash report: <path>` line, the report file, exit
code, and stderr tail together. A shared Runtime Adapter failure closes all
roots using that adapter; recovery requires a fresh adapter and root.

**Where this is enforced:** `crates/react-gpui-host/src/main.rs` (panic hook
and `REACT_GPUI_CRASH_DIR`), `crates/react-gpui/src/transport.rs`
(`fatal_runtime_failure`), and `packages/react-gpui/src/transport.ts`
(stderr-tail and crash-path extraction).

For field-level constraints and wire ownership, use [protocol.md](protocol.md).
For application recipes, use [getting-started.md](getting-started.md). For
environment variables and tap reporting, see the root
[Debugging section](../README.md#debugging).
