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

`draggable.type` and optional `draggable.exportFiles` are sent over the native
protocol; `data` stays
in JavaScript for application-side association. GPUI starts an internal drag
after its native pointer threshold and renders a compact neutral 96×32
`Moving item` preview centered under the pointer; custom drag previews are not
exposed. `onDragOver` is a notification
only and its return value is not a native `can_drop` decision. `onDrop`
receives the drag type, while `onExternalFileDrop` receives ordered local
filesystem paths from the desktop drop. `onDragOver` and `onDrop` are
independent capabilities and may be used on a drop target without `draggable`.
Browser runtimes do not promise filesystem path drops.
Enter/leave/move lifecycle events are intentionally not exposed in this
minimal surface; `onDragOver` is the target notification.

Set `draggable.exportFiles` to offer host-local files when the internal drag
leaves the viewport:

```tsx
<View draggable={{ type: "artifact", exportFiles: [artifactPath] }}>
  <Text>Drag artifact to Finder</Text>
</View>
```

`exportFiles` accepts 1..8 non-empty paths using the same 1024-byte UTF-8
validation as `Image.source`. The host checks directory metadata when the
gesture is promoted and offers GPUI's native `Files` payload. There is no
JavaScript completion/error callback for the platform drag: a failed start
remains a platform concern and GPUI may retry while the gesture continues.
macOS and Wayland Linux provide native file dragging; X11 and Windows use the
pinned platform default that cannot start an outbound drag. Text/string
payloads are not exposed because the pinned GPUI `ExternalDragPayload` only
supports `Files`.
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
- `flexDirection` — `"row"`, `"column"`, `"row-reverse"`, or `"column-reverse"`; setting it activates GPUI flex layout for the container. Container properties such as `gap`, `justifyContent`, and `alignItems` also activate flex; reverse values mirror the physical flex axis and are not an RTL base direction.
- `justifyContent` — `"flex-start"`, `"center"`, `"flex-end"`, `"space-between"`, `"space-around"`, or `"space-evenly"`.
- `borderColor`, `backgroundColor`, and `color` — `#RRGGBB` or `#RRGGBBAA`, packed as RGBA `u32` values on the wire.
- `fontWeight` — `"normal"` (400), `"medium"` (500), `"semibold"` (600), `"bold"` (700), or `"heavy"` (900); it applies to `Text` and `RawText`.
- `overflow` — `"visible"`, `"hidden"`, or `"scroll"`; only `"scroll"` makes a `View` a scrollable container and routes native wheel scrolling to it.
- `lineClamp` — a positive integer from `1` through `100`; setting it implies `overflow: "hidden"` when `overflow` is omitted, while an explicit `overflow` value takes precedence.
- `textOverflow` — `"clip"` clears text truncation or `"ellipsis"` maps to GPUI's ellipsis behavior.
- `marginTop`, `marginRight`, `marginBottom`, and `marginLeft` — finite, non-negative pixel values.
- `fontStyle` — `"normal"` or `"italic"`; `textDecoration` — `"none"`, `"underline"`, or `"lineThrough"`.
- `lineHeight`, `minWidth`, `maxWidth`, `minHeight`, `maxHeight`, and `flexShrink` — finite, non-negative pixel/flex values.
- `alignSelf` — `"start"`, `"end"`, `"flex-start"`, `"flex-end"`, `"center"`, `"baseline"`, or `"stretch"`.
- `position` — `"relative"` (default post-layout correction), `"absolute"` (anchored to the closest positioned ancestor/origin), or `"overlay"` (a deferred, anchored layer painted above normal siblings and fit within the viewport); `left` and `top` are finite pixel offsets, including negative values, for all positions. `right` and `bottom` are also supported for `"absolute"` but are rejected for `"overlay"`.
- `cursor` — `"default"`, `"text"`, `"pointer"`, `"grab"`, `"grabbing"`, `"not-allowed"`, `"context-menu"`, `"crosshair"`, `"vertical-text"`, `"alias"`, `"copy"`, `"no-drop"`, `"move"`, `"ew-resize"`, `"ns-resize"`, `"nesw-resize"`, `"nwse-resize"`, `"col-resize"`, or `"row-resize"`. Windows may fall back to Arrow for unsupported variants; headless backends do not render cursors.
- `textAlign` — `"left"`, `"center"`, or `"right"` for physical text alignment on `Text` and `RawText`; logical RTL start/end alignment is unsupported.
- `boxShadow` — one shadow object or a two-element tuple of shadow objects. Each
  shadow accepts finite `offsetX`/`offsetY`, non-negative finite
  `blurRadius`/`spreadRadius`, an `#RRGGBB` or `#RRGGBBAA` color, and an optional
  `inset` boolean. At most two shadows are accepted.
