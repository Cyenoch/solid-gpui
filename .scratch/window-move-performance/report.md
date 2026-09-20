# Window movement invalidation: implementation and verification

Investigation: 2026-09-19. Implementation verification: 2026-09-20.
Scope: issue [#2](https://github.com/Cyenoch/solid-gpui/issues/2), shared invalidation, native provider recovery, and popup movement. The implementation and evidence below supersede the historical investigation sections that follow. Windows/Linux native qualification remains outstanding.

## Implemented repair and current evidence

- Shared bounds callbacks always sample native state and notify observers. They
  refresh for viewport, scale, display identity, visual state, and pointer changes
  relevant to client content/tooltips or an active drag. Passive movement outside
  content does not force a tree rebuild, including in active windows.
- SolidRoot reconciles position-only popup movement from painted anchors after
  releasing the owner-window borrow. Viewport/revision freshness and observer
  identity are checked again at flush time. Resize and pending commits retain
  post-layout reconciliation.
- Windows preserves parked callbacks through repeated minimization, checks the
  restored surface size, consumes a one-shot recovery flag, and publishes display
  metadata changes. X11 publishes same-size maximization/fullscreen/tiling changes.
  Wayland retains its existing forced-redraw configure path.
- macOS queues popup frame changes after the GPUI borrow ends. Native popup
  creation exposed a nil-screen crash in maximize sampling; unmapped panels now
  return false before sending `visibleFrame`. The failing native creation and
  successful popup opening after the fix were both observed.

### Executed measurements

The macOS native probe used GPUI 0.3.5, the development profile, no HUD, 500 rows,
an 800x600 logical viewport and scale 2. Same executable SHA-256:
`f69279eefb163bef6b3e33bb243ce6d4e2198c5a90e0cbdf8d8a96da86d7a479`.
The `--legacy-refresh` option explicitly refreshes from the observer; it is a
forced-refresh reference, not an original historical checkout.

| Native movement sequence | Bounds callbacks | Root renders |
| --- | ---: | ---: |
| Forced-refresh reference | 116 | 116 |
| Candidate | 116 | 0 |

Raw logs: [candidate](native-candidate.txt), [reference](native-reference.txt).
These are render counts, not frame rate, latency or CPU savings. The automated
sequence used native AppKit `setFrameOrigin`, 120 attempts plus origin restoration;
delivered callbacks, not attempted calls, define the comparison.

Physical titlebar dragging moved x=220 to x=400 while callbacks rose 1 to 3 and
renders stayed 7. Hover changed the button color and a click incremented the
counter. Native edge dragging changed the viewport from 800x600 to 700x500;
the counter remained functional and the retained rows scrolled afterward.
See [interaction counters](native-interaction.txt),
[resized hover state](native-resize-hover.png), and
[resized scroll state](native-resize-scroll.png).

Shared GPUI's final suite passed **387/387**, including the 8 new bounds cases
and current list regressions. Spring timing uses the executor clock. Popup
scheduling's missing-anchor regression and both end-to-end passive-move/resize
popover tests passed. Native macOS provider checking and the profiling host build
passed. [Successful native popup opening](native-popup-open.png) confirms that
new-panel visual-state sampling no longer crashes.

The earlier full SolidRoot run reported **221 passed, 13 failed, 1 ignored**
while other subsystems were changing. On the resumed native-verification run,
the current workspace passed **387/387 shared GPUI tests** and **233 SolidRoot
tests, with 1 ignored**. The profiling host was rebuilt successfully before
native interaction. This supersedes the earlier whole-workspace failure status.

### Resumed macOS popup qualification

Host: `cargo build --locked -p solid-gpui --bin solid-gpui-profile --features
frame-profile`, with `SOLID_GPUI_PROFILE_HUD=0`. Rebuilt binary SHA-256:
`d3922d26431cbf61924bcc93deccf75fba21fa5a81af2f03f841d2470efa5782`.
The scratch fixture's `--open-nested` option opens both levels on bounded startup
timers for continuous native geometry/lifetime checks without repeated trigger
clicks. It does not modify production popup lifecycle or dismissal behavior.

- Real keyboard input changed `Ada` to `Grace`; the owner's saved-name label
  updated and retained the value after popup dismissal and reopening. See
  [edited popup](native-popup-edited.png).
- Two native popup levels were simultaneously visible. Moving the owner from
  `(464,259)` to `(-1800,300)` moved the outer popup from `(484,439)` to
  `(-1780,480)` and the inner popup from `(500,571)` to `(-1764,612)`.
  Both relative offsets were preserved across displays, including negative
  desktop coordinates. See [native geometry](native-popup-geometry.json) and
  [moved popup family](native-nested-moved.png).
- Programmatic native resize changed the owner outer frame from `800x632` to
  `700x532`; both popup windows remained at their top-anchored positions in the
  geometry capture. Subsequent foreground loss dismissed them, so this is a
  native geometry check, not a claim of uninterrupted visual focus.
- Pressing the owner's identified native `AXCloseButton` with two descendants
  open removed all three windows; the host exited with status 0. See
  [owner-close evidence](native-popup-owner-close.json).
- Physical titlebar dragging dismissed the popup family as an outside action.
  Passive following was therefore checked using native AX position changes,
  not by claiming that the dismissed popup followed a physical drag.

Repeated external focus changes interfered with Escape delivery. Some attempts
left all windows present; another removed both popups. Innermost-first Escape
ordering is **not qualified** by this run. AX setValue was not counted as text
entry; the successful editing check used actual keyboard events. Windows/Linux
native restoration, mixed-DPI movement, and platform visual-state checks remain
unverified. The native popup follow result does not independently measure root
render counts; use the earlier movement measurements and deterministic popup
tests for that claim.

The native probe sources remain in this diagnostic directory; temporary package
example registration was removed. English/Chinese performance and SystemPopover
guides plus all affected vendor patch records were synchronized; website imports
those guides directly. The website suite passed 8 tests / 3,950 assertions,
including the content test's 1,716 assertions. No public API or generated SDK
contract changed, so API generation was not needed. No browser WASM build or
Windows/Linux cross-compilation is claimed.

## Historical investigation (before implementation)

## Decision

Fix invalidation ownership in shared GPUI, not by suppressing Win32 messages, throttling input, or changing Solid rendering. A bounds notification must synchronize native state and notify observers; it must not automatically invalidate every cached view.

The production rule should be:

- Refresh for changed layout/raster inputs: logical viewport, scale, display, and visual window state.
- Preserve pointer-dependent redraws when movement changes client-relative pointer state for an interested window.
- Preserve application-owned invalidation from bounds observers, commits, animations, input, and popup positioning.
- Keep surface recovery/presentation as an explicit platform responsibility, independent of geometry equality.
- Leave the DirectX resize algorithm and the frame scheduler alone in this repair.

A three-field `viewport/scale/display` comparison proves the main optimization but is not a complete production contract. The retained experimental patch adds a conservative hovered-pointer check; it still does not implement visual window-state tracking or explicit restore invalidation. Do not merge that patch as the whole fix.

## Source and measurement identity

- Checkout HEAD observed: `a542603c275bdcbc336fddd12825584fdc661157`.
- Workspace dependencies and Cargo.lock resolve `gpui-pre` and native providers at **0.3.5**, patched to `vendor/` by [Cargo.toml](../../Cargo.toml). The performance skill's older 0.3.3 reference is not this linked implementation.
- Windows provider provenance: Zed `d89e9c2124b2786a390c7a451c7488601b4da2e1`, recorded in [vendor/gpui-windows/Cargo.toml](../../vendor/gpui-windows/Cargo.toml).
- Original and restored `vendor/gpui/src/window.rs` SHA-256: `267ccdbc63c05218acba134023c6d461f66b9320eb4222d9d82f007c9fc72170`. Production source was restored byte-for-byte after the experiment.
- Probe: macOS host, `cargo test`, GPUI TestAppContext, no native window/GPU, logical viewport 800x600, initial test-platform scale 2.0, changed scale 1.25. No monitor/HUD, vsync, physical refresh rate, or input-to-photon measurement applies.
- Test profile reports unoptimized plus debuginfo; workspace package overrides optimize GPUI, Taffy and slotmap at level 3. No CPU-time comparison is claimed.
- Issue measurements belong to the reporter's Windows 11 / Ryzen 7 9700X environment at checkout `f3f8590b`. They are accepted as reported, not reproduced or extrapolated here.

## 1. Proven mechanism

```text
WM_MOVE / WM_SIZE
  -> Windows moved / resize callback
  -> Window::bounds_changed
  -> Window::refresh
       refreshing = true
       invalidator dirty = true
  -> frame request
  -> draw + present
```

Source locations below refer to the restored tree:

1. [events.rs:190-221,239-292](../../vendor/gpui-windows/src/events.rs): move updates origin/display and fires `moved`; size updates logical size and fires `resize`.
2. [window.rs:1841-1855,2667-2677](../../vendor/gpui/src/window.rs): both callbacks call `bounds_changed`, which samples scale/viewport/display/mouse, unconditionally refreshes, then runs bounds observers.
3. [window.rs:2248-2252](../../vendor/gpui/src/window.rs): `refresh` sets both dirty state and `refreshing`.
4. [view.rs:386-418,503-508](../../vendor/gpui/src/view.rs): cached prepaint/paint reuse requires `!window.refreshing`. This is a whole-window cache-bypass request, not merely a request to present the previous scene.
5. [window.rs:1783-1802,3392-3463](../../vendor/gpui/src/window.rs): dirty or forced frames construct/layout/prepaint/paint the tree and present; clean frames can present the existing scene without rebuilding it.

This explains element-count-dependent work. It does not prove that Taffy alone accounts for the reporter's CPU delta; element construction, layout, prepaint, paint and native event overhead were not separately profiled here. Text shaping and other subsystem caches can still reuse their own results.

### Presentation is not rebuilding

`Window::present` submits the retained scene via `platform_window.draw` ([window.rs:3327-3341](../../vendor/gpui/src/window.rs)). That can still involve GPU/backend work. The objective is zero unnecessary **tree rebuilding**, not a promise of zero presentation or zero process CPU.

Windows requests frames continuously: its vsync thread calls `RedrawWindow(... RDW_INVALIDATE)` for every tracked window ([platform.rs:366-406](../../vendor/gpui-windows/src/platform.rs)). A modal move/menu loop also uses a timer to pump work and request drawing ([events.rs:295-334](../../vendor/gpui-windows/src/events.rs)). Neither makes every request a tree draw; the shared dirty/force check decides that.

Do not claim exactly one draw per move or per vblank from source alone. Dirty requests can coalesce, and multiple frame sources exist. Also, plain dirty requests without force or next-frame callbacks bypass the inactive-window throttle ([window.rs:1729-1740](../../vendor/gpui/src/window.rs)); that throttle is not the fix.

## 2. Executed red/green mechanism experiment

The probe crosses the real `Window::bounds_changed`, platform resize/scale callbacks, observer dispatch and frame-request callback. Its root render counter observes actual `Render::render` invocations, not a reimplemented predicate.

Command used for baseline and candidate:

```sh
BOUNDS_PROBE_EXPECT_CLEAN=1 cargo test --locked -p gpui-pre --lib investigate_bounds_invalidation -- --nocapture
```

| Probe case | Original root renders | Experimental gate root renders |
| --- | ---: | ---: |
| 60 clean requests with `require_presentation=true` | 0 | 0 |
| 60 unchanged `bounds_changed` callbacks, passive observer | 60 | 0 |
| Observer notifications in that case | 60 | 60 |
| 60 same-size resize callbacks | 60 | 0 |
| One real resize to 640x480 | 1 | 1 |
| One scale change to 1.25, logical size unchanged | 1 | 1 |
| Unchanged bounds with an observer calling `cx.notify()` | 1 | 1 |
| One forced frame request with required presentation | 1 | 1 |

Baseline: the intended clean-bounds assertion failed with `left: 60, right: 0`. Candidate: **1 passed, 0 failed**, 376 unrelated tests filtered out.

Important experimental limits:

- TestAppContext auto-draws dirty windows while flushing effects ([app.rs:1753-1765](../../vendor/gpui/src/app.rs)). Thus the original output label `coalesced_same_size_callbacks` is misleading: those 60 callbacks were **not coalesced into one production frame**. The table correctly calls them same-size callbacks. These are work-bound results, not cadence results.
- The test platform has no move simulator and returns a fixed mouse position. The unchanged-bounds case exercises the shared callback target, not Win32 `WM_MOVE`, actual origin displacement, stationary-pointer hover, or popup positioning.
- Forced requests use `require_presentation=true` to avoid the inactive-window throttle. This checks preservation of the force branch, not actual GPU device recovery. An initial probe with false was correctly throttled and was corrected before the red/green runs.
- An initial external integration-test attempt could not reach private `TestAppContext::test_window`; the probe was moved to an internal GPUI test module. No public API was widened.
- Both successful compilation runs emitted the existing `block 0.1.6` future-incompatibility warning.

Retained artifacts:

- [Probe source](window_bounds_probe.rs).
- [Baseline output](baseline-output.txt).
- [Candidate output excerpt](candidate-output.txt).
- [Experimental core-only patch](experimental-core-gate.patch).

The probe and patch are archived investigation artifacts, not enrolled in the test suite. To replay in a disposable checkout, copy the probe into `vendor/gpui/src/window_bounds_probe.rs` and temporarily add this at module scope in `window.rs`:

```rust
#[cfg(test)]
#[path = "window_bounds_probe.rs"]
mod window_bounds_probe;
```

Run the command against the original code, apply the experimental patch, then run it again. Restore the module declaration and production source afterward. Do not run both configurations concurrently.

## 3. Correctness boundaries missed by a three-field fix

### Pointer state: moving a window can change its pixels

Windows samples `GetCursorPos` and converts it through `ScreenToClient` ([Windows window.rs:805-816](../../vendor/gpui-windows/src/window.rs)). A stationary screen cursor changes client-relative position when the window moves underneath it.

GPUI redraw recomputes the hit test ([window.rs:3459](../../vendor/gpui/src/window.rs)); hover styles are computed during paint ([div.rs:2516-2523](../../vendor/gpui/src/elements/div.rs)); drawing resets the cursor style ([window.rs:3279](../../vendor/gpui/src/window.rs)). Suppressing all move redraws can therefore leave hover styling/cursor stale until unrelated input.

Recommended conservative rule: when client-relative pointer position changes and the window owns the pointer, retain a redraw. Use the existing `is_window_hovered` semantics; on macOS it means active, whereas Windows/Linux use tracked hover ([window.rs:2794-2807](../../vendor/gpui/src/window.rs)). Include active drag-dependent rendering where applicable; the drag visual reads mouse position ([window.rs:3449-3453,5693-5703](../../vendor/gpui/src/window.rs)). Do not synthesize application pointer-move events from WM_MOVE: [ADR-0010](../../docs/adr/0010-opt-in-high-frequency-event-streams.md) preserves their distinct ordering and opt-in contract.

A raw unconditional `old_mouse != new_mouse` term would revive full redraws while moving under a distant cursor. Conversely, comparing only hitbox identities can miss tooltip/drag coordinate dependencies. Neither is the recommended first optimization. Hover ownership and pointer-dependent cases require native acceptance; the issue does not establish the cursor's location in its synthetic benchmark, so no speedup is promised for all hover configurations.

### Visual window state can change without changing the three geometry fields

Titlebar rendering branches on `window.is_maximized()` ([title_bar.rs:285-290](../../vendor/gpui-kit/crates/component/src/title_bar.rs)). Client-side border geometry depends on decoration/tiling state ([window_border.rs:89-105,131-150](../../vendor/gpui-kit/crates/component/src/window_border.rs)). Wayland updates fullscreen/maximized/tiling during configure ([Wayland window.rs:1071-1108](../../vendor/gpui-linux/src/linux/wayland/window.rs)).

A shared gate needs a previous/current comparison of these visual state inputs, not just width/height/DPI/display. Track the small native snapshot privately, initialized with the window; include maximized/fullscreen and `Decorations` (which includes tiling). Keep existing appearance, control-layout, visual-viewport, insets and activation refresh paths. Do not create a public change-mask protocol or a second notification API for this fix.

Some platform state changes already have explicit appearance/redraw callbacks. Preserve them rather than assuming every change arrives as resize. Same-size maximize/restore and tiled decoration transitions belong in acceptance even if ordinary OS placement usually also changes size.

### Native popup observers are not redundant

The bounds observer in [renderer.rs:1198-1203](../../crates/solid-gpui/src/renderer.rs) schedules window observation and calls `cx.notify()` while a popup observer exists. Rendering defers popup reconciliation ([renderer.rs:1289-1294](../../crates/solid-gpui/src/renderer.rs)); reconciliation compares anchor, owner bounds, scale and display, then repositions the native popup ([host/popup.rs:116-162](../../crates/solid-gpui/src/host/popup.rs)).

Owner movement is part of the documented [SystemPopover contract](../../docs/system-popover.md): owner bounds changes reposition the live native window without a JS geometry subscription.

Therefore:

- Keep bounds observers unconditional and after state synchronization.
- Keep this popup `cx.notify()` in the correctness-first repair. Removing it without replacing its positioning path would break popups.
- Explicitly expect continued owner-tree redraws while a native popup is open. The core optimization is not sufficient to remove that separate cost.
- A later, separately measured popup optimization can reconcile native owner-position changes without rebuilding the root. It must preserve the post-layout anchor path for resize, commits and scrolling. Anchors come from rendered bounds, which are cleared at render start and populated during painting; calling reconciliation blindly inline can use stale/missing anchors or re-enter the owner window. Do not replace the notification with an inline callback as part of a one-line optimization.

Closed popups remove their observer when the final anchor is removed ([host/popup.rs:309-314](../../crates/solid-gpui/src/host/popup.rs)). There is no demonstrated permanent listener leak here.

### Window observation and application state

`on_next_frame` callbacks run before the dirty check ([window.rs:1765-1783](../../vendor/gpui/src/window.rs)), so a clean frame can still deliver the native window observation. `emit_window_observation` deduplicates width/height/scale ([renderer.rs:1214-1259](../../crates/solid-gpui/src/renderer.rs)). Preserve that path; do not manufacture JS resize events for pure movement or remove the frame wakeup it needs.

The probe confirms a bounds observer can still explicitly notify and redraw under the candidate gate. Custom consumers whose output depends on screen position must own that notification rather than depend on an accidental blanket refresh.

## 4. Windows surface lifecycle must remain explicit

### Same-size resize is already cheap at the GPU-resize boundary

[DirectXRenderer::resize:485-489](../../vendor/gpui-windows/src/directx_renderer.rs) returns immediately if physical width/height are equal. Genuine size changes execute `ResizeBuffers` and recreate resources at lines 501-518.

Do not rewrite this algorithm, stretch stale content during live resize, or debounce native resize messages. The same-size waste under investigation is above this boundary.

### Restore currently relies on implicit invalidation

[events.rs:239-268](../../vendor/gpui-windows/src/events.rs): minimize parks the frame callback; restore reinstalls it and skips `renderer.resize` for that restoring message. Shared `bounds_changed` currently guarantees dirtying even if geometry is equal. Its removal must not silently remove the only guaranteed redraw on a non-activating, same-size restore.

Recommended companion change: when restoring the callback, mark one pending forced render through the existing `force_render_pending` mechanism, before the next normal frame request. Avoid synchronous nested draw calls. Keep activation and visibility notifications unchanged. This conservative one-shot restore redraw removes dependence on geometry side effects without imposing any per-move work.

There is also a **separate source-level inconsistency**: the restore comment refers to `update_drawable_size_even_if_unchanged`, but this implementation suppresses renderer resizing on restore. If client size changed while minimized, logical and swapchain sizes may diverge. Visible failure has not been reproduced here. Add that exact Windows scenario to acceptance; any repair should allow the existing equality-checked renderer resize to run, not invent a new forced swapchain allocation. Do not claim the invalidation gate fixes this pre-existing issue.

### Device loss and OS paint must survive

Device-loss recovery already sets `force_render_pending`, marks the renderer drawable and uses forced rendering to rebuild stale atlas references ([events.rs:1304-1320,1329-1371](../../vendor/gpui-windows/src/events.rs)). Preserve this path, including deferral across re-entrant draws. Preserve `ValidateRect` and OS paint handling.

Microsoft's primary contracts distinguish [WM_MOVE](https://learn.microsoft.com/en-us/windows/win32/winmsg/wm-move), [WM_DPICHANGED](https://learn.microsoft.com/en-us/windows/win32/hidpi/wm-dpichanged), and [WM_PAINT](https://learn.microsoft.com/en-us/windows/win32/gdi/wm-paint). In particular DPI changes require applying the suggested position/size; the current backend's `SetWindowPos` can synchronously emit nested move/size messages. The repair must retain those callbacks and state ordering.

## 5. Proposed implementation scope

### Shared GPUI

Modify `Window::bounds_changed` in place, without changing its public signature:

1. Read current platform state once per field.
2. Compare against prior viewport, scale, display, and the small visual-state snapshot.
3. Determine whether updated pointer coordinates require current pointer-dependent visuals to redraw.
4. Store the new state unconditionally, before notifying consumers.
5. Call `refresh` only for those output-relevant changes.
6. Notify all bounds observers regardless of the refresh decision.

Work bound: constant-size core comparisons plus existing observer work. Passive same-display moves no longer cause O(tree size) rebuilding. Interested observers can still invalidate; their cost is not hidden or suppressed.

### Windows provider

Make restore's one-shot rendering demand explicit using the existing pending-force mechanism. Preserve move/display detection, native DPI updates, renderer resize/error handling, modal-loop pumping, paint validation and device recovery. Validate the separate restore-size inconsistency before deciding its correction.

### solid-gpui

Retain current window-observation and popup lifecycle behavior. No protocol/schema/TypeScript change is necessary for the core repair. Document the popup-open redraw exception rather than calling the entire application move path zero-cost.

### Tests and maintenance

Keep only behavioral cases that detect a plausible regression: no unnecessary render with observers preserved; actual resized/scaled painted output; pointer-dependent hover/cursor; visual window-state changes; restore/force rendering; popup placement. Add a real test-platform move/pointer seam if keeping automated move tests: directly calling `bounds_changed` cannot validate origin movement.

On implementation, update `vendor/gpui/PATCHES.md` and the Windows provider patch note. Synchronize applicable guidance in `docs/performance-analysis.md` and its explicit Chinese copy if the guide gains a reusable contract; the website imports those sources. No generated SDK/API change is implied.

## 6. Release acceptance

Run correctness first, then native timing serially with all compilation finished. Keep the HUD disabled for headline CPU measurements. Record binary hashes, profile/features, window size, scale, monitor, input rate, dataset, background load, and raw samples following [performance-analysis.md](../../docs/performance-analysis.md).

| Scenario | Required evidence |
| --- | --- |
| Passive, same-display pure move; no popup/animation; cursor outside client | WM_MOVE count increases; steady-state root draw count remains flat; content stays correct |
| Real titlebar drag and programmatic movement under a stationary cursor | Hover, cursor, tooltip and active-drag visuals remain correct; necessary redraws are attributed separately |
| Same-size resize, genuinely unchanged visual state | No unnecessary root render; no ResizeBuffers |
| Narrow -> wide -> narrow, including top/left edge drags | Correct text, controls, hit testing, viewport and surface dimensions throughout |
| Cross-display equal DPI; mixed DPI 100/125/150/200% | Current display and scale, sharp text, correct layout and popup placement; no stale scaled scene |
| Same-size maximize/restore/fullscreen; Linux tiling/decorations | Correct control icon, border/shadow and client insets without relying on size changing |
| Minimize/restore without activation, plus updates while minimized | Restored content is current and a frame is not stranded |
| Size or DPI changes while minimized | Logical content and swapchain dimensions agree after restore |
| Open popup, scrolling/removal of its anchor, nested popup, move owner | Popup follows/closes correctly; popup-open redraw cost reported separately |
| Animations/typing/IME while moving; multiple windows | Independent notifications and frame work continue; caret/candidate placement is checked natively |
| Device loss/recovery, occlusion/exposure, re-entrant paint | Forced refresh and presentation still work without nested App borrows or busy paint loops |

Report at least native move/size event counts, root draw count, present count, CPU-ms per delivered event and idle-subtracted CPU delta. Use both a bare window and the real consumer tree at 20/60/120 Hz, several serial runs, identical baseline/candidate geometry. Verify how many messages actually arrived rather than treating loop iterations as delivered move events. Synthetic movement does not qualify physical titlebar dragging.

The expected improvement is removal of **tree-size-dependent** work in output-neutral movement, not zero CPU or a guaranteed 2.7x speedup. The reporter's measured ratio is specific to one dev bare-window experiment.

## 7. Rejected alternatives and limits

- Delete `refresh()` outright: loses automatic resize/scale invalidation.
- Compare only viewport/scale/display: incomplete pointer and visual-window-state semantics; also removes implicit same-size restore drawing.
- Compare pointer position unconditionally: reintroduces per-move redraws even for an uninterested window.
- Disable callbacks or coalesce application input: breaks observers/state ordering and changes event contracts.
- Remove popup notify as redundant: breaks the current owner-position reconciliation path.
- Add another view cache: `refreshing` bypasses it; a cache with incomplete native-state dependencies can freeze content.
- Rewrite vsync to be demand-driven in the same patch: separate scheduler/lifecycle change. Existing continuous frame delivery is load-bearing for callbacks, modal loops and deferred recovery. Its overhead remains after the core fix.
- Replace WM_MOVE/WM_SIZE with WM_WINDOWPOSCHANGED handling here: broader platform-message migration than needed for this root cause.

No Windows hardware/session was available to this investigation. Hover movement, multi-monitor/DPI, titlebar mode changes, popup relocation, IME and real surface restore/recovery are **not** qualified by the headless probe. macOS/X11 callbacks share this invalidation seam, but no cross-platform native speedup is claimed. LSP references could not run because rust-analyzer exited with code 0; targeted source searches were used instead.

At the end of the initial investigation, production source was restored and no public docs were changed. That historical state is superseded by the implementation, synchronized guides, and verification recorded at the top of this report.
