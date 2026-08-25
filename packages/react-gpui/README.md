# @react-gpui/core

`@react-gpui/core` is a React custom renderer for the Rust GPUI host. React components, hooks, context, fragments, and JavaScript event closures keep their normal React semantics; each React commit becomes one atomic Commit Batch: a Snapshot bootstrap for a new surface epoch or a Patch afterward.

[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](./LICENSE)

[Getting started](../../docs/getting-started.md) · [Protocol](../../docs/protocol.md) · [Contributing](../../CONTRIBUTING.md)

## JSX

```tsx
import { useCallback, useState } from "react";
import { Pressable, StyleSheet, Text, View, createRoot } from "@react-gpui/core";

const styles = StyleSheet.create({
  root: { flexDirection: "column", gap: 8, padding: 16 },
  button: { padding: 8, backgroundColor: "#2d6cdf" },
  label: { color: "#ffffff" },
});

function Counter() {
  const [count, setCount] = useState(0);
  const increment = useCallback(() => setCount((current) => current + 1), []);
  return (
    <View style={styles.root}>
      <Text>Count: {count}</Text>
      <Pressable style={styles.button} onPress={increment}>
        <Text style={styles.label}>Increment</Text>
      </Pressable>
    </View>
  );
}

const root = createRoot(transport, { surfaceId: 1, epoch: 1 });
root.render(<Counter />);
```

`View`, `Text`, and `Pressable` accept `ref` values that resolve to typed host nodes. `Pressable` accepts `onPress`; callbacks stay in JavaScript and receive a semantic press notification. There is intentionally no synchronous `preventDefault()` because the transport cannot cancel a native action synchronously.
`View`, `Text`, `Pressable`, and `Image` accept `onLayout={(frame) => ...}`.
The callback receives `{ x, y, width, height }` in window pixels after native
post-layout measurement. The first report is delivered on the next frame;
identical finite frames are deduplicated. Layout reports are asynchronous and
are not available for `TextInput`, `VirtualList`, or virtualized rows.
## Drag and drop

`View` and `Pressable` support internal drag sources and drop targets:

```tsx
<View
  draggable={{ type: "activity", data: { id: activity.id } }}
  onDragOver={(type) => setDragType(type)}
  onDrop={(type) => moveActivity(type)}
  onExternalFileDrop={(paths) => importFiles(paths)}
/>
```

`draggable.type` is the only value sent over the native protocol; `data` stays
in JavaScript for application-side association. GPUI starts an internal drag
after its native pointer threshold and renders a fixed 24×24 translucent
preview; custom drag previews are not exposed. `onDragOver` is a notification
only and its return value is not a native `can_drop` decision. `onDrop`
receives the drag type, while `onExternalFileDrop` receives ordered local
filesystem paths from the desktop drop. Browser runtimes do not promise
filesystem path drops.
Enter/leave/move lifecycle events are intentionally not exposed in this
minimal surface; `onDragOver` is the target notification.
## Accessibility

Accessibility metadata is forwarded to GPUI's AccessKit-backed tree when the
node has a recognized role and stable host ID:

| React role | GPUI/AccessKit role | Label/description | Checked/selected | Value |
| --- | --- | --- | --- | --- |
| `button` | `Button` | supported | selected supported | supported |
| `text` | `Label` | supported | selected supported | supported |
| `textbox` | `TextInput` | supported | selected supported | supported |
| `checkbox` | `CheckBox` | supported | `checked` maps to toggled true/false; selected supported | supported |
| `heading` | `Heading` | supported | selected supported | supported |
| `generic` | GPUI's role-less container | not exposed as an AX node | not exposed | not exposed |

`accessibilityDisabled` is retained and validated on the wire, but GPUI 0.2.2
does not expose a public AX disabled-state builder. Pressable interaction and
focus behavior still honor `disabled`; only the AccessKit disabled flag is
unsupported until GPUI exposes that surface. Stock headless/TestPlatform runs
cannot activate or inspect an AccessKit tree; verify the final tree on a real
desktop adapter (for example with `Window::debug_a11y_tree_json` and a screen
reader).



## Styles

`StyleSheet.create` is a validated, frozen style-recipe helper; styles are not CSS. V3 fields are:

- `width`, `height`, `flexGrow`, `padding`, `gap`, `borderRadius`, and `borderWidth` — finite, non-negative numbers.
- `fontSize` — a finite, positive number in pixels; it applies to `Text` and `RawText`.
- `flexDirection` — `"row"` or `"column"`.
- `justifyContent` — `"flex-start"`, `"center"`, `"flex-end"`, `"space-between"`, `"space-around"`, or `"space-evenly"`.
- `alignItems` — `"flex-start"`, `"center"`, `"flex-end"`, `"stretch"`, or `"baseline"`.
- `borderColor`, `backgroundColor`, and `color` — `#RRGGBB` or `#RRGGBBAA`, packed as RGBA `u32` values on the wire.
- `fontWeight` — `"normal"` (400), `"medium"` (500), `"semibold"` (600), `"bold"` (700), or `"heavy"` (900); it applies to `Text` and `RawText`.
- `overflow` — `"visible"`, `"hidden"`, or `"scroll"`; only `"scroll"` makes a `View` a scrollable container.
- `lineClamp` — a positive integer from `1` through `100`; setting it implies `overflow: "hidden"` when `overflow` is omitted, while an explicit `overflow` value takes precedence.
- `textOverflow` — `"clip"` clears text truncation or `"ellipsis"` maps to GPUI's ellipsis behavior.
- `marginTop`, `marginRight`, `marginBottom`, and `marginLeft` — finite, non-negative pixel values.
- `fontStyle` — `"normal"` or `"italic"`; `textDecoration` — `"none"`, `"underline"`, or `"lineThrough"`.
- `lineHeight`, `minWidth`, `maxWidth`, `minHeight`, `maxHeight`, and `flexShrink` — finite, non-negative pixel/flex values.
- `alignSelf` — `"start"`, `"end"`, `"flex-start"`, `"flex-end"`, `"center"`, `"baseline"`, or `"stretch"`.
- `position` — `"relative"` (default post-layout correction) or `"absolute"` (anchored to the closest positioned ancestor/origin); `left`, `top`, `right`, and `bottom` — finite pixel offsets, including negative values.

The transport uses one fixed positional 38-slot style tuple: slots `0..19`
remain unchanged, the existing fields occupy `20..32`, and positioning appends
`33=position`, `34=left`, `35=top`, `36=right`, `37=bottom`; omitted fields
are encoded as `null` except position, whose default code `0` means relative.
Negative inset offsets are passed through to GPUI/Taffy. No `zIndex` field is
exposed; overlay layering follows subtree paint/hit-test order.

`fontFamily` is intentionally not exposed: GPUI accepts `SharedString`, but the
backend does not guarantee a safe fallback for an arbitrary missing primary

- Unknown fields, invalid colors, non-finite values, negative numeric fields other than positioning insets, zero `fontSize`, invalid alignment/weight/overflow/font-style/
  decoration values, invalid easing values, and duplicate transition properties
  are rejected.

## Image

`Image` uses a host-side file path and an `objectFit` value (`"fill"`,
`"contain"`, `"cover"`, `"scaleDown"`, or `"none"`):

```tsx
<Image source="assets/logo.png" objectFit="contain" style={{ width: 120, height: 48 }} />
```

Paths are resolved by the host process: relative paths use the host process
working directory, while absolute paths are recommended for production
packaging. Image nodes do not accept children and reuse generic style fields
such as width, height, and border radius. Loading is asynchronous through
GPUI's image cache; a missing or undecodable file renders as silent blank
space with no JavaScript failure callback in this protocol version.

For a sidecar asset shipped next to an ESM/Bun example, derive an absolute
host-visible path and ship the file alongside the example:

```tsx
const avatarSource = new URL("./todo-avatar.svg", import.meta.url).pathname;
<Image source={avatarSource} style={{ width: 32, height: 32 }} />;
```

## Transport and framing

`createRoot` accepts a `Transport` with two directions:

- `submit(frame)` sends a complete renderer-to-host frame. The frame is a four-byte little-endian payload length followed by MessagePack bytes.
- `onData(listener)` delivers host-to-renderer bytes. Input may be fragmented or coalesced; the package incrementally decodes frames and enforces the 16 MiB maximum frame size.

- The complete v3 wire reference—message matrix, node/host-property/accessibility/style slots, event and command directories, limits, fixture walkthroughs, and evolution rules—is [`../../docs/protocol.md`](../../docs/protocol.md). This README keeps only the framing and transport lifecycle summary.

