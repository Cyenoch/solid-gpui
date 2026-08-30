# Scratch evidence index

This index answers “where is the proof?” for the tracked `.scratch` evidence
set at the `v0.2.0` cut. The prior index covered **135 tracked files** across
The current inventory contains **149 evidence files
plus this index**, for **150 tracked files across 58 evidence areas**.
The index intentionally does not copy full evidence; each row points to the
source snapshot and gives the one-line number or verdict worth finding six
months from now.

`historical (stale-ok)` means a dated measurement, audit, rehearsal, or design
record that remains useful as provenance but is not a live status source.
`living` means a release/readiness reference expected to be consulted and
updated at a release boundary. All evidence areas in the current tracked
inventory are landed; no in-flight area is listed.

Refresh date: **2026-08-31**.

## File-count summary

| Area | Before refresh | Added since prior index | Final indexed |
| --- | ---: | ---: | ---: |
| a11y-second-pass | 6 | 0 | 6 |
| animation-completeness | 3 | 0 | 3 |
| audit-wiring | 1 | 0 | 1 |
| caret-follow | 1 | 0 | 1 |
| dist-verify | 1 | 0 | 1 |
| docs-dogfood | 1 | 0 | 1 |
| domain-docs | 1 | 0 | 1 |
| embedded-coverage | 1 | 0 | 1 |
| examples-smoke | 1 | 0 | 1 |
| export-audit | 1 | 0 | 1 |
| file-io | 2 | 0 | 2 |
| focus-order | 3 | 0 | 3 |
| font-loading | 2 | 0 | 2 |
| gate-perf | 1 | 0 | 1 |
| input-perf | 1 | 0 | 1 |
| interaction-completeness | 1 | 0 | 1 |
| interchange | 6 | 0 | 6 |
| kill-resilience | 1 | 0 | 1 |
| last-surface | 1 | 0 | 1 |
| license-audit | 2 | 6 | 8 |
| link-affordance | 2 | 0 | 2 |
| link-keyboard | 1 | 0 | 1 |
| menu-accelerators | 1 | 0 | 1 |
| observability | 3 | 0 | 3 |
| order-sweep | 1 | 0 | 1 |
| perf-event-storm | 8 | 0 | 8 |
| pointer-coords | 1 | 0 | 1 |
| pointer-move | 1 | 0 | 1 |
| prop-test | 1 | 0 | 1 |
| protocol-compatibility | 4 | 0 | 4 |
| protocol-cutover | 1 | 0 | 1 |
| pub-meta | 1 | 0 | 1 |
| rehearsal-2 | 1 | 0 | 1 |
| release-productionization | 7 | 9 | 16 |
| release-rehearsal | 1 | 0 | 1 |
| rich-perf | 3 | 0 | 3 |
| showcase | 1 | 0 | 1 |
| skrifa-spike | 1 | 0 | 1 |
| snapshot-perf | 3 | 0 | 3 |
| soak-test | 1 | 0 | 1 |
| storm-trim | 1 | 0 | 1 |
| supply-chain | 4 | 0 | 4 |
| surface-coverage | 1 | 0 | 1 |
| surface-lifecycle | 6 | 0 | 6 |
| surface-prop | 1 | 0 | 1 |
| test-determinism | 4 | 0 | 4 |
| testing-dx | 3 | 0 | 3 |
| text-input-completeness | 7 | 0 | 7 |
| theme-consistency | 4 | 0 | 4 |
| transport-resilience | 4 | 0 | 4 |
| ts-prop | 1 | 0 | 1 |
| undo-redo | 2 | 0 | 2 |
| upstream-assessment | 1 | 0 | 1 |
| version-const | 1 | 0 | 1 |
| virtual-list-completeness | 7 | 0 | 7 |
| vlist-scroll | 2 | 0 | 2 |
| window-controls | 4 | 0 | 4 |
| window-title | 1 | 0 | 1 |
| top-level `.scratch` | 1 | 0 | 1 |
| **Total** | **135** | **15** | **150** |

## a11y-second-pass (6 files)

| Path | What it proves | Key number / verdict | Freshness |
| --- | --- | --- | --- |
| `.scratch/a11y-second-pass/spec.md` | Accessibility second-pass decisions against pinned GPUI/AccessKit | Expanded and heading level implemented; live regions remain an upstream gap; tuple is `[role,label,description,disabled,checked,selected,value,expanded?,level?]` | historical (stale-ok) |
| `.scratch/a11y-second-pass/issues/01-expanded-and-heading.md` | Expanded-state and heading-level implementation seam | Positive heading-only level and optional expanded tail implemented; old seven-field tuple remains valid | historical (stale-ok) |
| `.scratch/a11y-second-pass/issues/02-live-regions.md` | Live-region API audit | AccessKit has `Live`, but pinned GPUI has no public live builder/write path; no wire field added | historical (stale-ok) |
| `.scratch/a11y-second-pass/issues/03-label-description.md` | Label/description mapping audit | GPUI and renderer set label and description independently for recognized roles | historical (stale-ok) |
| `.scratch/a11y-second-pass/issues/04-validation-parity.md` | TypeScript/Rust accessibility validation parity | Shared optional tail, heading requirement, and positive-u32 level checks; goldens cover both directions | historical (stale-ok) |
| `.scratch/a11y-second-pass/issues/05-roles-table.md` | Public accessibility documentation correction | README now includes Expanded and Level columns and the live-region boundary | historical (stale-ok) |

## animation-completeness (3 files)

| Path | What it proves | Key number / verdict | Freshness |
| --- | --- | --- | --- |
| `.scratch/animation-completeness/spec.md` | Transition property audit and bounded implementation scope | Four supported properties: opacity, backgroundColor, width, height; borderRadius and generic transforms remain unsupported | historical (stale-ok) |
| `.scratch/animation-completeness/issues/01-transition-lifecycle.md` | Removal, retarget, delay, and completion semantics | Omitted transition metadata is reused; retarget samples presentation value; one completion per generation | historical (stale-ok) |
| `.scratch/animation-completeness/issues/02-gpui-animation-surface.md` | GPUI animation primitive feasibility | No native border-radius/transform transition path; no wire growth or per-frame style mutation | historical (stale-ok) |
## audit-wiring (1 file)

