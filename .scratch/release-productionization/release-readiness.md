# Release-readiness inventory

Generated: 2026-08-30T12:26:13+0800
Decision owner: human release owner  
Current HEAD: `d74467e` (`build(ci): clarify transitive font advisory`)

This is an evidence package, not a release approval. It records the post-release
cycle position, fresh command runs, current contract coverage, documentation
consistency, known boundaries, and inputs for the human publication decision.
Version `0.2.0` was cut and tagged locally; no artifact has been published or
pushed.

## Executive readout

- All five fresh gates passed at exit code 0 on candidate `d74467e`, serially and
  exclusively under `/usr/bin/time -p`: `make ci`, `make embedded-bun`,
  `make host-candidate-smoke`, `make host-embedded-candidate-smoke`, and
  `make bun-pack-smoke`.
- The current candidate is `0.2.0` on macOS ARM. Release scripts prove archive
  consistency and CLI/runtime behavior, not display-backed GUI behavior. Both
  process and embedded candidate rehearsals passed their expected timeout,
  help, version, and commit/press checks.
- The final `CHANGELOG.md` Unreleased section contains **6 Added capability,
  5 Testing, 3 Fixed, and 2 Changed entries**. Counting Added plus Testing as
  release-facing additions gives **11 additions / 3 fixes / 2 changes**.
- `git log --oneline c7cb1a0..d74467e` reports **5 delta commits**. The
  crosswalk below records the dependency audit sweep, evidence-index refresh,
  advisory gate, skrifa migration, and deny-policy correction.
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
`/usr/bin/time -p` at candidate HEAD `d74467e`, in this order: `make ci`,
`make embedded-bun`, `make host-candidate-smoke`,
`make host-embedded-candidate-smoke`, and `make bun-pack-smoke`. Candidate
timeout exits are expected successful rehearsal behavior.

