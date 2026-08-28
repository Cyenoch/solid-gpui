# Text clipboard byte limit

Status: resolved
Verdict: implemented

## Question

Is the 1 MiB clipboard limit symmetric and consistent with the dual-length
semantics, including multibyte strings?

## Evidence

- TypeScript defines UTF-8 byte counting and the 1 MiB constant at
  `packages/react-gpui/src/protocol.ts:6-9,29-30`.
- `Root.setClipboardText` rejects oversized writes before framing, and
  `getClipboardText` validates returned text with the same helper at
  `packages/react-gpui/src/renderer/root-container.ts:692-710`.
- Rust command decode and host dispatch use `String::len()` against the same
  byte cap at `crates/react-gpui/src/protocol/wire/command.rs:326-333` and
  `crates/react-gpui/src/renderer/commands.rs:272-315`.
- Rust command-result text decoding applies the same cap at
  `crates/react-gpui/src/protocol/wire/event.rs:667-681`.
- TextInput selection/max-length remains UTF-16 by ADR-0004; clipboard resource
  limits are UTF-8 bytes.

## Decision and action

The contract is symmetric: exactly 1 MiB is accepted, one additional UTF-8 byte
is rejected, and values are never truncated. The independent
`packages/react-gpui/tests/interchange.test.tsx` test locks both sides of the
boundary with three-byte CJK and four-byte emoji strings, including write
framing, local rejection, and read-result validation.
