# Multi-surface example coverage

Status: resolved

## Evidence

- `packages/react-gpui/examples/multi-surface.tsx:54-94` demonstrates `createSurfaceHost`, opening one child surface, registering its returned id, rendering it, and logging `onClose`.
- The example does not provide a fake automatic reopen or a live gallery pairing; it stays safe for the documented host process.

## Decision

The example is sufficient for the supported open/close flow. The lifecycle edge cases are covered in focused tests and protocol documentation; adding a control that attempts same-id reopen would demonstrate a deliberate failure rather than a useful application pattern and would add weight without improving the public API.
