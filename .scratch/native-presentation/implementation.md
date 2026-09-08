# Native presentation implementation record

Date: 2026-09-08
Scope: owned SystemPopover Surfaces and the first-placement regression.

## Delivered design

`SystemPopover` is a core Solid component with controlled visibility, a visual
trigger slot, and an owner-aware content factory. Its child Surface shares the
application connection and context while owning an independent tree, input
model, event stream, and disposer. Host presentation sessions bind owner Surface,
epoch, opening request, and painted anchor. Cancelling the opening request works
before a native Surface ID is acknowledged. Owner retirement closes descendants
in reverse creation order; closed IDs cannot be reused.

Popup work is bounded by at most 32 live sessions. The host records only active
anchors during prepaint, intersects them with the content mask, and reconciles
after drawing. Geometry changes trigger native placement, without a JavaScript
geometry stream or an idle animation timer. The cache includes owner bounds,
display identity and work area, scale, and clipped anchor. Requested content size
is retained independently of work-area constraints.

The linked providers are vendored from gpui-pre 0.3.3 at the upstream package
revision recorded in each provider's `SOLID-GPUI.md`. macOS uses owned NSPanel,
Windows an owned WS_POPUP, X11 an override-redirect transient window, and Wayland
an ungrabbed reactive xdg_popup. Web reports the unsupported capability explicitly.
The public contract and current platform limits live in
[System popovers](../../docs/system-popover.md).

## First-placement regression

The reported macOS screenshot showed the popup at the screen origin, separated
from its trigger. The native probe opens an actual owner and AnchoredPopup,
compares initial bounds with an identical runtime reposition, and rejects either
inconsistent placement or a detached horizontal anchor:

```sh
cargo run -p solid-gpui --example popup_geometry
```

On the attached macOS display (DisplayId 1, scale 2), the owner origin was
(100, 100), its logical content was 400 by 300, and the trigger was at
(50, 50) with size 100 by 32. Before the fix, initial popup origin was (0, 0),
while repeating the same placement yielded (150, 222): FAIL. After the fix,
both origins were (150, 222): PASS. The titlebar accounts for the additional
vertical offset in the window bounds.

`MacWindow::open` resolved the anchor correctly, then its generic top-level
`setFrameTopLeftPoint` overwrote the result. Anchored placement now runs at the
final native placement step, before showing or focusing the panel.

## Verification

- TypeScript transport tests cover shared context with independent child commits,
  cancellation before acknowledgement, native dismissal with one cleanup, and
  opening a new child ID after closure.
- Host tests cover painted-anchor admission, cancellation, ID retirement, owner
  epoch replacement, and persistent-only QuickJS generation validation.
- Geometry tests cover monitor selection with negative and vertical origins,
  monitor gaps, edge flipping, oversized content, and requested-size recovery.
- The native macOS probe enumerates every attached display. This machine currently
  has one attached 2x display; that result is not a physical multi-monitor test.
- macOS native compilation, the Windows GNU cross-target provider check, and a
  Linux Docker compile with X11 and Wayland enabled have passed. These checks do
  not establish Windows/Linux desktop input behavior.

The TypeScript renderer/application/popup suite passes 42 tests; the host suite
passes 5 tests, including native-focus ordering and late-Snapshot retirement.
The website build passes using wasm-bindgen-cli 0.2.121. Its first attempt found
an obsolete absolute icon path in cached build-script output from a different
checkout; regenerating the existing asset build scripts corrected that local
cache. No source compatibility path was added.

## Remaining qualification

Run the native fixture on physical horizontal/vertical display layouts, negative
origins, mixed DPI, monitor disconnection, work-area changes, and owner cross-screen
movement. Check Windows and X11 window-manager behavior in real sessions. Wayland
requires compositor qualification for keyboard focus and outside dismissal of
ungrabbed popups; no guessed activation serial or explicit menu grab is used.
X11 keeps its backend's desktop-wide scale contract.

