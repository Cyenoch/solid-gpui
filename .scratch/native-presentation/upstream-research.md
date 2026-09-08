# Native presentation and SwiftUI interoperability: upstream research

Verified on 2026-09-08 against platform-owned documentation and protocol source.
This note establishes upstream contracts; it does not establish what the current
GPUI fork implements. The supplied screenshots demonstrate intended interactions,
but cannot establish QuickGUI's implementation, accessibility, lifecycle, or
cross-platform behavior.

## Distinct capabilities

| Capability | Platform contract and design implication |
| --- | --- |
| In-window popover | An application-rendered overlay is a separate product choice. It remains part of the owner's rendering surface; it does not acquire the system popover contract merely by using the same component API. |
| Native AppKit popover | `NSPopover` anchors to a real `NSView`, accepts an `NSViewController`, tracks the positioning view, and offers system-managed close behaviors. Use it when AppKit presentation behavior and appearance are wanted. [NSPopover](https://developer.apple.com/documentation/appkit/nspopover) |
| Custom popup outside its owner's bounds | An `NSPanel` is an auxiliary `NSWindow`. `addChildWindow(_:ordered:)` maintains relative ordering and makes an owner's movement move its attached child. These APIs do not define a complete popover interaction or collision policy. A custom child panel therefore needs an application-owned presentation controller. [NSPanel](https://developer.apple.com/documentation/appkit/nspanel), [attached windows](https://developer.apple.com/documentation/appkit/nswindow/addchildwindow(_:ordered:)) |
| Bidirectional SwiftUI hosting | `NSHostingView`/`NSHostingController` insert SwiftUI into AppKit; `NSViewRepresentable`/`NSViewControllerRepresentable` insert AppKit into SwiftUI. The common seam is a view or controller, not a top-level window. [Hosting view](https://developer.apple.com/documentation/swiftui/nshostingview), [hosting controller](https://developer.apple.com/documentation/swiftui/nshostingcontroller), [view representable](https://developer.apple.com/documentation/swiftui/nsviewrepresentable), [controller representable](https://developer.apple.com/documentation/swiftui/nsviewcontrollerrepresentable) |

**Recommendation:** keep content hosting and presentation independent. An
embeddable GPUI surface can feed an AppKit container, an `NSPopover`, or a
SwiftUI representable. A custom owner-attached popup can initially use a separate
GPUI window. Its success does not prove that GPUI is embeddable.

## View hosting and rendering constraints

- SwiftUI controls the represented AppKit view's outer `frame` and `bounds`;
  changing them independently conflicts with SwiftUI layout. A GPUI host should
  accept the assigned bounds and lay out its internal content. The representable
  coordinator is the intended route for native callbacks to SwiftUI state.
  [NSViewRepresentable](https://developer.apple.com/documentation/swiftui/nsviewrepresentable)
- `sizeThatFits` can run repeatedly with different proposals in a single layout
  pass. **Recommendation:** measurement must be free of mount, presentation, and
  state-changing side effects; keep one layout authority at each boundary.
  [Sizing contract](https://developer.apple.com/documentation/swiftui/nsviewrepresentable/sizethatfits(_:nsview:context:))
- A hosting view reports SwiftUI size preferences and coordinates event delivery.
  `NSHostingView.sizingOptions` can generate minimum, ideal, and maximum
  constraints; fewer options reduce measurements when a fixed-size contract is
  sufficient. **Recommendation:** choose constrained versus intrinsic sizing
  explicitly rather than letting both layout engines resize each other.
  [Hosting view](https://developer.apple.com/documentation/swiftui/nshostingview),
  [sizing options](https://developer.apple.com/documentation/swiftui/nshostingview/sizingoptions)
- AppKit distinguishes layer-backed views from layer-hosting views. Apple
  explicitly disallows adding `NSView` subviews to a layer-hosting view, whose
  custom layer is assigned before enabling `wantsLayer`. A layer-backed Metal
  view created through `makeBackingLayer` is a different arrangement.
  **Recommendation:** inspect GPUI's setup before choosing the insertion point;
  use a regular AppKit container with a GPUI rendering view and native hosting
  views as siblings when necessary.
  [wantsLayer](https://developer.apple.com/documentation/appkit/nsview/wantslayer),
  [Apple's custom Metal view](https://developer.apple.com/documentation/metal/creating-a-custom-metal-view)
- Metal presentation is asynchronous to Core Animation by default, so their
  content need not appear in the same frame. Transaction-based presentation has
  a specific submission procedure, including waiting until the command buffer
  is scheduled; enabling a flag alone is insufficient. **Recommendation:** test
  native/Metal alignment during scrolling and live resize before changing the
  presentation path, and measure any synchronization cost.
  [presentsWithTransaction](https://developer.apple.com/documentation/quartzcore/cametallayer/presentswithtransaction)

## Anchoring and lifecycle constraints

`NSPopover.show` requires a content controller and view, does nothing for an
invisible positioning view, and can update the anchor of an already shown
popover. The edge is a preference. A subrectangle inside a positioning view must
be updated when its own geometry changes; AppKit cannot infer a GPUI element's
virtual layout changes from an unchanged native host view.
[Show contract](https://developer.apple.com/documentation/appkit/nspopover/show(relativeto:of:preferrededge:)),
[positioningRect](https://developer.apple.com/documentation/appkit/nspopover/positioningrect)

**Recommendation:** represent an anchor as a surface identity plus local bounds
and visibility. Resolve it at the presentation boundary. For a custom panel,
convert through the current view/window to screen coordinates and use the
current target display's available region. `NSScreen.visibleFrame` excludes
occupied system areas and must not be cached indefinitely; Apple recommends
backing-coordinate conversion methods over manually multiplying layout values
by the backing scale.
[View conversion](https://developer.apple.com/documentation/appkit/nsview/convert(_:to:)-6u9ir),
[window conversion](https://developer.apple.com/documentation/appkit/nswindow/converttoscreen(_:)),
[visibleFrame](https://developer.apple.com/documentation/appkit/nsscreen/visibleframe),
[backing scale guidance](https://developer.apple.com/documentation/appkit/nswindow/backingscalefactor)

Transient `NSPopover` dismissal is not an exact portable outside-click algorithm:
Apple leaves the precise interactions unspecified and excludes some menu and
panel interactions. The default behavior is application-defined, so a backend
must select its policy deliberately.
[Transient behavior](https://developer.apple.com/documentation/appkit/nspopover/behavior-swift.enum/transient),
[default behavior](https://developer.apple.com/documentation/appkit/nspopover/behavior-swift.property)

An ordinary `performClose` request can be vetoed and can fail while a nested
popover or child window exists. `close` forces closure and cascades to nested
popovers. `popoverDidClose` is the completion notification.
**Recommendation:** distinguish a requested state change from actual native
closure; reconcile controlled Solid state with native dismissal and make
teardown idempotent. A close request is not sufficient evidence that native
resources have disappeared.
[performClose](https://developer.apple.com/documentation/appkit/nspopover/performclose(_:)),
[close](https://developer.apple.com/documentation/appkit/nspopover/close()),
[popoverDidClose](https://developer.apple.com/documentation/appkit/nspopoverdelegate/popoverdidclose(_:))

`NSView` and SwiftUI representables are main-actor APIs. A view can be removed
from its window, and SwiftUI provides an explicit dismantle hook for cleanup.
**Recommendation:** retain a stable native host for a mounted component; update
its content/state instead of recreating it on each reactive update. On disposal,
close owned presentations, invalidate callback identities, cancel subscriptions
and frame scheduling, remove observers/monitors, then release host resources on
the UI thread. Delayed callbacks must not revive a disposed Solid owner.
[NSView](https://developer.apple.com/documentation/appkit/nsview),
[viewDidMoveToWindow](https://developer.apple.com/documentation/appkit/nsview/viewdidmovetowindow()),
[dismantleNSView](https://developer.apple.com/documentation/swiftui/nsviewrepresentable/dismantlensview(_:coordinator:))

## Focus, text input, and accessibility

- The real AppKit responder chain must remain authoritative for native controls.
  `makeFirstResponder` can fail when the existing responder refuses to resign;
  it can also return success with the window itself as first responder when the
  requested responder refuses. **Recommendation:** verify the resulting
  responder, preserve a valid return-focus target, and avoid blanket key
  forwarding between rendering systems.
  [makeFirstResponder](https://developer.apple.com/documentation/appkit/nswindow/makefirstresponder(_:))
- `NSPanel.becomesKeyOnlyIfNeeded` is not a universal setting for form popovers.
  Apple recommends it chiefly when most controls are not text fields; a
  nonactivating panel consults the hit view's `needsPanelToBecomeKey`.
  [Panel key behavior](https://developer.apple.com/documentation/appkit/nspanel/becomeskeyonlyifneeded)
- Custom text views must implement the text input contract, including marked
  text, selection, and character geometry. `firstRect` returns **screen
  coordinates**. **Recommendation:** retest GPUI IME geometry in the actual
  embedded host/window, preserve composition across state updates, and let IME
  cancellation take precedence over an unconditional Escape-to-close handler.
  [NSTextInputClient](https://developer.apple.com/documentation/appkit/nstextinputclient),
  [firstRect](https://developer.apple.com/documentation/appkit/nstextinputclient/firstrect(forcharacterrange:actualrange:))
- Local event monitors do not see events consumed by nested menu/control/window
  tracking loops; returning `nil` suppresses delivery. **Recommendation:** custom
  dismissal needs explicit owner/app lifecycle signals as well as carefully
  scoped monitoring. It must preserve the destination click and recognize nested
  presentations as part of the active presentation family.
  [Local event monitor](https://developer.apple.com/documentation/appkit/nsevent/addlocalmonitorforevents(matching:handler:))
- Standard AppKit controls provide accessibility; custom-drawn controls need
  explicit semantic elements, actions, and notifications. **Recommendation:**
  combine the native island and GPUI accessibility trees with correct parents,
  coordinates, order, and focus, rather than expose duplicate wrapper elements.
  Native hosting alone does not make custom GPUI content accessible.
  [Accessibility for AppKit](https://developer.apple.com/documentation/appkit/accessibility-for-appkit),
  [NSAccessibilityElement](https://developer.apple.com/documentation/appkit/nsaccessibilityelement-swift.class)

## Cross-platform boundary

Win32 `WS_CHILD` windows are confined to their parent; an owned `WS_POPUP` is the
appropriate starting point for escaping an owner's bounds. Ownership controls
stacking, owner destruction, and minimization, but merely hiding the owner does
not hide its owned windows. This differs from AppKit child-window terminology.
[Microsoft window contracts](https://learn.microsoft.com/en-us/windows/win32/winmsg/window-features)

Wayland `xdg_popup` uses parent-relative `xdg_positioner` rules and compositor
constraints. Explicit grabs require a triggering user-event serial and can be
denied. Nested popups must be destroyed in reverse creation order; compositor
dismissal emits `popup_done`. Reactive positioning and repositioning require
protocol version 3 or later. A portable API should carry anchor intent and
activation context, then observe the backend's actual placement/lifecycle.
[Official xdg-shell protocol, interfaces `xdg_positioner` and `xdg_popup`](https://gitlab.freedesktop.org/wayland/wayland-protocols/-/blob/main/stable/xdg-shell/xdg-shell.xml)

## Recommended proof sequence

These are engineering recommendations, not upstream guarantees:

1. Prove custom owner-attached GPUI popup behavior independently, if that is the
   immediate product need. Do not couple this milestone to SwiftUI embedding.
2. Prove an externally hosted GPUI `NSView` inside an AppKit container, with
   lifecycle, bounds, text input, and accessibility independent of a GPUI-owned
   top-level window.
3. Wrap that surface with `NSViewRepresentable`; host it in SwiftUI and in an
   `NSPopover` content controller. This proves GPUI-in-native composition.
4. Add `NSHostingView` islands to the GPUI host container with explicit clipping,
   z-order, layout, and focus boundaries. Only then demonstrate the complete
   GPUI → SwiftUI → GPUI nesting from the screenshot.

Keep the decisive validation cases: (1) editing and IME across both boundaries,
(2) owner movement and anchor scrolling across mixed-scale displays, (3) nested
dismissal plus unmount during an outstanding callback, (4) native/GPUI overlap
and clipping during resize, (5) keyboard and VoiceOver traversal, and (6) idle
and resize/scroll performance against an equivalent pure-GPUI baseline. These
cases validate ownership boundaries rather than mirror implementation details.

## Remaining uncertainty

No screenshot establishes native API choice or production correctness. This
research does not verify the current GPUI rendering-view arrangement, multi-root
runtime behavior, surface extraction cost, platform availability in the project's
deployment target, full-screen/Spaces behavior, or native-island clipping under
arbitrary transforms. Those require code inspection and a measured platform
prototype. The Wayland source was read from upstream `main` (interface version 7)
on the verification date; it is not a pinned dependency claim.
