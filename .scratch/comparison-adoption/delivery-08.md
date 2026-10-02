# Ticket 08: signed application-owned updater

Branch: `adopt/08-signed-updater`. Base ancestry: `e879b662` verified with
`git merge-base --is-ancestor e879b66 HEAD` before edits. Work performed only in
the assigned worktree; no active application/install, credentials, release,
remote publishing, or external communications were modified.

## Delivered interface and ownership

- Rust feature `signed-updater` opts into `solid_gpui::updater` on native targets.
  No default host registers update authority. The desktop example registers
  `native_module(None)` and demonstrates a cancellable generated status command.
- `SignedUpdater::new(UpdaterConfig, Arc<dyn gpui::http_client::HttpClient>)`
  constructs the application-owned service. Rust configuration pins app/channel,
  feed/public key, current monotonic release sequence, exact bundle/executable,
  installation path, download/extraction budgets, deadline and loopback fixture
  permission. `native_module(Some(Arc<SignedUpdater>))` registers authority.
- Generated client commands: `updateStatus`, `checkUpdate`, `installUpdate`,
  `rollbackUpdate`, `confirmUpdate`. Offers are one bounded latest native resource;
  install accepts its opaque token, never a URL/key/path from JavaScript.
- `CommandDefinition::blocking` permits captured native service state while
  preserving the existing bounded executor/blocking admission and cooperative
  `NativeCallContext`. `sync` delegates to it; no alternate executor/HTTP stack.
- Solid owns consent and restart UX. Rust owns acquisition, cryptography,
  filesystem transaction and backup. No automatic quit, spawn, or relaunch.
  Installation and rollback report pending restart. Confirmation requires a host
  compiled with the new release sequence; the app performs its own health check.

## Security and installation assumptions

`solid-gpui-update-v1` uses exact UTF-8 JSON manifest bytes in lowercase hex,
plus an Ed25519 signature in lowercase hex. `verify_strict` authenticates the
manifest before parsing and again before staging; the signed artifact SHA-256
and exact byte count authenticate the bounded acquired tar. Every release binds
app/channel/platform (`macos-aarch64` or `macos-x86_64`), monotonically increasing
sequence, display version, URL, archive format, size/hash and bundle/executable.
Feed is 32 KiB, decoded payload is 12 KiB; artifact at most 256 MiB, extraction
at most 1 GiB/100,000 entries. Config may lower budgets. HTTPS exact-origin
artifact URLs, no redirects, credentials or fragments; literal loopback HTTP
only when native configuration explicitly permits fixtures. Weak keys rejected.

`app-tar-v1` is uncompressed USTAR with one exact `.app` root and only regular
files/directories. Traversal/absolute/Windows-shaped paths, links, devices,
extended/sparse headers, duplicates, deep paths and nonzero trailing data fail.
No ownership, xattrs, privileged modes or installer shell execution. Executable
must exist with execute permission. The app bundle must be self-contained with
no framework symlinks. Native download/extraction checks cancellation; stalled
HTTP cancels without waiting for its full timeout. One service operation runs
at a time; the OS lock prevents a second service instance owning the bundle.

macOS is the real default installation backend. Private sibling staging and
recovery record use the install filesystem; files/directories are synchronized,
then `renameatx_np(RENAME_SWAP)` atomically exchanges directories. Installation
path is never missing. Failed/cancelled commit attempts preserve or exchange
back the original; rollback failures retain the transaction and explicit error.
Old/new inode/device identities distinguish precommit cleanup from pending
confirmation after owner restart. The backup persists until explicit confirm.
Canonical paths and private current-user-owned transaction directories are
required. Unexpected identities stop recovery rather than guessing.

Trust assumes release signing code/key, native config and application-owned
installation parent. This is not an elevated installer and does not defend
against another process authorized to rewrite that directory concurrently.
Code-signing/notarization, Gatekeeper, health checks, multiple app processes and
user document persistence belong to the application/release qualification.
Windows/Linux constructor rejects installation before touching paths; disabled
bindings remain available. WASM does not expose this capability. Source/test
gates exist; macOS results do not qualify other OS backends.

## Ticket 02 packaging integration

The packager must assign/compile the native app/channel/platform/sequence,
finish bundling and OS signing policy, then verify its extracted app before
producing an additional USTAR update artifact. A normal distributable ZIP,
tar.gz, checksum sidecar or install-time native build is not an updater artifact.
Compute SHA-256/byte count over the final uncompressed tar. The signing tool
signs exact manifest bytes and wraps them in `{payload, signature}` hex fields;
signing keys remain outside the application and repository. Publish nothing in
this task. Full fields/example: `docs/signed-updates.md#feed-and-packaging-format`.
A new archive format requires a new explicit interface, never a fallback.

## Qualification

Key tests run through the actual generated native dispatcher with deterministic
fixture signing keys, local TCP/HTTP and canonical temporary `.app` directories:

1. Signed check -> artifact download -> atomic install -> explicit rollback;
   observe new/old executable bytes, backup/restart state.
2. Corrupt signature and wrong app/channel/platform identities reject before
   staging; signed wrong artifact hash leaves the old app intact.
3. Authenticated unsafe archives reject traversal, symlink/hardlink, unpacked
   size limit and duplicate paths without outside writes or install changes.
4. Pending backup survives service restart; old sequence cannot confirm,
   restarted new sequence can confirm and release the backup.
5. Dropping a generated request during stalled HTTP headers releases worker
   ownership and preserves the app.
6. Oversized/redirected feeds and a symlink installation path reject.
7. OS permission failure rejects the atomic exchange, preserves the original,
   cleans the candidate and permits a later retry.
