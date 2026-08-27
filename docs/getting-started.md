# React GPUI: getting started

This guide is for an application consumer, not a protocol contributor. It gets
a small React surface running first, then points to the deeper references.
The single authoritative wire reference is [Protocol v3](protocol.md); this
guide stays at the application level.

## Five-minute start

### 1. Install the pinned toolchain

The repository is locked to:

- Bun `1.4.0` (`.bun-version`)
- Rust `1.97.1` (`rust-toolchain.toml`)

A consumer package needs React 19 and the core package:

```sh
bun add react @react-gpui/core
```

The native host is a separate executable. A published/release workflow uses
the host binary from the `dist/react-gpui-host-*.tar.gz` candidate archive
(produced by `make host-release-bundle`). During local development, build and
run the host from the repository instead:

```sh
cargo run -p react-gpui-host -- --runtime process bun run path/to/counter.tsx
```

The `--runtime process` host starts Bun as a child process and connects its
stdin/stdout to `StdioTransport`. The host command must own that pipe; running a
`StdioTransport` entry directly without a host does not create a native
surface. The optional embedded runtime is a separate macOS/JSC build:

```sh
cargo run -p react-gpui-host --features embedded-bun -- \
  --runtime embedded path/to/counter.tsx
```

### 2. Render one surface

Save this as `counter.tsx` and run it with the process-host command above. The
example is intentionally complete rather than importing repository-only
helpers:

```tsx
import React, { useState } from "react";
import { Pressable, StdioTransport, Text, View, createProcessTerminationHandler, createRoot } from "@react-gpui/core";

function Counter() {
  const [count, setCount] = useState(0);
  return (
    <View style={{ flexDirection: "column", gap: 8, padding: 16 }}>
      <Text>Count: {count}</Text>
      <Pressable onPress={() => setCount((current) => current + 1)}>
        <Text>Increment</Text>
      </Pressable>
    </View>
  );
}

const root = createRoot(new StdioTransport(), {
  surfaceId: 1,
  epoch: 1,
  onTransportTermination: createProcessTerminationHandler(),
});
root.render(<Counter />);
```

`surfaceId` identifies the native Surface and `epoch` identifies its current
lifecycle generation. Start with one root/surface; use `createSurfaceHost` only
when the application needs multiple native windows.

## Build a small app

Once the counter is running, grow it in the same order most small desktop
surfaces need: form state, a bounded list, keyboard focus, and an overlay. The
examples are complete runnable entries; use them as the source of truth for the
composition rather than copying a large tutorial listing.

### 1. Start with form state

Use [`todo.tsx`](../packages/react-gpui/examples/todo.tsx) as the starting
point. Keep the draft in `TextInput` state, update it from `onChangeText`, and
append the native value received by `onSubmitEditing`. Clear the draft only
after the item is added. This keeps the form controlled and makes Enter
submission deterministic.

### 2. Add a bounded list

Render the items with the [`VirtualList` portion of
`todo.tsx`](../packages/react-gpui/examples/todo.tsx), giving it a finite
height, a stable `itemKey`, and an `emptyState`. The list owns visible-range
work on the native side, while `data` and `renderItem` remain ordinary
JavaScript state and closures. Keep the form and list in one parent so a
successful submit updates the same `data` array.

### 3. Make the flow keyboard-friendly

Use [`focus-flow.tsx`](../packages/react-gpui/examples/focus-flow.tsx) to add
focusable `Pressable` controls around the form actions. Give each control
`onFocus` and `onBlur` handlers that select its focused style, then route Tab
and Shift-Tab to `root.focusNext()` and `root.focusPrev()`. Focus is native
state; React only renders the visual state reported by the notifications.

### 4. Add a dropdown without a portal

Copy the composition in
[`dropdown.tsx`](../packages/react-gpui/examples/dropdown.tsx): keep the
trigger and overlay under a `position: "relative"` anchor, render the menu
with `position: "overlay"`, and close it from
`onPointerDownOutside`. The trigger and enabled menu item also handle Escape
through `onKeyDown`; mark unavailable actions `disabled`. This gives the form
a native-dismissable action menu without introducing DOM or browser event
semantics.

At this point the counter has become a small native app: React owns state and
composition, while GPUI owns layout, focus traversal, overlay hit testing, and
list virtualization. The four focused entries in
[`packages/react-gpui/examples`](../packages/react-gpui/examples/) keep each
capability easy to inspect when the combined app needs debugging.

## Core model

React GPUI is a renderer, not a DOM implementation:

```text
React render/commit
        │
        ▼
Commit Batch (Snapshot first, Patch afterward)
        │  MessagePack frame through Runtime Adapter
        ▼
Rust host → validated Host Node tree → GPUI/Taffy layout and drawing
        │
        └── Native Event frame → matching JavaScript Root callback
```

React/Bun own Fiber, hooks, context, fragments, and JavaScript closures. The
host owns the validated retained tree, native input/focus state, layout, and
painting. A Commit Batch is atomic at the protocol seam: it is not a stream of
per-prop DOM mutations. Native Events are semantic notifications such as
press, text input, key, pointer, scroll, list range, layout, window lifecycle,
and command acknowledgement.

Important differences from web React:

- There is no DOM, CSS cascade, browser event cancellation, browser storage, or
  browser URL/navigation model.
- Styles are a validated GPUI/Taffy-oriented subset with 42 positional slots,
  not CSS. Read [protocol.md](protocol.md) for the exact slot contract and the
  package README for the consumer-facing names.
- Text, images, lists, focus, and commands are native host concepts. A
  `VirtualList` uses a bounded native list with per-row layout measurement; its
  `estimatedItemSize` is an initial hint for unmeasured/placeholder rows, not a
  fixed height.
- `Runtime Adapter` is the transport seam. `ProcessAdapter` is the normal
  process host; `EmbeddedBunAdapter` is an optional in-process runtime.

## Capability quick reference

### Components and common props

| Host kind        | Common props                                                                                                                                               | Notes                                                                                                                               |
| ---------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------- |
| `View`           | `style`, `children`, `focusable`, `onKeyDown`, `onPointerDown`, `onPointerUp`, `onHoverChange`, `onFocus`, `onBlur`, `onPointerDownOutside`, `onScroll`, `onLayout`, accessibility props | Generic layout/container Host Node. Focus callbacks require `focusable`; an overlay (`position: "overlay"`) can dismiss through `onPointerDownOutside`. |
| `Text`           | `children`, `style`, `selectable`, `onLayout`, accessibility props                                                                                         | Text styling applies to its text content; raw strings must be direct children. `selectable` enables host-owned mouse drag selection, per-row highlighting, and Cmd/Ctrl-C clipboard copy without JS selection state/events. |
| `Pressable`      | `children`, `style`, `onPress`, `focusable`, `onKeyDown`, `onFocus`, `onBlur`, `disabled`, pointer/hover/layout handlers, accessibility props                 | Pointer and native keyboard activation share the press path. Focus callbacks require `focusable`; disabled removes interaction and reports disabled accessibility state. |
| `VirtualList<T>` | `data`, `itemKey`, `renderItem`, `estimatedItemSize`, `overscan`, `initialNumToRender`, `onEndReached`, `emptyState`, `style`                              | Native GPUI measures visible/overdraw rows at natural heights; `estimatedItemSize` is the initial hint for unmeasured rows. Only the committed range becomes Host Nodes. Empty data renders `emptyState` without a native VirtualList node.             |
| `Image`          | `source`, `fallbackSource`, `objectFit`, `style`, `onLayout`, accessibility props                                                    | `source` and optional `fallbackSource` are host-resolved paths; GPUI shows the fallback while loading or when the primary image fails. |

`Image` cannot have children. Text input, list, and image host properties are
validated tagged tuples. Empty native TextInput value renders `placeholder` as
muted visual guidance without changing `value`, selection, or UTF-16 length;
selection callbacks include the UTF-16 head-orientation `reversed` bit.
Selectable `Text` selection is visual-only and intentionally has no JS callback
or wire event; its final native cursor and clipboard behavior should be checked
on a display-backed desktop host. See
[protocol.md](protocol.md) for the node tuple, accessibility tuple,
host-property tags, and every wire constraint.

### Styles by class

The public `Style` names map to a fixed 42-slot tuple. Do not hand-author the
wire tuple; use the object fields and `StyleSheet.create`:

- **Layout:** `width`, `height`, `flexDirection`, `flexGrow`, `flexShrink`,
  `justifyContent`, `alignItems`, `alignSelf`, `overflow`, `lineClamp`,
  `textOverflow`.
- **Spacing:** `padding`, `gap`, `marginTop`, `marginRight`, `marginBottom`,
  `marginLeft`.
- **Typography:** `fontSize`, `fontWeight`, `fontStyle`, `fontFamily`,
  `textDecoration`, `lineHeight`, `color`.
- **Visual:** `backgroundColor`, `borderColor`, `borderWidth`, `borderRadius`,
  `boxShadow`, `opacity`.
