# Epoch validation and generation semantics

Status: resolved

## Evidence

- `packages/react-gpui/src/renderer/root-container.ts:819-831` requires event `surfaceId` and `epoch` to match the owning root before sequence/revision acceptance.
- `crates/react-gpui/src/tree/validation.rs:48-65` rejects patch identity mismatches, including epoch.
- `crates/react-gpui/src/renderer/commands.rs:146-158` validates command identity before native effects.
- `crates/react-gpui-host/src/main.rs:186-195` allocates surface IDs monotonically.

## Decision

Epoch validation is substantive and has no wire gap. Epoch is a renderer/native generation discriminator; a changed epoch may bootstrap a new tree, but it does not authorize reuse of a retired surface ID. The TypeScript host now rejects same-id recreation and documents this distinction.
