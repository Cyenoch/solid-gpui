# Malformed event frames must terminate the transport

Status: ready-for-agent
Type: task

MessagePack fuzzing currently proves the TypeScript decoder does not throw, but
that contained `null` result is silently discarded by both receive paths. A
later frame can therefore be accepted by a transport whose protocol state is
already unprovable.

Evidence: `packages/react-gpui/src/protocol.ts:378-428` (framing decoder only
throws on an oversized length), `packages/react-gpui/src/protocol.ts:430-436`
(`decodeWire` catches and returns `null`),
`packages/react-gpui/src/protocol.ts:761-797` (`decodeEvent` returns `null`),
`packages/react-gpui/src/renderer/root-container.ts:767-777` (decoder errors
and null events are dropped), and
`packages/react-gpui/src/surface-host.ts:158-170` (same behavior for shared
roots). The fuzz contract is decoder no-panic containment, not transport
survival (`packages/react-gpui/tests/protocol-fuzz.test.ts:101-127`).

## Design

Treat a framing failure or invalid event payload as a protocol termination.
Receive paths construct `TransportTerminatedError` with
`{ kind: "protocol", detail }`, mark the root or shared `SurfaceHost`
transport endpoint terminated, and reject all pending commands. Preserve the
existing direct `FrameDecoder` no-panic/oversize behavior; only transport
integration changes. Do not retry, skip, or resynchronize later frames.

## Acceptance

- A process-backed/stdio transport input sequence containing a valid frame,
  malformed framed payload, then another valid frame terminates exactly once.
- Pending commands reject with the same protocol termination error, later root
  commands and the root/surface-host submit paths reject, and later bytes are
  ignored.
- Shared SurfaceHost roots receive the same terminal error.