| Path | What it proves | Key number / verdict | Freshness |
| --- | --- | --- | --- |
| `.scratch/audit-wiring/notes.md` | Advisory audit gate wiring | `make audit` runs both Bun audits and cargo-deny; Bun reports 12/59 clean and cargo-deny reports `advisories ok` with three documented unmaintained-crate exceptions | historical (stale-ok) |


## caret-follow (1 file)

| Path | What it proves | Key number / verdict | Freshness |
| --- | --- | --- | --- |
| `.scratch/caret-follow/reproduction.md` | Multiline TextInput caret visibility regression | Six lines measured 156 px in a 40 px viewport; bounded scroll offset keeps end caret in bounds | historical (stale-ok) |

## dist-verify (1 file)

| Path | What it proves | Key number / verdict | Freshness |
| --- | --- | --- | --- |
| `.scratch/dist-verify/notes.md` | Standalone release archive and external package verification | **PASS** for archive host + packed core tarball + external app; protocol-v3 startup, snapshot exchange, help/version, and reproducibility checks passed | historical (stale-ok) |

## docs-dogfood (1 file)

| Path | What it proves | Key number / verdict | Freshness |
| --- | --- | --- | --- |
| `.scratch/docs-dogfood/findings.md` | Fresh-consumer onboarding walkthrough and corrected path | Initial path found five install/host/toolchain/example blockers; corrected tarball + manifest-path flow reached protocol-v3 startup and timed out cleanly | historical (stale-ok) |

## domain-docs (1 file)

| Path | What it proves | Key number / verdict | Freshness |
| --- | --- | --- | --- |
| `.scratch/domain-docs/refresh.md` | Domain vocabulary and ADR refresh | Records 14 new terms and ADR-0009 through ADR-0012; rejected duplicate ADR candidates are named | historical (stale-ok) |

## embedded-coverage (1 file)

| Path | What it proves | Key number / verdict | Freshness |
| --- | --- | --- | --- |
| `.scratch/embedded-coverage/spec.md` | Embedded Bun startup/lifecycle coverage design | Four representative entries run serially; Snapshot, bounded tree, expected text, Fast Refresh, and clean status-0 are required | historical (stale-ok) |

## examples-smoke (1 file)

| Path | What it proves | Key number / verdict | Freshness |
| --- | --- | --- | --- |
| `.scratch/examples-smoke/notes.md` | Release-host launch coverage for every example entry | **14/14 PASS**; every entry emitted protocol-v3 startup, exited with expected SIGTERM, and left no process-group leaks; warm loop 0.579 s | historical (stale-ok) |

## export-audit (1 file)

| Path | What it proves | Key number / verdict | Freshness |
| --- | --- | --- | --- |
| `.scratch/export-audit/2026-08-30.md` | Public TypeScript core/dev export consumer audit | Tier 2 leaves **3 true orphans**; `PointerAction` removed, `AnimationCompleteEvent` and `TransportTerminationDetails` kept as future seams; dev surface has zero orphans | historical (stale-ok) |

## file-io (2 files)

| Path | What it proves | Key number / verdict | Freshness |
| --- | --- | --- | --- |
| `.scratch/file-io/spec.md` | Root text-file command contract | Commands 25/26; asynchronous bounded UTF-8 read/write; frame-safe cap is `MAX_FRAME_SIZE - 1024` | historical (stale-ok) |
| `.scratch/file-io/issues/001-protocol.md` | File command wire additions | Absolute paths, bounded payloads, value tag 6, and allowlists are shared across Rust and TypeScript | historical (stale-ok) |

## focus-order (3 files)

| Path | What it proves | Key number / verdict | Freshness |
| --- | --- | --- | --- |
| `.scratch/focus-order/spec.md` | Keyboard traversal and focused-unmount contract | View, Pressable, TextInput, and selectable Text participate; disabled controls skip; blur precedes restoration | historical (stale-ok) |
| `.scratch/focus-order/issues/01-keyboard-traversal.md` | Native tab-stop correction | GPUI tab stops are explicit; tree-order traversal, wrapping, disabled skipping, and per-window isolation pass | historical (stale-ok) |
| `.scratch/focus-order/issues/02-focused-unmount.md` | Focused-node removal lifecycle | Detached focus emits one terminal blur and restores a live ancestor/target; ordinary detached events remain rejected | historical (stale-ok) |

## font-loading (2 files)

| Path | What it proves | Key number / verdict | Freshness |
| --- | --- | --- | --- |
| `.scratch/font-loading/spec.md` | Runtime custom-font contract | Root `loadFont(path)` returns metadata family; registration must precede first layout; TTF/OTF required | historical (stale-ok) |
| `.scratch/font-loading/issues/01-runtime-command.md` | Command-29 implementation assignment and boundary | Root-only bounded absolute-path loading and GPUI registration are required; status snapshot says claimed | historical (stale-ok) |

## gate-perf (1 file)

| Path | What it proves | Key number / verdict | Freshness |
| --- | --- | --- | --- |
| `.scratch/gate-perf/timing.md` | CI gate timing baseline and outlier explanation | Warm `make ci` samples include 54.14 s and 50.57 s; cold/concurrent timing is explicitly contaminated, not a recipe regression | historical (stale-ok) |

## input-perf (1 file)

| Path | What it proves | Key number / verdict | Freshness |
| --- | --- | --- | --- |
| `.scratch/input-perf/measurement.md` | TextInput typing-performance attribution | 10,000-char single-line p50/p99 is 1.252/1.586 ms; shaping is 99.8% of measured host path; verdict is platform-bound guard-only | historical (stale-ok) |

## interaction-completeness (1 file)

| Path | What it proves | Key number / verdict | Freshness |
| --- | --- | --- | --- |
| `.scratch/interaction-completeness/issues/03-word-line-selection.md` | Double/triple-click TextInput selection semantics | UAX word and logical-line selection, reversed drag expansion, and UTF-16/UTF-8 preservation implemented; focused Rust suite reached 30 unit tests | historical (stale-ok) |

## interchange (6 files)

