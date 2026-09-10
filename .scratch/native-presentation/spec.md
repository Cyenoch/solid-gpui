# Native popovers and SwiftUI composition

Status: design baseline; SystemPopover implementation tracked separately
Date: 2026-09-08
Scope: original research and recommendation. The repository audit below records
pre-implementation behavior, not current capability.

Owner-attached popovers now have a [public guide](../../docs/system-popover.md).
See the [implementation record](implementation.md) for changes, verification,
and remaining native acceptance. SwiftUI/AppKit embedding stages remain proposed.

## Recommendation

Build an owner-attached `SystemPopover` with its own Surface first. Then separate
GPUI's rendering view lifetime from native window ownership, and use that seam
for AppKit/SwiftUI hosting in both directions. Keep the existing `Popover` as an
in-window overlay. A native popup window and an AppKit `NSPopover` are distinct
presentations; neither should silently substitute for the other.

The screenshots establish desired behavior, not QuickGUI's implementation or
production quality. They show two independently useful capabilities:

1. Solid content in a child popup that may extend beyond its owner window.
2. A GPUI page containing SwiftUI, whose popover contains another GPUI view.

This proposal follows the existing [domain invariants](../../CONTEXT.md),
[Native Module decision](../../docs/adr/0016-rust-owned-native-modules.md), and
[input ownership decision](../../docs/adr/0012-host-owned-input-models.md).
The separate [primary-source research](upstream-research.md) records Apple,
Win32, and Wayland contracts. Proposed names below are design sketches, not SDK
exports or an accepted ADR.

## What the repository actually supports

Audited working tree: `a9b98bb24d5052643c760acddd0489ba4433ceb3`, with pre-existing
uncommitted changes. The linked core is `gpui-pre 0.3.3` patched to
[`vendor/gpui`](../../vendor/gpui); macOS is the registry package
`gpui-pre-macos 0.3.3`, not the Zed reference checkout. Its package metadata names
Zed revision `5b055fa789a8b8d38ac951a6e0cde272f66b4495`.
[`Cargo.toml`](../../Cargo.toml) and [`Cargo.lock`](../../Cargo.lock) are the
dependency authority. `cargo tree --locked -p website-host -i gpui-pre-macos
-e features --depth 3` confirmed the actual dependency path; it includes test
features and is not a production performance qualification.

| Area | Evidence | Consequence |
| --- | --- | --- |
| Existing Popover | [`components/popups.rs`](../../crates/solid-gpui/src/components/popups.rs), [`gpui-base/popover.rs`](../../vendor/gpui-kit/crates/base/src/popover.rs) | Native GPUI overlay in the current window, with controlled state, focus, Escape, and outside dismissal. Its `NativeSlot` is not an OS view. |
| Multiple Surfaces | [`surface-host.ts`](../../packages/solid-gpui/src/surface-host.ts), [`renderer.ts`](../../packages/solid-gpui/src/renderer.ts), [`host/mod.rs`](../../crates/solid-gpui/src/host/mod.rs) | Independent trees, routing, teardown, and retired IDs already exist. A native child Surface can reuse this infrastructure. |
| Surface/window association | `Surface.window`, `NativeStateRegistry.windows: HashMap<WindowId, u32>`, `insert_surface`, `window_closed` in `host/mod.rs` | Currently one registered Surface per GPUI window. Multiple embedded Surfaces within one native window require a structural change. |
| Popup primitive | [`platform/popup.rs`](../../vendor/gpui/src/platform/popup.rs), `WindowKind::AnchoredPopup` in [`platform.rs`](../../vendor/gpui/src/platform.rs) | Parent-relative logical coordinates, anchor, gravity, constraints, offset, and grab are already modeled. Extend this seam instead of inventing absolute-position JSX commands. |
| macOS support | Registry `gpui-pre-macos-0.3.3/src/platform.rs`, `MacPlatform::open_window`, line 659 | Explicitly returns `PopupNotSupportedError` for `AnchoredPopup`. `Floating` uses panel level but is not an owner-anchored popup contract. |
| Other providers | Registry `gpui-pre-linux-0.3.3/src/linux/{wayland,x11}/window.rs`, `gpui-pre-windows-0.3.3/src/window.rs`; [`web/platform.rs`](../../vendor/gpui-web/src/platform.rs) | Wayland implements anchored popups; Windows, X11, and Web explicitly reject them. Source inspection is not native platform acceptance. |
| Embedding | Registry macOS `window.rs`: `MacWindow::open`, `Drop`, `make_backing_layer`, `a11y_init`, `first_rect_for_character_range`, `get_frame` | NSWindow, GPUIView, renderer, input, frame callbacks, and accessibility are managed together. An exposed raw NSView pointer is not a supported embedding lifetime. |
| Native modules | [`native/component.rs`](../../crates/solid-gpui/src/native/component.rs), [`renderer/extensions.rs`](../../crates/solid-gpui/src/renderer/extensions.rs) | Typed props/events and retained `NativeView` instances are reusable, but their GPUI elements are not AppKit views. |

