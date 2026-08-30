# Release-readiness inventory

Generated: 2026-08-30T10:17:41+0800
Decision owner: human release owner  
Current HEAD: `a32b53d` (`refactor(api): prune dead alias and document audit follow-ups`)

This is an evidence package, not a release approval. It records the post-release
cycle position, fresh command runs, current contract coverage, documentation
consistency, known boundaries, and inputs for the human publication decision.
Version `0.2.0` was cut and tagged locally; no artifact has been published or
pushed.

## Executive readout

- All five fresh gates passed at exit code 0 on `a32b53d`, serially and
  exclusively under `/usr/bin/time -p`: `make ci`, `make embedded-bun`,
  `make host-candidate-smoke`, `make host-embedded-candidate-smoke`, and
  `make bun-pack-smoke`.
- The current candidate is `0.2.0` on macOS ARM. Release scripts prove archive
  consistency and CLI/runtime behavior, not display-backed GUI behavior. Both
  process and embedded candidate rehearsals passed their expected timeout,
  help, version, and commit/press checks.
- The final `CHANGELOG.md` Unreleased section contains **3 Added capability,
  4 Testing, 3 Fixed, and 1 Changed entries**. These are exact counts from the
  file at this HEAD; the four new Testing entries are not misplaced top-level
  bullets. Counting Added plus Testing as release-facing additions gives
  **7 additions / 3 fixes / 1 change**.
- `git log --oneline 794183d..HEAD` reports **5 delta commits**. The crosswalk
  below records the surface lifecycle property invariants, renderer soak,
  TypeScript commit-emission property test, export audit, and API cleanup.
- The implementation backlog for the current 0.2.0 contract remains empty.
  Current capability evidence includes deterministic protocol fuzz coverage,
  the verified distribution pair, onboarding dogfood's corrected path,
  actionable error messages, the contributor guide, documentation index,
  multi-surface examples, and the new property/soak safety nets.

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
  restores a live focus target; disabled controls are skipped and per-window
  isolation is preserved.
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
  Final checked-in API locks are **core 115 / dev 20** name-and-kind exports.
- The troubleshooting guide provides symptom-to-diagnosis-to-repair;
  onboarding documents both local checkout/Cargo and standalone archive plus
  packed-package paths, with repository-only example imports called out.
- The documentation index is linked from the root workspace map and contributor
  guide. The contributor guide records toolchain, verification ladder,
  conventions, evidence, and architecture wayfinding.
- The property safety net checks tree reachability, parent-chain acyclicity,
  recursive text content, live renderer side maps, and untouched rich-text
  cache entries after every generated patch. It covers **32 seeds x 64 valid
  create/update/delete/move operations** and retains seed zero as a named
  regression.

### New evidence notes

- **Surface property invariants (`be5fe29`):** 16 seeds x 48 operations cover
  fresh and retired-ID reopen, closure bookkeeping, cross-surface
  focus/activation, live-root side-map ownership, and surface/epoch mismatch
  rejection. The seed-zero regression and seeded sequence both passed (2
  property tests); no epoch/reuse violation was observed.
- **Renderer soak plateau (`486fb71`):** the CI test performs 2,000
  deterministic patch/draw iterations over a fixed 500-node rich-text,
  TextInput, and VirtualList tree. Side maps stayed constant, all IDs remained
  live, and RSS rose only 608 KiB after allocator/GPUI warm-up. The measured
  shape is **plateau/flat**, not an unbounded leak; the run took 16.15 s and
  stayed below its 50 MiB finding threshold.
- **TypeScript emission property (`60b3ddf`):** 24 seeds x 40 randomized
  mutations (960 post-bootstrap commits plus 24 snapshots) exercised real
  `createRoot().render()` commits, frame decode/encode round trips, node graph,
  rich-text, listener, parent/index, and revision invariants. The targeted run
  passed with **362,716 assertions in 1.75 s**; no violation or source fix was
  needed.