| Path | What it proves | Key number / verdict | Freshness |
| --- | --- | --- | --- |
| `.scratch/interchange/spec.md` | Cross-boundary data interchange audit | Clipboard images are platform-uneven, file paths stay path-only, text cap is UTF-8 bytes, generated images use host-visible paths | historical (stale-ok) |
| `.scratch/interchange/issues/01-clipboard-images.md` | Native clipboard-image capability audit | macOS/Windows native image paths exist; current X11/Wayland writes are text-only; feature deferred at that snapshot | historical (stale-ok) |
| `.scratch/interchange/issues/02-external-drop-metadata.md` | External file-drop metadata decision | `onExternalFileDrop` remains ordered path-only; size/mtime/content are app-composed | historical (stale-ok) |
| `.scratch/interchange/issues/03-clipboard-events.md` | Clipboard observer audit | GPUI exposes pull reads, not a public cross-platform change observer; keep `getClipboardText()` pull-only | historical (stale-ok) |
| `.scratch/interchange/issues/04-text-clipboard-limit.md` | Symmetric clipboard byte-limit proof | Exactly 1 MiB accepted and one UTF-8 byte over rejected, including CJK and emoji | historical (stale-ok) |
| `.scratch/interchange/issues/05-generated-images.md` | Generated-image ingestion recipe | Persist encoded bytes to an app-owned host-visible temporary/sidecar path; data URLs/base64 remain rejected | historical (stale-ok) |

## kill-resilience (1 file)

| Path | What it proves | Key number / verdict | Freshness |
| --- | --- | --- | --- |
| `.scratch/kill-resilience/notes.md` | Host, renderer, and process-group force-kill cleanup | **6 scenarios, zero orphans**; host-kill stdin EOF, renderer-kill typed signal 9, and group-kill all passed at 0.008–0.058 s latency | historical (stale-ok) |

## last-surface (1 file)

| Path | What it proves | Key number / verdict | Freshness |
| --- | --- | --- | --- |
| `.scratch/last-surface/notes.md` | Final native surface close lifecycle | `cx.quit()` is verified at the headless seam: first close keeps host alive, final close empties registry and requests quit; process exit remains display-backed | historical (stale-ok) |
## license-audit (8 files)

| Path | What it proves | Key number / verdict | Freshness |
| --- | --- | --- | --- |
| `.scratch/license-audit/2026-08-30.md` | Rust/Cargo, host archive, and Bun/npm license compliance audit | **STOP publication**: the default host graph reaches 3 GPL-licensed pinned Zed crates and 2 crates with missing manifest license fields; cargo-deny remains strict/red and no dependency was changed | living |
| `.scratch/license-audit/issues/01-gpl-and-unknown-host-licenses.md` | Host distribution license blocker and release-owner decision | **Stub landed, STOP narrowed**: the GPL trio is eliminated from the distributed host graph; `gpui_shared_string`/`gpui_util`, `self_cell`, weak-copyleft notices, and artifact-level legal review remain | living |
| `.scratch/license-audit/feasibility.md` | Engineering feasibility of four GPL-remediation options | No feature-off or observed-upstream-pin-bump clearance; only a maintained GPUI/`sum_tree` source fork can remove the GPL chain, while missing-license crates and legal review remain | living |
| `.scratch/license-audit/spike-option3.md` | Narrow Option 3 patch-fork proof | **Proven conditionally and narrowly**: a two-crate GPUI/`sum_tree` fork clears 3/3 GPL package IDs in the target no-dev graph, but carries medium/high maintenance cost and leaves license/legal obligations open | living |
| `.scratch/license-audit/spike-option3-stub.md` | Narrow Option 3′ local stub proof | **Proven for the current macOS host graph**: a 17-line independent Apache-2.0 proc-macro stub removes the 3 GPL package records; missing-license crates and the release STOP remain | living |
| `.scratch/license-audit/landing-option3.md` | Option 3′ execution and distribution-boundary record | **Stub landed, STOP narrowed**; Option 3′ is **EXECUTED**, the GPL trio is eliminated from the distributed host graph, and missing license fields, `self_cell`, weak-copyleft notices, and artifact-level legal review remain | living |
| `.scratch/license-audit/upstream/01-license-fields.md` | Prepared upstream issue for missing Cargo license metadata | `gpui_shared_string` and `gpui_util` lack `license`/`license-file` fields at the pinned revision and checked `origin/main`; posting checklist preserves holder-neutral wording | historical (stale-ok) |
| `.scratch/license-audit/upstream/02-apache-gpui-gpl-ztracing.md` | Prepared upstream issue for Apache GPUI and GPL ztracing dependency | Pinned `gpui` depends ordinarily on GPL-3.0-or-later `ztracing` (plus `zlog`/`ztracing_macro`) through direct SVG and `sum_tree` paths; issue asks for licensing/relicensing clarification without legal conclusion | historical (stale-ok) |

## link-affordance (2 files)

| Path | What it proves | Key number / verdict | Freshness |
| --- | --- | --- | --- |
| `.scratch/link-affordance/summary.txt` | Interactive Text focused affordance implementation | One opaque high-contrast 1 px quad per wrapped line; focused Rust test passed 2 cases; cursor experiment did not pass | historical (stale-ok) |
| `.scratch/link-affordance/issues/01-per-run-cursor.md` | Per-run cursor evidence and honest boundary | Parent `InteractiveText` range mechanism retained; reliable pointer/display distinction remains a follow-up requirement | historical (stale-ok) |

## link-keyboard (1 file)

| Path | What it proves | Key number / verdict | Freshness |
| --- | --- | --- | --- |
| `.scratch/link-keyboard/spec.md` | Keyboard-accessible nested Text run contract | Enter-only activation, native tab stop, existing listener wire, and no Space activation; depth-2 nesting remains rejected | historical (stale-ok) |

## menu-accelerators (1 file)

| Path | What it proves | Key number / verdict | Freshness |
| --- | --- | --- | --- |
| `.scratch/menu-accelerators/spec.md` | Static menu accelerator upstream audit | Pinned `MenuItem` has no explicit accelerator field; macOS can derive a shortcut indirectly from keybindings, Linux/Windows discard the keymap | historical (stale-ok) |