- `fontFamily` — a non-empty font-family string up to 64 Unicode characters.
  GPUI resolves the requested family through its configured fallback stack when
  the primary family is unavailable.
The exported `BoxShadow` type describes one layer, while `BoxShadowInput`
accepts that type or a two-element tuple of layers.

The transport uses one fixed positional 42-slot style tuple: slots `0..39`
retain the existing fields, `40=boxShadow`, and `41=fontFamily`; omitted fields
are encoded as `null` except position and text alignment, whose default codes
are `0` (relative and unset respectively), and cursor, whose default code `0`
means Arrow.
Negative inset offsets are passed through to GPUI/Taffy for relative and absolute positioning. Overlay offsets are supplied to GPUI's local anchored placement so its fit logic can flip a dropdown back into the viewport. No `zIndex` field is exposed; normal layering follows subtree paint order, while explicit overlays are deferred above normal siblings.
- `pointerEvents` is intentionally not exposed. GPUI's default normal
  hitboxes do not occlude underlying hitboxes, so an overlay with no listener
  already permits basic pass-through; a declaration that suppresses only this
  node's callbacks would not express listener-present pass-through or partial
  occlusion.

- Unknown fields, invalid colors, non-finite values, negative numeric fields other than positioning insets, zero `fontSize`, invalid alignment/weight/overflow/font-style/
  decoration values, invalid easing values, duplicate transition properties,
  malformed shadows, and invalid font-family strings are rejected.

## Image

`Image` uses a host-side file path, an optional `fallbackSource`, and an
`objectFit` value (`"fill"`, `"contain"`, `"cover"`, `"scaleDown"`, or `"none"`):

```tsx
<Image
  source="assets/avatar.png"
  fallbackSource="assets/avatar-fallback.svg"
  objectFit="contain"
  style={{ width: 40, height: 40 }}
/>
```

Paths are resolved by the host process: relative paths use the host process
working directory, while absolute paths are recommended for production
packaging. `source` and `fallbackSource` must each be non-empty host paths of
at most 1024 UTF-8 bytes with no control characters. Image nodes do not accept
children and reuse generic style fields such as width, height, and border
radius. Loading is asynchronous through GPUI's image cache; when a fallback is
provided, GPUI displays it while the primary image is loading and if it fails.
There is still no JavaScript `onError` callback in this protocol version.

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

Error handling is deliberately split by seam: unhandled React render errors
are synchronously thrown to the consumer, bad Commit Batches are fatal to the
shared Runtime Adapter, expected Image load failures render `fallbackSource`
when supplied and otherwise remain blank, and GPUI paint panics are host-fatal
rather than caught per node. See
[ADR-0008](../../docs/adr/0008-error-handling-philosophy.md) for the recovery
contract and rejected alternatives.

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
const surfaceId = await root.openSurface({
  title: "Inspector",
  width: 640,
  height: 480,
  kind: "floating",
  resizable: false,
  minSize: [320, 240],
});
const inspector = host.createRoot({
  surfaceId,
  onClose: () => console.log("Inspector closed"),
});
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

