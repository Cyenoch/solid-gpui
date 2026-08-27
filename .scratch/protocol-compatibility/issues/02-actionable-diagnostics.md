# Reject mismatches with actionable diagnostics

Type: task
Status: resolved

## Change

Every Rust wire decoder now returns
`ProtocolError::UnsupportedProtocol { received, expected }` before body
validation when the decoded header version differs. Its display names both
versions and tells the developer to update the host binary or pin
`@react-gpui/core` to the host's v3 release (`crates/react-gpui/src/protocol.rs:1205-1208`,
`protocol/wire/snapshot_patch.rs:29-34,73-78`, `protocol/wire/command.rs:179-184`,
`protocol/wire/event.rs:20-25`).

TypeScript raises `ProtocolVersionMismatchError` for a valid MessagePack array
whose version is not its own (`packages/react-gpui/src/protocol.ts:11-20,773-785`).
Both `SurfaceHost` and `RootContainer` catch this exception and pass its message
through the existing typed fail-fast termination path, preserving
`{ kind: "protocol", detail }` and rejecting pending commands rather than
silently returning (`packages/react-gpui/src/surface-host.ts:167-191`,
`packages/react-gpui/src/renderer/root-container.ts:795-812`).

## Tests

- `packages/react-gpui/tests/protocol-golden.test.ts` rejects v2 and v4 Event
  payloads and asserts both versions in the diagnostic.
- `packages/react-gpui/tests/surface-host.test.ts` and
  `packages/react-gpui/tests/renderer.test.tsx` assert a mismatched frame causes
  typed protocol termination and retains both versions in `cause.detail`.
- `crates/react-gpui/src/tests/protocol.rs` applies v2/v4 mutations to valid
  Snapshot, Patch, Command, and Event payloads and checks the typed error and
  diagnostic.

## Acceptance

A mismatch remains fail-fast, but the error points directly to the incompatible
artifact and the compatible pin. Malformed non-version frames retain the prior
`received malformed event frame` behavior.
