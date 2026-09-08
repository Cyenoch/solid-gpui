# Cross-platform anchored popup implementation notes

Date: 2026-09-08
Scope: implementation guidance from the unmodified registry packages
`gpui-pre-windows 0.3.3` and `gpui-pre-linux 0.3.3`; this note does not establish
platform acceptance. The parent implementation task owns production changes,
the shared GPUI interface, macOS, the Surface protocol, and Solid composition.

## Recommended boundary

Keep `WindowKind::AnchoredPopup(PopupOptions)` and add an error-returning runtime
anchor update on `PlatformWindow`, forwarded by `Window`. Its coordinates remain
logical, parent-surface-local bounds. Parent identity and grab policy are fixed
for the window's lifetime; a runtime update must not silently reparent or change
input ownership. A shared pure placement function can resolve anchor, gravity,
offset, flip, slide, and resize for absolute-coordinate backends. The provider
owns coordinate conversion and monitor/work-area selection. Wayland uses its
positioner instead. This extends the current
[popup contract](../../vendor/gpui/src/platform/popup.rs) and
[platform-window interface](../../vendor/gpui/src/platform.rs), which currently
exposes resize but no anchor mutation.

Keep requested logical content size separately from the last constrained actual
size. Otherwise moving a resized popup away from a screen edge cannot restore
its requested size. Validate finite, positive size and anchor dimensions before
native conversion; do not turn invalid public requests into arbitrary defaults.
Coalesce unchanged geometry and owner notifications, and execute provider work
after prepaint. These are implementation recommendations, not claims that the
current providers already satisfy them.

## Windows

The exact baseline sources are [window.rs][win-window],
[platform.rs][win-platform], [events.rs][win-events], and
[display.rs][win-display].

| Patch site | Concrete change |
| --- | --- |
| `WindowsPlatform::open_window`, `platform.rs:578` | Resolve `options.parent` against `raw_window_handles` using existing `window_from_hwnd` and `WindowsWindowInner.handle`. Reject a missing owner. Pass the resolved owner into creation rather than using `GetActiveWindow`. Existing helpers are at `platform.rs:216` and `window.rs:1435`. |
| `WindowsWindow::new`, `window.rs:424–545` | Remove the anchored-popup rejection and add its own style branch: `WS_POPUP`, `WS_EX_TOOLWINDOW`, plus the existing DirectComposition flag when applicable. Supply the owner HWND as `CreateWindowExW.hwndParent`. Do not reuse `WindowKind::PopUp`, which adds `WS_EX_TOPMOST` at line 472. |
| `WindowsWindowInner`, `WindowCreateContext` | Retain popup options, requested size, weak owner identity, and last applied device bounds. Split modal parent restoration from popup ownership; the existing `parent_hwnd` is a modal-dialog field. |
| `handle_destroy_msg`, `events.rs:339` | Run `EnableWindow` and foreground restoration only for modal dialogs. It currently foregrounds every `parent_hwnd`, which would steal focus when a popup closes after the user clicks another application. |
| `resize`, `window.rs:629`, and the new anchor updater | Route anchored popup changes through one placement method. `SetWindowPos` updates use `SWP_NOACTIVATE | SWP_NOZORDER`; ordinary geometry updates must not reactivate the popup or raise it globally. |
| `WM_MOVE`, `WM_SIZE`, `WM_DPICHANGED`, `WM_DISPLAYCHANGE` handlers | Invalidate live dependent popup geometry after updating owner state. Re-resolve placement after DPI/work-area changes. Keep the existing renderer resize path and avoid recursively performing the same placement inside synchronous window messages. |