Creation options are host-owned and apply only while the new window is being
created. `kind` accepts `"normal"`, `"floating"`, or `"dialog"`; floating means
above the owning app window where the platform supplies that relationship, not
global always-on-top. `resizable` and `minSize: [width, height]` map to GPUI's
creation-time fields. `maxSize`, a generic window-level/always-on-top switch,
runtime setters, and a centered toggle are intentionally unsupported. The
initial host window remains a centered `800×600` window configured outside the
React protocol before JavaScript starts.

| Creation option | macOS | Windows | X11 | Wayland | Web/test |
| --- | --- | --- | --- | --- | --- |
| `floating` | floating level | not topmost | transient parent | parent-linked | rejected/adapter-defined |
| `dialog` | sheet/modal | modal parent | dialog/transient | optional modal extension | rejected/adapter-defined |
| `resizable` | native style | native style | ignored by pinned adapter | no portable toggle | adapter-defined |
| `minSize` | native content minimum | `WM_GETMINMAXINFO` | WM minimum hint | `xdg_toplevel` minimum | adapter-defined |

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

`showNotification` submits a root-scoped request. Optional action buttons are
limited to three bounded `{ id, label }` pairs:

```tsx
await root.showNotification({
  title: "Build finished",
  body: "Artifacts are ready.",
  actions: [{ id: "open", label: "Open" }],
});
```

Register `onNotificationResponse` in the root options to receive
`{ tag, actionId }`; `actionId` is `null` for body activation. The host
generates tags and drops responses for closed surfaces. Delivery and response
support are platform best effort; Web/test platforms may be no-ops.

Static application menus use string action names and optional `disabled` and
`checked` state:

```tsx
await root.setMenus([
  {
    title: "File",
    items: [
      { type: "action", name: "open", disabled: !canOpen, checked: isOpen },
      { type: "separator" },
      { type: "submenu", title: "More", items: [{ type: "action", name: "other" }] },
    ],
  },
]);
```

Register `onAction` in the root options to receive a selected action string.
Menu replacement is static: state changes re-send the complete `setMenus`
definition, with omitted flags defaulting to `false`; there is no incremental
menu-state command. Disabled actions are unavailable to native activation, and
checked actions use GPUI's toggled indicator. Web and test platforms may not
install native menus; headless tests exercise wire and dispatch behavior.

`root.setKeybindings` replaces this surface's complete binding set; pass `[]`
to clear it. Entries use GPUI's whitespace-separated keystroke syntax:

```tsx
await root.setKeybindings([
  { keystrokes: "cmd-shift-p", actionName: "palette.open" },
  { keystrokes: "ctrl-k ctrl-1", actionName: "menu.other" },
]);
```

Each command accepts at most 64 entries, with each sequence capped at 64 UTF-8
bytes and each action name at 64 Unicode characters. Modifiers are joined to
their key with `-`; multiple chords are separated by ASCII whitespace.
Bindings are retained per root but installed as the union in GPUI's
process-global keymap. A matched action routes through the same `onAction`
callback as a menu action and is delivered to the active surface. Context
predicates and menu shortcut fields are intentionally not exposed. The host
pre-validates every chord with GPUI before replacing bindings, so an invalid
keystroke returns a rejected command without disturbing the previous set.

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
time, and prints duration, overall frame rate, frames by kind, total and patch
byte rates, a one-second frame/byte timeline, a byte-size histogram,
event-subtype counts, request-ID-correlated command success, and p50/p95 frame
intervals. `malformed_frames` counts metadata records classified as unknown;
`malformed_records` counts invalid JSON/object/timestamp lines skipped by the
report. A tap cannot observe transport queue depth/backpressure or renderer
commit timing, and `REACT_GPUI_LOG=debug` does not currently add such timing
lines.

Rates use the elapsed time between the first and last actual frame; the
synthetic `tap_stopped` capacity marker is excluded from frame rates and
timeline buckets.

