# Ticket 02: standalone application delivery

Integrated branch: `integrate/comparison-adoption`; owner merge `a03db0fc`.

## Delivered behavior

The Vite CLI creates exact-version stock and custom Rust applications with an
explicit Bun or QuickJS runtime. A paired candidate manifest pins the source
revision, schema lock, host, generated bindings and package archives by hashes.
Host acquisition validates those artifacts before selection; an unpublished
candidate never silently installs the older public package with the same version.
No install hooks compile native code and there is no source fallback.

Generic packaging uses portable Bun archive creation/extraction for ordinary
applications. Executable modes, empty directories and Unicode/PAX paths survive
extraction. The extracted native application validates its committed extension
tree with the production build-envelope, DTO, parent and composition validator.
The custom starter renders its Rust greeting reactively without writing text
into the framed Bun protocol stream.

Consumer workspace dependency resolution participates in native build provenance.
Generated native projects and the source artifact include Cargo configuration;
Vite selects the actual workspace lock and Cargo-reported executable path.
Release protocol checks derive their version from the canonical schema lock.
The manual candidate workflow builds paired artifacts and qualifies four
consumers without publishing. Workflow and local setup action were reviewed:
actions are pinned, credentials are disabled, and token permissions are read-only.

## Verification

Portable archive, corrupted acquisition, exact-version scaffold, project
preparation/preview, compiler and tooling tests passed. Final package/tool/fixture
typechecks passed. The Rust workspace gate passed 459 tests, with two existing
ignored experiments/documentation cases; extracted admission and build-provenance
regressions are included. Local paired-artifact and four clean-consumer checks
are the remaining integration qualification, recorded in `integration.md`.

## Documentation and limits

Getting started, Vite, distribution, runtime/CI guides, explicit Chinese copies,
website content and compiler exports are synchronized. This machine qualifies
macOS aarch64. The workflow has not been executed remotely. Windows/Linux,
notarized releases and public publication remain unqualified and unauthorized.
Local development-profile checks must not be described as release qualification.