`StdioTransport` observes input `end`/`close`/`error` and output `close`/`error`.
The first such failure enters an idempotent terminated state, removes stream
listeners, clears pending frames, and notifies `createRoot` through
`onTransportTermination`; the callback receives a contextual
`TransportTerminatedError`. Output write failures and host disappearance are
therefore explicit renderer termination, not uncaught stream exceptions.
`createProcessTerminationHandler(exit?)` is provided for process examples: it
logs the termination and exits with code `1`, with an injectable exit function
for tests. `MemoryTransport` remains an in-memory healthy transport.

Advanced custom I/O uses the exported `ByteInput`/`ByteOutput` contracts and
their `ByteInputListener`, `ByteInputEventListener`, `ByteOutputEventListener`,
and `DrainListener` types. `StdioTransportOptions` and
`DEFAULT_MAX_PENDING_BYTES` tune the pending queue; `TransportChunk`,
`TransportListener`, `TransportTerminationListener`,
`TransportTerminatedError`, and `createProcessTerminationHandler` cover chunk
conversion, termination diagnostics, and injected process exits.

## Multiple native surfaces

Use `createSurfaceHost` when several native windows share one transport. It
owns one reader and routes each event to the root registered for that
`surfaceId`; `SurfaceHost` itself has no global open-window command. The
compatibility `createRoot(transport)` path remains valid for a single surface.

```tsx
import { createSurfaceHost } from "@react-gpui/core";

const host = createSurfaceHost(transport);
const root = host.createRoot({ surfaceId: 1 });
root.render(<Main />);

// The request belongs to `root`; omitted values become "" and [0, 0].
const surfaceId = await root.openSurface({ title: "Inspector", width: 640, height: 480 });
const inspector = host.createRoot({
  surfaceId,
  onClose: () => console.log("Inspector closed"),
});
inspector.render(<Inspector />);
```

The `openSurface` promise resolves with the new positive native `surfaceId`
from CommandResult value tag `1`. Only after that handshake should the caller
register the new root and render it. The command is sent with the requesting
root's `surfaceId` and `nodeId=1`; unknown surfaces are rejected rather than
implicitly opened. A native close emits `EVENT_SURFACE_CLOSED` before teardown,
routes only to its root, and invokes `onClose`. Closing the final native window
terminates the host runtime/process. Headless tests cover demultiplexing and
close routing; actual Quartz multi-window display behavior requires a
macOS display-backed host run.

## Native file dialogs

File dialogs are asynchronous root commands backed by the host operating
system; JavaScript does not recreate a system picker:

```tsx
const selected = await root.pickFiles({
  title: "Choose files",
  directories: false,
  multiple: true,
});
const savePath = await root.pickSavePath({ defaultName: "report.json" });
```

`pickFiles` maps `directories` to an exclusive file/directory choice
(`files = !directories`) and `multiple` to multi-selection. It resolves to a
non-empty `string[]`, or `null` when the user cancels. `pickSavePath` resolves
to a selected path or `null` on cancellation; an empty `defaultName` leaves
the native suggestion unset. Save dialog titles are intentionally not exposed:
GPUI's raw save-picker interface accepts only an initial directory and
suggested filename. Platform picker failures reject the promise.
Headless tests cover command validation, asynchronous completion, cancellation,
and value routing. Actual NSOpenPanel/NSSavePanel interaction requires a
display-backed macOS Quartz host run and is not exercised in headless CI.

## System notifications and menus

`showNotification` is a one-way, root-scoped request:

```tsx
await root.showNotification({ title: "Build finished", body: "Artifacts are ready." });
```

Success means the host submitted a `SystemNotification`; operating-system
authorization and delivery are best effort. The host owns the internal tag and
does not expose actions, dismissal, or response callbacks in this interface.
Windows AppUserModel identity is a host packaging concern.

Static application menus use string action names:

```tsx
await root.setMenus([
  {
    title: "File",
    items: [
      { type: "action", name: "open" },
      { type: "separator" },
      { type: "submenu", title: "More", items: [{ type: "action", name: "other" }] },
    ],
  },
]);
```

Register `onAction` in the root options to receive a selected action string.
Menu replacement is static for this version: dynamic enablement, accelerator
labels, and keybinding registration are intentionally out of scope. Web and
test platforms may not install native menus; headless tests exercise wire and
dispatch behavior.

## Debugging

Set `REACT_GPUI_TAP` to a JSONL path before constructing a `StdioTransport` or
native runtime adapter to observe protocol metadata:

```sh
REACT_GPUI_TAP="${TMPDIR:-/tmp}/react-gpui-tap-$$.jsonl" bun run examples/counter.tsx
python3 ../../scripts/protocol-tap-report.py "${TMPDIR:-/tmp}/react-gpui-tap-$$.jsonl"
```