The tap-on overhead claim is informational: one 10,000 seven-byte snapshot
microbench measured tap off at 18.03 ms and tap on at 33.65 ms (about 1.56 μs
incremental wall time per frame). The later event-storm audit measured with the
tap disabled and did not revalidate tap-on overhead, so do not treat that
figure as a current performance guarantee.

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
`View` and `Pressable` also accept `onFocus` and `onBlur` when `focusable` is
true. The callback receives `{ type, target }` as native focus enters or leaves
the node; these notifications use the existing Focus/Blur event codes with a
null payload (TextInput keeps its tag-1 text/selection payload). Native focus
observers are tied to the mounted node and listener identity, so stale focus
events are discarded after unmount or replacement.

An overlay is a `View` or `Pressable` with `style={{ position: "overlay", left, top }}`.
It is placed with GPUI's local anchored/deferred path and may provide
`onPointerDownOutside={({ x, y }) => ...}`. The callback fires on a
capture-phase mouse down only when the point is outside both the rendered
overlay and its direct anchor subtree; `x` and `y` are logical window
coordinates. Keep Escape handling in `onKeyDown` when an overlay should also
dismiss from the keyboard.
`TextInput` remains always focusable and accepts `onKeyDown` through the same
bubble key listener wire. GPUI checks text-input/IME preference before keymap
bindings, so this listener does not use capture phase. Escape edit cancellation
is demonstrated in `examples/todo.tsx`.

`Root.focusNext()` and `Root.focusPrev()` delegate traversal to the native
tab-stop graph and return Promise acknowledgements. A `CommandResult` may omit
its value when the command has no typed return value; value tags are
`[1,number]`, `[2,[width,height]]`, `[3,bool]`, and `[4,string]`.
`Root.setTitle(title)` sends root command `COMMAND_SET_TITLE=6`; title must be
non-empty and at most 256 Unicode code points. The command returns a Promise
resolved by the native CommandResult.
`Root.resize(width, height)` resizes the window client area in integer pixels
from `1` through `16384`. `Root.getWindowSize()` returns a
`Promise<[number, number]>` of logical client-area pixels via
`COMMAND_GET_WINDOW_SIZE=13`. `Root.zoom()` and `Root.toggleFullscreen()`
expose GPUI's platform toggle semantics (they are not absolute state setters).
The command path is covered in headless tests; the visual zoom effect requires
a display-backed desktop adapter because the pinned TestWindow does not
implement native zoom.
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

`createRoot` accepts `onWindowResize(width, height, scaleFactor?)` and
`onWindowActivation(active)` options. Resize values are logical pixels and the
resize callback receives the latest size once per frame after coalescing; it
also receives one initial size after the first native frame. Native resize
events always include a positive display `scaleFactor`. A scale-factor-only
change is still a resize observation and invokes the callback.

The resize callback is root-level, so React applications need a small explicit
state bridge. The package exports `createWindowSizeStore` and `useWindowSize`;
there is intentionally no implicit global root:

```tsx
import { createRoot, createWindowSizeStore, Text, useWindowSize } from "@react-gpui/core";

const windowSizeStore = createWindowSizeStore();
const root = createRoot(transport, {
  onWindowResize: (width, height, scaleFactor) => windowSizeStore.set(width, height, scaleFactor),
});

function App() {
  const { width, height, scaleFactor } = useWindowSize(windowSizeStore);
  return (
    <Text>
      {width < 720 ? "Compact" : "Wide"} ({height}px high @ {scaleFactor}x)
    </Text>
  );
}

root.render(<App />);
```

`examples/todo.tsx` uses the same exported store/hook pattern. The store
starts from a caller-provided estimate (`{ width: 1024, height: 720, scaleFactor: 1 }`
by default) and updates when the native callback fires.

