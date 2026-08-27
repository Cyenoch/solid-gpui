# Malformed event frames must terminate the transport

Status: resolved
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

- A transport input sequence containing a valid frame, malformed framed
  payload, then another valid frame terminates exactly once.
- Pending commands reject with the same protocol termination error, later root
  commands and the root/surface-host submit paths reject, and later bytes are
  ignored.
- Shared SurfaceHost roots receive the same terminal error.

## Comments

- `RootContainer.receive` now validates every decoded event before dispatch and
  turns framing/validation failures into a protocol-cause termination; it
  detaches listeners, rejects pending commands, and ignores subsequent bytes.
  `SurfaceHostImpl.receive` applies the same fail-fast rule to its shared
  endpoint and all routed roots.
- `packages/react-gpui/tests/renderer.test.tsx:1910-1953` drives a coalesced
  valid/malformed/valid sequence with all 16 root command families pending;
  every Promise rejects with the same `{ kind: "protocol" }` error and later
  commands/bytes are rejected or ignored. `surface-host.test.ts:193-218`
  proves shared-root fan-out.
- Decoder fuzz/golden suites still pass because direct `FrameDecoder` and
  `decodeEvent` retain their no-throw/null containment contract; only the
  transport receive integration is fail-fast.
