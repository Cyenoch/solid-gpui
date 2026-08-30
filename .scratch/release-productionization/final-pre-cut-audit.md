# Final pre-cut audit

Generated: 2026-08-26 (HEAD `c304f29`)

This is an internal cross-cutting audit for the 0.2.0 cut. It is evidence, not
release approval. The human release owner still controls signing/notarization,
registry publication, version/date, tag creation, and the unavailable
`cargo-audit` security scan.

## Verdict matrix

| Area | Result | Evidence |
| --- | --- | --- |
| Change-history completeness | ✓* | 75 commits on `main`; all subjects match Conventional Commits syntax (custom project types are used consistently, and the single `Revert "…"` subject is standard). Recent feature/refactor topics map to implementation plus changelog or an explicit historical/boundary entry; four unreachable loose commits are documented below as superseded alternatives. |
| Documentation reverse verification | ✓ | 20 claims sampled across commands, events, styles, components, hooks, transport, and development helpers; every sampled claim has a current source symbol and matching behavior. |
| Public API/orphan review | ✓ | API snapshots lock core **101** and dev **18** exports. The two newest core exports (`BoxShadow`, `BoxShadowInput`) now have direct consumer documentation; no semantic zero-document/zero-example public API was found. |
| Dead-code/dead-branch review | ✓ | `make ci` Clippy is warning-clean. Production `unreachable!` sites are post-validation invariants; no production TODO/FIXME/dead-code suppression or constant dead branch was found. |
| Gate and workflow alignment | ✓ | Every Makefile target exists; workflow invocations resolve to Makefile targets. `ci`/release workflows cover their intended subsets, while generators, bundle, soak, and leaf gates are explicitly manual or composed. |
| Scratch evidence set | ✓ | Readiness, cut checklist, dependency audit, and upstream tracking note agree on HEAD, 42-slot style, Image fallback, scale factor, pointer/zoom boundaries, current counts, and gate evidence. |
| Final gate evidence | ✓ | Current `make ci` (including Clippy) passed; serial embedded, candidate, package, and both generator checks passed with zero fixture drift. |

## 1. Change history and changelog reconciliation

### History scan

`git log --oneline --all` contains 75 reachable commits. Subject validation used
`^[a-z][a-z0-9-]*(scope)?: description` plus the standard `Revert` form and found
zero nonconforming subjects. The project consistently uses meaningful custom
Conventional Commit types such as `protocol`, `input`, `delivery`,
`dev-toolkit`, `ts-renderer`, and `renderer-core`.

The recent topic chain has no unexplained implementation commit:

- `ee0fa1b` VirtualList implementation → `7ab1da1` migration changelog.
- `629a03b` boxShadow/fontFamily implementation → `a24bd0d` style/structure
  changelog and API snapshot follow-up.
- `2ba4b1b` host/tree/input structural cleanup → its Changed entry in
  `CHANGELOG.md`.
- `0561ceb` Image fallback and scale-factor implementation → `d05171f` Added
  entries.
- `d4b865a` pointer/zoom disposition documentation → existing capability-boundary
  changelog items; it introduces no new runtime feature.
- `c304f29` final audit repairs → this evidence and no runtime behavior change.

The changelog parser finds **91** current Unreleased entries: **82 Added**, **4
Fixed**, and **5 Changed**. The earlier 89-entry inventory was before the two
Image/scale-factor Added entries. Representative entry-to-commit checks cover
VirtualList (`ee0fa1b`/`7ab1da1`), style (`629a03b`/`a24bd0d`), Image and scale
factor (`0561ceb`/`d05171f`), host/tree cleanup (`2ba4b1b`), command round trips
(`4dd0129`/`8f20c7f`), and long-run stability (`c4522a8`).

### Findings fixed during this audit

- `docs/getting-started.md` still said “40 positional slots” after the 42-slot
  style extension. The stale wording was corrected in `d4b865a`.
- ADR-0007 and ADR-0008 still described every Image failure as blank output after
  `fallbackSource` landed. They now describe loading/error fallback and blank
  output only when no fallback is supplied (`c304f29`).
- The new `BoxShadow` and `BoxShadowInput` type exports lacked literal consumer
  API names. Their shape and two-layer union are now stated in the package README
  (`c304f29`).
- `bun-ci` was omitted from `.PHONY`, weakening the Makefile gate declaration.
  It is now listed and remains composed by `ci` (`c304f29`).

### Reachability note

