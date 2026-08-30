# Posting checklist

> - [ ] Post this issue in `zed-industries/zed` (not a downstream project).
> - [ ] Before posting, verify both manifests and the cited dependency lines at the current `main`; update the revisions and line numbers if they have moved.
> - [ ] Keep the issue focused on missing Cargo license metadata; do not infer or select a license on behalf of the copyright holders.
> - [ ] Attach or paste the relevant `cargo deny check licenses` output from the exact downstream revision being audited, with local/private repository details removed.

# Issue title

Missing `license` field in `gpui_shared_string` and `gpui_util` manifests

# Issue body

## Summary

The published/package manifests for `gpui_shared_string` and `gpui_util` do not contain a Cargo `license` (or `license-file`) field. This is true at the Zed revision we currently consume, `6805d952f9f3d702f760aa11b1547df8a625fa16`, and at the `origin/main` snapshot we checked, `1662f5f3f6497c5f80830ccdca1edfd1fc0c6c6a` (2026-08-29).

This report is about missing package metadata, not a claim that either crate is unlicensed. We are asking the maintainers/copyright holders to record the intended distribution terms in the manifests.

## Reproduction

At the pinned revision, the relevant manifests are:

- `crates/gpui_shared_string/Cargo.toml:1-5` identifies `gpui_shared_string` 0.1.0 and has no `license` field before the remainder of the package metadata (`:7-16`).
- `crates/gpui_util/Cargo.toml:1-5` identifies `gpui_util` 0.1.0 and has no `license` field before its dependency tables (`:7-15`).

The crates are ordinary dependencies of GPUI rather than optional dependencies:

- `crates/gpui/Cargo.toml:46-107`, specifically `gpui_shared_string.workspace = true` at line 61 and `gpui_util.workspace = true` at line 96.
- The same two package manifests and GPUI dependency entries are present at the `origin/main` snapshot above; no license-field addition was present when checked.

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

The exact output should be re-captured against the revision being reported before posting.

## Why this matters to downstream consumers

Downstream projects that link GPUI code into a binary need to identify the distribution terms for every included dependency. Without a manifest `license` field (or an explicit `license-file`), Cargo metadata and tools such as `cargo-deny` cannot classify these two packages. That prevents a downstream distributor from clearing its license audit and, in our case, blocks publication while the terms are confirmed with the relevant rights holders.

This also affects consumers that do not otherwise change GPUI features: both entries are ordinary `[dependencies]` entries in `gpui/Cargo.toml`, so they remain in the resolved graph.

## Suggested fix

Could the maintainers add an appropriate SPDX `license` field to both manifests (or otherwise provide equivalent package metadata), consistent with the intended licensing of each crate? We are not presuming which license the copyright holders will choose. If these crates are intended to follow the workspace/GPUI licensing intent, stating that explicitly in each manifest would let downstream tooling classify them.

- Pinned checkout: `crates/gpui_shared_string/Cargo.toml:1-16`
- Pinned checkout: `crates/gpui_util/Cargo.toml:1-15`
- GPUI dependency table: `crates/gpui/Cargo.toml:46-107`
- Audit record: `.scratch/license-audit/2026-08-30.md` (downstream evidence; the repository is private)

Thank you for maintaining these crates and for considering this small metadata fix. It would make downstream license audits substantially easier.