Each process opens its own path with truncation; do not use one shared path for
multiple processes. Tap records include monotonic milliseconds, direction,
peer, message kind, complete framed byte count (including the four-byte
length), and a strictly increasing per-file sequence. Event and command
subtype numbers, command request IDs, and command-result success values are
included when available. Payload contents are never written.

The tap is capped at 64 MiB. On reaching the cap it writes a final
`tap_stopped` record with `reason="capacity"` and then disables itself. An
unopenable path prints one warning to stderr and leaves transport operation
unchanged; tapping is never fatal. `MemoryTransport` does not tap.

The report script accepts multiple JSONL files, merges records by monotonic
time, and prints duration, frame rate, kind counts, byte min/median/max,
event-subtype counts, request-ID-correlated command success, and p50/p95 frame
intervals.

The tap-on overhead was measured once with 10,000 seven-byte snapshot frames
through `StdioTransport` (tap off 18.03 ms, tap on 33.65 ms, about 1.56 μs per
frame of incremental wall time); this is informational and not a performance
gate.

## Commit and event semantics

A completed React commit produces exactly one atomic Commit Batch: a complete Snapshot during bootstrap or one incremental Patch afterward, rather than one transport call per host mutation or prop. Listener IDs remain stable while a host node remains mounted; changing a callback function updates only the JavaScript callback table, while 0↔nonzero listener transitions update the native listener field. TextInput native edits are acknowledged with editSeq and preserve uncontrolled defaults. Press, TextInput, VisibleRange, AnimationComplete, Keyboard, Pointer, Hover, and Scroll events are semantic notifications, not cancellable browser events.

## Keyboard

`View` and focusable `Pressable` nodes can opt into native keyboard delivery
with `focusable` and `onKeyDown`; `TextInput` accepts `onKeyDown` while always
remaining focusable. The callback receives a semantic notification with the
native key name, a compact modifier list, and an action:

```tsx
<Pressable
  focusable
  onKeyDown={({ key, modifiers, action }) => {
    console.log(action, [...modifiers, key].join("+"));
  }}
/>
```

GPUI registers key handlers on the focused dispatch path. `down` is emitted for
the initial key press, `repeat` for a held-key notification reported by GPUI,
and `up` when the key is released. Modifier names are `cmd`, `ctrl`, `alt`,
`shift`, and `function`; `cmd` represents GPUI's platform modifier slot.
Keyboard events are semantic notifications and cannot be synchronously canceled.

## Pointer and focus

`View` and `Pressable` accept `onPointerDown`, `onPointerUp`, and
`onHoverChange`. Pointer events are semantic notifications with button codes
left/right/middle/back/forward, the compact modifier names above, and a
`clickCount`; Pressable's existing `onPress` notification remains unchanged.
Hover uses a null wire payload and alternates the `onHoverChange` boolean on
ordered enter/leave edges from GPUI's `.on_hover` callback.

`View` refs expose `focus()`, `blur()`, and `isFocused(): Promise<boolean>`;
focusable Pressables use the same native focus graph but their public ref does
not add synchronous focus state. These commands reject for non-focusable views.
Focusable `View` and `Pressable` nodes participate in the native tab-stop
graph when `focusable` is true and a listener is present. Pressable Enter/Space
activation is supplied by GPUI's `.on_click` keyboard synthesis: an unmodified
keydown→keyup pair with stable focus emits the existing `onPress` notification,
so applications do not add a second keyboard activation handler.
`Pressable.disabled` removes focus, press/key/pointer/hover interaction and
automatically reports `accessibilityDisabled`; it does not change opacity.
`TextInput` remains always focusable and accepts `onKeyDown` through the same
bubble key listener wire. GPUI checks text-input/IME preference before keymap
bindings, so this listener does not use capture phase. Escape edit cancellation
is demonstrated in `examples/todo.tsx`.