`git fsck --no-reflogs --unreachable` found four unreachable historical commits:
`e560c2a`/`71c3d416` are alternate menu-state drafts superseded by reachable
`a0a9718`, while `ca9bd23`/`a2bca2e` are alternate file-dialog drafts
superseded by reachable `9fab40d`. They have no branch or tag refs and do not
represent unaccounted current-tree work; they are retained as historical loose
objects and were not pruned.

## 2. Reverse verification: 20 documentation claims

Each claim was read from README/getting-started/package docs and checked against
the implementation symbol named in the evidence column.

| # | Documented claim | Implementation evidence |
| ---: | --- | --- |
| 1 | Snapshot bootstrap and Patch commits are atomic | `packages/react-gpui/src/renderer.ts:144-165`; `crates/react-gpui/src/renderer.rs:202-214` |
| 2 | `createSurfaceHost` and `openSurface` route multiple windows | `packages/react-gpui/src/surface-host.ts:15-18,104-116,195-196`; `renderer.ts:133` |
| 3 | File dialogs are asynchronous and cancellation returns `null` | `packages/react-gpui/src/renderer.ts:230-236`; `root-container.ts:428-470` |
| 4 | Notifications and static menus are root-scoped | `renderer.ts:238-244`; `crates/react-gpui/src/renderer/commands.rs:39-42,168-192` |
| 5 | `View`, `Text`, `Pressable`, `Image`, and `VirtualList` are host components | `packages/react-gpui/src/index.ts:119-126,186-188` |
| 6 | Pressable callbacks receive semantic press events | `packages/react-gpui/src/renderer/nodes.ts:183-186`; `renderer/dispatch.ts` press path |
| 7 | Layout callbacks are available on the documented components | `packages/react-gpui/src/renderer/nodes.ts:202-204`; native measurement in `crates/react-gpui/src/renderer/paint.rs` |
| 8 | Drag/drop keeps data in JavaScript and reports type/path payloads | `nodes.ts:195-200`; `renderer/dispatch.ts` drag path; `crates/react-gpui/src/protocol/wire/event.rs:147-164` |
| 9 | Image `fallbackSource` covers loading and failure | `packages/react-gpui/src/renderer/props.ts:189-205`; `crates/react-gpui/src/renderer/paint.rs:519-528` |
| 10 | VirtualList uses persistent native variable-height ListState | `crates/react-gpui/src/renderer.rs:77-80,313-316`; list paint path |
| 11 | Style has 42 slots, with boxShadow and fontFamily tails | `packages/react-gpui/src/style.ts:104-147,631-633`; `packages/react-gpui/README.md:120-124` |
| 12 | BoxShadow supports one/two tagged layers and validation | `style.ts:216-254`; `crates/react-gpui/src/protocol/wire/node.rs:76-82,187-205` |
| 13 | Font-family fallback uses GPUI's resolver | `crates/react-gpui/src/renderer/paint.rs:1124-1127`; `docs/protocol.md:253` |
| 14 | Window resize carries optional scaleFactor and reports scale-only changes | `packages/react-gpui/src/renderer/dispatch.ts:63,109-110`; `crates/react-gpui/src/renderer.rs:417-452` |
| 15 | `createWindowSizeStore`/`useWindowSize` are explicit hooks | `packages/react-gpui/src/hooks.ts:17-43` |
| 16 | TextInput selection preserves UTF-16 reversed orientation | `packages/react-gpui/src/renderer/types.ts:59-75`; `crates/react-gpui/src/renderer/input.rs` |
| 17 | Transition completion is emitted once | `packages/react-gpui/README.md:547-575`; `crates/react-gpui/src/renderer/animation.rs` |
| 18 | Accessibility roles/labels are forwarded with host validation | `packages/react-gpui/src/renderer/props.ts`; `crates/react-gpui/src/tree/validation.rs` |
| 19 | `StdioTransport` observes termination and bounds pending output | `packages/react-gpui/src/transport.ts:111-153,239-264` |
| 20 | Dev `render`/event helpers use real core MemoryTransport frames | `packages/react-gpui-dev/src/testing.ts:249-280`; `packages/react-gpui-dev/README.md:35-73` |

No sampled documentation claim was missing its implementation. The audit also
searched for stale 40-slot and pre-fallback wording; the only remaining 40-slot
reference is the intentional legacy decoder compatibility statement in
`docs/protocol.md`.

## 3. Public API and orphan review

