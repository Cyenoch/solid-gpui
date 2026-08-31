# Release-readiness inventory

Generated: 2026-08-31T08:46:53+0800
Decision owner: human release owner  
Current HEAD: `dd32a70` (`build(release): third-party notices bundle in host archive`)

**Decision queue:** [`DECISION-QUEUE.md`](DECISION-QUEUE.md) is the user's
one-pager and the primary human-action reference. D2 (legal/artifact
confirmation) is the critical path; D1's upstream drafts are ready to post.

This is an evidence package, not a release approval. It records the post-release
cycle position, fresh command runs, current contract coverage, documentation
consistency, known boundaries, and inputs for the human publication decision.
Version `0.2.0` was cut and tagged locally; no artifact has been published or
pushed.

## Executive readout

- **STOP — publication remains blocked, but narrowed materially.** Option 3′ is
  executed: GPL is eliminated from the distributed host graph by the
  independently authored local no-op stub. Terms for `gpui_shared_string` and
  `gpui_util` are **identified as Apache-2.0 by each crate's explicit
  `LICENSE-APACHE` evidence**, while Cargo license metadata remains missing.
  The generated `THIRD-PARTY-NOTICES.md` inventory is shipped beside `LICENSE`
  in the host archive. STOP now rests only on D2 legal confirmation plus
  remaining standard obligations: `self_cell`, weak-copyleft/file-scope
  notices, and exact artifact-level review before push, registry publication,
  signing, notarization, or any other external distribution. This is a legal
  decision, not an engineering waiver.
- The option matrix is now: **Option 1 (feature off) evidence-dead**; **Option 2
  (pin bump) evidence-dead** at observed `origin/main`; **Option 3′ executed**
  and reversible, with the README documenting deletion of the local patch and
  stub to restore the upstream dependency; **Option 4 (accept GPL terms)
  legal-only** and not selected. The GPL trio is absent from the resolved host
  graph; remaining legal/artifact obligations stay open.
- All five final gates passed at exit code 0 on candidate `dd32a70`, serially
  and exclusively under `/usr/bin/time -p`: `make ci`, `make embedded-bun`,
  `make host-candidate-smoke`, `make host-embedded-candidate-smoke`, and
  `make bun-pack-smoke`.
- The current candidate is `0.2.0` on macOS ARM. Release scripts prove archive
  consistency and CLI/runtime behavior, not display-backed GUI behavior. Both
  process and embedded candidate rehearsals passed their expected timeout,
  help, version, and commit/press checks.
- The current `CHANGELOG.md` Unreleased section contains **7 Added capability,
  6 Testing, 5 Fixed, and 3 Changed entries**. Counting Added plus Testing as
  release-facing additions gives **13 additions / 5 fixes / 3 changes**.
- `git log --oneline bb31de6..dd32a70` reports **5 delta commits**. The
  crosswalk below records all five commits and their reachable evidence.
- The implementation backlog for the current 0.2.0 contract remains empty;
  publication is nevertheless stopped by the narrowed D2 legal/artifact
  blocker.

## Current position

> **STOP (narrowed) — LICENSE BLOCKER:** Do not push, publish, sign, notarize, or
> otherwise distribute this cut until D2 obtains written legal/artifact
> confirmation for the exact distributed versions and artifacts. GPL is
> eliminated from the distributed host graph by Option 3′, and
> `gpui_shared_string`/`gpui_util` terms are identified as Apache-2.0 by
> explicit per-crate `LICENSE-APACHE` evidence. The remaining standard
> obligations are `self_cell`'s permitted option, weak-copyleft/file-scope
> notices, and exact artifact-level review. Publication is a legal decision,
> not an engineering waiver.

The decision evidence is bounded: **Options 1 and 2 are evidence-dead**
(feature-off and the observed upstream pin bump do not clear the findings);
**Option 3′ is executed** by the local independently authored Apache-2.0
identity proc-macro stub, which removes the GPL trio from the resolved host
graph while preserving the documented no-op attribute semantics; and **Option 4
is legal-only**, preserving upstream code while requiring written treatment of
GPL terms and artifact obligations. The patch is reversible, and the stub README
documents how to restore the upstream dependency. The remaining license and
artifact review items are listed in [issue 01](../license-audit/issues/01-gpl-and-unknown-host-licenses.md).

