# Development Rules by Design Problem

## Layout and painting

Consider block layout and margins for ordinary document paragraphs or stacked
cards. Use flex for alignment, stretching, or distributing remaining space. This
is a selection criterion, not a global flex replacement rule. Verify GPUI `gap`
semantics against the container's display mode; solid-gpui's style adapter enables
flex automatically for gap/alignItems/justifyContent.

Give single-line buttons an explicit lineHeight and derive height from line height,
padding, and borders; verify icon and text bounds. Keep multiline descriptions,
inputs, and dynamic cards content-sized. Constrain scroll panes' available and
minimum sizes and shrink behavior; place fixed headers/footers outside them.
Relayout must read new constraints rather than feed rounded previous bounds back
as the next size source.

Attribute costs to rendering, layout measurement, text shaping, prepaint, scene
construction, or the GPU before changing visual effects. Extracting functions or
components does not automatically reduce native element counts or layout passes.

## Lists and identity

Use native `uniform_list` only for genuinely equal-height rows. Use `list` with a
retained ListState when heights vary. Create scroll handles/ListState in the entity
that owns the state and reference them during render. Use stable item IDs: after
filtering/reordering, index is not identity. Work should grow with visible items
and bounded overscan rather than the entire dataset.

Verify nonempty first paint, empty-to-populated updates, filtering from the end,
reinsertion, resize, and independent inner/outer scrolling. Inspect actual row
bounds and visible ranges; a correct itemCount with no visible rows still fails.
Evaluate initial layout cost before measuring every row to fix a scrollbar.
Validate equal-height estimates against changes to fonts, width, and content.

## Entities and caches

Create persistent Entity instances, input models, and scroll state during
initialization. Notify local entities for local changes and combine related
updates into one notification. `RenderOnce` describes component construction; it
does not guarantee a single paint.

GPUI `.cached(style)` considers bounds including origin, content mask, text style,
dirty state, and other conditions. Moving cards may miss the cache. A cached layout
shell needs correct size constraints. Inspect the current implementation before
using it and test that dependency changes invalidate it.

Custom caches must cover every relevant change in content, fonts/line height,
theme, image readiness, native input state, animation, available size, scale, and
clipping. Native state can change without a protocol Patch.

## Foreground tasks and events

`cx.spawn` preserves foreground execution. Prepare CPU-intensive work on the
background executor, then apply a bounded result on the UI thread. Even consumers
that await can monopolize the UI when a queue stays ready. Bound work per poll and
check fairness under continuous production while preserving event order, completion
counts, and backpressure.

Bind task handles to owners. Store cancellable Tasks and drop them on replacement
or destruction. Use weak entities across awaits and exit normally when a window
closes. Use GPUI executor timers in GPUI tests; timers from another runtime require
that runtime. Subscriptions also need explicit retention and release.

Register high-frequency input only for consumers that need it; aggregate logs over
intervals. Sampling/coalescing input changes semantics and must be an explicit
choice. Real scrolling tests must observe content displacement as well as events.

## Animations and monitors

Request the next frame only while an animation is running; stop when it completes.
Decorative animation respects reduced motion. Monitors use small fixed-size graphs
and bounded samples, with explicit cadence/duration definitions and statistical
windows. A monitor on the UI thread cannot paint while that thread is blocked.
After recovery, sample actual elapsed time rather than assume an on-time timer.

Prefer passive sampling during paint, retaining the latest valid readings while
idle. Active refresh creates extra window work. Publish activity-segmentation
thresholds and the limitation that long stalls may be classified as idle; active
FPS alone cannot detect every stall. Compare monitor enabled/disabled under the
same workload rather than subtracting a constant estimate of monitor frames.
