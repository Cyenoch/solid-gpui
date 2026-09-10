# Native scroll performance

## Layout contract

GPUI's native style mapper has a finite supported surface; it is not CSS. A
`flexGrow` value grows a **child**, but does not make its parent a flex container.
Use `flexDirection: "column"` on a vertical parent, including router wrappers.
A bounded scrolling work area needs `minWidth: 0`, `minHeight: 0`, and an
appropriate flex parent all the way to the window.

For a pane that consumes **remaining space**, give it a zero initial size on
its parent's main axis, then grow it:

```tsx
<View style={{ height: 0, flexGrow: 1, flexDirection: "row", minHeight: 0 }}>
  <View style={{ width: 190, flexShrink: 0, minHeight: 0, overflow: "scroll" }}>{navigation}</View>
  <View style={{ width: 0, flexGrow: 1, minWidth: 0, minHeight: 0, overflow: "scroll" }}>{content}</View>
</View>
```

`height: 0` is an initial flex size, not the final height. This work area grows
into the space left after header/footer layout. An automatic initial size
requires measuring its full content before shrinking it back into the window.
Do not apply this rule to content-sized cards, labels, or intrinsic columns.
Their natural size is part of their contract.

## Reproduction and regression

The original routed Gallery regression was removed with the old application.
Its measurements below are historical evidence, not current website acceptance.
Run the retained native scroll regressions and the website navigation check:

```sh
cargo test -p solid-gpui --lib renderer::scroll_tests -- --nocapture
bun run task website-navigation-check
```

For current website measurements, launch `bun run task website-native-profile`
and follow [performance analysis](performance-analysis.md). Choose timing budgets
for the measurement machine; deterministic native tests do not measure GPU
presentation.

Always also exercise the native application with continuous navigation/content
scrolling, clicks after scrolling, resize, route changes, and both themes.
A benchmark result alone is not acceptance of the visible application.

## 2026-09-05 diagnosis

With 466 actual Gallery nodes, the old two-draw cycle had p95 around 26 ms.
After separating the second draw, navigation p95 was 12.04 ms with the
`slotmap` optimization already enabled. Giving the work area a zero vertical
basis reduced p95 to 6.21/6.39 ms (navigation/content); zero initial sizes for
the shell and content width brought it to 5.66/5.87 ms on the same machine.
All independent-scroll assertions passed for that initial viewport. The user
initially confirmed smooth native scrolling, then reported lag on wide/resized
Overview and Drag & Drop even without resize. **The initial acceptance did not
cover those cases.** Earlier smooth window resizing is separate evidence.

The expanded regression includes fresh 560/800/1280/1680-wide windows in both
themes, the real Drag & Drop route, and the same window at 800 → 1280 → 800 →
1280 followed by navigation. Each stage applies the actual Solid patches from
the same root. It checks that the already-scrolled sidebar stays in place.
This sequence catches retained-state bugs that fresh snapshots cannot.

Horizontal remaining-space columns also needed zero initial widths:
`ResponsiveRow` applies this only to positive-growing, non-stacked columns;
`grow: 0` columns keep their intrinsic width. Card and section headings use the
same rule when beside actions. Icon/label buttons now own their content and
colors directly, removing unnecessary nested rows.

The route baseline showed wide Overview content p95 13.34 ms and Drag & Drop
16.78 ms. After those changes one run measured 7.19 and 7.75 ms respectively;
small/light Drag & Drop measured about 3 ms. Other runs showed occasional
8–11 ms tails, including an explicit 8.333 ms budget failure. These local CPU
measurements show a substantial improvement, **not stable 120 Hz acceptance**.
Do not hide variability by repeatedly rerunning until a budget passes.
Native continuous trackpad/GPU presentation still requires fresh acceptance;
CUA synthetic scrolling did not visibly move this app, so it is not usable as
positive evidence. Native theme switching and Benchmark single-step updates
were visually verified.

Navigation had a separate lifetime defect: a small sibling-navigation test
mounted the layout 11 times instead of once. `RouterProvider`, `NativeMatch`,
and `Outlet` now observe memoized route identity/render status for mounting;
match data remains reactive through context. Test both mounting lifetime and
actual native scroll position; do not paper over remounts with scroll offsets.

Controlled experiments and constraints:

- Optimizing `slotmap` alone improved the old cycle by only about 6%; retain
  the existing optimized `gpui-pre` and `taffy` development dependencies too.