Editable input, CJK composition, accessibility traversal, clipboard/undo, focus
return, nested dismissal, and reload with an open form require native acceptance
on each shipping platform. Compilation and deterministic tests do not replace it.

SwiftUI/AppKit hosting stages remain unimplemented. They require separating GPUI
render-view lifetime, input coordinates, and accessibility from NSWindow ownership;
this delivery does not expose or reparent a donor window's NSView.


## Nested activation regression

The native QuickJS fixture reproduced the reported crash automatically with
both controlled popovers initially open. Before the fix, the platform key window
was Surface 3 while its deferred GPUI active flag was still false. The dismissal
pass constructed an empty active family, closed parent Surface 2 and child 3,
and rejected a subsequent commit to Surface 2 as unknown, terminating the host.

The deterministic host regression reaches that same ordering between native
activation and its observer notification. It failed with "nested activation must
retain its parent" before the fix and passes after it. Dismissal now uses one
platform focus source. macOS also checks application activation; Windows resolves
the foreground window rather than another application's last thread-active HWND.

A second red regression rejected a valid late Snapshot for a retired Surface.
The host now discards retired traffic and acknowledges a late initial Snapshot
with a terminal SurfaceClosed event. Uninitialized windows do not emit a bogus
Surface 0 close event. Never-allocated IDs remain errors, and retired IDs cannot
be reused. These rules cover native closure racing the JS acknowledgement and
child registration without retaining hidden child windows.


## Trigger remount regression

An automatic native fixture that unmounted and remounted an open nested trigger
failed after its first cycle with "cannot insert a host node from a different
Solid GPUI root". The TypeScript regression reproduced the same error with the
real Solid renderer, conditional mounting, nested child acknowledgements, and
independent Surface commits.

`withRootTransaction` previously rebound its caller's Solid owner. A popup
rendered under its captured parent owner could therefore overwrite the parent's
Host Tree association. Root ownership is now bound explicitly when creating the
new Solid root; a temporary render/dispatch transaction no longer rewrites it.
The regression passes and verifies that reopening sends its command to the parent
Surface and the next nested opening belongs to the fresh child Surface.


The native lifecycle fixture subsequently passed three consecutive nested
unmount/remount cycles and confirmed the owner answered a native bounds command
after each mount. It is retained as
[`fixtures/system-popover-lifecycle.tsx`](../../fixtures/system-popover-lifecycle.tsx).
Website tests pass 6 cases, including all catalog example type checks and real
DOM-free preview/navigation rendering. Actual CJK/IME and multi-monitor hardware
acceptance remains separate from these results.


The user subsequently confirmed that the basic interactions worked in their
manual test. This confirmation is not recorded as CJK/IME, accessibility, or
physical multi-monitor qualification. Initial AppKit placement errors propagate
through normal popup admission instead of panicking during window construction.


## Final verification summary

- Core TypeScript tests: 77 passed, including context, transport, nested lifecycle,
  and remount ownership. Core and native fixture type checks pass.
- Host presentation/application tests: 5 passed.
- Cross-language protocol golden check: passed, with all 40 command kinds and
  invalid popup ownership/cancellation vectors; 4 Rust integration tests passed.
- Protocol regeneration is byte-stable. Native generated bindings match their
  hosts; the public API surface matches the built SDK.
- Website build and all 6 website tests passed after the final SDK ownership fix.
- Native macOS first-placement probe passed on the single attached 2x display.
  The native nested lifecycle probe passed three consecutive remount cycles.
- Windows provider cross-compilation and the Linux X11/Wayland host compilation
  passed. No Windows/Linux desktop or physical multi-monitor run was available.

Documentation synchronization includes both System popover guides, the guide
index, component presentation scope, hot-reload and protocol contracts, native
composition capability notes, the SDK README, website navigation and translations,
and the Popover catalog description. API and protocol outputs were regenerated
from their authoritative sources. The public guide includes both native fixtures
and retains explicit SwiftUI embedding and platform qualification limits.