- **Positioning:** `relative` is the default post-layout correction; `absolute` anchors to the closest positioned ancestor/origin and leaves no layout space; `overlay` uses a local `left`/`top` anchor, deferred priority, and viewport fit for dropdowns/popovers. Overlay is not a general portal or z-index; its `right`/`bottom` fields are rejected, while `left`/`top` remain finite pixel offsets (including negatives).
- **Animation:** `transition` for `opacity`, `backgroundColor`, `width`, and
  `height`.

Colors are `#RRGGBB` or `#RRGGBBAA`; numeric fields are validated before a
Commit Batch is emitted. `boxShadow` accepts one or two shadow objects and
renders through GPUI's native shadow primitive; `fontFamily` accepts a
non-empty family name up to 64 Unicode characters and uses GPUI's configured
fallback stack. The complete positional table and enum codes live in
[protocol.md](protocol.md#style-tuple-all-42-slots).

### Root commands

| Domain         | Root methods                                                                 |
| -------------- | ---------------------------------------------------------------------------- |
| Window         | `setTitle`, `resize`, `getWindowSize`, `zoom`, `toggleFullscreen`, `openUrl` |
| Surfaces       | `createSurfaceHost`, `root.openSurface`, `host.createRoot`, `root.onClose`   |
| Focus          | `focusNext`, `focusPrev`                                                     |
| Clipboard      | `setClipboardText`, `getClipboardText`                                       |
| Files          | `pickFiles`, `pickSavePath`                                                  |
| User-facing OS | `showNotification`, `setMenus`, `setKeybindings`                    |

Node refs expose narrower commands: TextInput focus/blur/selection, View or
Pressable focus/blur where supported, and VirtualList `scrollToIndex`/
`scrollToEnd`. Commands return Promises and rejected validation/native results
must be handled by the application. See [protocol.md](protocol.md#4-command-directory)
for root-only versus Host Node ownership and CommandResult value tags.

### Native Events

- **Interaction:** `Press`, TextInput `Change`/`Selection`/`Focus`/`Blur`,
  `Submit`, `Key`, `Pointer`, and `Hover`.
- **Scrolling and lists:** `Scroll`, `VisibleRange`, and `Layout`.
- **Window/surface lifecycle:** `WindowResize`, `WindowActivation`,
  `WindowAppearance`, `SurfaceClosed`, `Action`, and `NotificationResponse`.
- **Command acknowledgement:** `CommandResult`, including optional typed
  number, pair, boolean, string, and newer surface values.
Callbacks are semantic notifications, not cancellable browser events. Event
18 appearance values are `"light"` and `"dark"`; palette selection is owned by
the application. See [protocol.md](protocol.md#3-event-directory) for payload
validation and root/node ownership.

## Common tasks

### Form submission

`TextInput` is controlled, and `onSubmitEditing` receives the native text:

```tsx
function AddForm({ add }: { add: (value: string) => void }) {
  const [value, setValue] = useState("");
  return (
    <TextInput
      value={value}
      onChangeText={setValue}
      onSubmitEditing={(submitted) => {
        add(submitted);
        setValue("");
      }}
      placeholder="New item"
    />
  );
}
```

### List with empty state

An empty `VirtualList` renders ordinary React nodes instead of a native list:

```tsx
<VirtualList
  data={items}
  itemKey={(item) => item.id}
  renderItem={(item) => <Text>{item.title}</Text>}
  estimatedItemSize={28}
  emptyState={<Text>No items yet.</Text>}
  style={{ height: 400 }}
/>
```

### Responsive layout

Bridge the root callback to an explicit store; do not make a module-global
window-size singleton:

```tsx
const sizes = createWindowSizeStore();
const root = createRoot(new StdioTransport(), {
  onWindowResize: (width, height, scaleFactor) => sizes.set(width, height, scaleFactor),
});
function Screen() {
  const { width, scaleFactor } = useWindowSize(sizes);
  return <View style={{ flexDirection: width < 720 ? "column" : "row", opacity: scaleFactor < 1 ? 0.9 : 1 }} />;
}
root.render(<Screen />);
```

### System appearance

The renderer reports a two-value semantic appearance; it does not choose your
palette. Bridge each root to an explicit store, then select semantic color
tokens. The example family keeps this pattern in
[`packages/react-gpui/examples/theme.ts`](../packages/react-gpui/examples/theme.ts):

```tsx
import { StdioTransport, createAppearanceStore, createRoot, Text, useAppearance, View } from "@react-gpui/core";
import { useTheme } from "../packages/react-gpui/examples/theme";

const appearanceStore = createAppearanceStore();
const root = createRoot(new StdioTransport(), {
  onAppearance: (value) => appearanceStore.set(value),
});
function Screen() {
  const theme = useTheme(useAppearance(appearanceStore));
  return (
    <View style={{ backgroundColor: theme.canvas, borderColor: theme.border, borderWidth: 1 }}>
      <Text style={{ color: theme.text }}>System-aware</Text>
      <Text style={{ color: theme.textMuted }}>State colors come from the same token set.</Text>
    </View>
  );
}
root.render(<Screen />);
```

Use `theme.surface`, `theme.text`, `theme.textMuted`, `theme.border`,
`theme.accent`, `theme.accentHover`, `theme.accentSoft`, `theme.focusRing`,
and the state tokens for controls instead of scattering light-only hex
values through component styles. Palette selection remains application-owned.

### Keyboard navigation

Focusable `View` and `Pressable` nodes use the native Tab-stop Graph. Let
native Enter/Space activation produce the existing `onPress` notification:

```tsx
const root = createRoot(new StdioTransport());
root.render(
  <Pressable
    focusable
    onPress={() => console.log("activated")}
    onKeyDown={({ key, action }) => console.log(key, action)}
  >
    <Text>Keyboard action</Text>
  </Pressable>,
);
```

For an application-level Tab shortcut, call `root.focusNext()` from a
focusable key listener. `disabled` removes a Pressable from interaction.

### Open another Surface

Several native windows share one transport through `createSurfaceHost`:

```tsx
const host = createSurfaceHost(new StdioTransport());
const main = host.createRoot({ surfaceId: 1 });
main.render(<Main />);
const id = await main.openSurface({
  title: "Inspector",
  width: 640,
  height: 480,
  kind: "floating",
  resizable: false,
  minSize: [320, 240],
});
const inspector = host.createRoot({ surfaceId: id, onClose: () => console.log("closed") });
```
`kind`, `resizable`, and `minSize` are creation-time options. `"floating"`
means above-parent where the platform supports it; it is not a portable global
always-on-top guarantee. `maxSize` and runtime window-option setters are not
exposed because pinned GPUI has no corresponding portable API. The first host
window stays a host-owned centered `800×600` window and is created before
JavaScript starts.

### File selection

File dialogs are asynchronous root commands. Cancellation resolves to `null`:

```tsx
const paths = await root.pickFiles({ title: "Choose files", multiple: true });
const savePath = await root.pickSavePath({ defaultName: "report.json" });
if (paths !== null) console.log(paths);
if (savePath !== null) console.log(savePath);
```

### Notification and menus

Notifications are one-way; menus are static, state-driven definitions:

```tsx
await root.showNotification({ title: "Build finished", body: "Artifacts ready." });
await root.setMenus([
  {
    title: "File",
    items: [{ type: "action", name: "open", disabled: !canOpen, checked: isOpen }],
  },
]);
```

Re-send the complete `setMenus` definition when `disabled` or `checked` state
changes. Omitted flags default to `false`; disabled actions are unavailable to
native activation and checked actions use the native toggled indicator.
Register `onNotificationResponse` in `createRoot` options for
`{ tag, actionId }` responses; body activation uses `actionId: null`, and a
closed surface drops late responses.

### Keyboard bindings

`root.setKeybindings` is a full replacement for this surface's bindings;
passing `[]` clears that surface's set. Each entry uses a whitespace-separated
GPUI keystroke sequence and an action name:

```tsx
await root.setKeybindings([
  { keystrokes: "cmd-shift-p", actionName: "palette.open" },
  { keystrokes: "ctrl-k ctrl-1", actionName: "menu.other" },
]);
```

Chords use `-` between modifiers and key (`ctrl`, `alt`, `shift`, `fn`,
`secondary`, `cmd`/`super`/`win`); multiple chords use ASCII whitespace.
Bindings are held per surface but installed into one process-global GPUI
keymap, so they are active in the focused window and conflicting entries use
deterministic replacement order. `onAction` receives the same action strings as
menu activation. Context predicates and menu shortcut fields are not part of
this API. Invalid GPUI keystrokes are rejected by the host without changing the
previous binding set.

### Transport termination and crash diagnostics

Keep the process termination handler in a standalone entry:

```tsx
const root = createRoot(new StdioTransport(), {
  onTransportTermination: createProcessTerminationHandler(),
});
root.render(<App />);
```

For host crash artifacts, set `REACT_GPUI_CRASH_DIR` (the host also accepts
`REACT_GPUI_LOG=off|error|info|debug`). See the [README debugging
section](../README.md#troubleshooting) for crash files, `REACT_GPUI_TAP`, and
the metadata-only tap report.

## Error and recovery boundaries

The failure owner determines the recovery behavior. React Error Boundaries are
consumer code; the other rows are host/runtime contracts:

| Failure                                      | Current behavior                                                                                                         | Owner/recovery                                                                                                        |
| -------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------- |
| React render error without an Error Boundary | `root.render()` throws synchronously and no invalid Commit Batch is submitted.                                           | Add an Error Boundary where the application can render a useful fallback; the renderer does not invent one.           |
| Bad Snapshot/Patch frame or tree invariant   | The host rejects it, shuts down the Runtime Adapter, and exits; it does not drop the frame or retry.                     | Fix the producer/protocol mismatch. A shared Runtime Adapter failure closes every registered Surface on that runtime. |
| Image resource failure                     | The Image node remains in the tree; GPUI renders `fallbackSource` for loading/error states when supplied, otherwise blank output; no `Image` error callback exists in this protocol. | Ship/validate the primary and fallback assets or handle the visual fallback in the tree. |
| GPUI paint panic/internal invariant          | The host panic hook writes crash diagnostics, but there is no safe node-level paint boundary or resume-after-panic path. | Treat the host/window as failed; inspect the crash report rather than relying on a partially painted frame.           |

The strict protocol choice is intentional: v3 revisions and Surface/epoch
identity require both sides to agree on the same tree. The full rationale,
rejected resync/ignore/catch-and-continue alternatives, and focused test
evidence are in [ADR-0008](adr/0008-error-handling-philosophy.md).

## Testing without a display

Install the dev testing package for component-level tests:

```sh
bun add -d @react-gpui/dev
```

Its `render` helper uses a real core `MemoryTransport`, exposes submitted
frames/decoded commits, and injects Press/Key/TextInput/Submit/VisibleRange
events. For behavior-level tests, `renderTestApp` adds accessibility-label
and text locators plus ergonomic interactions over the same dispatch path. It
does not open a window. See
`packages/react-gpui-dev/README.md#behavior-level-testapp-recipe`.

## Known boundaries
- There is no DOM, CSS cascade, or browser event cancellation. `letterSpacing`
  and `zIndex` remain unsupported style fields.
- `pointerEvents` is intentionally not exposed: an overlay with no native
  listener already leaves underlying GPUI hitboxes available, while a
  declaration that suppresses only this node's callbacks cannot express
  listener-present pass-through or partial occlusion.

- Generic transforms (`transform.scale`, `transform.translateX`,
  `transform.translateY`) are unsupported. Positioning is the explicit
  `relative`/`absolute`/`overlay` subset; use `overlay` for anchored
  dropdowns/popovers rather than treating it as a general portal.
- `flexDirection: row-reverse/column-reverse` is a physical layout mirror only; explicit container/text base direction (RTL) and bidi caret/IME semantics are unsupported pending upstream GPUI APIs.
- On Windows, some cursor variants (`alias`, `copy`, and similar) fall back to the default arrow; cursor changes are a no-op in headless environments.
- `Image.source` and optional `Image.fallbackSource` are host-local paths.
  GPUI renders the fallback while loading and after a primary load failure when
  supplied; there is no `Image` `onError` callback, remote URL fetch, or inline
  image-byte transport.
- `TextInput.secureTextEntry` and `keyboardType` are unsupported on the
  desktop GPUI surface.
- Multiline `TextInput` uses native wrapped-line geometry for painting,
  point-to-character mapping, and IME bounds; display-backed testing is still
  required for candidate-window placement. Double-click selects a UAX #29 word
  and triple-click selects the clicked logical line; dragging extends the
  selected range by that granularity. `Cmd-C` on macOS and `Ctrl-C` elsewhere
  copy the selected UTF-8 text to the native clipboard. These are host-owned
  interactions with no JavaScript copy callback.
- `VirtualList` assumes a bounded viewport. Native GPUI measures visible and
  overdraw rows at their natural heights; `estimatedItemSize` supplies the
  initial height hint for unmeasured or not-yet-committed rows. Use `emptyState`
  for an empty list.
- File dialogs require a display-backed native host for actual interaction;
  headless tests cover command/value routing, not OS picker UI.
- Notifications are best-effort platform submissions; delivery and OS
  authorization are not guaranteed.
- Window centering, `revealPath`, and `openWithSystem` are unsupported.
- `Root.zoom()` command routing is covered in headless tests, but its visual
  display-backed toggle effect requires a real desktop adapter; no test-only
  call-count seam is provided.
- A process entry still needs a host Runtime Adapter. A Bun script alone does
  not create a native Surface.

For protocol field-level constraints, invalid values, event payloads, command
ownership, style slot numbers, and evolution rules, use
[docs/protocol.md](protocol.md). For contribution and native-reference work,
use [CONTRIBUTING.md](../CONTRIBUTING.md).
