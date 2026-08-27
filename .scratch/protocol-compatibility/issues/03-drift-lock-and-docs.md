# Lock protocol version drift and document upgrade policy

Type: task
Status: resolved

## Change

The supported constants remain deliberately duplicated once per language:
TypeScript `PROTOCOL_VERSION = 3` (`packages/react-gpui/src/protocol.ts:11`) and
Rust `PROTOCOL_VERSION: u32 = 3` (`crates/react-gpui/src/protocol.rs:7`). No
code-generation dependency was added. Focused tests pin each constant to v3,
prove the TypeScript encoder emits it, and exercise decoder rejection for both
adjacent versions. Existing cross-language fixture bytes remain unchanged.

`docs/protocol.md:62-76` now defines the lockstep compatibility contract and
explicitly rejects negotiation/dual-version support. `CHANGELOG.md:46-48`
records the user-visible diagnostics and typed fail-fast behavior.

## Rejected alternatives

- Generated shared constants: rejected as disproportionate to one version
  number; fixture bytes plus literal lock tests catch drift without a build
  pipeline or generated-file ownership.
- Negotiation / dual-version support: rejected because it would hide an
  incompatible positional schema and multiply the supported matrix.
- A host `--version` format change: not taken. The existing host CLI surface is
  outside this bounded protocol change and no candidate-smoke expectations need
  migration.

## Acceptance

The two language constants cannot drift unnoticed by focused tests, the golden
fixtures remain byte-identical, and the upgrade/pinning remedy is documented at
the protocol reference.
