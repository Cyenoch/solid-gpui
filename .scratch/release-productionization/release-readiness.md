# Release-readiness inventory

Generated: 2026-08-31T01:01:19+0800
Decision owner: human release owner  
Current HEAD: `2b3f39a` (`chore(meta): publish readiness`)

This is an evidence package, not a release approval. It records the post-release
cycle position, fresh command runs, current contract coverage, documentation
consistency, known boundaries, and inputs for the human publication decision.
Version `0.2.0` was cut and tagged locally; no artifact has been published or
pushed.

## Executive readout

- All five final gates passed at exit code 0 on candidate `2b3f39a`, serially and
  exclusively under `/usr/bin/time -p`: `make ci`, `make embedded-bun`,
  `make host-candidate-smoke`, `make host-embedded-candidate-smoke`, and
  `make bun-pack-smoke`. The first CI attempt was an environment-load flake:
  its event-storm test timed out after 2,416.86 s; no orphan cargo/Bun/test
  processes were found, the isolated event-storm rerun passed in 23.61 s, and
  warm `make ci` passed in 64.36 s.
- The current candidate is `0.2.0` on macOS ARM. Release scripts prove archive
  consistency and CLI/runtime behavior, not display-backed GUI behavior. Both
  process and embedded candidate rehearsals passed their expected timeout,
  help, version, and commit/press checks.
- The final `CHANGELOG.md` Unreleased section contains **6 Added capability,
  5 Testing, 5 Fixed, and 2 Changed entries**. Counting Added plus Testing as
  release-facing additions gives **11 additions / 5 fixes / 2 changes**.
- `git log --oneline d74467e..2b3f39a` reports **4 delta commits**. The
  crosswalk below records truthful renderer version wiring, root test scoping,
  the 0.3.0 rehearsal, and publication metadata readiness.
- The implementation backlog for the current 0.2.0 contract remains empty.

## Current position

**v0.2.0 cut locally + tagged; publication (push/npm/signing) pending human
action.** The annotated local tag `v0.2.0` still points to release commit
`72a520c`; no tag move or publication operation is part of this refresh. The
candidate archive was rebuilt by `make host-candidate-smoke` and is represented
by the identical SHA-256 pair in the gate table below.

`.scratch/` is gitignored by default; this readiness report and its companion
checklist follow the repository's force-add evidence convention. No artifacts
were published.

## 1. Capability inventory

### Protocol and wire contract

- Command codes `1..35` and semantic event codes `1..23` are complete. Cross-
  language golden vectors lock producer bytes and semantic decoding.
- Deterministic protocol fuzz coverage remains **35/35 command seeds and 23/23
  event seeds** in both Rust and TypeScript records.
- Retained-tree validation covers ownership, contiguous indexes, revisions,
  listeners, host properties, accessibility, focusability, malformed patches,
  rollback, and one-level rich Text runs. The 42-slot style tuple remains the
  current style contract.
- Root APIs cover rendering, surfaces, window controls, focus traversal,
  clipboard, file dialogs and text-file I/O, fonts, notifications, menus,
  keybindings, close policy, and transport termination.

### Runtime, host, and examples

- `ProcessAdapter` uses framed stdin/stdout with bounded ordered event delivery,
  explicit shutdown/EOF/failure states, retained failure reporting, and
  non-sensitive wire identity in fatal diagnostics.
- `EmbeddedBunAdapter` runs pinned Bun/JSC on a dedicated runtime thread with a
  bounded callback bridge and serialized Fast Refresh lifecycle.
- Focus traversal reaches View, Pressable, TextInput, and selectable Text nodes;
  focused-node unmount emits blur and restores a live focus target; disabled
  controls are skipped and per-window isolation is preserved.
- The flagship gallery includes mixed-style rich text, keyboard-accessible
  links, selectable multi-run text, shared theme state, multiple surfaces, and
  dirty-state close confirmation. The core README indexes fourteen examples.
- `make examples-smoke` launched all **14/14** runnable examples through the
  real process-mode Bun/stdio path with `protocol=v3` startup diagnostics and
  clean bounded teardown. It does not touch live gallery processes.
- Headless host lifecycle coverage proves six close-policy/lifecycle scenarios:
  clean allow, confirmation request, duplicate pending request, denied
  resolution, allowed deferred resolution, and two-surface first/final close.
  The first close keeps the host alive; the final close requests `cx.quit()`
  after the registry is empty. The verdict is **zero orphan / all clean**.
  Native user-close process exit timing is display-backed and not claimed.
- Display-backed Quartz dialogs, menus, clipboard/window behavior, glyph pixels,
  native drag visuals, tooltip appearance, pointer positioning, and
  accessibility tree behavior remain acceptance boundaries rather than
  headless claims.