## observability (3 files)

| Path | What it proves | Key number / verdict | Freshness |
| --- | --- | --- | --- |
| `.scratch/observability/spec.md` | Local protocol-tap and host identity observability contract | Report adds frame/patch rates, timelines, histograms, malformed counters; host lines include `protocol=v3`; no queue/commit telemetry | historical (stale-ok) |
| `.scratch/observability/issues/01-runtime-metrics-gaps.md` | Queue/commit timing limitation | Tap has no queue depth, drain latency, or commit timer; do not infer backpressure from timestamp gaps | historical (stale-ok) |
| `.scratch/observability/issues/02-tap-report-and-version.md` | Report/version implementation result | Existing JSONL gains aggregates without payload persistence; historical tap-on overhead is 1.56 µs/frame, not a current gate | historical (stale-ok) |

## order-sweep (1 file)

| Path | What it proves | Key number / verdict | Freshness |
| --- | --- | --- | --- |
| `.scratch/order-sweep/sweep.md` | Test-order independence sweep | Ten randomized Bun seeds, Rust parallel/single-thread modes, and repeated host examples all passed with zero failures | historical (stale-ok) |

## perf-event-storm (8 files)

| Path | What it proves | Key number / verdict | Freshness |
| --- | --- | --- | --- |
| `.scratch/perf-event-storm/spec.md` | End-to-end native event-storm attribution | 143-node path is below the 8.33 ms 120-Hz interval; 10,000-node stateful path is engine-bound; no production coalescing justified | historical (stale-ok) |
| `.scratch/perf-event-storm/issues/01-scroll-storm.md` | Scroll storm measurement | 143 nodes at 240 Hz: 0.407/0.721 ms p50/p99; 10,000 nodes: 8.744/12.412 ms; all events committed | historical (stale-ok) |
| `.scratch/perf-event-storm/issues/02-drag-over-storm.md` | Drag-over storm measurement | 143 nodes at 240 Hz: 0.487/1.178 ms; 10,000 nodes: 9.506/13.084 ms; all events committed | historical (stale-ok) |
| `.scratch/perf-event-storm/issues/03-listener-fanout.md` | Listener dispatch isolation | 10,000-node no-op control is 0.001/0.004 ms p50/p99 for 240 events; fan-out is not the hotspot | historical (stale-ok) |
| `.scratch/perf-event-storm/issues/04-protocol-stages.md` | Patch build/encode/submit split | Scroll stages total 0.029/0.143 ms and drag-over 0.013/0.056 ms p50/p99; Rust 10,000-event encodes stay below 21 ms | historical (stale-ok) |
| `.scratch/perf-event-storm/issues/05-layout-dedupe.md` | Native layout callback deduplication | 128 nodes over two draws schedule 256 callbacks but emit exactly 128 layout events | historical (stale-ok) |
| `.scratch/perf-event-storm/issues/06-writer-queue.md` | Process writer saturation boundary | Queue cap is 32 frames/16 MiB; frame-cap approach is 0.133–0.299 s at tested rates; no silent drops | historical (stale-ok) |
| `.scratch/perf-event-storm/issues/07-react-engine-bound.md` | Reconciler/host commit-diff attribution | More than 99% of 10,000-node stateful latency is outside protocol/listener stages; stress result retained as a guard | historical (stale-ok) |

## pointer-coords (1 file)

| Path | What it proves | Key number / verdict | Freshness |
| --- | --- | --- | --- |
| `.scratch/pointer-coords/spec.md` | Pointer down/up coordinate wire contract | Exact seven-field payload `[6,button,modifiers,action,clickCount,x,y]`; logical non-negative coordinates; legacy five-field form rejected | historical (stale-ok) |

## pointer-move (1 file)

| Path | What it proves | Key number / verdict | Freshness |
| --- | --- | --- | --- |
| `.scratch/pointer-move/spec.md` | Opt-in high-frequency pointer movement performance | 143 registered nodes produced 240 events/commits; p50/p99 0.660/1.040 ms at 240 Hz versus 4.17 ms interval | historical (stale-ok) |

## prop-test (1 file)

| Path | What it proves | Key number / verdict | Freshness |
| --- | --- | --- | --- |
| `.scratch/prop-test/notes.md` | Tree patch property invariants and side-map cleanup | **4 product bugs found and fixed**; 32 seeds × 64 operations passed in 0.97 s, with a 20,000-node host phase of 0.104 ms | historical (stale-ok) |

## protocol-compatibility (4 files)

| Path | What it proves | Key number / verdict | Freshness |
| --- | --- | --- | --- |
| `.scratch/protocol-compatibility/spec.md` | Lockstep protocol-v3 policy | Snapshot/Event/Patch/Command all carry version; adjacent versions fail fast with actionable diagnostics; no negotiation | historical (stale-ok) |
| `.scratch/protocol-compatibility/issues/01-audit-current-mismatch.md` | Baseline mismatch behavior | Before fix TS returned `null`, Rust Event accepted v4 while other Rust decoders rejected it; exact real-decode baseline recorded | historical (stale-ok) |
| `.scratch/protocol-compatibility/issues/02-actionable-diagnostics.md` | Typed version-mismatch diagnostics | Rust and TS identify received/expected versions and route termination as `{ kind: "protocol" }`; focused tests cover v2/v4 | historical (stale-ok) |
| `.scratch/protocol-compatibility/issues/03-drift-lock-and-docs.md` | Version drift lock and upgrade policy | TS and Rust constants are each 3; fixture bytes stay unchanged; docs reject dual-version support | historical (stale-ok) |

## protocol-cutover (1 file)

| Path | What it proves | Key number / verdict | Freshness |
| --- | --- | --- | --- |
| `.scratch/protocol-cutover/spec.md` | Clean v3 wire cutover | Current forms are TextInput 13, Image 4, Drag 5, WindowResize 3, CommandResult 7, and string Submit; historical arities are rejected | historical (stale-ok) |
## pub-meta (1 file)