`createRoot` also accepts `onAppearance(appearance)`, where the current
values are `"light"` and `"dark"`. It emits an initial value on the first
window observation frame and coalesces later changes with resize/activation.
Bridge it explicitly with `createAppearanceStore` and `useAppearance`; the
store is per root and the library does not impose a color palette. For a
small application, keep semantic colors in one plain token module (the
examples use [`examples/theme.ts`](examples/theme.ts)) and select that set
from the appearance value:

```tsx
import { createAppearanceStore, createRoot, Text, useAppearance, View } from "@react-gpui/core";
import { useTheme } from "./examples/theme";

const appearanceStore = createAppearanceStore();
const root = createRoot(transport, {
  onAppearance: (appearance) => appearanceStore.set(appearance),
});

function ThemeAwareSurface() {
  const theme = useTheme(useAppearance(appearanceStore));
  return (
    <View style={{ backgroundColor: theme.canvas, borderColor: theme.border, borderWidth: 1 }}>
      <Text style={{ color: theme.text }}>System-aware</Text>
      <Text style={{ color: theme.textMuted }}>Use theme.accentSoft for focused/hovered surfaces.</Text>
    </View>
  );
}

root.render(<ThemeAwareSurface />);
```

The shared module exports `lightTheme`, `darkTheme`, and the `Theme` union.
Its tokens cover canvas/surface layers, border and text hierarchy, accent
hover/pressed/soft variants, focus rings, success/danger, disabled, input,
and shadow colors. Keep state styles token-backed too; do not hand-roll
light-only hex values in individual examples.

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
wheel notifications. `VirtualList` uses GPUI's variable-height `list` state
and reports its visible range; scrollbar appearance and width remain GPUI
platform defaults and are not controlled by this protocol.

`Text` accepts `selectable`. When true, the host owns the selection anchor and
head: dragging with the primary mouse button paints native-shaped,
per-visual-row highlights, and `Cmd-C` on macOS (`Ctrl-C` on other platforms)
writes the selected UTF-8 text directly to the host clipboard. Selection is
deliberately visual-only: there is no JavaScript selection event, callback, or
mirrored React state, and the optional wire tail is omitted when false. The
flag is valid only on `Text`; it is unrelated to the `editable` `TextInput` selection
API. The GPUI test adapter exercises geometry, dragging, text-change clamping,
unmount cleanup, and simulated clipboard copy;
the final native cursor/clipboard behavior should also be checked on a
display-backed desktop host.

`TextInput` is controlled with `value`/`onChangeText` or initialized once with
`defaultValue`. It also supports `placeholder`, `onSelectionChange`, `onFocus`,
`onBlur`, `onSubmitEditing`, `onKeyDown`, `multiline`, `disabled`, `maxLength`,
and accessibility props; native edits carry UTF-16 selection/marked ranges and
editSeq acknowledgements. `onKeyDown` uses the bubble phase; GPUI gives
text-input/IME preference precedence over keymap bindings before dispatching
the listener.

Double-click a word or triple-click a logical line to select at that
granularity; dragging extends the selected range by the same granularity.
`Cmd-C`/`Ctrl-C` copy the selected UTF-8 text, `Cmd-X`/`Ctrl-X` cut it, and
`Cmd-V`/`Ctrl-V` paste native clipboard text at the current selection.
`Cmd-A`/`Ctrl-A` selects all text; `Option`/`Alt`-Left/Right moves by Unicode
word boundaries and Shift extends that movement. These interactions are
host-owned; there is no separate JavaScript copy, cut, paste, or select-all
callback. The runnable [`text-input.tsx`](examples/text-input.tsx) entry
includes the interaction guidance.

`Cmd-Z`/`Ctrl-Z` and Shift-`Cmd-Z`/Shift-`Ctrl-Z` do not provide native
TextInput undo/redo: the pinned GPUI custom input-handler surface has no
history primitive. Applications should not assume an undo stack for this
desktop TextInput.