- **Export audit baseline and follow-ups (`f0e725b`, `a32b53d`):** the audit
  baseline at `486fb71` recorded core 116/dev 20 exports, with Tier 1 strict
  counts core 41 used/10 docs-only/65 orphan and dev 9/11/0; Tier 2 promoted
  structural consumers to core 111 used/2 docs-only/3 orphan and dev 16/4/0.
  API cleanup removed the one true dead `PointerAction` alias, leaving final
  fixtures core 115/dev 20, and expanded docs for `createStyleSheet`,
  `Position`, transport termination details, root options, notification
  responses, and transition payloads. `AnimationCompleteEvent` and
  `TransportTerminationDetails` remain explicitly retained future-surface
  follow-ups.

## 2. Fresh five-gate evidence

All times below are fresh wall-clock `real` values from `/usr/bin/time -p` on
`a32b53d`. The commands ran serially and exclusively in this order:
`make ci`, `make embedded-bun`, `make host-candidate-smoke`,
`make host-embedded-candidate-smoke`, and `make bun-pack-smoke`. Every command
exited 0. Candidate timeout lines are expected successful rehearsal behavior.

| Command | Exit | Real time | Counts / fresh observed evidence |
| --- | ---: | ---: | --- |
| `make ci` | 0 | **76.89 s** | Rust workspace **216 passed / 0 failed**; Core Bun **132 pass / 0 fail / 453,263 expect() calls**; dev Bun **24 pass / 0 fail / 56 expect() calls**; formatting, checks, builds, typechecks, package smoke, property, soak, and performance guards passed. |
| `make embedded-bun` | 0 | **1.84 s** | `embedded_examples` **2 passed**; embedded counter **1 passed**; locked checks passed. |
| `make host-candidate-smoke` | 0 | **35.48 s** | Identical archive SHA-256 **`6c4a83aea2b6fcdfe8d28ac464f7d681bbff341eee1c6f38cdfbd081425c8b87`** matched on both archives; archive `react-gpui-host-0.2.0-aarch64-apple-darwin.tar.gz`; snapshot commit **312 bytes**; expected process timeouts **5.009 s/5.015 s**; version/help passed. |
| `make host-embedded-candidate-smoke` | 0 | **20.56 s** | Release embedded binary; `--smoke-press` reported `sent=true, commits=2, status=None`; expected timeout **4.534 s**; version/help passed. |
| `make bun-pack-smoke` | 0 | **2.10 s** | Frozen installs, both 0.2.0 JS/type builds and tarballs, external consumer install (**59 packages**), and `package tarball consumer smoke passed`. |

Measured sequential five-gate wall-time sum: **136.87 s**. Candidate timeout
exits are expected behavior, not gate failures. The archive SHA is the
process-candidate check's identical pair of SHA-256 values.

## 3. Quality evidence matrix

| Area | Current evidence |
| --- | --- |
| Rust workspace | Fresh `make ci`: **216 tests / 0 failures** across workspace suites, including surface property **2/2**, tree property **2/2**, and renderer soak **1/1**; `make embedded-bun`: startup matrix **2/2** and embedded counter **1/1**. |
| TypeScript renderer | Fresh `make ci`: **132/132 tests**, **453,263 assertions**, 0 failures; snapshot encode/wire-size guard and malformed-source diagnostics passed. |
| Development package | Fresh `make ci`: **24/24 tests**, **56 assertions**, expected malformed-source diagnostics contained and last-good tree preserved. |
| Fuzz / malformed input | Deterministic protocol fuzz coverage is **35/35 command seeds and 23/23 event seeds** in both Rust and TypeScript records; fresh `make ci` remained green. |
| Property and soak safety nets | Tree **32 x 64**, surface **16 x 48**, TS emission **24 x 40**, and soak **2,000 iterations** passed with their documented invariants and plateau verdict. |
| Performance smoke | Fresh Rust perf suites, TextInput typing-performance guard, TypeScript event-storm scenarios, snapshot encode/wire-size guard, and bounded soak passed. |
| Golden vectors | Rust golden and TypeScript protocol-golden checks passed; current vectors cover VirtualList offset, selectable rich Text, focusable Text-run, pointer-move, and window controls. |
| API surface locks | Checked-in name/kind fixtures lock **core 115 / dev 20** exports after `PointerAction` removal. |
| Release packaging | Process and embedded candidate scripts, deterministic archive checks, package tarball consumer smoke, and embedded package gate passed. Checksums establish reproducibility, not publisher authenticity. |