| Path | What it proves | Key number / verdict | Freshness |
| --- | --- | --- | --- |
| `.scratch/pub-meta/2026-08-30.md` | npm and crates.io publication metadata audit | Both package manifests are ready with explicit tarball allowlists (18/8 files); Cargo metadata is SPDX `Apache-2.0`; unsafe Bun/JSC FFI inventory is documented; no publish was run | historical (stale-ok) |

## rehearsal-2 (1 file)

| Path | What it proves | Key number / verdict | Freshness |
| --- | --- | --- | --- |
| `.scratch/rehearsal-2/notes.md` | Detached 0.3.0 release-prep rehearsal | Four sources synchronized to 0.3.0; CI passed after a known local file-dependency hydration refresh; deterministic 0.3.0 archive SHA `7a39f706…a4cd51e944`; worktree removed cleanly | historical (stale-ok) |

## skrifa-spike (1 file)

| Path | What it proves | Key number / verdict | Freshness |
| --- | --- | --- | --- |
| `.scratch/skrifa-spike/notes.md` | Direct font-parser migration and advisory boundary | `skrifa 0.44.0` replaces the application-owned parser; transitive `ttf-parser` remains in pinned GPUI paths, so its advisory stays open | historical (stale-ok) |

## version-const (1 file)

| Path | What it proves | Key number / verdict | Freshness |
| --- | --- | --- | --- |
| `.scratch/version-const/notes.md` | React DevTools renderer version source | `rendererVersion` is package metadata and release-prep now synchronizes it with Cargo and both package versions | historical (stale-ok) |


## release-productionization (16 files)

| `.scratch/release-productionization/v0.2.0-cut-checklist.md` | Executed local release cut and human gates | Final five gates exited 0 at gated HEAD `9ac496a`; host SHA is `bf737394…a6b7805f`; tag is local; narrowed license decision, signing, publication, and cross-platform runner evidence remain human/push-day work | living |
| `.scratch/release-productionization/release-notes-0.2.0.md` | Consumer-facing 0.2.0 scope and boundaries | Protocol v3 has 35 commands/23 events; notes unsigned macOS ARM, incomplete cross-platform runner evidence, and display-backed limits | historical (stale-ok) |
| `.scratch/release-productionization/release-readiness.md` | Release readiness capability and gate inventory | Gated HEAD `9ac496a`; five gates 0 at 95.77/4.29/49.20/19.68/2.13 s; core/dev API locks are 115/20; GPL trio eliminated by Option 3′, narrowed STOP remains | living |
| `.scratch/release-productionization/upstream-dependencies.md` | Pinned dependency and upstream-gap boundaries | GPUI is pinned to Zed revision `6805d952`; true gaps include base-direction, bidi geometry, letter spacing, and secure input; project limits are separated | historical (stale-ok) |
| `.scratch/release-productionization/style-gap-matrix.md` | GPUI style comparison and scope choice | Current bridge has 42 slots; B1 selects eight native fields while C-class expansions and upstream gaps stay deferred | historical (stale-ok) |
| `.scratch/release-productionization/issues/03-embedded-build-coverage.md` | Embedded gate ownership and coverage | Separate `make embedded-bun` gate runs four isolated examples plus refresh; 8.68 s is about 4x the 2.16 s baseline and below the 10x budget | historical (stale-ok) |
| `.scratch/release-productionization/release-artifacts.md` | Local artifact manifest and push-day handoff | Host archive SHA `d0dfc86e8c5b98d790d99eae0ec2ebb485e3b1a8cc28939cdafbf9a973e25ab` matches the checklist; package tarballs are CI-ephemeral; local tag peels to `72a520c`; no artifact has been published, pushed, signed, or notarized | living |
| `.scratch/release-productionization/dependency-audit.md` | Dependency and version governance audit | Cargo.lock has **672 packages** and 13 direct dependency declarations; GPUI remains pinned to Zed `6805d952`; `block 0.1.6` warns; cargo-audit is unavailable; latest CI/embedded gates passed | historical (stale-ok) |
| `.scratch/release-productionization/final-pre-cut-audit.md` | Final cross-cutting audit before the 0.2.0 cut | **Cut-ready from an engineering perspective**; 91 Unreleased entries (82/4/5), API locks core 101/dev 18, serial final gates passed, while human publication/signing/display/platform decisions remain | historical (stale-ok) |
| `.scratch/release-productionization/issues/01-unsigned-notarization.md` | Candidate artifact signing and notarization boundary | Candidate archive is unsigned and not notarized; SHA-256 checksums verify file consistency, not publisher identity; workflow stops before release/package publication | historical (stale-ok) |
| `.scratch/release-productionization/issues/02-block-future-incompat.md` | `block` future-incompatibility disposition | `block 0.1.6` remains the Apple GPUI warning; latest crates.io release is still 0.1.6 and only an unreviewed unsigned fork is available, so status is `ready-for-human` | historical (stale-ok) |
| `.scratch/release-productionization/issues/04-quartz-no-display.md` | Quartz display-backed host smoke boundary | No active display is available, so display-dependent GPUI paths were not re-tested; CLI, process/runtime, and package smoke coverage remains available and a display-backed run is still needed | historical (stale-ok) |
| `.scratch/release-productionization/issues/05-protocol-decode-hardening.md` | Continuous malformed-frame/fuzz coverage boundary | Rust exercises **1,039** and TypeScript **1,034** deterministic mutation cases; scoped tests pass, while long-term maintenance ownership remains `needs-triage` | historical (stale-ok) |
| `.scratch/release-productionization/issues/06-cross-platform-host.md` | Cross-platform GPUI backend and target validation boundary | Process-mode target matrix now enables Linux Wayland/X11 and adds runner workflows, but macOS probes cannot validate Linux/Windows; display-backed runner smoke and non-macOS embedded-Bun port remain open | historical (stale-ok) |
| `.scratch/release-productionization/spec.md` | Release productionization scope | Covers the macOS ARM process-runtime host candidate, package artifacts, CI gates, and pre-publication evidence; it does not publish artifacts or treat embedded Bun as standalone | historical (stale-ok) |
| `.scratch/release-productionization/DECISION-QUEUE.md` | Human release decision queue and push-day handoff | Internal contract is production-ready, but external distribution remains held at the narrowed license boundary; upstream issue, legal review, push/publication/signing decisions, next cut, and optional gallery restart are queued | living |