`maxLength` is enforced natively before an edit enters the Rust input state;
JavaScript also clamps the value delivered to controlled `onChangeText`
callbacks using UTF-16 units. `onSubmitEditing` is emitted for Enter on a
focused single-line input; Enter in a multiline input remains text insertion.
Empty native text renders `placeholder` as muted guidance text without changing
the value, selection, UTF-16 length, or IME ranges. `onSelectionChange`
includes a `reversed` head-orientation bit; `setSelection(start, end)` remains
an ordered-range command and does not set orientation. IME candidate positioning
and point-to-character lookup use the cached GPUI shaped layout for single-line
and multiline content, including explicit empty and trailing-newline lines.
Multiline IME candidate placement still needs display-backed verification.

`onSubmitEditing` has the breaking type `(value: string) => void`; the value is
the authoritative native text at Enter time, including a valid empty string.
Submit events always carry that string payload; `examples/todo.tsx` submits
native text directly instead of relying on a draft closure.
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

Only the committed range is reconciled into host rows, so a 100,000-item array
does not produce 100,000 host nodes. A `VirtualListHandle` ref exposes
Promise-returning `scrollToIndex(index)` and `scrollToEnd()` methods. The native
list keeps one variable-height `ListState` per node: committed/visible rows are
measured at their natural heights, while uncommitted rows use
`estimatedItemSize` as a size hint. The next-frame range is reported through
the existing `VisibleRange` event.

`onEndReached` is based on that reported committed/required range, not an exact
viewport-bottom signal. Native scroll ranges include `overscan`, and
placeholder row requests may extend the required range on the next frame. The
callback runs when the reported range end reaches `data.length`, including
initial or near-end layout, at most once until a later range ends before the
data length; it may run again after that retreat. Identical range reports are
deduplicated.

The list must have a finite viewport height (for example
`style={{ height: 400 }}`) or be inside a parent that supplies a bounded
height; the native list uses that bound to render only visible rows.
`scrollToIndex` accepts only an existing index (`0 <= index < data.length`);
`index === data.length` rejects and sends no command. Valid scroll commands
work while rows are still estimated, and the requested row is made visible
(it is not required to become the first row); `scrollToEnd` anchors the
bottom.

The native list state preserves its logical item/offset when data grows or
shrinks, clamping an anchor past the new count to the new end. An explicit
old-end anchor stays at the end when data grows. JavaScript retains a prior
committed range through filter/unfilter unless a newer native range supersedes
it. Native restoration is index/offset based because `itemKey` is not on the
wire; changed committed row content is remeasured.

Rows with long text, images, or cards may therefore have different native
heights. A wrong estimate is only an initial hint: each committed row replaces
it when measured, while a never-committed row can remain estimated and
scrollbar/content drift converges as rows are visited. Overlapping keyed rows
retain their React state, but a row removed from the committed range is
unmounted and remounts when it returns. Keep state that must survive eviction
outside the row (for example in the parent or application data).

When `data.length === 0`, `emptyState` is rendered instead of a native
`VirtualList` node, so no rows are emitted. Empty-to-non-empty restoration and
placeholder flicker are display-backed behaviors; the native boundary keeps
list wheel handling inside the list and does not change that limitation.

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
single immediate layout update. A transition with omitted `easing` uses
`easeInOut`; `delayMs` is held before the easing curve starts. If a changed
style omits `transition`, the previous transition metadata is reused for
supported target changes: removing `opacity` animates back to `1`, and
removing `backgroundColor` fades through the transparent variant of its last
color. Width/height transitions require numeric endpoints, so removing one
(`auto`) changes immediately. Retargets sample the current presentation value,
start a new generation, and restart the declared delay. Background colors use
straight RGBA/sRGB-channel interpolation.
`transform.scale`, `transform.translateX`, and `transform.translateY` are
intentionally unsupported: GPUI's public
`Transformation`/`with_transformation` contract is SVG-only, and a generic
transformed element would require a new hitbox/layout/painting contract.
`borderRadius` and other low-value scalar transitions are also not exposed.
GPUI retains one animation state per node, samples easing on frame ticks,
retargets from the current sample, cancels deleted nodes, and honors reduced
motion. Completion is emitted once as the tagged AnimationComplete event.