The `make ci` output now includes the advisory audit step after the existing
Bun gates: each package emits Bun's `No vulnerabilities found (checked N
packages)` result, followed by `==> cargo deny check advisories` and
`advisories ok`. The run also emitted the known unrelated `block v0.1.6`
future-incompatibility warning.

| Command | Exit | Real time | Counts / fresh observed evidence |
| --- | ---: | ---: | --- |
| `make ci` | 0 | **94.81 s** | Rust workspace green; Core Bun **132 pass / 0 fail / 453,263 expect() calls**; dev Bun **24 pass / 0 fail / 56 expect() calls**; formatting, checks, builds, typechecks, package smoke, property, soak, performance guards, both Bun audits (**12** and **59** packages, no vulnerabilities), and cargo-deny (`advisories ok`) passed. |
| `make embedded-bun` | 0 | **4.37 s** | `embedded_examples` **2 passed**; embedded counter **1 passed**; locked checks passed. |
| `make host-candidate-smoke` | 0 | **55.00 s** | Identical archive SHA-256 **`abb0aeb784d99d9761636d128bb104331f934bf1378ae72b31b311d1092ccdef`** matched on both archives; archive `react-gpui-host-0.2.0-aarch64-apple-darwin.tar.gz`; snapshot commit **312 bytes**; expected process timeouts **5.019 s/5.013 s**; version/help passed. |
| `make host-embedded-candidate-smoke` | 0 | **20.26 s** | Release embedded binary; `--smoke-press` reported `sent=true, commits=2, status=None`; expected timeout **4.820 s**; version/help passed. |
| `make bun-pack-smoke` | 0 | **3.57 s** | Frozen installs, both 0.2.0 JS/type builds and tarballs, external consumer install (**59 packages**), and `package tarball consumer smoke passed`. |

Measured sequential five-gate wall-time sum: **178.01 s**. Candidate timeout
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
5 Testing, 3 Fixed, and 2 Changed entries**. Counting all Added and Testing
bullets as release-facing additions gives **11 additions / 3 fixes / 2 changes**.
The six Added entries include the large-tree guard, flagship gallery and
multi-surface work, examples launch smoke, the advisory audit gate, and process
kill resilience. The five Testing entries are tree patch property, renderer
soak, surface lifecycle property, host last-surface lifecycle, and TypeScript
commit-emission property. The three Fixed entries cover diagnostics and patch
reconciliation/state cleanup. The two Changed entries are `PointerAction`
removal and the `ttf-parser` to `skrifa` migration.

The audit sweep commit changed the dev package's Babel dependency and added
supply-chain evidence, but did **not** add a bullet to the current Unreleased
section; the dependency-governance Fixed bullet visible in the changelog is
under the already-cut `0.2.0` section and is not counted here.

### (b) Refresh-delta commit crosswalk from `c7cb1a0`

`git log --oneline c7cb1a0..d74467e` reports **5 commits**:

| Commit | Reachable evidence |
| --- | --- |
| `0b88b2b` | Security audit sweep: Bun advisory scan, `@babel/core` **7.28.4 -> 7.29.6**, and three tracked Rust advisory issues in `.scratch/supply-chain/`; no current Unreleased changelog bullet. |
| `c5cc405` | Refreshed `.scratch/EVIDENCE.md` to index **127 tracked files across 51 evidence areas**; navigation-only documentation change. |
| `f7bc498` | Added strict `make audit` and wired it into `make ci`; current Unreleased **Added** bullet; `deny.toml` acknowledges the three tracked unmaintained advisories. |
| `8626ec2` | Migrated the application-owned font-family parser to `skrifa`; current Unreleased **Changed** bullet; focused malformed/Tuffy tests and supply-chain issue update. |
| `d74467e` | Corrected deny-policy wording to state direct `ttf-parser` elimination while transitive pinned-GPUI/fontdb occurrences remain; no changelog section change. |

### (c) Documentation, capability, and upstream alignment

The verified distribution pair, onboarding dogfood verdict, actionable error-string
alignment, contributor guide, documentation index, export audit, 14/14 examples
launch smoke, kill-resilience zero-orphan matrix, and last-surface lifecycle test
are represented in the current evidence set. The supply-chain path is now
explicitly **scan -> gate -> direct-dependency elimination**: the scan found and
patched the Babel advisory, `make ci` enforces Bun audit and cargo-deny, and
`skrifa` removes the application-owned parser edge. Remaining `ttf-parser`
instances are transitive upstream GPUI/fontdb/rustybuzz paths and remain tracked,
not silently reclassified as resolved.

ADR-0008 remains aligned with Error-Boundary ownership for render failures,
fail-fast protocol/tree rejection, local image fallback, and host-fatal GPUI
paint panic. The GPUI drift verdict is **no released unblocks** for per-run
typography, explicit direction, letter spacing, secure obscuring, live regions,
runtime position setters, or portable Linux image seams.

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

- **Recommendation only:** because Unreleased has matured and includes the
  breaking `PointerAction` removal, the next cut should be **0.3.0**, not
  **0.2.1**, under pre-1.0 SemVer; no version is selected.

## 6. Post-release cycle position

**v0.2.0 cut locally + tagged; publication (push/npm/signing) pending human
action.** Tag `v0.2.0` remains anchored to release commit `72a520c`; no tag move
or publication operation is part of this refresh. The candidate archive SHA is
**`abb0aeb784d99d9761636d128bb104331f934bf1378ae72b31b311d1092ccdef`**.

The current cycle contains **6 Added, 5 Testing, 3 Fixed, and 2 Changed**
entries; its release-facing count is **11 additions / 3 fixes / 2 changes**.
All five fresh gates passed serially under `/usr/bin/time -p` on `d74467e`, and
the companion checklist records the same gate matrix, archive checksum,
capability notes, crosswalk, supply-chain boundary, and human publication
boundaries.

No publication option is selected by this inventory; human action is required
for push, npm publication, signing/notarization, and any Rust crate release.
