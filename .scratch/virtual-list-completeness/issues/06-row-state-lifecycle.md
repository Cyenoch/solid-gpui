# Row unmount/remount state

Status: resolved
Type: audit

Question: Is a row's React subtree preserved when it leaves and re-enters the
committed range?

Evidence: JavaScript creates only children in the current range
(`packages/react-gpui/src/index.ts:165-177`), keyed by `itemKey`; native rows
look up only the committed offset and otherwise return an estimated placeholder
(`crates/react-gpui/src/renderer/paint/virtual_list.rs:70-89`). The visible-range
callback changes that committed child range (`packages/react-gpui/src/index.ts:145-149`).
React therefore preserves overlapping keyed fibers, deletes evicted children,
and mounts fresh fibers when those keys return.

Decision: This is expected virtualization behavior, not a bug. Add a renderer
test with row-local state that is changed, evicted, and returned. Document that
state needed across eviction belongs outside the row (or in application data),
while overlapping rows retain state by key.

## Comments
- Added a keyed row-local-state regression: an overlapping row remains keyed,
  while an evicted row unmounts and remounts with initial state.
- Verified by `bun test tests/renderer.test.tsx` (65 passed).