- Making every `ResponsiveRow` child wrapper a flex column worsened scroll
  p95 to about 21.8 ms. That experiment was removed.
- Sampling pointed primarily at Taffy block/flex layout. A sleeping transport
  thread must not be counted as CPU use merely because it appears in a sample.
- `MeasuredElement` delegates request-layout; it is not an extra Taffy box.
- No wheel events were throttled or dropped. No scene/geometry cache was added.
  GPUI view caches require definite dimensions and include origin and content
  mask in their key; moving cards are not automatically useful cache units.
- Temporary `[DEBUG-scroll-live]` output was removed. Console wheel logs do not
  establish frame latency.
- Giving the provider wrapper fixed viewport pixels or optimizing only the local
  renderer did not establish a useful additional gain; both experiments were
  removed. Kanban cross-axis alignment changes likewise had no clear benefit.

## Theme regressions

Use ordinary theme surfaces for statistics, boards, previews, and event logs;
reserve `codeBg` + `codeText` for code. Fixed-color swatches need a foreground
chosen from their own luminance, independent of app theme. Shared buttons own
icon/label styling so nested `Text` cannot silently fall back to black. Input
placeholder tests check composited contrast in both modes, not an opacity
constant. The former 20% foreground gave only 1.48:1 on the light input.

When this regresses, start with the actual routed test and a fresh native
capture. Pin the binary, viewport, package outputs, profile, and timing scope;
change one variable at a time. Keep production changes separate from test
calibration and reject improvements that break pane geometry or interaction.

## VirtualList sizing and range recovery

The Gallery list must fill its 320-pixel flex-column viewport. The native
VirtualList event boundary owns the public style; its inner GPUI List fills
that boundary. Applying flex growth only to the inner List collapses the
unstyled boundary. A parent-only flex change did not fix the blank viewport.
The routed native regression now checks real initial-row bounds and an inner
wheel movement while the outer viewport stays stationary.

Also exercise empty → populated data and filtering after scrolling near the
end. Validate **committed rows and range**, not only itemCount. Previously an
empty initial range remained empty after population, and filtering could emit
an inverted range such as 90..1. The committed range now restarts from the
initial window when the previous range is empty or beyond the new data.

## Capturing sustained lag after the three-column breakpoint

The latest user reproduction is **continuous lag after resizing Kanban into
three columns**, not a one-time cold wheel. A cold pixel sequence moved every
requested delta, but that does not close this report. Removing the column
wrappers regressed narrow layout performance; removing the card-list growth
or the button toolbar scroll wrapper did not establish a useful gain. These
experiments were reverted.

For actual native input and presentation measurements, run:

```sh
bun run task website-native-profile 2> /tmp/solid-gpui-frame-profile.log
```

The optional `frame-profile` Cargo feature enables GPUI's native profiler.
It reports interval sample counts, draw p50/p95, invalidation-to-present p95,
and input-to-present p95 alongside viewport dimensions and active-window state.
It does not inject inputs, schedule redraws, or log every wheel. Histograms
are differenced between observations; resize-spanning intervals are discarded.
A zero input sample count provides no input-latency evidence. Presentation here
means GPUI's submission boundary, not a measured physical display scanout.

Compare several seconds of actual scrolling in single-column and three-column
states at a stable size. Keep the exact binary/profile and interval; do not
combine this with the TestPlatform CPU timings or claim a native fix from a
passing geometry test. CUA currently sends clicks but its wheel operation does
not visibly scroll this application; real trackpad input is required for this
remaining reproduction.

The user subsequently exercised the diagnostic application with a real trackpad.
At 793×733, one stable active interval measured draw p95 4.432 ms and input to
presentation p95 6.861 ms (68 draws / 67 input samples). At 1223×733, active
intervals measured draw p95 11.805–11.837 ms and input to presentation p95
57.967–100.663 ms. A 1147×733 interval measured 8.172 ms / 34.472 ms. These are
native observations, not TestPlatform results. Exclude idle-spanning intervals
and zero-input intervals; the profiler counts general native inputs, not only
wheel events. CPU sampling during a second user scroll confirmed Taffy flex and
block layout as the main active hotspot. Header-only and provider-flex changes
also failed to establish a useful improvement and were reverted.

## Confirmed Kanban fix and repeatable whole-gallery audit

