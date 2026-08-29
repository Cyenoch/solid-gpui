# 01 — Targeted rich-text cache invalidation

Status: resolved
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

## Implementation

Implemented in `ReactRoot::apply_payload` with a two-pass patch boundary. Before
the store mutation, the renderer records the decoded operation operands and
their pre-patch ancestor IDs. After `NodeStore::apply_patch` succeeds, it walks
each touched ID against the post-patch tree and removes cached entries for the
touched nodes, all final ancestors, and deleted nodes. Pre-patch ancestors are
also invalidated for moves/deletes so detaching content cannot leave an old
rich parent cached. Snapshot replacement still clears the complete cache.

The ancestor-set approach was chosen over lazy content fingerprints: it does a
bounded tree walk once per committed patch and keeps cache reads on the render
hot path to a hash lookup, rather than recomputing paragraph content every
frame. Focused coverage proves unrelated input updates preserve both paragraph
caches, nested run updates rebuild only the changed paragraph, a same-patch
`View` → `Text` → `RawText` create chain assembles once, and deleting that Text
drops its cache entry.

## Answer

Targeted rich-text cache invalidation is implemented and verified. `cargo test
-p react-gpui --lib` and the repository gates are recorded with the landed
change.
## Comments

The current full-clear behavior is an intentional correctness-first baseline;
this issue records the measured over-invalidation tradeoff rather than claiming
that targeted dependency tracking is already designed.
