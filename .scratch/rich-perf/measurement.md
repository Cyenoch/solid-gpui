# Rich Text Performance Audit

## Method

Headless GPUI draws used the established `#[gpui::test]` renderer harness. Each
scenario rendered a 50-run or 200-run paragraph for 32 measured draws after one
warm draw. Percentiles use the existing p50/p99 rank convention. A test-only
assembly counter increments only when `rich_text_parts` actually flattens
children and constructs the `TextRun` array; it is compiled out of production.

## Before cache

| Runs | Scenario | p50 (ms) | p99 (ms) | assemblies / 32 draws |
| ---: | --- | ---: | ---: | ---: |
| 50 | no change | 0.153 | 0.177 | 32 |
| 50 | style-only change | 0.148 | 0.155 | 32 |
| 50 | full text change | 0.140 | 0.150 | 32 |
| 200 | no change | 0.433 | 0.469 | 32 |
| 200 | style-only change | 0.438 | 0.472 | 32 |
| 200 | full text change | 0.441 | 0.483 | 32 |

No-change assembly count equaled draw count, and the 200-run p99 was 2.65x the
50-run p99. This confirms O(runs) flattening and run-array assembly on every
render, even when the tree is unchanged.

## After cache

| Runs | Scenario | p50 (ms) | p99 (ms) | assemblies / 32 draws |
| ---: | --- | ---: | ---: | ---: |
| 50 | no change | 0.152 | 0.174 | 0 |
| 50 | style-only change | 0.142 | 0.159 | 1 |
| 50 | full text change | 0.137 | 0.146 | 1 |
| 200 | no change | 0.392 | 0.418 | 0 |
| 200 | style-only change | 0.383 | 0.425 | 1 |
| 200 | full text change | 0.390 | 0.427 | 1 |

The behavioral guard is assembly count: unchanged redraws perform zero
assemblies; a tree patch touching the rich paragraph performs one rebuild.
Wall-clock timing is secondary and remains roughly stable because GPUI shaping
still runs in `RichTextElement::prepaint` on each draw. A StyledText-level shape
cache was intentionally not attempted in this change because its layout and
window-width correctness boundary is deeper and unmeasured.

## Fix

`ReactRoot` stores assembled rich parts by Text node ID. Snapshots clear the
entire cache, while successful patches now invalidate only touched nodes, their
post-patch content ancestors, and any cached entries for deleted nodes. Move
and delete operations also invalidate pre-patch ancestors so detached content
cannot remain stale. The two-pass boundary handles same-patch create chains
after the store has established final ancestry. This preserves cache hits for
unrelated updates while retaining the correctness guard for descendant
create/update/move/delete operations.