## release-rehearsal (1 file)

| Path | What it proves | Key number / verdict | Freshness |
| --- | --- | --- | --- |
| `.scratch/release-rehearsal/report.md` | Detached-worktree release-prep rehearsal | Version sync, frozen locks, package candidates, and deterministic host archive passed; package outputs lived under `/tmp` and were cleaned | historical (stale-ok) |

## rich-perf (3 files)

| Path | What it proves | Key number / verdict | Freshness |
| --- | --- | --- | --- |
| `.scratch/rich-perf/measurement.md` | Rich-text assembly and shape benchmark | 200-run unchanged redraws went from 32 to 0 assemblies; residual shaping is only about 11–13% of forced total | historical (stale-ok) |
| `.scratch/rich-perf/issues/01-targeted-invalidation.md` | Targeted rich cache invalidation | Patches invalidate changed nodes and ancestors while unrelated paragraphs retain cache; snapshot replacement still clears all | historical (stale-ok) |
| `.scratch/rich-perf/issues/02-shaped-text-cache.md` | StyledText shape-cache decision | Two runs measured 12.7%/13.0% shape share, below the 30% savings threshold; deeper cache is resolved-as-wontfix | historical (stale-ok) |

## showcase (1 file)

| Path | What it proves | Key number / verdict | Freshness |
| --- | --- | --- | --- |
| `.scratch/showcase/coverage.txt` | Example showcase capability coverage | Multi-surface controls, runtime font loading, bundled Tuffy asset, and clipboard-image recipe are represented | historical (stale-ok) |

## snapshot-perf (3 files)

| Path | What it proves | Key number / verdict | Freshness |
| --- | --- | --- | --- |
| `.scratch/snapshot-perf/measurement.md` | Initial snapshot, patch, and first-frame performance audit | Targeted reconciliation fix reduced one-operation host apply to 0.131 ms; remaining 20k first draw is GPUI-bound at about 196 ms | historical (stale-ok) |
| `.scratch/snapshot-perf/issues/01-affected-reconciliation-and-gpui-draw-boundary.md` | Affected-workset fix and honest GPUI draw boundary | 20k snapshot host stages 70.705 ms; one-op patch host stages 0.131 ms; no speculative element-tree cache | historical (stale-ok) |
| `.scratch/snapshot-perf/ts-encode.md` | TypeScript snapshot encode scaling and wire size | 20k total encode/handoff 3.759 ms p50 / 5.612 ms p99; payload 0.410 MiB, 2.559% of frame limit | historical (stale-ok) |

## soak-test (1 file)

| Path | What it proves | Key number / verdict | Freshness |
| --- | --- | --- | --- |
| `.scratch/soak-test/notes.md` | Long-running renderer state and memory stability | **2,000 iterations** passed; RSS rose 608 KiB then plateaued after warm-up, with side maps bounded and no leak found | historical (stale-ok) |

## storm-trim (1 file)

| Path | What it proves | Key number / verdict | Freshness |
| --- | --- | --- | --- |
| `.scratch/storm-trim/notes.md` | Event-storm sample budget verdict | `23.6s = minimum stable sample; quarter fails variance/margin gate; no trim` | historical (stale-ok) |

## supply-chain (4 files)

| Path | What it proves | Key number / verdict | Freshness |
| --- | --- | --- | --- |
| `.scratch/supply-chain/2026-08-30.md` | Bun and Rust dependency advisory audit | Bun core/dev audits clean after `@babel/core` 7.28.4 → 7.29.6; cargo-deny found three unmaintained crates; `paste` and `rustybuzz` remain needs-triage, while direct `ttf-parser` exposure is eliminated and its transitive issue stays open | historical (stale-ok) |
| `.scratch/supply-chain/issues/01-paste.md` | `paste` advisory disposition | `paste` 1.0.15 remains an upstream GPUI/Apple compile-time dependency with no safe upgrade; needs upstream triage | historical (stale-ok) |
| `.scratch/supply-chain/issues/02-rustybuzz.md` | `rustybuzz` advisory disposition | `rustybuzz` 0.20.1 remains an upstream `usvg`/`resvg` shaping dependency with no safe graph-local replacement; needs upstream triage | historical (stale-ok) |
| `.scratch/supply-chain/issues/03-ttf-parser.md` | Direct font parser advisory disposition | Direct `react-gpui` exposure is eliminated by `skrifa 0.44.0`; `ttf-parser` remains in transitive pinned-GPUI paths, so the issue stays open | historical (stale-ok) |

## surface-coverage (1 file)

| Path | What it proves | Key number / verdict | Freshness |
| --- | --- | --- | --- |
| `.scratch/surface-coverage/coverage.txt` | Protocol command/event coverage matrix | Commands 1..35 and events 1..23 are 35/35 and 23/23 across fuzz/golden suites; embedded gate also passed | historical (stale-ok) |

## surface-lifecycle (6 files)

| Path | What it proves | Key number / verdict | Freshness |
| --- | --- | --- | --- |
| `.scratch/surface-lifecycle/spec.md` | Multi-surface identity, close, failure, and focus contract | Surface/epoch identity is validated; root-local render errors differ from shared-runtime fatal failures; retired IDs cannot revive | historical (stale-ok) |
| `.scratch/surface-lifecycle/issues/01-isolation.md` | Per-surface versus shared-runtime isolation | Surface 95 failure leaves sibling 97 usable; isolated renderer rerun passed 66 tests | historical (stale-ok) |
| `.scratch/surface-lifecycle/issues/02-close-ordering.md` | Close ordering and ID retirement | Native close emits before removing registry entries; TS IDs are monotonic/retired and explicit reuse raises `SurfaceIdReusedError` | historical (stale-ok) |
| `.scratch/surface-lifecycle/issues/03-focus-routing.md` | Cross-surface focus/activation routing | Two-window headless dispatch proves surface 1 blur and surface 2 focus retain identity | historical (stale-ok) |
| `.scratch/surface-lifecycle/issues/04-epoch-validation.md` | Epoch and generation validation | Event/patch/command identity checks reject mismatches; changing epoch does not bypass retired-ID protection | historical (stale-ok) |
| `.scratch/surface-lifecycle/issues/05-example-coverage.md` | Supported multi-surface example path | Example opens/registers/renders/closes a child surface; it deliberately does not demonstrate same-ID reopen | historical (stale-ok) |

