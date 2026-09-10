# Native incremental commits

Baseline: `c6aec827c3e0fb44afcaa8ae28c9a338dd93a18b`, clean worktree.

## Objective

Make local Solid updates local in native commit processing, with measured
rendering decisions. Preserve transactional publication, native instance identity,
event generations, text editing, list state, and layout correctness.

## Work

- [x] Instrument the actual decode, commit, reconciliation, and render entrypoints;
  replace the duplicated stage benchmark and retain baseline artifacts.
- [x] Apply patches through an undo transaction that includes extension validation;
  publish revision only after success and remove full-store cloning.
- [x] Produce one dependency-aware change set and use it for validation, route
  updates, native reconciliation, and cache invalidation.
- [x] Compare local-update rendering and real application scenarios; retain a
  render-region/cache change only if its measured benefit and invalidation
  contract justify it.
- [x] Run focused correctness tests, relevant broader checks, documentation and
  website checks; record evidence and qualification limits.

## Work bounds

Property-only commits visit changed nodes and their dependency closure. Structural
commits may additionally visit removed descendants, changed child lists, shifted
siblings, and old/new ancestors. Adapter validation can inspect descendant
composition, so ancestor contracts must be revalidated. Native state changes and
layout constraints remain independent sources of rendering invalidation.

Tree mutation and validation are one rollback scope. Native side effects occur
only after acceptance. No legacy transaction path or new wire protocol is needed.

## Evidence

Raw local artifacts live in `artifacts/`. Deterministic GPUI tests attribute CPU
work and verify geometry; they do not measure GPU presentation or display scanout.
Production measurements must exclude test-support, run serially after compilation,
and identify binary, features, runtime, geometry, and monitor state.

## Outcome

See [the delivery report](report.md) for the final isolated A/B/B/A measurements,
checks, and native verification limits. The cached-pane experiment was rejected
because it retained stale rendered text; no unsafe region cache was shipped.