An owned `WS_POPUP` stays above its owner, hides when its owner is minimized, and
is destroyed when its owner is destroyed. These properties come from ownership;
they do not require global topmost status. Hiding the owner alone does not hide
owned windows, so application hide/close policy still needs explicit handling.
[Microsoft window features](https://learn.microsoft.com/en-us/windows/win32/winmsg/window-features)
and [SetWindowPos](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowpos)
define the relevant behavior.

Use physical screen coordinates for the entire platform placement calculation:

1. Scale the anchor corners using the **owner window's** current DPI, then call
   `ClientToScreen` on the owner. This accounts for the actual client origin and
   supports negative coordinates; adding GPUI's stored window origin is not an
   equivalent conversion. [ClientToScreen](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-clienttoscreen)
2. Select a monitor from the converted anchor with `MonitorFromRect`, then read
   `MONITORINFO.rcWork`. Do not assume the owner center or primary display is the
   relevant monitor. [MonitorFromRect](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-monitorfromrect)
   and [MONITORINFO](https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-monitorinfo)
3. Convert requested popup size to device pixels using the target display scale,
   constrain in the same screen space, and position the undecorated client area.
   Existing `WindowsDisplay` stores scale and visible bounds, but its logical
   global origins are each divided by their own display scale (`display.rs:35–76`).
   Mixing those origins across monitors produces an inconsistent coordinate
   space. Expose physical work bounds or query `rcWork` directly.
4. On `WM_DPICHANGED`, retain logical requested size, update rendering scale, and
   schedule a fresh anchor calculation. The existing handler applies the
   suggested native rectangle (`events.rs:865–941`); reconcile that with anchored
   placement without an update loop. [WM_DPICHANGED](https://learn.microsoft.com/en-us/windows/win32/hidpi/wm-dpichanged)

Do not add `WS_EX_NOACTIVATE` to an editable form popup. Preserve native focus,
IME, and accessibility callbacks. Dismissal should inspect whether the next
active HWND belongs to the same presentation family; the existing
`handle_activate_msg` at `events.rs:799` drops `WM_ACTIVATE.lParam`, so a provider
implementation of family-aware dismissal needs that destination HWND. Treat
Escape after IME handling and explicit focus restoration as host policies.

## Linux X11

The exact baseline sources are [window.rs][x-window], [client.rs][x-client], and
[display.rs][x-display].

| Patch site | Concrete change |
| --- | --- |
| `X11Client::open_window`, `client.rs:1605` | Match `AnchoredPopup` and resolve the explicit parent using `window.state.borrow().handle`. The current path always selects the keyboard-focused window. Use the parent's X screen/root and scale for popup construction. |
| `X11WindowState::new`, `window.rs:466–555` | Remove the rejection. Create an undecorated root-child window with `override_redirect = true`, independent of the owner's clipping. Store popup options and requested size. Bypass the unconditional `x + 2` positioning workaround for anchored placement. |
| Atoms and setup, `window.rs:60–88`, `603–660` | Add `WM_TRANSIENT_FOR` for the explicit owner and a semantically appropriate `_NET_WM_WINDOW_TYPE`. Existing `PopUp` uses `_NET_WM_WINDOW_TYPE_NOTIFICATION`, which is not a general form-popover type. For a generic form use `_NET_WM_WINDOW_TYPE_NORMAL`; menu/combobox types require an explicit semantic distinction. Do not mark this window modal. |
| Child ownership, `window.rs:258`, `635`, `1146–1180` | Replace `FxHashSet<Window>` with child metadata containing whether input is blocked; dialogs are blocking, popups are not. Preserve descendant cleanup. Currently `is_blocked` returns true for any child, so simply registering a popup makes its owner modal. |
| `resize`, `window.rs:1422`, and new anchor updater | Resolve parent client coordinates to root coordinates, constrain the requested size, then issue a single `ConfigureWindow` with changed x/y/width/height. Do not use the parent's cached `ConfigureNotify` origin as an unconditional root origin; window-manager reparenting matters. |
| `activate`, `window.rs:1507`, `map_window`, `window.rs:1590` | Add a focused-popup map/activation path. Current activation sends `_NET_ACTIVE_WINDOW` to the window manager, which does not manage an override-redirect window. Use `set_input_focus` only after the popup is mapped, under the actual user-activation policy. |
| Events, `client.rs:951–1004` | Refresh placement on owner geometry changes. Process relevant root properties before the current per-window lookup, which currently discards root property events. Keep FocusIn/FocusOut and XIM state routed to the actual popup; do not mistake moving into a descendant for application exit. |

X11 coordinate translation should use `xcb.translate_coordinates(owner, root, ...)`.
Server-provided coordinates account for the hierarchy; `ConfigureWindow`
coordinates are relative to the actual X parent. An override-redirect popup
therefore uses the root coordinate space. [Xlib coordinate and window functions](https://www.x.org/releases/X11R7.5/doc/libX11/libX11.html)
describe both operations. `XSetInputFocus` requires the target to be viewable;
retain FocusIn/FocusOut as the observed state rather than assuming the request
succeeded. [XSetInputFocus](https://xorg.freedesktop.org/archive/X11R7.5/doc/man/man3/XSetInputFocus.3.html)

EWMH explicitly recommends `WM_TRANSIENT_FOR` for override-redirect popups.
Its window types distinguish ordinary windows, menu popups, combo popups,
tooltips, and notifications. [Compositing-manager hints](https://specifications.freedesktop.org/wm/latest/ar01s08.html)
and [window types](https://specifications.freedesktop.org/wm/latest/ar01s05.html)
are the authoritative contracts. Ownership hints do not replace application
teardown or input management.

**Monitor/work-area work is required.** Current `X11Display::new` represents an
entire X screen at `(0, 0)` and does not override `visible_bounds`; it cannot
constrain a popup to one monitor or avoid panels. There is already RandR query
code in `client.rs:2610–2644` (`randr_get_monitors`) and CRTC queries at
`1938–1954`. Reuse that connection/extension support for active monitor rectangles;
select the monitor containing the anchor. Read the current desktop's
`_NET_WORKAREA`, translate from its viewport coordinate space as needed, and
intersect it with that monitor. Add the relevant atoms and invalidate cached
monitor/work-area facts on RandR/root-property events. EWMH work areas are per
desktop, not per monitor; do not interpret the array as one rectangle per output.
[Root-window properties](https://specifications.freedesktop.org/wm/latest/ar01s03.html)

If the environment lacks monitor or work-area information, record that
limitation in capabilities/acceptance instead of claiming full multi-monitor
qualification. Preserve the provider's existing X11-wide scale policy; a popup
patch does not by itself implement independent per-monitor X11 DPI scaling.

## Linux Wayland

The exact baseline sources are [window.rs][wl-window], [client.rs][wl-client],
and [serial.rs][wl-serial]. Anchored creation already exists at
`window.rs:198–243`; it stores `PopupOptions` and `next_reposition_token`, creates
an `xdg_popup`, and registers a nonblocking child. Extend this implementation.

Protocol facts: anchors are relative to parent window geometry; mapped popups
can reposition starting at version 3. A reactive positioner is reconstrained
when owner conditions change. Each reposition replaces the previous positioner,
and its configuration must be acknowledged. Grabs need a triggering user-event
serial; a grabbed popup's parent must be a toplevel or another grabbed popup.
The topmost grabbed popup receives keyboard focus. Destroy nested popups in
reverse creation order, and destroy the role before its `xdg_surface`.
These are defined in the primary
[xdg-shell XML](https://raw.githubusercontent.com/wayland-mirror/wayland-protocols/main/stable/xdg-shell/xdg-shell.xml),
sections `xdg_positioner`, `xdg_surface`, and `xdg_popup`. The canonical
[Freedesktop source](https://gitlab.freedesktop.org/wayland/wayland-protocols/-/blob/main/stable/xdg-shell/xdg-shell.xml)
was unavailable to the browser; the linked XML mirror contains the protocol
source, not a third-party interpretation.

Concrete changes inferred from that contract and the inspected implementation:

1. Add an anchor update that replaces the stored geometry and calls the existing
   `reposition_popup` (`window.rs:436–457`) with requested size and current
   `parent.window_geometry()` (`894–903`). Reject changed owner/grab fields.
   Return an explicit unsupported error when the negotiated popup version is
   below `REQ_REPOSITION_SINCE`; the current method silently does nothing.
2. Set `xdg_positioner.set_reactive` in `build_popup_positioner` (`313–367`) when
   supported. Keep the existing conversion from surface-local coordinates to
   parent window geometry. A scroll-induced anchor change still needs a new
   positioner; reactive positioning does not know about GPUI element layout.
3. Keep the latest pending anchor and requested size before mapping. The current
   `resize` (`1658–1677`) simply drops changes before `is_configured()`. Moreover,
   `is_configured()` (`1009`) checks only the frame loop, not first buffer
   presentation. Gate requests on actual mapping and drain pending placement
   after first presentation (`draw`, `1933`), not merely the first configure.
4. Preserve configured x/y as well as size if actual placement is reported.
   `handle_popup_event` (`1325–1350`) currently discards x/y. Apply the result at
   the corresponding `handle_xdg_surface_event` (`1059`), preserving normal
   acknowledge/renderer sequencing. Keep desired size independent from the
   compositor's constrained result.
5. For parent resize synchronization, retain the current parent configure serial
   and pass `set_parent_configure`/`set_parent_size` only for the matching parent
   state. Avoid guessing a serial or claiming a future size that does not match
   the parent buffer committed with it.

### Input admission and teardown defects to avoid

`client.rs:1054–1065` currently chooses `max(mouse_press, key_press)` and converts
zero to no grab. `SerialTracker` already has arrival-order and valid-zero
regression cases for selection serials (`serial.rs:44–59`, `112–137`); the popup
path should not reintroduce a numeric-order assumption. Add native popup
activation state containing seat, source surface, actual triggering serial,
press eligibility, and consumption. Reject `grab = true` when that state is
missing or invalid. A selection serial or any earlier press is insufficient,
and `Option::flatten` must not silently downgrade to an ungrabbed popup.

The Solid/native seam must create or arm the eligible popup during native input
dispatch; an asynchronous JavaScript callback cannot guarantee an input grab.
Programmatic/ungrabbed popovers need their own explicitly described focus and
dismissal contract. The protocol guarantees keyboard focus for the topmost
grabbed popup; it does not make that guarantee for the ungrabbed path.

`WaylandWindow::drop` (`749–799`) destroys the current role before its deferred
`state_ptr.close()` walks descendants. `close()` (`1493–1514`) also iterates child
IDs from an unordered hash map. Refactor destruction around an idempotent native
teardown operation: snapshot children in reverse creation order, release borrows,
destroy descendant roles first, destroy this role and remaining surface
resources, then retire the client entry and complete the callback exactly once.
Do not rely on asynchronous GPUI close callbacks alone to order protocol
destructors. Apply the same operation to explicit close, owner disposal,
`PopupDone`, and failed setup; cancel frame/pending-position work at entry.

`params.show` and `params.focus` are not read in this provider's `window.rs`;
`WaylandWindow::new` commits the initial surface unconditionally at line 865.
If the shared host uses prepare-then-show, add a real map gate on first content
buffer attachment. `show = false` alone does not establish that contract here.

## Acceptance and verification limits

Keep only behavioral tests with a material failure mode:

- Shared placement: requested versus constrained size, flip/slide at edges,
  negative screen origin, oversized content, and an owner/popup DPI mismatch.
- Native lifecycle: owner close with nested popups, external application focus,
  stale anchor updates after unmount, and repeated open/close without active
  callbacks or retained native windows.
- Native input: editable form with CJK IME, Tab traversal, Escape during marked
  text, and family-aware dismissal. X11 requires a real focus/XIM session;
  Wayland requires valid-grab and unavailable-grab sessions.
- Wayland ordering: anchor update before first mapping, denied grab, serial
  rollover/zero, version below 3, and sibling/nested reverse destruction. A fake
  request log can prove ordering; compositor sessions prove actual admission.

The available host is macOS. This research ran no Windows or Linux build,
compositor session, native input/accessibility session, or performance capture.
Cross-compilation can prove API/type correctness but cannot qualify these native
behaviors. The main task must label implemented and runtime-verified support
separately. This internal implementation note changes no public API or website
capability claim; public guide and website synchronization belongs to the
production change in the parent task.

[win-window]: /Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gpui-pre-windows-0.3.3/src/window.rs
[win-platform]: /Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gpui-pre-windows-0.3.3/src/platform.rs
[win-events]: /Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gpui-pre-windows-0.3.3/src/events.rs
[win-display]: /Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gpui-pre-windows-0.3.3/src/display.rs
[x-window]: /Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gpui-pre-linux-0.3.3/src/linux/x11/window.rs
[x-client]: /Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gpui-pre-linux-0.3.3/src/linux/x11/client.rs
[x-display]: /Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gpui-pre-linux-0.3.3/src/linux/x11/display.rs
[wl-window]: /Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gpui-pre-linux-0.3.3/src/linux/wayland/window.rs
[wl-client]: /Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gpui-pre-linux-0.3.3/src/linux/wayland/client.rs
[wl-serial]: /Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gpui-pre-linux-0.3.3/src/linux/wayland/serial.rs