`fixtures/api-surface.core.txt` and `fixtures/api-surface.dev.txt` contain the
current checked-in 101+18 name/kind locks, and `make api-surface-generate`
regenerated both with zero diff. `BoxShadow` and `BoxShadowInput`, the only new
core export names in the latest feature extension, are directly documented in
`packages/react-gpui/README.md` and their `boxShadow` field is demonstrated by
the gallery.

A literal identifier-only scan reports some type aliases without their exact
PascalCase spelling in consumer prose (for example `PressableProps` and
`WindowResizeHandler`). These are not semantic orphans: their component,
callback, and payload contracts are documented and implemented. The dev package
README lists its complete public API explicitly. No exported value or type was
found to have both zero documentation and zero example/usage context.

Production dead-code review found only two `unreachable!` forms, both after
validated enum/command domains (`commands.rs` and `paint.rs`). Test-only
`allow(unused_imports)` is confined to `crates/react-gpui/src/tests/support.rs`.
`make ci` Clippy with `-D warnings` and all tests passed.

## 4. Gate and workflow alignment

Makefile target inventory:

- Composed by `ci`: `rust-format`, `rust-check`, `bun-ci`; `bun-ci` composes
  `bun-format`, `bun-typecheck`, `bun-test`, and `bun-pack-smoke` (which composes
  `bun-build`).
- Workflow-covered independent gates: `embedded-bun`, `host-release-check`,
  `host-candidate-smoke`, `host-embedded-candidate-smoke`, and `release-prep`.
- Explicitly manual/leaf targets: `bun-install`, `bun-format`, `bun-typecheck`,
  `bun-test`, `bun-build`, `bun-pack-smoke`, `protocol-golden-generate`,
  `api-surface-generate`, `host-release-bundle`, and `soak-smoke`.
- `host-release-check` and candidate smoke are composed in the host-release
  workflow; embedded gate and embedded candidate are composed in their workflow.
  The release-prep workflow runs `release-prep`, `ci`, and both package pack
  commands. No workflow invokes a missing Makefile target.

The `.PHONY` inventory now includes every declared target, including `bun-ci`.
README and the package docs explain that golden/API generators, embedded builds,
release bundle, and soak are deliberate explicit/manual gates rather than hidden
parts of ordinary `ci`.

## 5. Scratch four-piece consistency

| Evidence file | Lines | Current consistency |
| --- | ---: | --- |
| `release-readiness.md` | **315** | HEAD `c304f29`; Unreleased 91 (82/4/5); 42-slot style; Image fallback; scale factor; pointer/zoom boundaries; serial 5-gate matrix; API 101+18 |
| `v0.2.0-cut-checklist.md` | **116** | Same HEAD/counts/protocol/style/features; references dependency and upstream notes |
| `dependency-audit.md` | **222** | Current lock/toolchain inventory; latest CI 9.24 s, embedded 3.15 s; explicit cargo-audit unavailable caveat; references the other evidence |
| `upstream-dependencies.md` | **93** | Three classes remain explicit: true upstream gaps, re-reviewable project limits, and test-platform limits; image fallback/scale/pointer/zoom dispositions agree |

The reports agree on the latest audit commit, current style/protocol/API facts,
feature boundaries, and gate totals. The `cargo-audit` absence, unsigned/notarized
artifact, and display-backed Quartz/zoom checks remain explicit human or
platform-owned release boundaries, not hidden green claims.

## 6. Gate evidence index

The serial final evidence used for this audit:

- `make ci`: **PASS**, **9.24 s**, Clippy included; Rust 116 tests, Bun core
  91 tests/54,053 assertions, dev 16 tests/39 assertions.
- `make embedded-bun`: **PASS**, **3.15 s**, embedded counter 1 test.
- `make host-candidate-smoke`: **PASS**, **31.65 s**, deterministic archive SHA
  `ac8f087ffb89f68ac022c3ead414fcc05aab8ebd20c3e1a15cad4fdacee470fc`, snapshot
  312 bytes, expected 5.017/5.015 s timeouts.
- `make host-embedded-candidate-smoke`: **PASS**, **19.52 s**, `sent=true`,
  `commits=2`, expected 5.016 s timeout.
- `make bun-pack-smoke`: **PASS**, **1.97 s**, 59-package external consumer.
- `make protocol-golden-generate`: **PASS**, **0.90 s**, zero fixture diff.
- `make api-surface-generate`: **PASS**, **1.35 s**, zero fixture diff; core
  101/dev 18.

