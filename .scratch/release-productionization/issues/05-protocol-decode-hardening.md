# Protocol decode hardening needs continuous fuzz coverage

Status: needs-triage
Type: task

The process boundary now has deterministic malformed-input regression coverage for
MessagePack Commit Batches and native event frames. The fuzz tests are intended to
remain a productionization asset as protocol fields and decoder implementations
evolve.

Evidence: `crates/react-gpui/tests/protocol_fuzz.rs` and
`packages/react-gpui/tests/protocol-fuzz.test.ts`.

## Comments

- Rust uses a fixed-seed xorshift64 PRNG (`0x4d595df4d0f33173`) with four legal
  Snapshot/Patch/Event/Command seed payloads, round-trip assertions, truncation,
  byte flips, wrong length prefixes (including over-16 MiB), tuple arity,
  invalid tags/kinds/modifiers/enums, invalid UTF-8, huge counts, NaN/infinity,
  negative floats, and u32 overflow mutations. It exercises 1,039 deterministic
  mutation cases and catches decoder panics around `read_frame`,
  `Snapshot::decode`, `Patch::decode`, `Command::decode`, and `Event::decode`.
- TypeScript uses a fixed-seed xorshift32 PRNG (`0x4d595df4`) with the same
  malformed-frame categories and explicit fragmented/coalesced FrameDecoder
  cases. It exercises 1,034 deterministic mutation cases, including explicit
  assertions that `decodeEvent` returns `null` for `0xd9 0x01 0xff` and `0x90`.
- The Rust scoped test (`cargo test -p react-gpui --test protocol_fuzz --locked`)
  passes 1/1. The TypeScript scoped test (`bun test
  tests/protocol-fuzz.test.ts`) passes 3/3 with no uncaught exceptions.
- No Rust panic or TypeScript uncaught decoder exception was found. On the TS
  side, malformed MessagePack is contained by `decodeWire`'s `try/catch`, which
  returns `null`; `decodeEvent` then rejects `null` and any decoded array whose
  length is not 10. Rust's `rmp-serde` tuple deserialization returns an error for
  the empty fix-array arity case (`0x90`), and the test asserts that all four
  decoders return `Err` without panicking.

- The Rust and TypeScript fuzz assets described above are now present and
  passing; the issue remains `needs-triage` for deciding their long-term
  maintenance/extension policy rather than claiming the coverage is missing.

Coverage is implemented and runs as part of the regular `make ci` gates; the
remaining open question is maintenance ownership, so this issue stays
`needs-triage`.