The user confirmed smooth native scrolling after repeatedly resizing into and
out of the three-column Kanban layout. The load-bearing change is in the shared
Gallery `Button`: its single-line label has an explicit line height, and the
control height is derived from that line height plus padding and borders. The
previous intrinsic height caused repeated measurement through the surrounding
nested flex rows. Removing the action buttons in a diagnostic experiment reduced
the wide-page cost; restoring all actions with explicit control metrics retained
the gain. A comparable CPU loop fell from roughly 7.8–8.4 ms p95 to 4.8–5.0 ms
including repeated resize stages. A subsequent native scrolling interval at
1303×835 recorded draw p95 around 6–7 ms and input-to-present p95 around 7 ms;
its viewport differs from the earlier capture, so this is not a matched-window
percentage comparison. The user's native confirmation is the acceptance signal.

This contract is for **single-line controls**, not content cards, arbitrary
children, or multiline inputs. Do not assign fixed heights to flowing content
to hide layout work. The native regression checks actual Gallery variant labels
fit within their controls. Shared Button usage carries the fix to other pages;
existing fixed-height benchmark cells, swatches and virtual rows were also
inspected.

The Gallery-specific all-route geometry harness described in this investigation
was retired with the old Gallery application. The shared website now has a
focused navigation regression check:

```sh
bun run task website-navigation-check
```

This verifies retained sidebar nodes and incremental updates when routes change.
It does not measure native scrolling or frame presentation. For the current
Showcase, follow [performance analysis](performance-analysis.md), verifying the
Collections list with real input and both narrow and wide windows. Keep CPU
measurement and native acceptance distinct; do not count idle intervals or
concurrent builds as scrolling evidence.

## Overview: flowing sections must not accumulate flex measurement

After Kanban was accepted, the user still reproduced sustained scrolling lag in
Overview's library directory. This is a separate acceptance case: a green
all-route geometry audit does not invalidate native feedback.

Controlled minimization identified the directory as the major contributor.
Removing the playground was only a modest improvement. Giving directory rows
fixed heights, zero flex basis, or flattening the row into its Link did not
establish a useful gain and was reverted. Fixed heights would also risk clipping
wrapped descriptions.

The retained change uses block flow for the page's sections, library categories,
and directory entries, with explicit margins preserving their spacing. Horizontal
icon/text/chevron rows remain flex layouts; description heights remain intrinsic.
Those outer vertical stacks do not distribute available space, so they do not
need flex's repeated intrinsic size passes through the nested descendants.

Adjacent 1280x600 CPU runs measured content p95 8.478 ms for the original and
4.462 ms for the change (48 measured wheel frames after 8 warmups). Retained
800 -> 1280 -> 800 -> 1280 -> 1680 resize measured Overview content p95
3.917-4.128 ms. Artifacts: `/tmp/solid-gpui-overview-control-final.log`,
`/tmp/solid-gpui-overview-block-final.log`,
`/tmp/solid-gpui-overview-block-resize.log`. This is CPU evidence, not a
native presentation guarantee. The user subsequently confirmed smooth scrolling
and resizing in the reloaded diagnostic window, completing Overview acceptance.
The corresponding active intervals at 1264x759 recorded draw p95 4.477-7.741 ms
and input-to-present p95 4.772-11.223 ms. Exclude idle intervals with no input;
these are interval percentiles, not one pooled percentile or a matched-viewport
before/after comparison. Raw evidence: `/tmp/solid-gpui-profile-app.log`.

Design rule: distinguish a flowing document from a flex allocation problem.
Use block flow with margins for independent vertical sections; use flex where
alignment or flexible allocation is actually needed. Do not globally replace
flex stacks, cache geometry without invalidation, or constrain multiline text
merely to meet a timing threshold. Compare adjacent controlled runs because
unrelated machine load changes absolute timings.

Final checks for this change: package typecheck, host Clippy with frame-profile,
format checks, native Button label bounds, and Overview independent scrolling
through retained resize and at 560 px passed. The compact paired CPU run also
improved (content p95 11.435 -> 8.355 ms), but remains near an 8.33 ms budget
under that machine load; do not advertise a universal 120 Hz guarantee.

## Native FPS monitor

