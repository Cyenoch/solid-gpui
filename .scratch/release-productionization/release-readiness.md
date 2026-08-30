# Release-readiness inventory

Generated: 2026-08-30T11:13:19+0800
Decision owner: human release owner  
Current HEAD: `3a1a9ea` (`test(host): last surface exit semantics`)

This is an evidence package, not a release approval. It records the post-release
cycle position, fresh command runs, current contract coverage, documentation
consistency, known boundaries, and inputs for the human publication decision.
Version `0.2.0` was cut and tagged locally; no artifact has been published or
pushed.

## Executive readout

- All five fresh gates passed at exit code 0 on candidate `3a1a9ea`, serially and
  exclusively under `/usr/bin/time -p`: `make ci`, `make embedded-bun`,
  `make host-candidate-smoke`, `make host-embedded-candidate-smoke`, and
  `make bun-pack-smoke`.
- The current candidate is `0.2.0` on macOS ARM. Release scripts prove archive
  consistency and CLI/runtime behavior, not display-backed GUI behavior. Both
  process and embedded candidate rehearsals passed their expected timeout,
  help, version, and commit/press checks.
- The final `CHANGELOG.md` Unreleased section contains **5 Added capability,
  5 Testing, 3 Fixed, and 1 Changed entries**. Counting Added plus Testing as
  release-facing additions gives **10 additions / 3 fixes / 1 change**.
- `git log --oneline cf985a4..3a1a9ea` reports **6 delta commits**. The crosswalk
  below records error-string documentation alignment, examples launch smoke,
  process kill resilience, and last-surface lifecycle evidence.
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
  ladder, conventions, evidence, and architecture wayfinding.

## 2. Fresh five-gate evidence

All five commands exited 0 and ran serially and exclusively under
`/usr/bin/time -p` at candidate HEAD `3a1a9ea`, in this order: `make ci`,
`make embedded-bun`, `make host-candidate-smoke`,
`make host-embedded-candidate-smoke`, and `make bun-pack-smoke`. Candidate
timeout exits are expected successful rehearsal behavior.

| Command | Exit | Real time | Counts / fresh observed evidence |
| --- | ---: | ---: | --- |
| `make ci` | 0 | **53.81 s** | Rust workspace green; Core Bun **132 pass / 0 fail / 453,263 expect() calls**; dev Bun **24 pass / 0 fail / 56 expect() calls**; formatting, checks, builds, typechecks, package smoke, property, soak, and performance guards passed. |
| `make embedded-bun` | 0 | **1.75 s** | `embedded_examples` **2 passed**; embedded counter **1 passed**; locked checks passed. |
| `make host-candidate-smoke` | 0 | **11.80 s** | Identical archive SHA-256 **`3f0a8835c2e946d2bab9ea32bd75ed490c0a502a719104c7870bc3524bd9e9a8`** matched on both archives; archive `react-gpui-host-0.2.0-aarch64-apple-darwin.tar.gz`; snapshot commit **312 bytes**; expected process timeouts **5.011 s/5.014 s**; version/help passed. |
| `make host-embedded-candidate-smoke` | 0 | **5.11 s** | Release embedded binary; `--smoke-press` reported `sent=true, commits=2, status=None`; expected timeout **4.593 s**; version/help passed. |
| `make bun-pack-smoke` | 0 | **2.38 s** | Frozen installs, both 0.2.0 JS/type builds and tarballs, external consumer install (**59 packages**), and `package tarball consumer smoke passed`. |

Measured sequential five-gate wall-time sum: **82.89 s**. Candidate timeout
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

The current `CHANGELOG.md` Unreleased section contains **5 Added capability,
5 Testing, 3 Fixed, and 1 Changed entries**. Counting all Added and Testing
bullets as release-facing additions gives **10 additions / 3 fixes / 1 change**.
The five Testing entries are tree patch property, renderer soak, surface lifecycle
property, host last-surface lifecycle, and TypeScript commit-emission property.
The Added entries include examples launch smoke and process kill resilience. The
Changed entry is the `PointerAction` removal.

### (b) Refresh-delta commit crosswalk from `cf985a4`

`git log --oneline cf985a4..3a1a9ea` reports **6 commits**:

| Commit | Reachable evidence |
| --- | --- |
| `576cdf6` | Aligned README, protocol, troubleshooting, and package README wording with rewritten actionable error strings; no changelog section change. |
| `fe0af9e` | Added `make examples-smoke`; 14/14 process-mode launch/teardown evidence. |
| `53a895e` | Completed the package README example table. |
| `9fd771e` | Added `make kill-resilience`; host/renderer/process-group SIGKILL coverage for counter and gallery. |
| `370684e` | Recorded the six-scenario kill matrix and zero-orphan verdict. |
| `3a1a9ea` | Added headless last-surface lifecycle Testing entry and focused two-surface final-close quit coverage. |

### (c) Documentation and upstream alignment

The verified distribution pair, onboarding dogfood verdict, actionable error-string
alignment, contributor guide, documentation index, export audit, 14/14 examples
launch smoke, kill-resilience zero-orphan matrix, and last-surface lifecycle test
are represented in the current evidence set. ADR-0008 remains aligned with
Error-Boundary ownership for render failures, fail-fast protocol/tree rejection,
local image fallback, and host-fatal GPUI paint panic. The GPUI drift verdict is
**no released unblocks** for per-run typography, explicit direction, letter
spacing, secure obscuring, live regions, runtime position setters, or portable
Linux image seams.

## 5. Known limits and human decision items

The implementation backlog for the current 0.2.0 contract remains empty. Issue
01 remains unsigned/not notarized; issue 02 remains future-incompatibility
review; issue 03 remains a separate embedded source build; issue 04 remains a
display-backed Quartz/native boundary; issue 05 remains human-owned fuzz
maintenance; issue 06 remains unvalidated cross-platform CI until runner jobs
execute. Upstream gaps remain explicit for secure/password display,
RTL/container direction and bidi-aware hit/caret/IME semantics, `letterSpacing`,
and JavaScript Image `onError`. Native TextInput undo/redo is host-owned and
implemented; `fallbackSource` remains the visual Image degradation path.

## 6. Post-release cycle position

**v0.2.0 cut locally + tagged; publication (push/npm/signing) pending human
action.** Tag `v0.2.0` remains anchored to release commit `72a520c`; no tag move
or publication operation is part of this refresh. The candidate archive SHA is
**`3f0a8835c2e946d2bab9ea32bd75ed490c0a502a719104c7870bc3524bd9e9a8`**.

The current cycle contains **5 Added, 5 Testing, 3 Fixed, and 1 Changed**
entries; its release-facing count is **10 additions / 3 fixes / 1 change**.
All five fresh gates passed serially under `/usr/bin/time -p` on `3a1a9ea`, and
the companion checklist records the same gate matrix, archive checksum,
capability notes, crosswalk, and human publication boundaries.

No publication option is selected by this inventory; human action is required
for push, npm publication, signing/notarization, and any Rust crate release.