The five-gate sequential sum is **66.91 s**. The latest post-audit `make ci`
rerun is 9.24 s; the remaining gate timings are from the same implementation
tree before docs-only audit repairs and are unaffected by those repairs.

## Final disposition

✓ **Cut-ready from a repository engineering perspective.** No blocking
implementation, documentation, orphan-API, dead-code, Makefile/workflow, or
scratch-evidence defect remains. Release approval itself remains human-owned,
with the explicitly recorded signing/notarization, cargo-audit, display-backed,
platform-target, and publication decisions still outstanding.

## 7. Incremental delta after the 44-round audit

This appendix covers the post-baseline deltas through the selectable-Text
feasibility archive. It is intentionally additive; the original seven-dimension
audit and its four repairs remain unchanged.

| New declaration or boundary | Reverse implementation check | Result |
| --- | --- | --- |
| Keybinding registration: root `setKeybindings`, command 22, full replacement and Event 17 action routing | `packages/react-gpui/src/renderer.ts:139-140,249-251`; `root-container.ts:548-577`; `crates/react-gpui-host/src/main.rs:249-296`; headless `test_support.rs:548-690` | ✓ The command validates all entries before atomically replacing the per-surface set and rebuilding the process-global union. Context-conditional bindings remain explicitly unsupported. |
| Outbound file drag: `draggable.exportFiles`, host-properties tag 4, legacy two-slot decode | `packages/react-gpui/src/renderer/props.ts:240-294`; `crates/react-gpui/src/renderer/paint.rs:372-380`; `docs/protocol.md:175`; `crates/react-gpui/src/renderer/paint.rs` outbound drag helper test | ✓ 1..8 host-local paths are validated and directory metadata is probed lazily. Files payload delivery has no JavaScript completion event; platform availability remains explicit. |
| Multiline TextInput geometry: `shape_text`/`WrappedLine`, UTF-16 line mapping, cross-line bounds, explicit line-height growth | `crates/react-gpui/src/renderer/paint.rs:90-157,192-195`; `crates/react-gpui/src/renderer/input.rs:31-210`; `renderer.rs:983-1029`; wrapped-layout regression tests | ✓ Single and multiline layouts are cached separately; positions map through line starts, cross-line selections union first/last visual-row bounds, and layout height follows wrapped line count. IME candidate placement remains display-backed. |
| Consumer troubleshooting workflow and README Debugging pointer | `docs/troubleshooting.md` (459 lines, 8 symptom entries); `README.md#debugging`; targeted Bun/Rust diagnostic commands | ✓ The guide preserves the environment quick reference while adding symptom → diagnosis → repair paths. |
| Selectable read-only Text feasibility re-review | `.scratch/release-productionization/upstream-dependencies.md:44-55`; `f8a5079` | ✓ Reclassified from hard upstream gap to feasible-bounded host-owned design, backed by GPUI TextRun/TextLayout APIs and Zed Markdown precedent. Wire/state half-product was rolled back; no selectable Text feature is shipped. |

The current checked-in evidence remains coherent after this delta: **22
commands / 21 events**, **42 style slots**, API locks **core 102 / dev 18**, and
**96 Unreleased entries (87 Added / 4 Fixed / 5 Changed)**. Readiness, checklist,
and dependency reports now name HEAD `5ca34e1`; the upstream boundary note carries
the current feature and deferred-design classification.

### Current serial gate refresh

The current-tree serial run after the selectable-Text rollback was:

- `make ci`: PASS, **20.35 s**, Clippy included; Rust **120** tests, Bun core
  **92 / 54,065 assertions**, dev **16 / 39 assertions**.
- `make embedded-bun`: PASS, **3.86 s**, embedded counter **1**.
- `make host-candidate-smoke`: PASS, **36.12 s**, deterministic SHA
  `ee823bc289e8b5f309ba119297c460c6e158909c377af8695df01c57b8146f63`,
  Snapshot 312 bytes.
- `make host-embedded-candidate-smoke`: PASS, **22.09 s**, `sent=true`,
  `commits=1`.
- `make bun-pack-smoke`: PASS, **2.25 s**, consumer 59 packages.
- `make protocol-golden-generate`: PASS, **1.62 s**, zero fixture drift.
- `make api-surface-generate`: PASS, **1.54 s**, zero fixture drift, core
  **102** / dev **18**.

The five-gate sequential sum is **84.67 s**. Selectable Text remains a
feasible-bounded design only; its half-complete wire/state edits were restored
to the last clean implementation tree before this evidence run.
