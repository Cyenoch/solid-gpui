# System popovers

`SystemPopover` mounts its content in an owned native window, so it can extend
past its owner's window boundary. Import it from `@solid-gpui/core`.
The generated `Popover` in `@solid-gpui/core/components` remains an in-window
presentation, including on the Web.

## Controlled form

```tsx
import { Pressable, SystemPopover, Text, TextInput, View } from "@solid-gpui/core";
import { createSignal } from "@solid-gpui/core/runtime";

export function EditName() {
  const [open, setOpen] = createSignal(false);
  const [name, setName] = createSignal("Ada");
  const [error, setError] = createSignal("");
  return (
    <View style={{ gap: 12 }}>
      <Text>Saved name: {name()}</Text>
      <SystemPopover
        open={open()}
        onOpenChange={setOpen}
        width={340}
        height={220}
        placement="bottom-start"
        gap={8}
        accessibilityLabel="Edit name"
        onError={(error) => setError(String(error))}
        slots={{ trigger: <Text>Edit name</Text> }}
        content={() => (
          <View
            accessibilityRole="dialog"
            accessibilityLabel="Edit name"
            style={{ padding: 20, gap: 12, widthPercent: 100, heightPercent: 100 }}
          >
            <TextInput accessibilityLabel="Name" value={name()} onChangeText={setName} />
            <Pressable focusable onPress={() => setOpen(false)}>
              <Text>Save {name()}</Text>
            </Pressable>
          </View>
        )}
      />
      <Text>{error()}</Text>
    </View>
  );
}
```

The trigger slot contains visual content. `SystemPopover` supplies its focusable
Pressable, button role, expanded state, and toggle action. Avoid nesting an
independently interactive button inside that slot. Give icon-only triggers an
`accessibilityLabel`.

`content` is a factory: create the elements inside it, in their destination
Surface. Do not return elements constructed in the parent tree. Solid context
and signals are shared through the existing owner. Keep state that should survive
closing, such as `name`, above the factory. Local content state and resources are
disposed when that popup closes. Unmounting the trigger disposes its presentation.
If controlled `open` is still true when the trigger is mounted again, it creates
a fresh child Surface. Creating or rendering a child root preserves the parent
component's Host Tree association.

## Properties

| Property                | Contract                                                                                                                                                                   |
| ----------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `open` / `onOpenChange` | Required controlled state. Trigger activation toggles it; native dismissal or creation failure requests `false`.                                                           |
| `width`, `height`       | Required logical content size; positive integers, at most 16384 per axis. Native work-area constraints may reduce the actual size.                                         |
| `placement`             | Default `bottom-start`. Supports `top`, `bottom`, `left`, `right`, and each side's `-start` / `-end` variants. Start/end refer to physical left/right or top/bottom edges. |
| `gap`                   | Logical spacing, 0–1024; default 8.                                                                                                                                        |
| `slots.trigger`         | Visual trigger content in the parent Surface.                                                                                                                              |
| `content`               | Factory returning the popup's Solid content.                                                                                                                               |
| `accessibilityLabel`    | Accessible name for the trigger. Label the content dialog separately.                                                                                                      |
| `onError`               | Receives native creation/placement admission failures. Without it, the failure reaches the captured Solid error owner.                                                     |

Size, placement, and gap are read when opening. Changing them takes effect on the
next open cycle. Trigger layout changes, scrolling, and owner bounds changes
reposition the live native window without a JavaScript geometry subscription.
A fully clipped or removed trigger closes its popup. Closed popups retain no
geometry observer or animation timer. The host admits at most 32 live popups.

## Platforms and displays

