# Release-readiness inventory

Generated: 2026-08-30T08:18:00+0800
Decision owner: human release owner  
Current HEAD: `61f82ee` (`test(tree): property-based patch sequence invariants`)

This is an evidence package, not a release approval. It records the post-release
cycle position, fresh command runs, current contract coverage, documentation
consistency, known boundaries, and inputs for the human publication decision.
Version `0.2.0` was cut and tagged locally; no artifact has been published or
pushed.

## Executive readout

- All five fresh gates passed at exit code 0 on `61f82ee`, serially and
  exclusively under `/usr/bin/time -p`: `make ci`, `make embedded-bun`,
  `make host-candidate-smoke`, `make host-embedded-candidate-smoke`, and
  `make bun-pack-smoke`.
- The current candidate is `0.2.0` on macOS ARM. Release scripts prove archive
  consistency and CLI/runtime behavior, not display-backed GUI behavior. Both
  process and embedded candidate rehearsals passed their expected timeout,
  help, version, and commit/press checks.
- The current `CHANGELOG.md` Unreleased section contains **3 Added capability
  entries, 1 Testing entry, 3 Fixed entries, and 0 Changed entries**. The
  release-facing count is **4 additions / 3 fixes / 0 changes** when the
  property-test safety net is counted with additions.
- `git log --oneline 5db2ba4..HEAD` reports **9 refresh-delta commits**. The
  crosswalk below records the encode guard, onboarding dogfood repairs,
  actionable diagnostics, verified distribution pair, contributor guide,
  style follow-up, multi-surface example, documentation index, and property
  test/fix round.
- The implementation backlog for the current 0.2.0 contract remains empty.
  Current capability evidence includes deterministic protocol fuzz coverage,
  the verified distribution pair, onboarding dogfood's corrected path,
  actionable error messages, the contributor guide, documentation index, and
  the property-test safety net.

## Current position

**v0.2.0 cut locally + tagged; publication (push/npm/signing) pending human
action.** The annotated local tag `v0.2.0` still points to release commit
`72a520c`; no tag move or publication operation is part of this refresh. The
current candidate archive was rebuilt by `make host-candidate-smoke` and is
represented by the identical SHA-256 pair in the gate table below.

`.scratch/release-productionization/v0.2.0-cut-checklist.md` is the companion
local checklist. `.scratch/` is gitignored by default; this readiness report and
checklist follow the repository's force-add evidence convention. No artifacts
were published.

## 1. Capability inventory

### Protocol and wire contract

- The command directory is complete for codes `1..35`, and the semantic event
  directory is complete for `1..23`. Current wire vectors cover TextInput (13),
  Image (4), Drag (5), Pointer (7), CommandResult (7), string Submit, and
  WindowResize (3), with one current form and explicit validation.
- Cross-language golden vectors lock producer bytes and semantic decoding.
  Deterministic protocol fuzz coverage is complete at **35/35 command seeds and
  23/23 event seeds** in both Rust and TypeScript records.
- Retained-tree validation covers ownership, contiguous indexes, revisions,
  listeners, host properties, accessibility, focusability, malformed patches,
  rollback, and one-level rich Text runs. The 42-slot style tuple remains the
  current style contract.
- Root APIs cover rendering, surfaces, window controls, focus traversal,
  clipboard, file dialogs and text-file I/O, fonts, notifications, menus,
  keybindings, close policy, and transport termination. VirtualList offset
  persistence, selectable Text, interactive Text runs, pointer coordinates,
  drag capabilities, and appearance/layout observations remain documented.

### Runtime, host, and examples

- `ProcessAdapter` uses framed stdin/stdout with bounded ordered event delivery,
  explicit shutdown/EOF/failure states, retained failure reporting, and
  non-sensitive wire identity in fatal diagnostics.
- `EmbeddedBunAdapter` runs pinned Bun/JSC on a dedicated runtime thread with a
  bounded callback bridge and serialized Fast Refresh lifecycle. The embedded
  startup matrix covers representative examples and the refresh probe.
- Focus traversal reaches View, Pressable, TextInput, and selectable Text nodes
  through explicit native tab stops. Focused-node unmount emits blur and
  restores focus to a live ancestor/restore target or first eligible stop;
  disabled controls are skipped and per-window isolation is preserved.
- The flagship gallery includes mixed-style rich text, keyboard-accessible
  links, selectable multi-run text, shared theme state, multiple surfaces, and
  dirty-state close confirmation. The core README indexes fourteen examples.
- Display-backed Quartz dialogs, menus, clipboard/window behavior, glyph
  pixels, native drag visuals, tooltip appearance, pointer positioning, and
  accessibility tree behavior are acceptance boundaries rather than headless
  claims.

