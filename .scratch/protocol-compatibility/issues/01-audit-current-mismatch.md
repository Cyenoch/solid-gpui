# Audit current protocol mismatch behavior

Type: research
Status: resolved

## Question

What headers carry the protocol version, and what does each TypeScript/Rust
encoder and decoder do when a valid frame uses an adjacent version?

## Answer

All four message arrays carry `version` at index 0 and a discriminator at index
1: Snapshot `1`, Event `2`, Patch `3`, Command `4`. The length prefix is only a
payload byte count. TypeScript emits all four tuple forms through
`encodePayload`/`encodeFrame`, but its production receive path decodes Events;
Rust exposes encoders/decoders for all four forms. See
`packages/react-gpui/src/protocol.ts:11-18,140-148,179-187,197-240,285-319`
and `crates/react-gpui/src/protocol.rs:7-11,87-125,128-190,233-282,566-1192`.

Baseline real-decode evidence:

- TypeScript `decodeEvent(encodePayload([4,2,1,1,1,1,0,0,1,null]))` returned
  `null` (the old condition at `protocol.ts:763-764` treated version mismatch as
  malformed).
- Rust Snapshot, Patch, and Command decoders returned
  `unsupported protocol version 4` after a valid encoded v3 payload's version
  byte was changed to 4 (`protocol/wire/snapshot_patch.rs:29-34,73-78`,
  `protocol/wire/command.rs:179-184`).
- Rust Event decoder returned `Ok(Event { protocol: 4, ... })`; it had no version
  check in the old `protocol/wire/event.rs:20-45` path.

The low-value Rust error and TypeScript `null` made upgrades hard to diagnose,
and Rust Event decoding allowed a mismatched frame through.

## Evidence command output

```text
TypeScript: decoded: null
Rust:
snapshot: err: unsupported protocol version 4
patch: err: unsupported protocol version 4
command: err: unsupported protocol version 4
event: ok
```

## Acceptance

The audit is preserved in `.scratch/protocol-compatibility/spec.md`, including
exact frame shapes, decoder locations, and executable baseline results.