The macOS source creates GPUIView with `setWantsLayer(YES)` and supplies its layer
through `makeBackingLayer`; do not infer that it uses the different AppKit
layer-hosting mode. Its AccessKit adapter is currently created with `for_window`.
IME rectangle conversion uses the owning window frame, which must be changed
for a viewport offset inside another view. These are concrete reasons that
reparenting the existing GPUIView is insufficient.

## Module responsibilities and ownership

Use two deep Modules with small Interfaces. The shared host keeps Surface
identity and transport; platform-specific Implementation stays below it.

| Module | Interface callers need | Implementation it owns |
| --- | --- | --- |
| Presentation | Declare anchor, content factory, requested visibility, size, and placement; observe semantic visibility changes | Surface preparation, native show/update/close, owner relationship, placement, focus restoration, dismissal, and generation cancellation |
| Native view host | Mount a declared native view, update typed props, unmount | Native view identity, viewport, AppKit containment, input handoff, accessibility composition, and resource release |

There are real Adapters at each seam: in-window vs child-window presentation;
owned-window vs externally hosted render view. Do not start by exposing a generic
platform plug-in framework or every AppKit property to JavaScript.

Solid owns application state and the Solid Owner Tree. Each Surface owns one
Host Tree and native editing state. AppKit owns native windows/views and the
responder chain; GPUI owns the layout/paint/input within its view. SwiftUI owns
its own view state and layout. Only bounded values and semantic events cross
these systems. Neither JS closures nor SwiftUI values enter the wire protocol.

### Child Surface composition

A popup is a new native render/input context, not a second painting location for
the parent's `NativeSlot`. Allocate a new Surface and Host Nodes for its content.
Keep the same JavaScript runtime and capture the Solid owner at the presentation
call site so contexts remain available. Mount under that owner with the child's
HostTree binding and explicitly retain a disposer. Owner context propagation
alone does not arrange cleanup of a separately created Solid root.

The current `createRootWithRouter` creates its root at call time. An asynchronous
open acknowledgement needs an explicit owner-aware mount operation, with the
parent owner's liveness checked after the await. `withRoot` and Solid ownership
must both target the intended child; reusing a parent-created JSX node is invalid.
Key acceptance includes theme/router context, callbacks, ErrorBoundary/Suspense,
and cleanup after a late native reply.

Closing disposes child-local state and retires the Surface ID. Reopening creates
a new ID. Application state that should survive closure belongs above the popup.
Keep a child window only while the presentation exists; defer hidden-window pools
until measurements justify their additional lifetimes.

### Interface sketch

Prefer the repository's `slots.trigger` convention over introducing a parallel
compound-component family. Make the content factory explicit initially; it is
evaluated in the destination Surface, not serialized as a function:

```tsx
// Proposed interface; this is not available in the current SDK.
<SystemPopover
  open={open()}
  onOpenChange={setOpen}
  placement="bottom-start"
  width={340}
  height={220}
  slots={{ trigger: <Button label="Edit profile" /> }}
  content={() => <ProfileEditor />}
/>
```

This requires a renderer composition Module plus native presentation support;
adding a generated `NativeView` that clones a slot is insufficient. Platform
capabilities remain Rust-declared. Any new Surface Command/Event is defined in
`protocol.bop` and regenerated; native component contracts use the existing
Rust exporter. The orchestration Module must not introduce another wire format,
untyped `invoke` escape hatch, or handwritten copies of generated native props.

Start with explicit finite dimensions and constrained child scrolling. Add
content sizing only with a bounded measurement contract: width is constrained,
height is measured against the work area, unchanged sizes do not relayout, and
alternating results cannot drive an endless resize loop. Placement names express
preferences; the platform reports the actual result after constraints.

## SystemPopover implementation

### Native backend

Implement `AnchoredPopup` in the linked macOS provider using an explicitly owned
child panel/window. Use `NSWindow.addChildWindow` for the relationship; implement
the popup's focus, dismissal, and geometry contract separately. A normal floating
panel or globally elevated `PopUp` does not establish that relationship.
Apple documents child-window ordering and movement in
[addChildWindow](https://developer.apple.com/documentation/appkit/nswindow/addchildwindow(_:ordered:)).

Provide a native way to update the anchor rectangle while the popup is alive;
the current public `Window`/`PlatformWindow` Interface does not expose updating
`PopupOptions.anchor_rect`. Wayland's internal size repositioning does not by
itself cover a scrolling trigger. Extend the existing popup seam and implement
the actual provider behavior before exposing repositioning to callers.

The macOS provider is not currently patched by this workspace. Vendor/patch that
exact package reproducibly, or land an upstream change and pin the accepted
version. Editing the Cargo registry or the unlinked Zed reference has no place
in the delivered implementation.

Do not implement SystemPopover as a true NSPopover merely because of its name.
An AppKit NSPopover remains a separate backend once embeddable GPUI content exists;
it owns system chrome, content controller, and system placement policies.
Use it when that native presentation is the requirement. Prefer direct AppKit
for that backend; SwiftUI is needed only when the content or enclosing UI uses it.
For this backend, distinguish a `performClose` request from confirmed closure;
it can be vetoed or blocked by nested presentation. Forced owner teardown needs
the native cascading close path and completion notification. See the
[upstream lifecycle evidence](upstream-research.md#anchoring-and-lifecycle-constraints).

### Geometry

The anchor identity is `(surface, epoch, node, mount/session generation)`.
Read its latest visible bounds and clipping from native prepaint. Resolve
window/view coordinates using AppKit conversion functions; logical points and
drawable pixels are distinct. Select the relevant display/work area, then
flip/slide/constrain according to policy. Account for titlebar offsets, flipped
views, fractional scale, multiple monitors with negative origins, RTL, and
oversized content. Close if the anchor is unmounted or fully clipped.

Update on anchor scroll/layout, owner move/resize, display/scale changes, and
content-size changes. Keep a latest-value geometry slot per live presentation;
apply at most one changed native update per frame. Never send anchor positions
through JS each frame. Do not mutate window hierarchy inside paint; schedule
reconciliation after layout with session checks. Parent-child window movement
does not remove the need to reconsider work-area constraints.

### Visibility and teardown

Use an explicit lifecycle: closed -> preparing -> visible -> closing -> closed.
Requested visibility is application state; native visibility and its session
number are host facts. Check owner/anchor/epoch validity before allocating, after
each asynchronous operation, before publishing the initial snapshot, and before
showing. Show after a coherent initial content/size is ready, avoiding an empty
flashing panel. A newer close/reopen request invalidates all older replies.

Opening failure rolls back both child Surface and native resources. OS-forced
closure retires the session and emits one semantic change; an unchanged stale
`open=true` must not immediately resurrect it. Require a fresh transition/request
before reopening. Closing the owner closes descendants before itself. Release
event monitors, tasks, subscriptions, focus references, and pending commands
exactly once. A child presentation does not count as an independent application
window for `lastWindowClose`; revisit the registry's current `surfaces.is_empty()`
check and application activation target selection.

QuickJS HMR currently requires snapshots for exactly the open Surface set in
`prepare_generation`. Child sessions must participate coherently in generation
staging or be retired before that set is captured. Do not preserve a native popup
whose Solid owner/epoch was replaced. Verify the chosen rule in process Bun and
embedded runtimes, where native replies can race with reload.

### Input and dismissal

Treat the owner and all descendant popup windows as one presentation family.
Moving focus into a child is not outside dismissal. Escape closes the topmost
dismissible presentation after active IME handling; parent clicks, clicks in other
applications, application deactivation, owner closure, and removed anchors have
explicit policies. Window deactivation alone cannot distinguish these cases.

Keep first-responder changes native. Restore focus to a still-live trigger only
when closing did not deliberately move focus elsewhere. Do not steal application
activation after another app was clicked. A form popup must accept keyboard
focus and native marked-text input. `NSPanel.becomesKeyOnlyIfNeeded` is a policy
choice, not a universal default for text forms. Mouse hit testing, keyboard
shortcuts, Tab/Shift-Tab, context menus, and accessibility focus must follow the
active Surface, not whichever window was registered first.

Wayland grabbed popups require the triggering native input serial. A JS
`onPress` -> asynchronous `openSurface` round trip cannot promise that contract.
Arm/create the eligible popup from native trigger handling with its actual serial,
or explicitly restrict that presentation mode. Do not invent/reuse a stale serial
or silently switch to an ungrabbed window. Model form-popover focus separately
from menu grabs; nested grabbed popups close topmost-first. See the official
[xdg-shell protocol](https://gitlab.freedesktop.org/wayland/wayland-protocols/-/blob/main/stable/xdg-shell/xdg-shell.xml).

## SwiftUI and AppKit composition

### GPUI inside SwiftUI or NSPopover

Expose a real embeddable GPUI NSView and a retained mount object. The mount owns
render state, input handler, subscriptions, viewport, and Surface; it borrows its
enclosing NSWindow and must never close it during unmount. Keep GPUI's internal
Window as a rendering/input context if useful, but remove assumptions that each
context owns a top-level OS window. Avoid a hidden donor window and view theft.

SwiftUI uses `NSViewRepresentable`/`NSViewControllerRepresentable` to construct,
update, size, and dismantle that host. SwiftUI controls the managed outer frame;
GPUI lays out only inside the assigned viewport. AppKit NSPopover instead owns a
content view controller containing this view. These requirements follow Apple's
[NSViewRepresentable](https://developer.apple.com/documentation/swiftui/nsviewrepresentable)
and [NSPopover](https://developer.apple.com/documentation/appkit/nspopover) contracts.

An external SwiftUI/AppKit application already owns NSApplication and its event
loop. Prove an attach/bootstrap mode on that main thread; do not start a second
application loop. Give app-owned vs borrowed windows explicit types/lifetimes.
Window commands such as close/title/resize need ownership-based admission when
the caller is an embedded Surface.

### SwiftUI inside GPUI

A macOS Native Module registers a typed view factory returning an NSHostingView
or NSHostingController. Rust owns the Native Component Instance and retained
bridge; Swift owns its SwiftUI model. Update props on the main thread without
recreating the hosting view or resetting editing state. Swift events enqueue
typed Native Events; they never call JavaScript synchronously.

Use an ordinary AppKit container with the GPUI drawing view and registered
native views, with explicit viewport and stacking rules. AppKit native views
cannot freely interleave with every GPUI draw primitive. Initially support
rectangular native-view regions with defined clipping and overlay ordering;
reject unsupported transforms/occlusion instead of pretending arbitrary JSX
composition works. Test scrolling clips and a GPUI overlay above/below a SwiftUI
region before broadening this Interface. Apple's
[NSHostingView](https://developer.apple.com/documentation/swiftui/nshostingview)
provides the SwiftUI-to-AppKit direction, not the GPUI scene integration.

Measure Metal/Core Animation alignment during native-view scrolling and live
resize. The linked provider already toggles transaction-based presentation in
resize paths; inspect that scheduling before changing it. A synchronization flag
alone is not proof that both views appear in the same frame; Apple's
[presentation contract](https://developer.apple.com/documentation/quartzcore/cametallayer/presentswithtransaction)
specifies command-buffer scheduling requirements and needs latency validation.

At the Rust/Swift seam use declared, versioned native factories with explicit
retain/release rules and bounded DTOs. Keep opaque pointers entirely native.
Use an Objective-C/C-compatible bridge built as an optional macOS artifact;
do not depend on Swift's generic struct layout across FFI. Qualify build, SDK
availability, deployment target, linking, and signing with a packaged fixture.
Plain GPUI SystemPopover should not require the Swift toolchain.

### Required refactor gates

1. Separate rendering view lifetime from NSWindow creation/destruction and main
   loop ownership. Hidden/detached views stop drawing and resume at current scale.
2. Replace the one-window/one-Surface index with membership plus explicit active
   input Surface; keep ordinary single-window lookup simple behind that Module.
3. Convert pointer, caret, drag, and IME geometry through the actual embedded view.
   `NSTextInputClient.firstRect` returns screen coordinates, as required by
   [Apple](https://developer.apple.com/documentation/appkit/nstextinputclient/firstrect(forcharacterrange:actualrange:)).
4. Compose AccessKit subtrees with real AppKit/SwiftUI accessible children. The
   current window-level `SubclassingAdapter::for_window` cannot be installed
   independently for each embedded Surface without an explicit composition
   design. Prove VoiceOver traversal, actions, and focus without duplicates.
5. Keep one live native instance per mounted Host Node; dismantle revokes all
   callbacks, cancels tasks, releases renderer resources, and retires the Surface.

The diagram's full nesting is the final fixture: GPUI page -> SwiftUI region ->
SwiftUI popover -> embedded GPUI form. Both view-host directions must be working
before that is called supported.

## Delivery sequence and evidence gates

| Stage | Deliverable | Gate to continue |
| --- | --- | --- |
| 0: native experiments | Small Rust child-popup fixture plus a separate embedded-NSView spike | Prove an owner-attached editable panel; identify feasible render-view/main-loop/AccessKit seams. Keep experiments isolated; no SDK promises. |
| 1: SystemPopover | macOS `AnchoredPopup`, anchor updates, Presentation Module, independent child Surface, capability reporting | Owner lifecycle, multi-monitor geometry, IME, dismissal, async races, HMR, and idle work acceptance. Ordinary Popover remains unchanged. |
| 2: embeddable view | Owned/borrowed window model and GPUI view mount; SwiftUI/NSPopover hosts GPUI | Two embedded Surfaces in one window with independent focus, correct IME coordinates, resize, accessibility, and cleanup. |
| 3: native view region | Typed SwiftUI factory in a GPUI layout region | Retained identity, native clipping/stacking, semantic event routing, packaged Swift linking. |
| 4: nested showcase and platforms | Full screenshot scenario; then qualify each additional native backend | Nested focus/dismissal, shared application state, generation cleanup, measured work bounds. Browser advertises in-window presentation only. |

Stages 2 and 3 share a native view seam but have separate acceptance criteria.
Do not block the useful child-window capability on a generalized cross-platform
embedding framework. Reopen domain invariants in an ADR only when an experiment
establishes which rendering/input responsibilities change; this research does
not silently amend `CONTEXT.md`.

## Only the key tests

Keep behavioral tests at the Module Interface. Do not test prop forwarding,
every setter, the state machine's private representation, or fixed frame-time
thresholds in unit tests.

| Test | Failure it must detect |
| --- | --- |
| Lifecycle race | Open -> close/unmount/reload before native readiness -> reopen: no orphan panel, stale callback, ID reuse, pending request, or surviving owner resource. Owner shutdown closes descendants. |
| Composition | Child reads parent context and updates shared state; async/error handling and disposal stay with the correct owner, Surface, epoch, and listener revision. Sibling editing state remains intact. |
| Placement | Bottom/right screen edge, negative monitor origin, mixed scale, moving/scrolled/clipped trigger, and oversized content produce a reachable popup without feedback loops. |
| Native editing and dismissal | Tab, Shift-Tab, Escape during/after CJK composition, copy/paste/undo, nested menu/popup, external app click, and focus restoration route to the intended view. |
| Embedded viewport and accessibility | Two GPUI regions and one SwiftUI region resize/scroll without stale hit areas or IME coordinates; VoiceOver reaches both trees without duplicates; detach leaves no callback. |

Deterministic tests prove routing/lifecycle/geometry invariants. Actual macOS
fixtures prove AppKit, IME, focus, GPU, and accessibility behavior. Run the
original nested scenario in light/dark, narrow/wide, 1x/2x, and repeated
open/close conditions. Additional platforms need their own real sessions.

## Work bounds and measurements

- Content edits invalidate the child Surface/Native Component Instance and only
  parent computations that consume changed application state. They do not mount
  the page, trigger, or SwiftUI root again.
- Geometry work is bounded by live presentations whose dependencies changed,
  coalesced once per frame. Reuse normal prepaint bounds instead of walking the
  full Host Tree. Scrolling geometry does not require a JS commit.
- A closed/detached presentation owns no frame timer or geometry monitor.
  Native tasks and observations end with its session. Measure retained windows,
  Surface/owner/task counts and GPU resources, not RSS alone (caches can retain memory).
- Measure trigger-to-first-present, CPU draw p50/p95/p99, input-to-present,
  reposition cadence, idle wakeups, and repeated-open resource growth separately.
  Compare an editable child popup against the same content in an ordinary GPUI
  window; compare each embedding direction against its native baseline.

Use [performance-analysis.md](../../docs/performance-analysis.md), production
features without `test-support`, fixed runtime/viewport/theme/scale, actual display
refresh, monitor on/off, and serial captures below five minutes. No performance
numbers or native acceptance are claimed by this research-only task.

## Documentation synchronization and verification

The component guide and website Popover recipe state the existing window-local
limit. The native composition guide and its explicit Chinese translation record
the absence of SwiftUI/AppKit view hosting. The documentation index links this
proposal without registering proposed components in the generated catalog.
No Native Contract or canonical wire schema changed, so regeneration is not
required. Website recipes and their explicitly named Chinese translation also
describe the current window-local behavior.

Verification on 2026-09-08:

- `bun run --cwd examples/website typecheck`: passed, including route generation.
- `bun --conditions=browser test examples/website/tests`: 6 passed, 0 failed.
  These checks include documentation highlighting, Markdown parsing, generated
  component example types, translations, and retained navigation.
- Oxfmt checks for both changed recipe files and scoped `git diff --check`: passed.
- Local Markdown link, fence, and new-file whitespace checks: passed.

No Rust implementation, Native Contract, or wire schema was changed by this task.
No native popup/embedding prototype, packaged build, VoiceOver session, or
performance measurement was performed. Platform support statements above come
from the linked source, and proposed acceptance gates remain future work.
