# ADR-0011: Single-form wire with optional node tails

- **Status:** Accepted
- **Date:** 2026-08-29

## Context

Protocol v3 accumulated several additions at the end of positional tuples:
selectable Text, tooltip text, pointer-move capability, accessibility state,
and typed command results. Some earlier rounds also left historical arities in
one or both decoders. That combination made an absent current capability easy
to confuse with an old renderer or host. The protocol cutover recorded in
`.scratch/protocol-cutover/spec.md` removed the historical decode paths while
retaining optional fields that current encoders intentionally omit.

The current positional schemas are implemented by
`packages/react-gpui/src/protocol.ts`,
`crates/react-gpui/src/protocol/wire/node.rs`,
`crates/react-gpui/src/protocol/wire/snapshot_patch.rs`,
`crates/react-gpui/src/protocol/wire/event.rs`, and
`crates/react-gpui/src/protocol/wire/command.rs`; the implemented contract is
indexed in `docs/protocol.md` §§2–4 and locked by `fixtures/protocol/`.

## Decision

- Each protocol message family has one current positional schema. Optional
  data may be absent only where that schema explicitly defines an optional
  suffix; absence is current data, not a legacy-peer compatibility mode.
- Node capability fields are appended as **Wire Tail Slots**. When a later tail
  needs an earlier optional position, the encoder emits the required
  placeholder (for example, selectable `false` before a tooltip or pointer
  move tail) so the current decoder remains unambiguous. An absent tail is
  omitted; it is never filled from a historical default.
- Optional accessibility fields and the typed `CommandResult` value remain
  optional tails under their own current validators. Optional command payloads
  such as open-surface options remain optional because current producers emit
  those forms intentionally; they are not collapsed into a mandatory null map.
- Data that is semantically required by the current contract is not made
  optional for compatibility. Pointer down/up coordinates are required, and
  Submit carries a string including the empty string. Historical tuple arities,
  null-to-empty translations, and default-filling compatibility branches are
  rejected.
- Every tail change updates both language implementations and the cross-
  language golden vectors before the protocol is considered complete.

## Alternatives rejected

- **Keep permissive legacy decoders:** rejected because accepting old arities
  hides mismatched renderer/host builds and leaves two protocol designs alive.
- **Version the whole protocol for every optional field:** rejected because a
  validated suffix expresses the current absence/presence contract without
  duplicating every message family and fixture.
- **Make every tuple position mandatory and encode null placeholders:** rejected
  because it adds weight to acknowledgements and would erase the distinction
  between an absent capability and a meaningful current value.
- **Use maps or untagged dynamic values instead of positional tails:** rejected
  because bounded positional tuples and tagged values provide deterministic,
  cross-language validation at lower structural cost.
- **Treat required additions as optional tails with defaults:** rejected because
  defaulting a real host value, such as a pointer coordinate, silently changes
  behavior and preserves a design the cutover intentionally removed.

## Consequences

- Protocol v3 remains a lockstep contract: a current optional tail is not a
  promise that a different protocol version can decode the frame.
- Encoders must understand placeholder rules, and decoders must validate both
  the current arity grammar and the field's semantic owner. Golden fixtures
  catch producer-byte and cross-language drift.
- Adding a new optional capability is cheap when it is a true suffix, but moving
  or reinterpreting an existing position is a wire decision. A future change
  that needs historical compatibility requires a new protocol design and ADR,
  not a permissive branch in the current decoder.