### Developer and distribution tooling

- Both Bun packages build ESM and declaration artifacts, expose built package
  exports, carry license text, and pass an external tarball consumer smoke.
  Checked-in API locks are **core 114 / dev 20** name-and-kind exports.
- The troubleshooting guide provides symptom→diagnosis→repair; README
  Debugging retains the environment quick reference. Onboarding now documents
  both a local checkout/Cargo path and a standalone archive plus packed package
  path, with repository-only example imports called out.
- The documentation index is linked from the root workspace map and contributor
  guide. The contributor guide records toolchain, verification ladder,
  conventions, evidence, and architecture wayfinding.
- The property safety net checks tree reachability, parent-chain acyclicity,
  recursive text content, live renderer side maps, and untouched rich-text
  cache entries after every generated patch. It covers **32 seeds × 64 valid
  create/update/delete/move operations** and retains seed zero as a named
  regression. The round found and fixed empty created-Text content,
  listener-bearing Text validation, deleted animation/style state, and deleted
  renderer side-map state; the focused unmount blur/restore and delete cleanup
  regression is now green.

## 2. Fresh five-gate evidence

All times below are fresh wall-clock `real` values from `/usr/bin/time -p` on
`61f82ee`. The commands ran serially and exclusively in this order:
`make ci`, `make embedded-bun`, `make host-candidate-smoke`,
`make host-embedded-candidate-smoke`, and `make bun-pack-smoke`. Every command
exited 0. Candidate timeout lines are expected successful rehearsal behavior.

| Command | Exit | Real time | Counts / fresh observed evidence |
| --- | ---: | ---: | --- |
| `make ci` | 0 | **45.10 s** | Rust **130 passed / 0 failed**; Core Bun **131 pass / 0 fail / 90,547 expect() calls** across 17 files; dev Bun **24 pass / 0 fail / 56 expect() calls** across 4 files; formatting, checks, builds, typechecks, package consumer smoke, property tests, and performance guards passed. |
| `make embedded-bun` | 0 | **1.70 s** | `embedded_examples` **2 passed**; embedded counter **1 passed**; locked checks passed. |
| `make host-candidate-smoke` | 0 | **33.79 s** | Identical archive SHA-256 **`1e27dbf0bc579449c332566e357b1ee12a2d921f188d33a390088011a6746a88`** matched on both archives; archive `react-gpui-host-0.2.0-aarch64-apple-darwin.tar.gz`; snapshot commit **312 bytes**; expected process timeouts **5.017 s/5.016 s**; version/help passed. |
| `make host-embedded-candidate-smoke` | 0 | **19.67 s** | Release embedded binary; `--smoke-press` reported `sent=true, commits=2, status=None`; expected timeout **4.584 s**; version/help passed. |
| `make bun-pack-smoke` | 0 | **2.10 s** | Frozen installs, both 0.2.0 JS/type builds and tarballs, external consumer install (**59 packages**), and `package tarball consumer smoke passed`. |

Measured sequential five-gate wall-time sum: **102.36 s**. Candidate timeout
exits are expected behavior, not gate failures. The archive SHA is the
process-candidate check's identical pair of SHA-256 values.

## 3. Quality evidence matrix

| Area | Current evidence |
| --- | --- |
| Rust workspace | Fresh `make ci`: **130 tests / 0 failures** across the workspace, including tree-property **2/2**; `make embedded-bun`: embedded startup matrix **2/2** and embedded counter **1/1**. |
| TypeScript renderer | Fresh `make ci`: **131/131 tests**, **90,547 assertions**, 0 failures; snapshot encode/wire-size guard and malformed-source diagnostics passed. |
| Development package | Fresh `make ci`: **24/24 tests**, **56 assertions**, expected malformed-source diagnostics contained and last-good tree preserved. |
| Fuzz / malformed input | Deterministic protocol fuzz coverage is **35/35 command seeds and 23/23 event seeds** in both Rust and TypeScript records; fresh `make ci` remained green. |
| Property-test safety net | Seed-zero regression plus **32 seeds × 64** valid create/update/delete/move operations passed, checking tree, text-content, renderer side-map, and rich-cache invariants after every patch. |
| Performance smoke | Fresh Rust perf suites, TextInput typing-performance guard, TypeScript event-storm scenarios, and snapshot encode/wire-size guard passed. |
| Golden vectors | Rust golden and TypeScript protocol-golden checks passed; current vectors cover VirtualList offset, selectable rich Text, focusable Text-run, pointer-move, and window controls. |
| API surface locks | Checked-in name/kind fixtures lock **core 114 / dev 20** exports. |
| Release packaging | Process and embedded candidate scripts, deterministic archive checks, package tarball consumer smoke, and embedded package gate passed. Checksums establish reproducibility, not publisher authenticity. |