## 4. Consistency and crosswalk audit

### (a) Current Unreleased inventory

The current `CHANGELOG.md` Unreleased section contains **3 Added capability,
4 Testing, 3 Fixed, and 1 Changed entries**. Counting all Added and Testing
bullets as release-facing additions gives **7 additions / 3 fixes / 1 change**.
The four Testing entries are placed under `### Testing`: tree patch property,
renderer soak, surface lifecycle property, and TypeScript commit-emission
property. The Changed entry is the `PointerAction` removal. No historical
top-level bullets remain in the freshly cut Unreleased section.

### (b) Refresh-delta commit crosswalk from `794183d`

`git log --oneline 794183d..HEAD` reports **5 commits**:

| Commit | Reachable evidence |
| --- | --- |
| `be5fe29` | Added deterministic surface lifecycle property invariants: fresh/reused IDs, closure, focus/activation, and mismatch routing. |
| `486fb71` | Added the CI renderer soak with bounded side maps/history/sequence and RSS sampling; verdict is plateau after warm-up. |
| `60b3ddf` | Added deterministic TypeScript commit-emission property coverage with wire round trips and node/listener/revision invariants. |
| `f0e725b` | Added the public TypeScript surface usage audit and corrected the getting-started accessibility wording; audit baseline is `486fb71`. |
| `a32b53d` | Removed dead `PointerAction`, updated the core API fixture, and documented the remaining export-audit follow-ups; added the Unreleased Changed entry. |

### (c) Documentation and upstream alignment

The verified distribution pair, onboarding dogfood verdict, actionable
error-message pass, contributor guide, documentation index, export audit, and
API cleanup follow-ups are linked from the current documentation map. ADR-0008
remains aligned with Error-Boundary ownership for render failures, fail-fast
protocol/tree rejection, local image fallback, and host-fatal GPUI paint panic.
The GPUI drift verdict is **no released unblocks** for per-run typography,
explicit direction, letter spacing, secure obscuring, live regions, runtime
position setters, or portable Linux image seams.

## 5. Known limits and human decision items

The implementation backlog for the current 0.2.0 contract remains empty. These
are explicit acceptance boundaries or human decisions, not hidden TODOs:

- Issue 01 remains `needs-triage`: the candidate is unsigned and not notarized;
  checksums do not authenticate the publisher.
- Issue 02 remains `ready-for-human`: `block 0.1.6` emits a future-
  incompatibility warning and no unreviewed fork should be patched automatically.
- Issue 03 remains `ready-for-human`: embedded source build is separate,
  green, expensive, and outside ordinary `make ci`.
- Issue 04 remains `needs-triage`: display-backed Quartz/native behavior needs
  a real display/hardware runner; headless evidence cannot replace it.
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
**`6c4a83aea2b6fcdfe8d28ac464f7d681bbff341eee1c6f38cdfbd081425c8b87`**.

The current cycle contains **3 Added, 4 Testing, 3 Fixed, and 1 Changed**
entries; its release-facing count is **7 additions / 3 fixes / 1 change**.
All five fresh gates passed serially under `/usr/bin/time -p` on `a32b53d`, and
the companion checklist records the same gate matrix, archive checksum,
capability notes, crosswalk, and human publication boundaries.

No publication option is selected by this inventory; human action is required
for push, npm publication, signing/notarization, and any Rust crate release.
