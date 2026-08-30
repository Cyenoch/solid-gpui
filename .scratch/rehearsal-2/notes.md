# 0.3.0 release-prep rehearsal

- Date: 2026-08-30
- Base: detached worktree at `064e4b0`
- Worktree: `/tmp/rehearsal-2`
- Sequence: confirmed clean detached worktree; temporarily moved the former
  `Unreleased` entries under `## [0.3.0]`, ran `make release-prep VERSION=0.3.0`,
  ran the requested gates, then removed the worktree. The temporary changelog
  heading and all generated release changes were confined to the worktree and
  were discarded with it.

## Four-source synchronization

Before release preparation, all four sources were `0.2.0`:

| Source | Before | After |
| --- | --- | --- |
| Cargo workspace package (`Cargo.toml`) | `0.2.0` | `0.3.0` |
| `packages/react-gpui/package.json` | `0.2.0` | `0.3.0` |
| `packages/react-gpui-dev/package.json` | `0.2.0` | `0.3.0` |
| `packages/react-gpui/src/renderer/host-config.ts` `rendererVersion` | `0.2.0` | `0.3.0` |

`make release-prep VERSION=0.3.0` completed successfully in 37.76 s (real
 time). Its diff changed exactly those four source lines; `Cargo.lock` also
updated the three workspace package entries from `0.2.0` to `0.3.0`, while both
Bun lockfiles stayed unchanged. A post-prep read verified all four values as
`0.3.0`.

The script's failure rollback includes the fourth source: its
`restore_on_failure` handler copies the backed-up `host-config.ts` alongside
`Cargo.toml`, both package manifests, `Cargo.lock`, and both Bun lockfiles
(lines 68-74). The backup is created before mutation (line 83), and the normal
update path rewrites the anchored `rendererVersion` line.

## Gates

Commands ran serially in the isolated worktree. Times are `/usr/bin/time -p`
real times:

| Command | Result | Real time | Evidence |
| --- | ---: | ---: | --- |
| `make ci` (first attempt) | 2 | 56.28 s | Rust gates passed; dev package type build could not resolve the local `@react-gpui/core` declaration package after the first package build. |
| Refresh: `cd packages/react-gpui-dev && bun install --frozen-lockfile` | 0 | 0.14 s | Refreshed the local core dependency copy; core declarations were present. |
| `make ci` (successful retry) | 0 | 59.17 s | Rust format/check/clippy/tests passed; core Bun 132 pass / 0 fail; dev Bun 24 pass / 0 fail; package tarball consumer smoke passed; both Bun audits clean (12 and 59 packages); cargo-deny `advisories ok`. |
| `make embedded-bun` | 0 | 137.63 s | Embedded examples 2 passed; embedded counter 1 passed; locked checks passed. |
| `make host-candidate-smoke` | 0 | 75.47 s | Release archive checks, allowlist, process timeout/snapshot, startup diagnostic, help and version all passed. |

The first CI failure was a local file-dependency hydration issue during the
rehearsal, not a release-prep version-sync failure. After the explicit frozen
install refresh, the complete CI gate passed. No four-source release-prep rot
was found.

## Candidate archive

- Archive: `dist/react-gpui-host-0.3.0-aarch64-apple-darwin.tar.gz`
- SHA-256 (both consecutive archives, identical):
  `7a39f7067025afa14f33f6874e54e9defc6ccf56fe33a83c1fe493a4cd51e944`
- The release check accepted only the expected entries: the bundle directory,
  `react-gpui-host`, `README.md`, `LICENSE`, and `SHA256SUMS`; embedded checksums
  passed.
- Candidate output: `react-gpui-host 0.3.0 protocol=v3`.
- Candidate process smoke observed a 312-byte snapshot commit, expected timeout
  exit 124 at both error and info levels, and the info startup diagnostic with
  `protocol=v3`; help output validation passed.

## Cleanup and boundaries

`git worktree remove --force /tmp/rehearsal-2` succeeded; `git worktree list`
then showed only the main worktree. The rehearsal did not edit the main tree and
did not touch live gallery processes. Concurrent metadata work in the main tree
was left intact; this commit contains only this force-added evidence file.
