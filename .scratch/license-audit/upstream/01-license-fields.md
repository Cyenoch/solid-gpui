# Posting checklist

> - [ ] Post this issue in `zed-industries/zed` (not a downstream project).
> - [ ] Before posting, verify both manifests and the cited dependency lines at the current `main`; update the revisions and line numbers if they have moved.
> - [ ] Keep the issue focused on missing Cargo license metadata; do not infer or select a license on behalf of the copyright holders.
> - [ ] Attach or paste the relevant `cargo deny check licenses` output from the exact downstream revision being audited, with local/private repository details removed.

# Issue title

Missing `license` field in `gpui_shared_string` and `gpui_util` manifests

# Issue body

## Summary

The published/package manifests for `gpui_shared_string` and `gpui_util` do not contain a Cargo `license` (or `license-file`) field. This is true at the Zed revision we currently consume, `6805d952f9f3d702f760aa11b1547df8a625fa16`, and at the fetched `origin/main` snapshot `1662f5f3f6497c5f80830ccdca1edfd1fc0c6c6a` (checked 2026-08-31).

This report is about missing package metadata, not a claim that either crate is unlicensed. Both crates carry an explicit per-crate `LICENSE-APACHE` symlink to Zed's root Apache-2.0 license text; the metadata omission prevents Cargo tooling from classifying that already-identifiable text. We are asking the maintainers/copyright holders to record the intended distribution terms in the manifests.

## Reproduction

At the pinned revision, the relevant manifests are:

- `crates/gpui_shared_string/Cargo.toml:1-5` identifies `gpui_shared_string` 0.1.0 and has no `license` field before the remainder of the package metadata (`:7-16`). Its sibling `crates/gpui_shared_string/LICENSE-APACHE` symlink targets `../../LICENSE-APACHE`.
- `crates/gpui_util/Cargo.toml:1-5` identifies `gpui_util` 0.1.0 and has no `license` field before its dependency tables (`:7-15`). Its sibling `crates/gpui_util/LICENSE-APACHE` symlink targets `../../LICENSE-APACHE`.

The linked file identifies the `Apache License, Version 2.0`; the control crate `gpui` carries the same `LICENSE-APACHE` symlink and identical contents. The terms are therefore determinable from Zed's shipped source evidence even though the Cargo fields are absent.

The crates are ordinary dependencies of GPUI rather than optional dependencies:

- Pinned `crates/gpui/Cargo.toml:46-107`, specifically `gpui_shared_string.workspace = true` at line 61 and `gpui_util.workspace = true` at line 96.
- Fetched `origin/main` GPUI manifest: `gpui_shared_string.workspace = true` at line 64 and `gpui_util.workspace = true` at line 99.
- The same two package manifests and per-crate Apache license files are present at the fetched `origin/main` snapshot; no license-field addition was present.

A strict downstream `cargo-deny` run reports the missing metadata. The following is an abridged summary of the observed result from `cargo-deny 0.20.2`; the full run also reports other findings because the downstream policy is intentionally strict:

```text
$ cargo deny check licenses
...
670 rejected records under the current license policy
2 no-license-field warnings:
  gpui_shared_string 0.1.0
  gpui_util 0.1.0
...
process exited with status 4
```

The exact output should be re-captured against the revision being reported before posting. The per-crate license files identify Apache-2.0 terms, but this does not remove the metadata warning.

## Why this matters to downstream consumers

Downstream projects that link GPUI code into a binary need to identify the distribution terms for every included dependency. Without a manifest `license` field (or an explicit `license-file`), Cargo metadata and tools such as `cargo-deny` cannot classify these two packages even though their per-crate `LICENSE-APACHE` files identify Apache-2.0 text. That prevents a downstream distributor from clearing its metadata audit and, in our case, publication remains held pending the metadata fix and confirmation that the identified terms govern the exact distributed artifacts.

This also affects consumers that do not otherwise change GPUI features: both entries are ordinary `[dependencies]` entries in `gpui/Cargo.toml`, so they remain in the resolved graph.

## Suggested fix

Could the maintainers add `license = "Apache-2.0"` or equivalent `license-file` metadata to both manifests, consistent with the explicit per-crate Apache-2.0 license files and the intended licensing of each crate? We are asking the copyright holders to confirm or correct that identification, not selecting terms on their behalf.

- Pinned checkout: `crates/gpui_shared_string/Cargo.toml:1-16`; `crates/gpui_shared_string/LICENSE-APACHE`
- Pinned checkout: `crates/gpui_util/Cargo.toml:1-15`; `crates/gpui_util/LICENSE-APACHE`
- GPUI dependency table: `crates/gpui/Cargo.toml:46-107` (pinned lines 61 and 96; fetched `origin/main` lines 64 and 99)
- Audit record: `.scratch/license-audit/recheck-2026-08-31.md` and `.scratch/license-audit/2026-08-30.md` (downstream evidence; the repository is private)

Thank you for maintaining these crates and for considering this small metadata fix. It would make downstream license audits substantially easier.