## surface-prop (1 file)

| Path | What it proves | Key number / verdict | Freshness |
| --- | --- | --- | --- |
| `.scratch/surface-prop/notes.md` | Surface/epoch identity and command-routing property test | No epoch/reuse violation across 16 seeds × 48 operations; 2 focused property tests passed in 0.14 s | historical (stale-ok) |

## test-determinism (4 files)

| Path | What it proves | Key number / verdict | Freshness |
| --- | --- | --- | --- |
| `.scratch/test-determinism/spec.md` | Three gate-flake hardening decisions | Writer EOF, per-instance tap failures, and commit-observed embedded smoke are bounded without weakening contracts | historical (stale-ok) |
| `.scratch/test-determinism/issues/01-process-event-writer-reader-eof.md` | Process writer/reader EOF repair | Focused loop passed 20/20 after change; waits are explicit and bounded, not sleep-based | historical (stale-ok) |
| `.scratch/test-determinism/issues/02-protocol-tap-cross-file-contamination.md` | Protocol-tap state isolation | Combined no-isolate loop improved from 10/20 failures to 20/20; full sample recorded 112 passes | historical (stale-ok) |
| `.scratch/test-determinism/issues/03-embedded-smoke-fixed-timeout.md` | Embedded smoke synchronization | Five post-change runs passed; each observed `commits=1` and expected 4.46–4.74 s timeout behavior | historical (stale-ok) |

## testing-dx (3 files)

| Path | What it proves | Key number / verdict | Freshness |
| --- | --- | --- | --- |
| `.scratch/testing-dx/spec.md` | TestApp facade design and honest boundary | Locators and event helpers use real MemoryTransport; host geometry/paint remains Rust/display-backed | historical (stale-ok) |
| `.scratch/testing-dx/issues/01-test-app-facade.md` | Consumer behavior-test facade implementation | Labels/exact text locate nodes; press, hover, key, input, submit, scroll, drag, focus, and command paths use Root dispatch | historical (stale-ok) |
| `.scratch/testing-dx/issues/02-docs-and-lock.md` | Facade docs and API lock | Dev API lock reached 20 exports; README recipe is copied from real tests; no host-geometry claim | historical (stale-ok) |

## text-input-completeness (7 files)

| Path | What it proves | Key number / verdict | Freshness |
| --- | --- | --- | --- |
| `.scratch/text-input-completeness/spec.md` | Native TextInput completeness audit | Host-owned C/X/V, Select All, word navigation, multiline copy, and maxLength/IME are implemented; secure input remains an upstream gap | historical (stale-ok) |
| `.scratch/text-input-completeness/issues/01-clipboard.md` | TextInput clipboard key semantics | Real mouse selection plus Cmd-C/X/V verifies copy, cut, paste, and multiline selection through native edit events | historical (stale-ok) |
| `.scratch/text-input-completeness/issues/02-undo.md` | Earlier undo/redo gap classification | Historical snapshot says upstream-gap because GPUI has no InputHandler undo; superseded by the later `undo-redo` reclassification/implementation | historical (stale-ok) |
| `.scratch/text-input-completeness/issues/03-selection-navigation.md` | Select All and word-wise navigation | Cmd/Ctrl-A and Alt/Option word movement with Shift extension reuse the same UTF-16/UAX model | historical (stale-ok) |
| `.scratch/text-input-completeness/issues/04-secure.md` | Secure/password input boundary | GPUI has no secure-display primitive; `secureTextEntry` and `keyboardType` remain unsupported without a masking fallback | historical (stale-ok) |
| `.scratch/text-input-completeness/issues/05-max-length-ime.md` | Marked-IME maxLength parity | Replacement truncates at UTF-16 boundaries for ordinary and marked paths; astral characters are not split | historical (stale-ok) |
| `.scratch/text-input-completeness/issues/06-multiline.md` | Multiline Enter and cross-line copy | Multiline Enter inserts text rather than submit; cross-paragraph copy preserves the internal newline | historical (stale-ok) |

## theme-consistency (4 files)

| Path | What it proves | Key number / verdict | Freshness |
| --- | --- | --- | --- |
| `.scratch/theme-consistency/spec.md` | Shared appearance-token architecture | Twelve visual examples use one `theme.ts`; roots own appearance stores and light/dark state does not change layout/behavior | historical (stale-ok) |
| `.scratch/theme-consistency/issues/01-shared-token-module.md` | Canonical token module | Immutable light/dark tokens cover canvas, controls, text, focus, status, input, and shadows; no renderer context added | historical (stale-ok) |
| `.scratch/theme-consistency/issues/02-example-conversions.md` | Example conversion scope | Gallery, dropdown, focus-flow, drag-reorder, todo, input, counter, keyboard, stress, list, selectable text, and multi-surface converted | historical (stale-ok) |
| `.scratch/theme-consistency/issues/03-docs-and-evidence.md` | Theme documentation and dual-mode evidence | README/getting-started point to the real module; process-only light/dark snapshots compare semantic surfaces | historical (stale-ok) |

## transport-resilience (4 files)

| Path | What it proves | Key number / verdict | Freshness |
| --- | --- | --- | --- |
| `.scratch/transport-resilience/spec.md` | Process-boundary resilience contract | Crash paths, malformed frames, typed causes, pending rejection, and restart pattern are fail-fast; no resynchronization protocol | historical (stale-ok) |
| `.scratch/transport-resilience/issues/01-crash-report-bridge.md` | Crash report path propagation | Host emits one stable path marker; JS exposes `crashReportPath` with bounded stderr; unwritable reports do not claim a path | historical (stale-ok) |
| `.scratch/transport-resilience/issues/02-malformed-frame-fate.md` | Malformed-frame termination behavior | Valid/malformed/valid sequence terminates once, rejects pending commands, and ignores later bytes; direct decoder containment remains | historical (stale-ok) |
| `.scratch/transport-resilience/issues/03-typed-termination-cause.md` | Programmatic termination taxonomy | Causes are shutdown/eof/exit/code/protocol/detail/io/detail; focused tests discriminate without message parsing | historical (stale-ok) |