8. Disabled module exports its real contract and rejects install authority.

Focused native command: `cargo test -p solid-gpui --locked --features
signed-updater --test signed_updater --test native_executor`. Updater: 8 passed;
executor: 2 passed. All normal focused Cargo commands use `CARGO_BUILD_JOBS=2`
and the requested shared `CARGO_TARGET_DIR`.
Focused Clippy (`--lib --test signed_updater -- -D warnings`), changed Rust
format checks, JavaScript formatting and `git diff --check` also pass.

Other successful checks: generated desktop bindings (`prepare --check`), desktop
typecheck + application integration test (1 passed), desktop Vite production
build, workspace JS package builds, website content/runtime tests (7 passed),
website route generation/typecheck/frontend build, actual WASM Rust release
build + wasm-bindgen, and regenerated third-party notices.

Verification limits/observed tooling issues:

- Windows GNU cross-check was attempted but stopped in transitive `aws-lc-sys`
  because `x86_64-w64-mingw32-gcc` is unavailable. No Windows compile/install
  success is claimed; explicit unsupported backend has a target-gated test.
- `build-web-wasm.sh` ignores custom target paths at its wasm-bindgen step. Rust
  build succeeded; ran wasm-bindgen against the requested target directory,
  then the frontend build passed. The script itself is outside ticket 08 scope.
- Workspace Bun's `solid-gpui` bin link was unavailable after initial install;
  used the exact same `packages/solid-gpui-vite/src/cli.ts` implementation for
  prepare/check/test. No workaround source wrapper was added.
- Concurrent shared Cargo output briefly reused ticket 03's newer proc macro;
  rebuilding this worktree's unchanged macro source corrected the desktop build.
- No active app, real signed/notarized release, power-loss injection, physical
  GUI acceptance or external process relaunch was exercised. Fixture success is
  filesystem/native-service qualification, not product release qualification.

## Documentation and merge notes

Synchronized authoritative `signed-updates.md` and explicit Chinese translation,
documentation index, English/Chinese distribution and Rust bridge links/API,
desktop example READMEs, website guide navigation/label translation, generated
desktop API, changelog and derived dependency notices. Website imports both new
guides from authoritative sources; no duplicated guide content was hand-edited.

Ticket 03 replaces native contract constructors/source identity. Update the
updater's one `ModuleDefinition::new`/`with_contract` call to its canonical
behavioral version API (version `1.0.0`), preserve the blocking closure seam,
and regenerate the combined desktop catalog from the merged host. The existing
desktop generated file was already stale at base for unrelated Kit additions;
the broad catalog diff is actual host regeneration, not hand-written metadata.
Reconcile Cargo manifest/lock and regenerate notices after all dependency merges.

Commit branch work before merging `integrate/comparison-adoption` into this
branch. Do not merge this branch into integration here; the parent owns that
operation and final combined qualification/review.

Implementation commit: `182db31`. The requested inbound integration merge uses
`91a63761` (ticket 10 compiler/docs plus integration tracking). Its only conflict
was the parallel Unreleased changelog additions; both entries were preserved.
This branch has not been merged back into integration.
After the inbound merge: rebuilt JavaScript packages, then compiler + website
tests passed (10 tests), and website typecheck/frontend build passed. The
incoming upstream `.patch` has intentional space-prefixed blank context lines
reported by `git diff --check --cached`; patch bytes were preserved. Ticket 08
files and the resolved changelog pass whitespace checks.

## Updater recovery fixes

Follow-up work is confined to `adopt/08-signed-updater`; it does not merge or
modify integration. `updateStatus` now acquires the service state mutex before
filesystem inspection. Installation/rollback/confirmation replies use the same
locked state, and restart tracking is owned by `State` rather than a separate
atomic. A paused pre-swap installation test verifies status cannot observe the
staged record/candidate window and returns the committed state after release.

The persistent record now has explicit `staged`, `restoring`, and `cleaning`
phases. Rollback intent is synchronized before exchanging bundles. Cleanup
synchronizes its retained installation identity and signed release before any
recursive candidate deletion, retains that metadata through deletion and its
directory barrier, then unlinks the record. Restart recovery resumes restoring
or cleaning with exact directory identities; cleaning accepts an already absent
candidate. Final record-unlink synchronization failure permits an absent record
or a cleaning record with no candidate, both corresponding to the same durable
retained installation. Confirmation cannot supersede recorded rollback intent.

Four focused service/restart tests inject: cleanup phase synchronization failure,
candidate removal failure, partial recursive candidate deletion, deletion barrier
failure, record removal failure, final record-unlink barrier failure, post-swap
rollback exchange/synchronization failures, and precommit record synchronization
failure. Fault injection is invocation-thread-local and compiled only into Rust
unit tests; production hosts expose no injection control. Existing local HTTP,
signature/archive/cancellation and generated-dispatcher tests remain in place.

Technical English/Chinese updater guides document serialization and restart
semantics; ticket-planning phrasing was removed from those public guides. Legal
attribution and dependency notices are preserved. No contract DTO changed, so
native binding regeneration is unnecessary for this fix.

Fix validation: 4 new concurrency/recovery tests, 8 existing signed updater
tests, and 2 native executor tests pass. `cargo clippy -p solid-gpui --locked
--features signed-updater --lib --tests -- -D warnings` passes. Website content
tests pass (7), and website typecheck/frontend build pass using the previously
generated WASM host. Changed Rust formatting and whitespace checks pass.
No current installation, credentials, external process, or release publication
was accessed. New tests inject deterministic operation failures into temporary
app directories; they do not establish physical power-loss durability.
