# Production hardening and runtime support

## Scope

- Make maintained code, comments, documentation, and agent guidance English-first.
  Preserve intentional Unicode samples and tests; Chinese documentation may live
  in explicitly named translated copies.
- Remove redundant retained state and obsolete compatibility glue. Comment
  transactional, scheduling, and protocol invariants where they are non-obvious.
- Correct renderer, native tree, routing, transport, and release failures using
  focused behavioral tests. Prefer simpler algorithms and bounded work.
- Support Rust-led native applications with SolidJS/GPUI UI under Bun or an
  embedded QuickJS runtime. Also support applications led by Bun business logic.
  Rust owns GPUI rendering in every mode; native contracts stay shared.
- Use Oxc instead of Babel if actual Solid universal compilation, TypeScript
  behavior, source maps, and application execution satisfy the existing contract.

## Acceptance evidence

The package build, types, key tests, tarball consumer, native protocol goldens,
Rust workspace tests and Clippy must pass. QuickJS must execute actual compiled
Solid JSX and preserve protocol ordering, resource bounds, cancellation, errors,
and the routing capabilities exposed by the project. Dependency inventory and
release preparation must remain reproducible.

Report measured improvements with their measurement scope. Do not equate passing
tests with universal algorithmic optimality or cross-platform release qualification.
