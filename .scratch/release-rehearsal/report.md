# Release-prep rehearsal

- **HEAD exercised:** `346e99b`
- **Rehearsal version:** `0.2.0`
- **Verdict:** version-cut tooling verified at HEAD 346e99b, after the approved Makefile ordering fix.

## Step map

1. `.github/workflows/release-prep.yml` is a manual `workflow_dispatch` workflow. It accepts a strict `version` input, sets `RELEASE_VERSION`, checks out the repository, installs the pinned Bun version from `.bun-version` (`1.4.0`), restores the Bun install cache, and runs `make release-prep VERSION="$RELEASE_VERSION"`.
2. `Makefile` delegates `release-prep` to `bash scripts/release-prep.sh "$(VERSION)"`.
3. `scripts/release-prep.sh` rejects a missing version, `0.0.0`, and anything other than strict `MAJOR.MINOR.PATCH`. It reads `[workspace.package] version` from `Cargo.toml` and the `version` fields from both package manifests. If all three already match, it exits successfully without changing locks. Otherwise it requires an exact `## [VERSION]` section in `CHANGELOG.md`.
4. The script snapshots `Cargo.toml`, both package manifests, `Cargo.lock`, and both Bun locks in a temporary directory. It then rewrites the single Cargo workspace version and both Bun package versions.
5. It refreshes the Cargo lock with `cargo update --workspace`, verifies Cargo with `cargo check --workspace --locked`, refreshes each Bun lock with `bun install`, and freezes each with `bun install --frozen-lockfile`. On failure before completion it restores all six snapshotted files.
6. It re-reads all three manifests, checks that all equal the requested version, prints `release-prep: synchronized version VERSION`, and prints a Git diff summary. The script itself does not pack archives.
7. The workflow then runs `make ci`. `ci` runs Rust format/check/clippy/test and Bun format/typecheck/test/package-pack-smoke. `bun-build` produces `dist` artifacts for both packages; the approved fix makes `bun-typecheck` depend on `bun-build` so a clean checkout has core declarations before dev typechecking.
8. The workflow packs candidates without publishing: `bun pm pack --filename "$RUNNER_TEMP/react-gpui-core-$RELEASE_VERSION.tgz"` and the corresponding dev command. It uploads those tarballs as artifact `react-gpui-release-prep-VERSION`.
9. Separately, `make host-release-check` calls `scripts/host-release.sh check`. It builds the host in release mode with `--locked`, stages the binary/README/LICENSE/checksums, creates two deterministic `ustar` + `gzip -n` archives, compares their SHA-256 values, extracts and verifies the archive, checks the extracted binary's help and version output, and removes temporary staging.

## Worktree rehearsal

A detached worktree was created exactly at `/tmp/vue-gpui-release-rehearsal` with `git worktree add --detach ... HEAD`; all release commands ran there. A temporary `## [0.2.0]` changelog section was added only in that worktree to satisfy the script's required changelog guard.

### Version and lock evidence

`make release-prep VERSION=0.2.0` succeeded. Its key output was:

```text
Updating react-gpui v0.1.0 -> v0.2.0
Updating react-gpui-bun v0.1.0 -> v0.2.0
Updating react-gpui-host v0.1.0 -> v0.2.0
Finished `dev` profile
bun install v1.4.0 ...
release-prep: synchronized version 0.2.0
release-prep diff summary:
 Cargo.lock                           | 6 +++---
 Cargo.toml                           | 2 +-
 packages/react-gpui-dev/package.json | 2 +-
 packages/react-gpui/package.json     | 2 +-
```

The final rehearsal values were:

```text
Cargo.toml [workspace.package] version = 0.2.0
@react-gpui/core package.json version = 0.2.0
@react-gpui/dev package.json version = 0.2.0
cargo metadata: react-gpui=0.2.0, react-gpui-host=0.2.0, react-gpui-bun=0.2.0
Cargo.lock: only the three workspace package version lines changed (0.1.0 -> 0.2.0)
```

Both explicit freeze checks passed with no lock changes:

```text
packages/react-gpui: Checked 14 installs across 15 packages (no changes)
packages/react-gpui-dev: 1 package installed (@react-gpui/core local dependency)
```

The Bun lockfiles were unchanged by the rehearsal. The release script's temporary backups were removed on successful completion.

### Initial CI finding and fix

The first clean-worktree `make ci` correctly exposed release-path rot at `bun-typecheck`: the core package had no `dist/`, so the dev package could not resolve `@react-gpui/core` declarations. It emitted `TS2307: Cannot find module '@react-gpui/core'` followed by cascading `TestNode`/implicit-any errors. This was a dependency ordering problem, not a source failure.

With the user's approval, the main `Makefile` change was one prerequisite change:

```diff
-bun-typecheck: bun-install
+bun-typecheck: bun-build
```

The same one-line prerequisite was applied in the throwaway worktree and `make ci` was rerun there. The fixed run passed Rust formatting, Cargo check/clippy/test, Bun formatting, both package builds, both typechecks, both test suites, and `scripts/package-pack-smoke.sh`:

```text
Rust: all tests passed (115 + 6 + 3 + 1 + 3 + 1 + 12 + 27 + 25 + 2; no failures)
Core Bun: 129 pass, 0 fail
Dev Bun: 20 pass, 0 fail
package tarball consumer smoke passed
```

The test output includes expected diagnostic text from negative Fast Refresh and protocol-tap cases; the suites still reported zero failures.

### Candidate package output

After the fixed build, the exact workflow pack commands succeeded and produced:

```text
/tmp/vue-gpui-release-candidates/react-gpui-core-0.2.0.tgz
/tmp/vue-gpui-release-candidates/react-gpui-dev-0.2.0.tgz
```

The package smoke additionally validated required JS/declaration/license/README contents and an external tarball consumer.

### Host release archive output

The single requested `make host-release-check` invocation in the rehearsal succeeded. It produced and validated:

```text
host release archive SHA256: f5adce841196159ae15e207cf71ace053403034848b7fdae2255531f691003ee
host release second archive SHA256: f5adce841196159ae15e207cf71ace053403034848b7fdae2255531f691003ee
react-gpui-host: OK
README.md: OK
LICENSE: OK
host release check passed: .../dist/react-gpui-host-0.2.0-aarch64-apple-darwin.tar.gz
```

Thus the archive is named for `0.2.0`, has the expected macOS ARM target suffix, and is byte-deterministic across consecutive builds.

## Main-tree changes and cleanup

- Product sources were not changed.
- `AGENTS.md` remained the user's existing modification and was not staged.
- Approved tooling fix: `Makefile` `bun-typecheck` now depends on `bun-build`.
- The rehearsal worktree and temporary outputs were removed.
- This report is the only scratch artifact added on main.
- Main focused verification after the fix: `make bun-typecheck` passed (both package builds and both typechecks).

The release-rehearsal commit is intentionally limited to this report and the approved Makefile ordering fix.