`Root.focusNext()` and `Root.focusPrev()` delegate traversal to the native
tab-stop graph and return Promise acknowledgements. A `CommandResult` may omit
its optional value for backward compatibility; value tags are
`[1,number]`, `[2,[width,height]]`, `[3,bool]`, and `[4,string]`.
`Root.setTitle(title)` sends root command `COMMAND_SET_TITLE=6`; title must be
non-empty and at most 256 Unicode code points. The command returns a Promise
resolved by the native CommandResult.
`Root.resize(width, height)` resizes the window client area in integer pixels
from `1` through `16384`. `Root.getWindowSize()` returns a
`Promise<[number, number]>` of logical client-area pixels via
`COMMAND_GET_WINDOW_SIZE=13`. `Root.zoom()` and `Root.toggleFullscreen()`
expose GPUI's platform toggle semantics (they are not absolute state setters).
`Root.openUrl(url)` accepts only non-empty `http://` or `https://` URLs up to
2048 characters; `file:` and other schemes are rejected. Centering,
`revealPath`, and `openWithSystem` are intentionally unsupported because GPUI
only exposes centering during initial `WindowBounds` construction and path
operations require a separate permission/Path design.
`Root.setClipboardText(text)` and `Root.getClipboardText()` use root commands
15 and 16. Clipboard strings are capped at 1 MiB UTF-8 bytes; oversized writes
reject locally, while an oversized host clipboard rejects with
`clipboard text exceeds the supported size`. A clipboard with no text content
rejects with `clipboard has no text content`; an empty string stored as text is
still a successful read.

`createRoot` accepts `onWindowResize(width, height)` and
`onWindowActivation(active)` options. Resize values are logical pixels and
the resize callback receives the latest size once per frame after coalescing;
it also receives one initial size after the first native frame. Activation
delivers its initial value on observer registration and then only changes.
Scale-factor-only changes have no independent callback.

The resize callback is root-level, so React applications need a small explicit
state bridge. The package exports `createWindowSizeStore` and `useWindowSize`;
there is intentionally no implicit global root:

```tsx
import { createRoot, createWindowSizeStore, Text, useWindowSize } from "@react-gpui/core";

const windowSizeStore = createWindowSizeStore();
const root = createRoot(transport, {
  onWindowResize: (width, height) => windowSizeStore.set(width, height),
});

function App() {
  const { width, height } = useWindowSize(windowSizeStore);
  return (
    <Text>
      {width < 720 ? "Compact" : "Wide"} ({height}px high)
    </Text>
  );
}

root.render(<App />);
```

`examples/todo.tsx` uses the same exported store/hook pattern. The store
starts from a caller-provided estimate (`{ width: 1024, height: 720 }` by
default) and updates when the native callback fires.

`createRoot` also accepts `onAppearance(appearance)`, where the current
values are `"light"` and `"dark"`. It emits an initial value on the first
window observation frame and coalesces later changes with resize/activation.
Bridge it explicitly with `createAppearanceStore` and `useAppearance`; the
store is per root and the library does not impose a color palette:

```tsx
const appearanceStore = createAppearanceStore();
const root = createRoot(transport, {
  onAppearance: (appearance) => appearanceStore.set(appearance),
});

function ThemeAwareLabel() {
  const appearance = useAppearance(appearanceStore);
  return <Text style={{ color: appearance === "dark" ? "#ffffff" : "#111827" }}>System-aware</Text>;
}
```

`"light"` includes GPUI light/vibrant-light appearances and `"dark"` includes
dark/vibrant-dark appearances. Palette selection remains application-owned.

## Scroll

`View` accepts `onScroll` for native wheel notifications. The callback receives
the delta kind (`"pixels"` or `"lines"`), finite `dx`/`dy` values, the window
coordinate (`x`/`y`), and the compact modifier list:

```tsx
<View
  onScroll={({ deltaKind, dx, dy, x, y, modifiers }) => {
    console.log(deltaKind, dx, dy, x, y, modifiers);
  }}
/>
```

GPUI's `ScrollDelta::Pixels` and `ScrollDelta::Lines` are preserved as the two
wire kinds. `touch_phase` is intentionally not serialized because scroll is a
semantic notification rather than a cancellable gesture lifecycle. `VirtualList`
continues to report its existing `VisibleRange` notification and does not also
accept `onScroll`.

`overflow: "scroll"` makes a `View` a scrollable container; `overflow: "visible"`
and `"hidden"` do not. The same View may use `onScroll` to observe its native
wheel notifications. `VirtualList` is unaffected and its `uniform_list` owns
scrolling. Scrollbar appearance and width remain GPUI platform defaults and
are not controlled by this protocol.

