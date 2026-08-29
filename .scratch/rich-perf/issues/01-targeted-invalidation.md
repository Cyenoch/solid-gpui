# 01 — Targeted rich-text cache invalidation

Status: needs-triage
Type: task

`ReactRoot` now caches assembled rich parts by `Text` node ID, but snapshots and
successful patches clear the entire cache. That conservative policy is correct
for descendant create/update/move/delete operations, including patches whose
parents are created in the same batch, but it rebuilds unrelated rich paragraphs
when only one paragraph changed.

Evidence: `.scratch/rich-perf/measurement.md:44-52` (tracked measurement and
current invalidation boundary).

## Acceptance

- A patch invalidates the changed rich paragraph and any rich-text ancestors
  that can be affected by the patch, without rebuilding unrelated cached
  paragraphs.
- Snapshot replacement still clears all entries, and create/update/move/delete
  cases (including same-patch parent creation) cannot leave stale assembled
  runs behind.
- The existing assembly-count contract remains true: unchanged redraws perform
  zero assemblies, while a changed paragraph performs one rebuild.
- Focused coverage exercises multiple rich paragraphs and verifies both
  correctness (new text/styles are rendered) and selective assembly counts.

## Comments

The current full-clear behavior is an intentional correctness-first baseline;
this issue records the measured over-invalidation tradeoff rather than claiming
that targeted dependency tracking is already designed.
