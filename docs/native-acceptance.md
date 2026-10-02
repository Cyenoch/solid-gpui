# Native acceptance testing

Retained native primitive regions replay the observations captured by the same
painted scene generation. Cache misses record fresh text and clipped geometry;
hits restore those painted facts without reading current store text as proof
of paint. Removing a region removes its observations on the next frame. This
keeps locators and geometry available across unrelated commits and allows
painted-content checks to detect stale region updates.

`NativeAcceptance` from `@solid-gpui/core/testing` lets a Bun test own an explicitly
launched native host. The application still uses `createRoot`, the ordinary
Snapshot/Patch decoder, production `NativeStateRegistry`, `SolidRoot`, GPUI
layout/paint, and native input handlers. Solid owns application state and callback
generations; GPUI owns editing, hit testing, selection, scrolling, and rendering.

`TestHost` remains a semantic protocol helper over `MemoryTransport`. It inspects
submitted styles and injects semantic events without native geometry. Use native
acceptance when an assertion depends on layout, paint, or native input routing.

## Build and launch

Build the opt-in executable with the matching native sources:

```sh
CARGO_BUILD_JOBS=2 cargo build -p solid-gpui --features native-acceptance --bin solid-gpui-acceptance
```

The Cargo feature includes GPUI test support. The regular production host exposes
no automation server. The acceptance executable refuses to start unless passed
`--native-acceptance deterministic` or `--native-acceptance gpu`; the public JS
controller appends that argument itself. Piped stdin never enables automation.
Requests use an owned length-prefixed byte channel, separate from the canonical
renderer protocol; commits and events inside it retain their canonical framing.
There are no TCP listeners, raw native pointers, or JS paint callbacks.

```ts
import { createRoot, Pressable, Text } from "@solid-gpui/core";
import { createSignal } from "@solid-gpui/core/runtime";
import { NativeAcceptance } from "@solid-gpui/core/testing";

const host = await NativeAcceptance.launch({
  command: ["./target/debug/solid-gpui-acceptance"],
  mode: "deterministic",
});
// Each owned native process initially opens Surface 1.
const root = createRoot(host.transport, { surfaceId: 1 });
try {
  root.render(() => {
    const [count, setCount] = createSignal(0);
    return Pressable({
      accessibilityLabel: "Increment",
      style: { width: 140, height: 40 },
      onPress: () => setCount(count() + 1),
      get children() {
        return Text({ children: String(count()) });
      },
    });
  });
  await host.click(await host.locate({ label: "Increment" }));
  const text = await host.locate({ text: "1" });
  if (text.bounds.width <= 0) throw new Error("Updated text did not paint");
} finally {
  root.unmount();
  await host.close();
}
```

For TSX, run through the public `solid-gpui test` runner so the application and
tests share their compiler and Solid runtime. Direct source tests in this repo
use Bun's `browser` and `solid-gpui-source` conditions. Build output is owned by
the selected checkout; do not share mutable package `dist` directories.

## Public interface

| Interface                                                      | Behavior                                                                                                                                                                                                                                                   |
| -------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `NativeAcceptance.launch({ command, mode, cwd?, timeoutMs? })` | Owns one Bun subprocess; requires an explicit mode. Default request/startup/cleanup timeout is 30 seconds. Startup failures reject.                                                                                                                        |
| `transport`                                                    | Pass to `createRoot(host.transport, { surfaceId: 1 })`. Uses real native admission and event dispatch. Additional surfaces use normal root/host commands.                                                                                                  |
| `capabilities`                                                 | Acceptance contract version, selected mode, platform, screenshot and clock support.                                                                                                                                                                        |
| `flush()`                                                      | Submit queued commits, draw native frames, route events back to Solid, and paint controlled acknowledgements. Non-settling application work fails after 32 feedback turns.                                                                                 |
| `snapshot(surfaceId = 1)`                                      | Detached painted nodes plus Surface, epoch and revision. Each node has stable protocol ID, parent ID, numeric native kind, listener ID, label, native text where observable, clipped logical-pixel bounds, and native input/scroll/selection observations. |
| `locate({ id?, label?, text? }, surfaceId = 1)`                | Exact matching; requires exactly one painted node. Labels are existing `accessibilityLabel` props. Use distinct labels for controls.                                                                                                                       |
| `click(target)`                                                | Move, left down, left up at the target's visible center through GPUI's actual hit testing. Overlays, disabled controls and native propagation govern the result.                                                                                           |
| `type(text, surfaceId = 1)`                                    | Dispatch Unicode keystrokes into native focus. Click an editor first. At most 512 Unicode scalar values per action; split longer input.                                                                                                                    |
| `key(keystroke, surfaceId = 1)`                                | Dispatch one GPUI keystroke, for example `secondary-a`, `backspace`, or `enter`.                                                                                                                                                                           |
| `drag(target, { x, y }, { from? }?)`                           | Left down at the target center or explicit `from` inside its painted bounds, eight native pointer moves, then left up at the destination. Coordinates are logical pixels in that Surface.                                                                  |
| `wheel(target, { x, y })`                                      | Native pixel scroll delta at the target center. Negative `y` moves content upwards.                                                                                                                                                                        |
| `advanceClock(milliseconds)`                                   | Advance the owned GPUI test clock and draw; at most 60 seconds per action. Does not control Bun timers or real network/OS services.                                                                                                                        |
| `screenshot(surfaceId = 1)`                                    | `{ width, height, png: Uint8Array }` from the native rendered scene, in physical pixels. Requires supported GPU mode.                                                                                                                                      |
| `clipboardText()`                                              | Read the owned GPUI test clipboard after a native copy action; returns text or `null`. Both modes isolate copy assertions from the user's desktop clipboard.                                                                                               |
| `close()`                                                      | Idempotent owned cleanup: paint an empty frame to release frame/input references, remove all windows and retained roots, end the channel, and await child exit. Returns `{ surfaces: 0, windows: 0, popups: 0 }`; nonzero cleanup exit rejects.            |