## ts-prop (1 file)

| Path | What it proves | Key number / verdict | Freshness |
| --- | --- | --- | --- |
| `.scratch/ts-prop/notes.md` | TypeScript snapshot/patch emission invariants | **362,716 assertions**, zero violations across 24 seeds × 40 mutations; targeted test passed in 1.75 s | historical (stale-ok) |

## undo-redo (2 files)

| Path | What it proves | Key number / verdict | Freshness |
| --- | --- | --- | --- |
| `.scratch/undo-redo/spec.md` | Host-owned TextInput history contract | Bounded 100 pre-edit snapshots; typing coalesces by caret continuity; Cmd/Ctrl-Z and redo emit ordinary change/selection events | historical (stale-ok) |
| `.scratch/undo-redo/reclassification.md` | Correction of the earlier undo classification | Renderer ownership makes bounded host history feasible without GPUI primitive or wire change | historical (stale-ok) |

## upstream-assessment (1 file)

| Path | What it proves | Key number / verdict | Freshness |
| --- | --- | --- | --- |
| `.scratch/upstream-assessment/2026-08-30-gpui-drift.md` | Released GPUI/Zed API drift assessment | No released unblock for requested boundaries; keep current pin/protocol and existing honest upstream/platform gaps | historical (stale-ok) |

## virtual-list-completeness (7 files)

| Path | What it proves | Key number / verdict | Freshness |
| --- | --- | --- | --- |
| `.scratch/virtual-list-completeness/spec.md` | Six-edge VirtualList contract | Range end semantics, restoration, index bounds, wrapper propagation, estimates, and row state are explicitly decided; wire unchanged | historical (stale-ok) |
| `.scratch/virtual-list-completeness/issues/01-on-end-reached.md` | `onEndReached` semantics | Fires on emitted range end >= data length, can fire near/initial end, and re-arms after retreat; no second wire event | historical (stale-ok) |
| `.scratch/virtual-list-completeness/issues/02-scroll-restore.md` | Data-change scroll restoration | Logical item/offset anchor survives reset, shrink clamps to end, growth preserves old-end anchor, affected rows remeasure | historical (stale-ok) |
| `.scratch/virtual-list-completeness/issues/03-scroll-commands.md` | Index/end command boundaries | Existing index and end commands work with estimates; `index === data.length` rejects without a frame | historical (stale-ok) |
| `.scratch/virtual-list-completeness/issues/04-boundary-wrapper.md` | List wheel boundary and empty/placeholder decision | Bubble wrapper stops outer propagation while list consumes wheel; empty/placeholder visuals remain display-backed | historical (stale-ok) |
| `.scratch/virtual-list-completeness/issues/05-estimated-item-size.md` | Estimate-to-measure convergence | Committed rows replace wrong estimates with natural heights; never-committed rows legitimately retain estimates | historical (stale-ok) |
| `.scratch/virtual-list-completeness/issues/06-row-state-lifecycle.md` | Virtualized row state lifetime | Overlapping keyed rows retain state; evicted rows unmount and remount with initial state; focused suite recorded 65 passes | historical (stale-ok) |

## vlist-scroll (2 files)

| Path | What it proves | Key number / verdict | Freshness |
| --- | --- | --- | --- |
| `.scratch/vlist-scroll/spec.md` | Precise VirtualList offset API | `getScrollOffset()` returns logical pixels; `scrollToOffset()` accepts finite non-negative pixels and delegates clamping to GPUI | historical (stale-ok) |
| `.scratch/vlist-scroll/issues/01-scroll-offset.md` | Commands 34/35 implementation result | Native ListState offset read/write, wire, handles, goldens, and docs are complete; pre-layout writes are acknowledged but have no native effect | historical (stale-ok) |

## window-controls (4 files)

| Path | What it proves | Key number / verdict | Freshness |
| --- | --- | --- | --- |
| `.scratch/window-controls/spec.md` | Root window-control contract | Commands 30–33 cover minimize, bounds tag 8, state tag 9, and activation; no runtime position setter exists | historical (stale-ok) |
| `.scratch/window-controls/issues/01-root-commands.md` | Window command implementation | All four commands are root-only/null-payload; bounds/state tags round-trip; minimize visible effect remains display-backed | historical (stale-ok) |
| `.scratch/window-controls/issues/02-position-boundary.md` | Position persistence limitation | Saved bounds can restore centered size through `openSurface`; exact global position restore is upstream-blocked | historical (stale-ok) |
| `.scratch/window-controls/issues/03-headless-limitations.md` | Headless native-control boundary | TestWindow reads bounds and false state values; minimize is unimplemented, so visible minimize/activation require a display | historical (stale-ok) |

## window-title (1 file)

| Path | What it proves | Key number / verdict | Freshness |
| --- | --- | --- | --- |
| `.scratch/window-title/spec.md` | Existing title command and notes example correction | `SetTitle` is command 6; no command 36/badge/subtitle path; notes derives title from basename and dirty state | historical (stale-ok) |

## Living-versus-historical guide

**Living:** `release-readiness.md`, `v0.2.0-cut-checklist.md`,
`release-artifacts.md`, the six living license-audit entries, and this `EVIDENCE.md`
are the release/push-day references. The checklist and manifest carry the
current local cut and pending human publication decisions; the license-audit
entries carry the active narrowed publication blocker and Option 3′ proof; the
index is the navigation map.

**Historical snapshots:** all other rows are dated audits, measurements,
rehearsals, issue decisions, and implementation evidence. They remain valuable
for provenance and regression comparison, but a future release should not infer
current status from a stale measurement without rerunning the named gate.

**Not included:** untracked/ignored scratch material (for example directories
that may exist locally but do not appear in `git ls-files .scratch`) is outside
this index's tracked-evidence contract. All tracked areas, including
`upstream-assessment` and `supply-chain`, are indexed as landed evidence.

