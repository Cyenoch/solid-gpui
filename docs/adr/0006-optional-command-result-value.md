# ADR-0006: Append an optional typed value to CommandResult

- **Status:** Accepted
- **Date:** 2026-08-25

## Context

Most commands only need an acknowledgement, but some root and node commands
need a typed result such as a window-size pair, a focus boolean, a number, or
clipboard text. The protocol must add values without making the current
single-form decoder accept historical tuple arities. The v3 cutover and
optional-tail contract are recorded in `.scratch/protocol-cutover/spec.md` and
[ADR-0011](0011-single-form-wire-optional-tails.md).

## Decision

CommandResult keeps its established positional prefix and appends one optional
seventh item:

`[2,requestId,command,nodeId,success,error,value?]`

The optional value is tagged and validated by its command contract. A missing
tail is a valid acknowledgement with no value, not a failure. Senders only
append the tail when a command has a value to return; receivers implement the
current six- or seven-field forms, not a separate historical compatibility
decoder.

## Alternatives rejected

- **Version the entire protocol for the value channel:** rejected because the
  change is an optional tail in the current schema; a protocol-version split
  would duplicate all existing message validation and fixtures for a small
  additive capability.
- **Make `value` mandatory and encode `null`:** rejected because it changes the
  established result shape, makes every acknowledgement pay for a field, and
  cannot distinguish an old peer from a command that intentionally has no
  value without extra rules.
- **Replace the positional result with a map:** rejected because the protocol
  deliberately uses bounded positional tuples for compact, deterministic
  cross-language encoding.
- **Use one untagged dynamic value:** rejected because number/pair/boolean/
  string validation would become ambiguous across TypeScript and Rust.

## Consequences

- Current six-field acknowledgements and seven-field value-bearing results are
  valid under the v3 schema. This ADR's former claim that six-field results
  were retained specifically for older peers is superseded by [ADR-0011](0011-single-form-wire-optional-tails.md).
- Each value-producing command needs a documented tag and a test for both
  success and malformed/mismatched values.
- The receiver's pending-command API can continue resolving `Promise<void>` for
  acknowledgement-only commands while value-aware commands expose typed
  wrappers.