The initial host-local passive overlay has been replaced by the reusable live
`gpui-fps::FpsMonitor` entity from the pinned GPUI Kit tree. The current HUD
starts with observed presentation cadence and can switch to estimated redraw
capacity. Its 500 ms readout refresh differs from the historical passive monitor.
See [current definitions and controlled comparisons](performance-analysis.md).

## VirtualList internal scrolling: owner retention and range feedback (2026-09-05)

The user clarified that the lag occurs inside the list. Two workload regressions
were reproduced independently of timing:

- Moving a ten-row committed window from `[0,10)` to `[1,11)` called renderItem
  ten more times. After retaining overlapping rows with Solid mapArray ownership,
  it creates one row, disposes the departing row, and retains the other nine.
  Same-key new values still update; unmount disposes all remaining owners.
- One native wheel produced conflicting ranges `(0,7)` then `(5,10)`: the GPUI
  scroll callback supplied the previous offset, whereas rendering supplied the
  newly visible rows without overscan. There is now one post-layout source:
  actual viewport intersection plus overscan once, producing `(3,12)` in the
  100px/20px-row regression. Native overdraw remeasurement must not grow this
  request; the same test invalidates forty measured rows and checks no extra event.

The ordinary Gallery CPU loop uses real synthetic wheel displacement within
already committed rows; it does not cover Bun round trips or native vsync.
Its initial p50/p95 was 3.207/3.506ms; later candidate runs were 4.485/11.166ms and
5.619/9.815ms. Multiple UnityShaderCompiler processes were subsequently observed
near 90–95% CPU each, so these measurements do not establish a comparable CPU
improvement. No unrelated process was stopped. The deterministic improvements
are fewer row constructions and one stable range notification.

Artifacts: `/tmp/list-identity-red.log`, `/tmp/list-identity-green.log`,
`/tmp/list-range-red.log`, `/tmp/list-core-tests.log`, `/tmp/list-host-tests.log`,
`/tmp/list-scroll-baseline.log`, `/tmp/list-scroll-candidate-repeat.log`.
Native automated wheel still did not visibly move content. The diagnostic app
was rebuilt and loaded for user trackpad verification. The user then confirmed
smooth scrolling after being asked to scroll internally and resize. This is user
experience acceptance, separate from the contaminated CPU comparison above.
Stable rules live in the solid-gpui skill application reference.

## Nested VirtualList wheel boundaries (2026-09-05)

GPUI List does not stop event propagation itself. The solid-gpui outer boundary
compares the logical scroll offset after List handles the wheel, stopping
propagation only when the list actually moves. At the top or bottom, an event
that causes no movement passes to an ancestor scroll container. Update the
comparison value for every event, including multiple wheel events between
paints. Unconditional stopping would trap scrolling at the boundary; removing
all interception would scroll both the list and its ancestor. If an event moves
the list to its boundary, the list consumes that event and passes the next
non-moving event to the ancestor.

Key native regression: `virtual_list_at_top_passes_wheel_to_outer_content`
verifies that outer content moves when the list is at the top and stays still
while the list can move.

The linked gpui-pre 0.3.3 List routes events through
`hitbox.should_handle_scroll(window)` and provides no direct API for locking
a scroll gesture's target. If page scrolling moves the list under the pointer,
subsequent events still scroll the list. At the user's request, this change
preserved native behavior without timer-based gesture inference or GPUI
dependency changes. Passing deterministic tests does not establish real
trackpad acceptance.

## Nested scroll ownership (2026-09-08)

A wheel event belongs to the deepest viewport that can move in its direction.
Clamp the new offset before deciding whether the viewport consumed the event.
Movement stops propagation; an unchanged offset at an edge or in non-overflowing
content lets an ancestor handle it. Reaching an edge consumes that event; only a
subsequent event at the edge hands off. A horizontal strip without vertical
overflow leaves vertical wheels to the containing page.

This policy is implemented in GPUI's shared Div interactivity and variable-height
List, covering ordinary overflow containers, UniformList, the component VirtualList,
and list-backed widgets. InputBase and ScrollableMask also consume actual movement.
Do not add per-example wheel blockers: they hide missing shared ownership and can
trap scrolling at boundaries. Scrollbar dragging owns mouse drag events separately.

The native regression matrix in `components::scroll_views::tests` checks child and
ancestor offsets together, including pixel/line deltas, boundaries, horizontal
lists, and non-overflowing content. A test asserting only the child moved cannot
catch simultaneous page movement.