Targets belong to the session that produced them and capture Surface/epoch/
revision/listener identity. After any application commit, locate the target
again. Actions reject stale revisions, replaced epochs, removed nodes, and
unpainted nodes instead of routing a captured target to a newer callback. IDs
remain stable while their Host Node is mounted. Snapshot bounds are native
layout boxes clipped by content masks, not glyph outlines or an occlusion map;
input uses GPUI hit testing even when a box is covered by another element.

Text observations contain the native Text/RawText content and the current core
TextInput display text, including its placeholder when empty. An extension's
internal widget tree is opaque: locate its outer Host Node by label/ID and assert
its public callback/state or screenshot. Nested rich-text spans are assembled by
the parent native Text; they need not have separate painted layout boxes. These
are paint-time observations, not OCR or complete accessibility-tree export.

Core TextInput `input` contains `value`, UTF-16 `selectionStart`/`selectionEnd`,
`reversed`, `editSeq`, `focused`, and nullable `markedStart`/`markedEnd`.
`scrollOffset` is the positive native VirtualList offset, or `null` for other
elements; ordinary overflow scrolling is observable through painted displacement.
`selectedText` reports the existing per-node selectable Text selection.
Renderer-wide selection/search remains the consuming native module's public
contract; drive it with native drag/key actions and inspect its public result or
`clipboardText()`. These observations do not replace application callbacks.

The channel admits at most 256 queued commits/4 MiB of commit bytes, 4096 queued
events/4 MiB of framed event bytes, and 16 MiB per JSON packet. Oversized work
fails explicitly. Screenshots are bounded to 4 million physical pixels and
2 MiB of PNG bytes. Flush between large application updates. The JS controller
serializes actions and drains feedback; Rust draws only on the owned native
foreground thread. Observations cost O(painted Host Nodes) and are compiled only
with the opt-in Cargo feature.

## Custom production hosts

A custom executable can call this runner with the same `HostProfile` used by its
application, including generated NativeModules and native component catalogs:

```rust
fn main() {
    if let Err(error) = solid_gpui::host::acceptance::run(my_application_profile()) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
```

Enable `solid-gpui/native-acceptance` for that executable. The profile's production
initialization, registry, window root, command admission and native handlers are
reused. The stock executable has `NoExtensions` and rejects application extension
contracts. GPU windows are positioned offscreen through native WindowOptions.

## Platforms and evidence

| Mode                      | Rendering and input                                                                                                            | Screenshots                                                                               | Qualification                                                                                                              |
| ------------------------- | ------------------------------------------------------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------- |
| `deterministic`           | Production native layout, scene construction and hit-tested dispatch using GPUI TestAppContext; test-platform text/OS services | Explicitly unsupported                                                                    | Automated native correctness exercised on macOS. Other desktop targets require their own build/run qualification.          |
| `gpu` on macOS            | Production native tree/input and real platform text/rendering in GPUI VisualTestAppContext, offscreen Metal window             | Native scene rendered to a Metal texture; PNG capture needs a working desktop/GPU session | Automated changed-pixel capture exercised on macOS; no screen-recording permission or visible desktop capture is required. |
| `gpu` elsewhere           | Explicit startup error: the linked GPUI visual context is macOS-only                                                           | Unsupported                                                                               | Do not infer Windows/Linux support from a different GPUI fork.                                                             |
| Physical foreground input | Normal application window, human mouse/trackpad and IME                                                                        | Separate platform tooling                                                                 | Manual qualification; acceptance does not synthesize OS hardware events.                                                   |

Native acceptance does not establish physical display latency, presentation FPS,
trackpad behavior, IME composition, or compositor blur. Deterministic layout/scene
tests and Metal screenshots should be reported separately. GPUI test HTTP/OS
services also do not prove a production network/service integration.

`root.unmount()` disposes the JS owner and event subscriptions. Always also await
`host.close()` in `finally` for native windows, retained entities, and process
cleanup. Within a running session, test removal by committing conditional content
changes; use a newer epoch for same-Surface remount tests. A skipped test caused
by an absent native executable or GPU opt-in is no evidence of support.

The repo's focused acceptance tests can be enabled explicitly:

```sh
SOLID_GPUI_ACCEPTANCE_BINARY="$PWD/target/debug/solid-gpui-acceptance" \
bun --conditions=browser --conditions=solid-gpui-source test packages/solid-gpui/tests/native-acceptance.test.ts

SOLID_GPUI_ACCEPTANCE_BINARY="$PWD/target/debug/solid-gpui-acceptance" \
SOLID_GPUI_ACCEPTANCE_GPU=1 \
bun --conditions=browser --conditions=solid-gpui-source test packages/solid-gpui/tests/native-acceptance.test.ts
```

Run the GPU lane on macOS with a working desktop session. Physical-input and
performance qualification follow [performance analysis](performance-analysis.md)
and the consuming application's acceptance workload.
