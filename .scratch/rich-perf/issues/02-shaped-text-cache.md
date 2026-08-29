# 02 — StyledText-level shape cache

Status: needs-triage
Type: research

Rich-run assembly is now cached, but `RichTextElement::prepaint` still invokes
GPUI shaping on every draw. The measured run-array cache removes repeated tree
flattening while leaving the per-frame shaping cost in place. A deeper
`StyledText`-level shape/layout cache could address that residual, but its
correctness boundary was not measured: shaped output depends on text runs,
font/style inputs, available window width, wrapping, and layout-affecting
changes.

Evidence: `.scratch/rich-perf/measurement.md:26-42` (tracked after-cache
measurements and explicit unmeasured boundary).

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

## Comments

This is a research-first performance issue, not permission to cache by text
alone. The current implementation deliberately stops at assembled `TextRun`
reuse because the window-width and layout correctness boundary is deeper and
unmeasured.