## 4. Consistency and crosswalk audit

### (a) Current Unreleased inventory

The current `CHANGELOG.md` Unreleased section contains **3 Added capability
entries, 1 Testing entry, 3 Fixed entries, and 0 Changed entries**. Counting
the property-test safety net with additions gives the release-facing total **4
additions / 3 fixes / 0 changes**. The Testing entry is the deterministic tree
patch property suite. The three Fixed entries cover actionable diagnostics,
affected-aware reconciliation, and the property-test findings. No historical
top-level bullets remain in this freshly cut Unreleased section.

### (b) Refresh-delta commit crosswalk

`git log --oneline 5db2ba4..HEAD` reports **9 refresh-delta commits**:

| Commit | Reachable evidence |
| --- | --- |
| `b49330e` | Added the TypeScript snapshot encode-performance and wire-size guard across 1k/5k/20k-node trees. |
| `80bb8c3` | Recorded onboarding dogfood findings and corrected the local/standalone consumer path. |
| `f91def6` | Made validation and command failures actionable with affected node/command context and corrective actions. |
| `d411b69` | Verified the standalone distribution pair: packed core tarball plus extracted process host and external app smoke. |
| `6c00923` | Added the contributor guide with toolchain, verification ladder, conventions, and architecture wayfinding. |
| `ec3b253` | Applied the Rust formatting follow-up after the diagnostics changes. |
| `5ed50c3` | Expanded the multi-surface example with shared state, activation/focus routing, and dirty close confirmation. |
| `19ddeab` | Added the documentation reading-order index and linked it from the workspace map and contributor guide. |
| `61f82ee` | Added deterministic tree-patch property tests plus fixes for empty Text content, listener-bearing Text validation, and deleted renderer side-map/animation state. |

### (c) Documentation and upstream alignment

The verified distribution pair, onboarding dogfood verdict, actionable
error-message pass, contributor guide, and documentation index are linked from
the current README/CONTRIBUTING map. ADR-0008 remains aligned with
Error-Boundary ownership for render failures, fail-fast protocol/tree rejection,
local image fallback, and host-fatal GPUI paint panic. The GPUI drift verdict is
**no released unblocks** for per-run typography, explicit direction, letter
spacing, secure obscuring, live regions, runtime position setters, or portable
Linux image seams.

## 5. Known limits and human decision items

The implementation backlog for the current 0.2.0 contract remains empty. These
are explicit acceptance boundaries or human decisions, not hidden TODOs:

- Issue 01 remains `needs-triage`: the candidate is unsigned and not notarized;
  checksums do not authenticate the publisher.
- Issue 02 remains `ready-for-human`: `block 0.1.6` emits a future-
  incompatibility warning and no unreviewed fork should be patched automatically.
- Issue 03 remains `ready-for-human`: embedded source build is separate,
  green, expensive, and outside ordinary `make ci`.
- Issue 04 remains `needs-triage`: display-backed Quartz/native behavior needs a
  real display/hardware runner; headless evidence cannot replace it.
- Issue 05 remains `needs-triage`: malformed-input coverage is implemented and
  green; long-term fuzz maintenance ownership is human-owned.
- Issue 06 remains `needs-triage`: Linux/Windows workflow is scaffolded at
  `.github/workflows/cross-platform.yml`, but runner validation is not claimed
  until push/PR jobs execute.
- Upstream gaps remain explicit: secure/password display, RTL/container text
  direction and bidi-aware hit/caret/IME semantics, `letterSpacing`, and
  JavaScript Image `onError`. Native TextInput undo/redo is host-owned and
  implemented; `fallbackSource` remains the visual Image degradation path.

## 6. Post-release cycle position

**v0.2.0 cut locally + tagged; publication (push/npm/signing) pending human
action.** Tag `v0.2.0` remains anchored to release commit `72a520c`; no tag move
or publication operation is part of this refresh. The candidate archive SHA is
**`1e27dbf0bc579449c332566e357b1ee12a2d921f188d33a390088011a6746a88`**.

The current cycle contains **3 Added capability entries, 1 Testing entry, 3
Fixed entries, and 0 Changed entries**; the release-facing count is **4
additions / 3 fixes / 0 changes**. Property testing covers **32 seeds × 64
operations**, including the named seed-zero regression, while all five fresh
gates passed serially under `/usr/bin/time -p`. The companion checklist records
this same gate matrix, archive checksum, crosswalk, and human publication
boundaries.

No publication option is selected by this inventory; human action is required
for push, npm publication, signing/notarization, and any Rust crate release.