`TextInput` is controlled with `value`/`onChangeText` or initialized once with
`defaultValue`. It also supports `placeholder`, `onSelectionChange`, `onFocus`,
`onBlur`, `onSubmitEditing`, `onKeyDown`, `multiline`, `disabled`, `maxLength`,
and accessibility props; native edits carry UTF-16 selection/marked ranges and
editSeq acknowledgements. `onKeyDown` uses the bubble phase; GPUI gives
text-input/IME preference precedence over keymap bindings before dispatching
the listener.
`maxLength` is enforced natively before an edit enters the Rust input state;
JavaScript also clamps the value delivered to controlled `onChangeText`
callbacks using UTF-16 units. `onSubmitEditing` is emitted for Enter on a
focused single-line input; Enter in a multiline input remains text insertion.

`onSubmitEditing` has the breaking type `(value: string) => void`; the value is
the authoritative native text at Enter time, including a valid empty string.
Old null-payload submit frames remain decoder-compatible and dispatch as `""`;
new hosts always send the text payload. `examples/todo.tsx` submits native text
directly instead of relying on a draft closure.
`secureTextEntry` and `keyboardType` are intentionally unsupported: the desktop
GPUI surface has no password-obscuring text primitive or soft-keyboard layout
semantic.

## VirtualList

`VirtualList<T>` keeps its `data`, `itemKey`, and `renderItem` closures in JavaScript:

```tsx
<VirtualList
  data={rows}
  itemKey={(row) => row.id}
  renderItem={(row) => <Text>{row.title}</Text>}
  estimatedItemSize={28}
  overscan={3}
  initialNumToRender={12}
  onEndReached={loadMore}
/>
```

Only the committed visible range is reconciled into host rows, so a 100,000-item array does not produce 100,000 host nodes. A `VirtualListHandle` ref exposes Promise-returning `scrollToIndex(index)` and `scrollToEnd()` methods. Native GPUI uses one persistent fixed-height `uniform_list` scroll handle per node and reports the actual next-frame range with overscan.
The list must have a finite viewport height (for example `style={{ height: 400 }}`) or be inside a parent that supplies a bounded height; `uniform_list` uses that bound to render only visible rows.

Rows are expected to remain within the supplied `estimatedItemSize`; dynamic
editing, wrapping, or multiline content does not provide a measurement
callback. There is no `emptyState`/`emptyRenderer` prop, so render an empty
message outside the list when `data.length === 0`.

## Native animation

`opacity`, `backgroundColor`, `width`, and `height` can transition natively
without per-frame JavaScript commits:

```tsx
<View
  style={{
    width: expanded ? 420 : 260,
    height: expanded ? 120 : 64,
    opacity: expanded ? 1 : 0.72,
    transition: {
      durationMs: 180,
      easing: "easeOut",
      properties: ["width", "height", "opacity"],
      onComplete: (generation) => console.log("completed", generation),
    },
  }}
/>
```

Width/height transitions rewrite layout dimensions on every native frame, so
they trigger layout reflow; static width/height without `transition` remains a
single immediate layout update. `transform.scale`, `transform.translateX`, and
`transform.translateY` are intentionally unsupported: GPUI's public
`Transformation`/`with_transformation` contract is SVG-only, and a generic
transformed element would require a new hitbox/layout/painting contract.
`borderRadius` and other low-value scalar transitions are also not exposed.
GPUI retains one animation state per node, samples easing on frame ticks,
retargets from the current sample, cancels deleted nodes, and honors reduced
motion. Completion is emitted once as the tagged AnimationComplete event.

## Examples

The source tree includes focused entries for the main host surfaces:

- [`counter.tsx`](examples/counter.tsx) — the smallest process-runtime smoke entry.
- [`gallery.tsx`](examples/gallery.tsx) — composed layout, drag, appearance, and animation coverage.
- [`todo.tsx`](examples/todo.tsx) — controlled text input, keyboard, and virtual-list integration.
- [`keyboard.tsx`](examples/keyboard.tsx) — focus and native key notifications.
- [`text-input.tsx`](examples/text-input.tsx) — two controlled text inputs and focus.
- [`virtual-list.tsx`](examples/virtual-list.tsx) — a large fixed-row list with overscan.

## Local commands

From this directory:

```sh
bun install --frozen-lockfile
bun run format
bun run typecheck
bun run test
bun run build
bun pm pack --dry-run
```

`bun run build` first removes `dist/`, then writes ESM JavaScript and
declarations there. The package export points only at those built files; source,
tests, and examples are not included in the tarball.

The counter example is intended to run from the repository root while
developing:

```sh
cargo run -p react-gpui-host -- bun run packages/react-gpui/examples/counter.tsx
```