The `v0.2.0` cut remains local and tagged: annotated tag `v0.2.0` still points
to release commit `72a520c`; no tag move or publication operation is part of
this refresh. The candidate archive was rebuilt by `make host-candidate-smoke`
and includes `THIRD-PARTY-NOTICES.md` next to `LICENSE`; its identical SHA-256
pair is represented in the gate table below.

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
  procedures, packaging boundaries, and the no-publication status.
- `THIRD-PARTY-NOTICES.md` inventories **659 Rust third-party packages / 1,152
  package-license records** and **76 Bun dependency records**; the host archive
  carries it beside `LICENSE`, and `make audit` compares a freshly generated
  inventory with the committed file.

## 2. Fresh five-gate evidence

All five commands exited 0 and ran serially and exclusively under
`/usr/bin/time -p` at candidate HEAD `dd32a70`, in this order: `make ci`,
`make embedded-bun`, `make host-candidate-smoke`,
`make host-embedded-candidate-smoke`, and `make bun-pack-smoke`. Candidate
timeout exits are expected successful rehearsal behavior.

| Command | Exit | Real time | Counts / fresh observed evidence |
| --- | ---: | ---: | --- |
| `make ci` | 0 | **58.40 s** | Rust workspace green; Core Bun **132 pass / 0 fail / 453,263 expect() calls**; dev Bun **24 pass / 0 fail / 56 expect() calls**; formatting, checks, builds, typechecks, package smoke, both Bun audits (**12** and **59** packages, no vulnerabilities), cargo-deny (`advisories ok`), and the committed `THIRD-PARTY-NOTICES.md` staleness `cmp` guard passed. |
| `make embedded-bun` | 0 | **1.73 s** | `embedded_examples` **2 passed**; embedded counter **1 passed**; locked checks passed. |
| `make host-candidate-smoke` | 0 | **11.65 s** | Identical archive SHA-256 **`9b1760cbea118a4c2053c9074b84bd03f0f1efac7c3767f7964d8584192464b1`** matched on both archives; archive includes `THIRD-PARTY-NOTICES.md` and `SHA256SUMS`; 312-byte snapshot; expected process timeouts **5.012 s / 5.007 s**; version/help passed. |
| `make host-embedded-candidate-smoke` | 0 | **5.02 s** | Release embedded binary; `--smoke-press` reported `sent=true, commits=2, status=None`; expected timeout **4.575 s**; version/help passed. |
| `make bun-pack-smoke` | 0 | **2.03 s** | Frozen installs, both 0.2.0 JS/type builds and tarballs, external consumer install (**59 packages**), and `package tarball consumer smoke passed`. |

Measured sequential five-gate wall-time sum: **78.83 s**. Candidate timeout
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
| Release packaging | Process and embedded candidate scripts, deterministic archive checks, package tarball consumer smoke, and embedded package gate passed. The archive includes the notices inventory; checksums establish reproducibility, not publisher authenticity. |

## 4. Consistency and crosswalk audit

### (a) Current Unreleased inventory

The current `CHANGELOG.md` Unreleased section contains **7 Added capability,
6 Testing, 5 Fixed, and 3 Changed entries**. Counting all Added and Testing
bullets as release-facing additions gives **13 additions / 5 fixes / 3 changes**.
The seventh Added bullet is the generated notices inventory; the five Fixed
entries include truthful renderer-version metadata wiring and root Bun test
discovery scoping, in addition to diagnostics and patch/state cleanup. The
three Changed entries are `PointerAction` removal, the `ttf-parser` to `skrifa`
migration, and the executed Option 3′ stub mitigation.

### (b) Refresh-delta commit crosswalk from `bb31de6`

`git log --oneline bb31de6..dd32a70` reports **5 commits**:

| Commit | Reachable evidence |
| --- | --- |
| `54c5993` | Ready-to-post upstream issue drafts in `.scratch/license-audit/upstream/01-license-fields.md` and `02-apache-gpui-gpl-ztracing.md`. |
| `ddd0e68` | User one-pager decision queue in `.scratch/release-productionization/DECISION-QUEUE.md`, with D2 first and D1 ready to post. |
| `f90681c` | Evidence-index drift corrections: tracked-file totals/area rows and corrected license/release evidence summaries. |
| `093b32a` | Terms-determinability recheck in `.scratch/license-audit/recheck-2026-08-31.md`, issue-01 wording, and synchronized upstream-draft/queue facts. |
| `dd32a70` | Generated `THIRD-PARTY-NOTICES.md`, archive inclusion/checksum wiring, candidate notices assertion, `make audit` staleness guard, and the Added changelog bullet. |

### (c) Documentation, capability, and upstream alignment

The upstream drafts are ready to post and ask neutral maintainer questions;
the decision queue is the user's one-pager and puts D2 first. The terms
recheck establishes that the two missing Cargo fields do not mean unknowable
terms: each crate's explicit per-crate `LICENSE-APACHE` evidence identifies
Apache-2.0. The notices bundle now ships in the host archive and its freshness
is enforced by the `make audit` staleness guard. The GPL trio remains absent
from the distributed graph by executed Option 3′; the only release STOP is D2
legal confirmation plus the remaining standard obligations.

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

The implementation backlog for the current 0.2.0 contract remains empty, but
the narrowed D2 legal/artifact blocker is release-critical. The release owner
and legal reviewer must confirm that the explicit Apache-2.0 marking in the
`gpui_shared_string`/`gpui_util` `LICENSE-APACHE` files governs the exact
versions and distributed artifacts, confirm `self_cell`'s permitted Apache
option, resolve weak-copyleft/file-scope notices and obligations, and complete
an artifact-level legal review before publication. Do not remove the STOP until
the written determination and artifact-level evidence are attached to issue 01.

The current engineering decision evidence is precise: Option 1 (feature off)
and Option 2 (pin bump to observed `origin/main`) are evidence-dead; Option 3′
is executed and reversible, with an independently authored Apache-2.0 no-op
stub eliminating the GPL trio from the distributed host graph; Option 4 is
legal-only, with no code clearance. The informational license check remains
red under the strict empty allow-list policy: GPL package rejection blocks are
**3 → 0** after the stub, while the two no-license-field records remain
metadata findings despite determinable Apache-2.0 file terms.

Issue 01 remains the active legal blocker; issue 02 remains future-incompatibility
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

**STOP (narrowed) — v0.2.0 is cut locally and tagged, but publication remains
blocked by D2's legal/artifact decision.** Push, npm publication, signing,
notarization, and any other distribution remain prohibited until D2 resolves
the exact Apache-marked crate treatment, `self_cell`, weak-copyleft notices,
and artifact review. GPL is eliminated from the distributed host graph; the
notices inventory is shipped and guarded for staleness, but it is an inventory
rather than legal clearance. The tag remains anchored to release commit
`72a520c`; no tag move or publication operation is part of this refresh. The
candidate archive SHA is **`9b1760cbea118a4c2053c9074b84bd03f0f1efac7c3767f7964d8584192464b1`**.

The current cycle contains **7 Added, 6 Testing, 5 Fixed, and 3 Changed**
entries; its release-facing count is **13 additions / 5 fixes / 3 changes**.
All five final gates passed serially under `/usr/bin/time -p` on `dd32a70`; the
gate matrix above records the exact 58.40/1.73/11.65/5.02/2.03-second runs.
The companion checklist records the same gate matrix, archive checksum,
crosswalk, license status, and human publication boundaries.

No publication option is selected by this inventory. Human action is required
for D2's legal/artifact confirmation first; D1's upstream drafts are ready to
post, then push, npm publication, signing/notarization, and any Rust crate
release remain separate human decisions.
