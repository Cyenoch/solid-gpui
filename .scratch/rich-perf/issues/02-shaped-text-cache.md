# 02 — StyledText-level shape cache

Status: resolved-as-wontfix
Type: research

Rich-run assembly is now cached, but `RichTextElement::prepaint` still invokes
GPUI shaping on every draw. The measured run-array cache removes repeated tree
flattening while leaving the per-frame shaping cost in place. A deeper
`StyledText`-level shape/layout cache could address that residual, but its
correctness boundary was not measured: shaped output depends on text runs,
font/style inputs, available window width, wrapping, and layout-affecting
changes.

Evidence: `.scratch/rich-perf/measurement.md:44-61` (split benchmark,
measured assembly/shaping costs, and verdict).

## Acceptance

- Establish a focused benchmark and correctness matrix for repeated draws,
  text/style changes, font changes, and available-width/wrapping changes before
  choosing a cache key or invalidation policy.
- If implemented, reuse shaped/layout data only when every layout-relevant input
  matches, and invalidate on text, run typography, font availability, width,
  line-height, and other changes that affect wrapping or geometry.
- Verify that wrapped-line geometry, hit testing, selection/copy ranges, link
  activation, and per-line focus affordances remain correct after cache hits and
  invalidation.
- Report assembly and shaping measurements separately so an improvement in one
  layer is not presented as an improvement in the other.

## Answer

The shaped-layout cache is not warranted by the measured boundary. Two repeat
runs of the focused 200-run paragraph benchmark (32 draws after one warm draw)
measured the `shape_text` call at 0.066 ms and 0.036 ms per draw, versus total
unchanged draw times of 0.520 ms and 0.276 ms. The shape call therefore consumed
12.7% and 13.0% of the frame respectively. Forced assembly runs measured
0.055 ms and 0.035 ms for shaping, with assembly at 0.067 ms and 0.046 ms; the
shape share of those total draws was 11.5% and 11.1%. All runs made 32 shape
calls; forced assembly made 32 assemblies. The measured saving is below the
30% threshold, so no application-level shaped-layout cache is being added.

The pinned GPUI source shows why this is a shallow boundary. `WindowTextSystem::shape_text`
rebuilds filtered `TextRun`s, font runs, and decoration runs on every call before
delegating each line to `LineLayoutCache::layout_wrapped_line`
(`~/.cargo/git/checkouts/zed-a70e2ad075855582/6805d95/crates/gpui/src/text_system.rs:506-634`).
The pinned line cache is keyed by text, font size, font runs, wrap width, and
force width, and reuses current/previous-frame entries
(`~/.cargo/git/checkouts/zed-a70e2ad075855582/6805d95/crates/gpui/src/text_system/line_layout.rs:454-476,574-636`).
Platform shaping (`PlatformTextSystem::layout_line`) occurs only on a line-cache
miss (`.../line_layout.rs:639-689`), so the benchmark's residual `shape_text`
cost is primarily GPUI request/assembly overhead after platform layout caching,
not repeated platform shaping.

`RichTextElement` bypasses GPUI's per-element `StyledText::TextLayout` state and
calls `shape_text` from `prepaint`
(`crates/react-gpui/src/renderer/paint/text_input.rs:155-186`). A cache placed
in `ReactRoot` would need at least node-parts version, available width, scale
factor, font/style/run identity, line height, wrapping/clamp, and font
registration invalidation. Width changes alter wrap boundaries; scale changes
alter device-pixel geometry; font registration can change family resolution;
and text/style/run/line-height changes alter glyphs or geometry. These are
correctness requirements for wrapping, hit testing, selection/copy ranges, link
activation, and focus affordances, not safe text-only cache inputs. The existing
GPUI line-layout cache already handles the platform-shaped layout boundary, so
the additional cache complexity is not justified by the measured residual.

This research issue is resolved as wontfix: retain the existing rich-parts
cache and per-frame GPUI line-layout cache; do not add a ReactRoot shaped-layout
cache without a materially different measurement.

## Comments

This is a research-first performance issue, not permission to cache by text
alone. The current implementation deliberately stops at assembled `TextRun`
reuse because the window-width and layout correctness boundary is deeper and
unmeasured.
