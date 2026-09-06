# gpui-component 0.6.0 transitive maintenance advisories

Status: needs-triage

## Evidence (2026-09-05)

`bun run audit` fails on three unmaintained advisories with the current lockfile:

- RUSTSEC-2025-0141: bincode 1.3.3 through syntect 5.3.0 → gpui-base → gpui-component.
- RUSTSEC-2024-0384: instant 0.1.13 through gpui-base/gpui-component and notify-types 1.0.1 → notify 7.0.0.
- RUSTSEC-2025-0134: rustls-pemfile 2.2.0 through gpui-pre-reqwest 0.12.15 → gpui-kit-assets → gpui-component.

The audit reports no direct safe upgrade for these dependency constraints.
Bun audit passed for 108 packages. No new advisory ignores were added.

## Resolution criteria

Update or refactor the owning upstream dependencies, verify native provider
behavior and the unified GPUI runtime family, and pass `bun run audit`.
Do not mark the whole CI/release green while these gates remain open.
