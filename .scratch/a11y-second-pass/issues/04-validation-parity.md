# Accessibility validation parity

Status: resolved
Type: task

Keep TypeScript and Rust validation and encode/decode semantics aligned as the accessibility tuple grows.

## Answer

The implemented tail is `[expanded?,level?]` after the unchanged seven-field core. TypeScript validates boolean expanded values and positive-u32 heading-only levels before encoding; Rust decodes omitted tails to `None` and validates heading-only level plus positive range in retained-tree validation. Golden generation covers the extended tuple in both directions. Legacy seven-field tuples remain valid.

Evidence: `packages/react-gpui/src/renderer/props.ts:36-45,142-168,319-324`; `crates/react-gpui/src/protocol/wire/node.rs:50-91,653-683`; `crates/react-gpui/src/tree/validation.rs:137-171`; `scripts/protocol-golden.ts` accessibility fixture.

## Comments

Verification stays at the protocol/encode/decode/apply seams. Headless TestPlatform cannot inspect AccessKit, so no fake AX-tree assertion is added.
