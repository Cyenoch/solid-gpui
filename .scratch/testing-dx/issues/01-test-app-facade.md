# TestApp facade

Status: resolved
Type: task

Implement the consumer-facing `renderTestApp` seam in
`packages/react-gpui-dev/src/testing.ts`, keeping all interactions on the real
`MemoryTransport`/Root dispatch path. Add the focused end-to-end and locator
coverage in `testing.test.tsx`; do not add per-method trivia tests.

## Acceptance

- Labels and exact text locate retained nodes without requiring a host kind.
- Missing locators report the query in a stable, useful error.
- Facade interactions return the latest decoded commit and cover the listed
  renderer-side event/command payloads without a fake event system.
- Existing low-level `render` behavior remains available.

## Comments

## Answer

Implemented `renderTestApp` in `packages/react-gpui-dev/src/testing.ts`.
Accessibility-label/predicate and rendered-text locators operate on the latest
retained tree and report the query on misses. Press, hover, key, input, submit,
scroll, pointer/multi-click, drag, outside-pointer, focus/blur, visible-range,
command-result, and raw-frame operations all use the existing Root receive path
through `MemoryTransport`; interactions return the latest decoded commit.
Focused tests cover the filtered-list flow and locator/error contract.