## Troubleshooting

`onTransportTermination` is the process-boundary failure hook. A host panic
writes a versioned report under `REACT_GPUI_CRASH_DIR` (or the system temporary
directory) and prints `react-gpui-host: crash report: <path>` on stderr. When
the process wrapper supplies that stderr tail, the callback error exposes the
same path as `error.crashReportPath`, alongside `error.exitCode` and
`error.stderrTail`:

```tsx
const root = createRoot(transport, {
  onTransportTermination: (error) => {
    console.error(error.cause?.kind, error.crashReportPath, error.exitCode, error.stderrTail);
  },
});
```

`error.cause` is a discriminated union. Branch on `error.cause.kind` rather
than matching `error.message`: `shutdown`, `eof`, `exit` (with `code`),
`protocol` (with `detail`), or `io` (with `detail`). A malformed or
oversized host-to-renderer frame is a `protocol` termination; the root or
shared surface host rejects pending commands and ignores later input. The
decoder's fuzz-level no-panic containment does not imply that a transport can
continue after a malformed frame.

The terminated transport and root are not reusable. If the application owns
the child process, supervise it and create a fresh child, `StdioTransport`, and
root after the old child closes:

```tsx
import { spawn } from "node:child_process";
import { createRoot, StdioTransport, type Root } from "@react-gpui/core";

let activeRoot: Root | undefined;

function mountHost(): Root {
  const child = spawn("react-gpui-host", ["--runtime", "process", "--", "bun", "run", "app.tsx"], {
    stdio: ["pipe", "pipe", "inherit"],
  });
  const root = createRoot(new StdioTransport(child.stdin, child.stdout), {
    surfaceId: 1,
    epoch: 1,
    onTransportTermination: (error) => {
      console.error("host terminated", error.cause?.kind, error.crashReportPath);
      if (activeRoot !== root) return;
      activeRoot = undefined;
      child.once("close", () => {
        activeRoot = mountHost();
      });
    },
  });
  root.render(<App />);
  return root;
}

activeRoot = mountHost();
```

Do not call `createRoot` again with the terminated transport, and do not treat a
rejected command as a recoverable native result. A process killed without
running the panic hook has no crash-report path; retain the exit/signal and
stderr diagnostics that the process wrapper provides.

## Examples

The source tree includes focused entries for the main host surfaces:

- [`counter.tsx`](examples/counter.tsx) — the smallest process-runtime smoke entry.
- [`gallery.tsx`](examples/gallery.tsx) — composed layout, pointer/scroll/drop, drag, appearance, and animation coverage.
- [`todo.tsx`](examples/todo.tsx) — controlled text input, keyboard, accessibility, and virtual-list integration.
- [`keyboard.tsx`](examples/keyboard.tsx) — focus/key notifications, keybindings, a native menu, and a fire-and-forget notification request.
- [`text-input.tsx`](examples/text-input.tsx) — controlled/uncontrolled text input, multi-click word/line selection, native copy, and focus handles.
- [`selectable-text.tsx`](examples/selectable-text.tsx) — host-owned text dragging, per-row highlighting, and Cmd/Ctrl-C clipboard copy.
- [`virtual-list.tsx`](examples/virtual-list.tsx) — a large fixed-row list with overscan and imperative scrolling.
- [`stress.tsx`](examples/stress.tsx) — a manual 10 ms process-runtime soak entry; use `make soak-smoke`.
- [`focus-flow.tsx`](examples/focus-flow.tsx) — focusable form controls with `onFocus`/`onBlur` styling and Tab/Shift-Tab navigation.
- [`dropdown.tsx`](examples/dropdown.tsx) — an anchored overlay with pointer-down-outside and Escape dismissal plus a disabled item.
- [`drag-reorder.tsx`](examples/drag-reorder.tsx) — a standalone draggable list with drag-over feedback, drop reordering, and the neutral native preview.
- [`multi-surface.tsx`](examples/multi-surface.tsx) — a second native window with per-surface resize and appearance bridges.