### Developer and distribution tooling

- Both Bun packages build ESM and declaration artifacts, expose built package
  exports, carry license text, and pass an external tarball consumer smoke.
  Checked-in API locks are **core 115 / dev 20** name-and-kind exports.
- Troubleshooting provides symptom-to-diagnosis-to-repair; onboarding documents
  local checkout/Cargo and standalone archive plus packed-package paths, with
  repository-only example imports called out.
- The documentation index and contributor guide record toolchain, verification
## 2. Fresh five-gate evidence

All five commands exited 0 and ran serially and exclusively under
`/usr/bin/time -p` at candidate HEAD `2b3f39a`, in this order: `make ci`,
`make embedded-bun`, `make host-candidate-smoke`,
`make host-embedded-candidate-smoke`, and `make bun-pack-smoke`. Candidate
timeout exits are expected successful rehearsal behavior.

The first `make ci` attempt exited 2 after **2,416.86 s** because
`perf-event-storm` timed out; `ps`/`pgrep` found no orphan cargo/Bun/test
processes. The isolated `/usr/bin/time -p bun test
tests/perf-event-storm.test.tsx` then passed (**1/1**, 78 assertions, **23.61
s**), and the warm CI rerun passed in **64.36 s**. This is recorded as an
environment-load flake, not a source regression. The successful CI row below
is the readiness result.

| Command | Exit | Real time | Counts / fresh observed evidence |
| --- | ---: | ---: | --- |
| `make ci` (warm rerun) | 0 | **64.36 s** | Rust workspace green; Core Bun **132 pass / 0 fail / 453,263 expect() calls**; dev Bun **24 pass / 0 fail / 56 expect() calls**; formatting, checks, builds, typechecks, package smoke, property, soak, performance guards, both Bun audits (**12** and **59** packages, no vulnerabilities), and cargo-deny (`advisories ok`) passed. Initial run: exit 2 / 2,416.86 s due to event-storm timeout; isolated rerun: 1/1 / 23.61 s. |
| `make embedded-bun` | 0 | **7.11 s** | `embedded_examples` **2 passed**; embedded counter **1 passed**; locked checks passed. |
| `make host-candidate-smoke` | 0 | **12.50 s** | Identical archive SHA-256 **`a6086a7f62be4b1604a88925fb186e1df5b6c90a2783f5c67729b4bba7b3076e`** matched on both archives; archive `react-gpui-host-0.2.0-aarch64-apple-darwin.tar.gz`; snapshot commit **312 bytes**; expected process timeouts **5.006 s/5.014 s**; version/help passed. |
| `make host-embedded-candidate-smoke` | 0 | **5.22 s** | Release embedded binary; `--smoke-press` reported `sent=true, commits=2, status=None`; expected timeout **4.647 s**; version/help passed. |
| `make bun-pack-smoke` | 0 | **2.11 s** | Frozen installs, both 0.2.0 JS/type builds and tarballs, external consumer install (**59 packages**), and `package tarball consumer smoke passed`. |

Measured sequential five-gate wall-time sum: **91.20 s**. Candidate timeout
exits are expected behavior, not gate failures. The archive SHA is the
process-candidate check's identical pair of SHA-256 values.

## 3. Quality evidence matrix

| Area | Current evidence |
| --- | --- |
| Rust workspace | Fresh `make ci` green; surface property **2/2**, tree property **2/2**, renderer soak **1/1**, and host lifecycle `command_roundtrip` **28/28**; `make embedded-bun`: startup matrix **2/2** and embedded counter **1/1**. |
| TypeScript renderer | Fresh `make ci`: **132/132 tests**, **453,263 assertions**, 0 failures. |
| Development package | Fresh `make ci`: **24/24 tests**, **56 assertions**, expected malformed-source diagnostics and last-good tree behavior. |
| Fuzz / malformed input | **35/35 command seeds and 23/23 event seeds** in both Rust and TypeScript records; fresh `make ci` green. |
| Property and soak safety nets | Tree **32 x 64**, surface **16 x 48**, TypeScript **24 x 40**, renderer soak **2,000 iterations**, and host lifecycle six-scenario matrix passed with clean bounded state. |
| Examples and process lifecycle | `make examples-smoke` verified **14/14** launches and bounded teardown; kill resilience verified six scenarios with zero orphans; last-surface close test passed headlessly. |
| API surface locks | Checked-in name/kind fixtures lock **core 115 / dev 20** exports after `PointerAction` removal. |
| Release packaging | Process and embedded candidate scripts, deterministic archive checks, package tarball consumer smoke, and embedded package gate passed. Checksums establish reproducibility, not publisher authenticity. |

