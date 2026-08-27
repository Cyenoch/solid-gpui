# Testing DX documentation and API lock

Status: resolved
Type: task
Blocked by: 01

Document the TestApp recipe and the TS-versus-display-backed testing split in
the dev README, keep the getting-started pointer additive, add the Unreleased
Added changelog entry, and regenerate the dev API fixture after the public
exports change.

## Acceptance

- README snippets are copied from real tests.
- No claims imply TypeScript can assert host-owned geometry or painted output.
- `fixtures/api-surface.dev.txt` matches built exports and generation is
  idempotent.

## Comments

## Answer

The dev README now includes the Bun `TestApp` behavior recipe and explicitly
states the TypeScript versus display-backed Rust testing split. The getting
started section links to that recipe without duplicating it. The Unreleased
changelog includes the facade. `make api-surface-generate` completed twice
after the protocol cutover with the same `dev: 20 exports` result; the fixture
adds `renderTestApp` and `TestApp` (+2 over the prior 18).