| Host                               | Implementation and limits                                                                                                                                                                                                                              |
| ---------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| macOS                              | Borderless, key-capable NSPanel attached to its actual owner. AppKit view-to-screen conversion uses logical points; the anchor selects the display and its visible frame. It is not an AppKit NSPopover or SwiftUI host.                               |
| Windows                            | Owned `WS_POPUP` with `WS_EX_TOOLWINDOW`. The owner client origin, monitor work area, owner DPI, and destination DPI are converted into one physical screen space. No global topmost flag or modal owner disabling.                                    |
| Linux X11                          | Override-redirect transient window. RandR monitor bounds intersect the WM's advertised work area. It uses the X11 backend's desktop-wide scale; independent per-monitor scaling is not promised. Requires an active RandR monitor.                     |
| Linux Wayland                      | `xdg_popup` with reactive positioner and repositioning; requires xdg-shell version 3. The compositor chooses final placement and scale. Keyboard focus of an ungrabbed popup is compositor policy and still needs qualification on the target desktop. |
| Web / hosts without native windows | Creation reports an explicit error. No automatic in-window substitute. Use ordinary `Popover` for browser UI.                                                                                                                                          |

Absolute-coordinate providers choose the display from the trigger rather than
assuming the primary display or owner center. Work-area placement can flip,
slide, and constrain oversized content. Requested size is retained, so moving to
a larger work area can restore it. Negative monitor origins and mixed scale
must stay in the provider's coordinate space; do not add JS `screenX`/`screenY`
or multiply logical coordinates by a global scale.

This API creates editable form popovers without an explicit menu grab. Wayland
requires a valid native input activation for a grab; an asynchronous JS command
cannot supply that guarantee. It never substitutes a guessed or stale serial.
The keyboard, IME, accessibility, and external-application dismissal behavior
must be checked in actual Windows and Linux sessions before release qualification.

## Dismissal and lifetime

Escape closes the innermost popup after child key handling. Clicking outside a
child in an owner window closes it. Moving activation outside the popup family
closes activated popups; activating a nested popup keeps its ancestors alive. The host uses the current
platform key window for this decision, so delayed activation observers cannot
close the parent during the transfer.
Explicit dismissal restores owner focus when the popup held activation. Dismissal
caused by another window does not request focus restoration.

Closing an owner or replacing its epoch closes descendants. Each popup gets a
fresh Surface ID and independent event/revision stream. Cancellation names the
original opening request, so closing or unmounting before its ID arrives cannot
leave an orphan window. Valid commits already in transit for a retired Surface
are discarded. A late initial Snapshot receives a terminal close event, allowing
the just-mounted child root to dispose itself. Unknown, never-allocated IDs are
still rejected. QuickJS generation validation retains persistent windows;
popups are disposed with the old generation and recreated from the new controlled
state after activation. Parent context is preserved, while callbacks from retired
Surfaces are rejected.

## Native acceptance fixture

Build the SDK, then run the editable, nested fixture:

```sh
bun run task build
SOLID_GPUI_FIXTURE=fixtures/system-popover.tsx SOLID_GPUI_FIXTURE_OUTPUT=/tmp/system-popover.js bun --bun vite build --config fixtures/vite.config.ts
cargo run -p solid-gpui --features quickjs --bin solid-gpui-host -- --runtime quickjs /tmp/system-popover.js
```

The automatic lifecycle fixture opens nested popovers, unmounts/remounts their
trigger three times, checks a native owner command after each mount, and exits.
Keep it in the foreground while it runs:

```sh
SOLID_GPUI_FIXTURE=fixtures/system-popover-lifecycle.tsx SOLID_GPUI_FIXTURE_OUTPUT=/tmp/system-popover-lifecycle.js bun --bun vite build --config fixtures/vite.config.ts
cargo run -p solid-gpui --features quickjs --bin solid-gpui-host -- --runtime quickjs /tmp/system-popover-lifecycle.js
```

On macOS, run the first-placement regression against all attached displays:

```sh
cargo run -p solid-gpui --example popup_geometry
```

Check opening at every screen edge, moving the owner between displays, scrolling
and removing the trigger, repeated open/close, nested Escape, external-app clicks,
Tab/Shift-Tab, CJK composition, clipboard/undo, and accessibility traversal.
Exercise the actual monitor layouts, scales, and desktop sessions shipped to users.
Deterministic routing/geometry tests and cross-platform compilation do not certify
native input or compositor behavior. Evidence and remaining checks live in the
[implementation record](../.scratch/native-presentation/implementation.md).

SwiftUI/AppKit view embedding is a separate stage; see
[native composition](native-composition.md#swiftui-and-appkit-view-hosting).
