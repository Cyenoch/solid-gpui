# ADR-0003: Use a three-layer cross-language golden vector contract

- **Status:** Accepted
- **Date:** 2026-08-25

## Context

The TypeScript renderer and Rust host must evolve the same MessagePack protocol
without requiring their independent encoders to emit identical bytes for every
numeric value. The workspace has protocol fixtures that exercise both
renderer-to-host and host-to-renderer directions. Integral values that model
Rust `f32` fields may legitimately be encoded as either an integer or a
float32, while protocol framing, tags, and positional structure remain part of
the wire contract.

## Decision

Every golden-vector row is checked at three layers:

1. **Producer byte stability:** each producer's encode output remains stable
   against its checked-in expected bytes.
2. **Cross-language semantic equivalence:** TypeScript and Rust decode the
   other producer's row to the same protocol meaning.
3. **Numeric dual-form acceptance:** integral `f32` fields accept both integer
   and float32 MessagePack forms where the protocol permits them; the semantic
   value, validation limits, and re-encoded bytes for that producer remain
   deterministic.

We do not require TypeScript and Rust to produce byte-identical encodings for
all semantically equal numeric values.

## Alternatives rejected

- **Require cross-library byte identity:** rejected because `@msgpack/msgpack`
  has no option that forces integer values to become float32 encodings. A
  value such as `x=10` can therefore differ in bytes while remaining equal in
  protocol meaning; making this a fixture failure would encode an accidental
  library detail as a protocol rule.
- **Assert semantic equality only:** rejected because producer byte stability
  catches accidental encoder changes, positional regressions, and fixture
  churn that a decode-only test can hide.
- **Normalize every number in the protocol layer:** rejected because it would
  add conversion work and complexity without improving the declared semantic
  contract; only numeric fields that permit both forms need dual-form
  validation.

## Consequences

- New protocol rows must identify the expected bytes for each producer and the
  semantic result expected from both decoders.
- A MessagePack dependency upgrade can require fixture updates when bytes
  change, but cannot silently change the semantic contract.
- Tests are more precise than a cross-library byte comparison while retaining
  byte-level regression coverage.