## Recipes

These are the four small compositions most applications need first. Each recipe
is implemented end to end in the linked example; the snippets show the seam
to preserve when combining them with application state.

### Focusable form flow

Make a control focusable, keep its visual state in React, and route keyboard
navigation through the root:

```tsx
<Pressable
  focusable
  style={focused === "name" ? styles.fieldFocused : styles.field}
  onFocus={() => setFocused("name")}
  onBlur={() => clearFocus("name")}
  onKeyDown={({ key, action, modifiers }) => {
    if (action === "down" && key === "tab") {
      void (modifiers.includes("shift") ? root.focusPrev() : root.focusNext());
    }
  }}
/>
```

`focusable` and a listener put `View`/`Pressable` nodes in the native
tab-stop graph. See [`focus-flow.tsx`](examples/focus-flow.tsx).

### Dropdown with outside dismissal

Keep the trigger and menu under a relative anchor. The overlay listener sees
capture-phase pointer downs outside both the overlay and its direct anchor
subtree; the trigger and enabled items can close on Escape:

```tsx
<View style={{ position: "relative" }}>
  <Pressable focusable onPress={() => setOpen((open) => !open)} onKeyDown={closeOnEscape}>
    <Text>{open ? "Close actions" : "Open actions"}</Text>
  </Pressable>
  {open ? (
    <View
      style={{ position: "overlay", left: 0, top: 42 }}
      onPointerDownOutside={() => setOpen(false)}
    >
      <Pressable focusable onKeyDown={closeOnEscape} onPress={() => setOpen(false)}>
        <Text>Refresh data</Text>
      </Pressable>
      <Pressable disabled>
        <Text>Export data (disabled)</Text>
      </Pressable>
    </View>
  ) : null}
</View>
```

See [`dropdown.tsx`](examples/dropdown.tsx) for the complete styled entry.

### Drag-reorder list

Encode the source row in the drag type, highlight valid targets from
`onDragOver`, and reorder keyed application data on `onDrop`:

```tsx
<View
  style={dragOverId === row.id ? styles.rowDragOver : styles.row}
  draggable={{ type: `activity:${row.id}`, data: row }}
  onDragOver={(dragType) => {
    if (dragType.startsWith("activity:")) setDragOverId(row.id);
  }}
  onDrop={(dragType) => moveActivity(row.id, dragType)}
>
  <Text>{row.title}</Text>
</View>
```

The host supplies a compact neutral preview; the callback return value is not
a native drop decision. See [`drag-reorder.tsx`](examples/drag-reorder.tsx).

### Responsive dark multi-surface layout

Give every root its own explicit size and appearance stores, then derive
layout and palette from those stores:

```tsx
const sizeStore = createWindowSizeStore({ width: 800, height: 600 });
const appearanceStore = createAppearanceStore();
const root = host.createRoot({
  onWindowResize: (width, height, scaleFactor) => sizeStore.set(width, height, scaleFactor),
  onAppearance: (value) => appearanceStore.set(value),
});

function Panel() {
  const { width, height } = useWindowSize(sizeStore);
  const appearance = useAppearance(appearanceStore);
  return (
    <View style={{ width, height, backgroundColor: appearance === "dark" ? "#111827" : "#f7f8fa" }}>
      <Text>{appearance}</Text>
    </View>
  );
}
```

After `await root.openSurface(...)` resolves, register the returned ID with
`host.createRoot` and wire a second pair of stores. See
[`multi-surface.tsx`](examples/multi-surface.tsx).

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
