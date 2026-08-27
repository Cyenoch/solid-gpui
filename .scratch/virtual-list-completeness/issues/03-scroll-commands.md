# VirtualList scroll commands

Status: resolved
Type: task

Question: Do `scrollToIndex` and `scrollToEnd` work with estimates and at
boundaries?

Evidence: Public handles submit the two commands
(`packages/react-gpui/src/renderer/nodes.ts:150-156`). TypeScript rejects
non-integers, values outside u32, and `index >= itemCount`
(`packages/react-gpui/src/renderer/root-container.ts:258-265`); native command
processing applies `ListState::scroll_to` for an existing index and
`scroll_to_end` for the tail (`crates/react-gpui/src/renderer/commands.rs:504-538`).
GPUI clamps `scroll_to` and explicitly uses the item count as the end anchor
(`references/zed/crates/gpui/src/elements/list.rs:659-674,597-611`).

Decision: Preserve the existing bounds contract: `index === data.length` is a
rejected Promise and no command frame is sent. Add real command round-trip
coverage against an unmeasured display-backed list and document the accepted
range plus estimate behavior.

## Comments
- Kept the existing strict index validation and added display-backed command
  coverage for estimated rows, `scrollToEnd`, and the rejected end index.
- Verified by `cargo test -p react-gpui virtual_list --locked` (7 passed) and
  `bun test tests/renderer.test.tsx` (65 passed).