## 4. Consistency and crosswalk audit

### (a) Current Unreleased inventory

The current `CHANGELOG.md` Unreleased section contains **6 Added capability,
5 Testing, 5 Fixed, and 2 Changed entries**. Counting all Added and Testing
bullets as release-facing additions gives **11 additions / 5 fixes / 2 changes**.
The five Fixed entries include truthful renderer-version metadata wiring and
root Bun test discovery scoping, in addition to diagnostics and patch/state
cleanup. The two Changed entries are `PointerAction` removal and the
`ttf-parser` to `skrifa` migration.

### (b) Refresh-delta commit crosswalk from `d74467e`

`git log --oneline d74467e..2b3f39a` reports **4 commits**:

| Commit | Reachable evidence |
| --- | --- |
| `a1e7683` | Truthful renderer version constant: `rendererVersion` now follows package metadata; `release-prep` synchronizes all four release sources; `.scratch/version-const/notes.md` records the rationale. |
| `064e4b0` | Root Bun test discovery now ignores generated `target/**`; `.scratch/testing-dx` documents the test entry and boundary (no separate root-buntest notes file). |
| `003060b` | Detached 0.3.0 cut rehearsal: all four version sources synchronized, archive reproducibility passed, and the known worktree file-dependency hydration difference was resolved by a frozen install refresh. |
| `2b3f39a` | Publish metadata readiness: npm tarball allowlists, crates SPDX metadata, and unsafe embedded Bun/JSC FFI inventory documented; no publication performed. |

### (c) Documentation, capability, and upstream alignment

The four-source release synchronization, root-test scoping, 0.3.0 rehearsal,
publication metadata, supply-chain status, and final five-gate matrix are
represented in the refreshed evidence set. The rehearsal's known worktree
hydration difference is explicit: detached worktree CI initially failed to
resolve the local core declaration package, then passed after a frozen install
refresh; no release-prep drift was found.

The publication audit records both package tarball allowlists (18 core / 8
dev files), Cargo SPDX `Apache-2.0` metadata, and the unsafe Bun/JSC FFI
inventory. Supply-chain status remains layered: Bun audits clean, cargo-deny
is wired and passes with three documented unmaintained-crate exceptions,
`skrifa` eliminates direct application-owned parser exposure, and transitive
GPUI-owned `ttf-parser` remains open.

ADR-0008 remains aligned with Error-Boundary ownership for render failures,
fail-fast protocol/tree rejection, local image fallback, and host-fatal GPUI
paint panic. The GPUI drift verdict is **no released unblocks** for per-run
typography, explicit direction, letter spacing, secure obscuring, live regions,
runtime position setters, or portable Linux image seams.

## 5. Known limits and human decision items

The implementation backlog for the current 0.2.0 contract remains empty. Issue
01 remains unsigned/notarized; issue 02 remains future-incompatibility review;
issue 03 remains a separate embedded source build; issue 04 remains a
display-backed Quartz/native boundary; issue 05 remains human-owned fuzz
maintenance; issue 06 remains unvalidated cross-platform CI until runner jobs
execute. Upstream gaps remain explicit for secure/password display,
RTL/container direction and bidi-aware hit/caret/IME semantics, `letterSpacing`,
and JavaScript Image `onError`. Native TextInput undo/redo is host-owned and
implemented; `fallbackSource` remains the visual Image degradation path.

- **Recommendation only:** because Unreleased has matured and includes the
  breaking `PointerAction` removal, the next cut should be **0.3.0**, not
  **0.2.1**, under pre-1.0 SemVer; no version is selected.

## 6. Post-release cycle position

**v0.2.0 cut locally + tagged; publication (push/npm/signing) pending human
action.** Tag `v0.2.0` remains anchored to release commit `72a520c`; no tag move
or publication operation is part of this refresh. The candidate archive SHA is
**`a6086a7f62be4b1604a88925fb186e1df5b6c90a2783f5c67729b4bba7b3076e`**.

The current cycle contains **6 Added, 5 Testing, 5 Fixed, and 2 Changed**
entries; its release-facing count is **11 additions / 5 fixes / 2 changes**.
All five final gates passed serially under `/usr/bin/time -p` on `2b3f39a` after
the isolated event-storm rerun established the initial timeout as an
environment-load flake. The companion checklist records the same gate matrix,
archive checksum, capability notes, crosswalk, supply-chain boundary, and
human publication boundaries.

No publication option is selected by this inventory; human action is required
for push, npm publication, signing/notarization, and any Rust crate release.
