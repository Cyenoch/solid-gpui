# Integration ownership and acceptance record

Integration branch: `integrate/comparison-adoption`.
Starting implementation/spec commit: `e879b66`.
Source baseline: `e9bc4389c5394e5de1fecb6758d2d06e48877fdb`.

All ten workstreams are concurrent parts of one authorized delivery, not delivery phases. The user requested every required outcome with high-quality fixes and refactoring.

## Workstream registry

All worktrees are below `/private/var/folders/sl/42r5kc756mj96rlxkglrpr900000gn/T/opencode/solid-gpui-adoption-20261002/`.

| Ticket | Branch/worktree suffix | Agent session | State |
| --- | --- | --- | --- |
| 01 | `01-native-acceptance` | `ses_f05081953ffeN4aQ8DSP0ruwr5` | Merged `67e8c37`; public deterministic/GPU acceptance: 4 passed |
| 02 | `02-delivery` | `ses_f05081953ffdcg3qfb1jmdkR5L` | Merged `a03db0fc`; source fixes and standalone qualification in progress |
| 03 | `03-contract-identity` | `ses_f05081949ffe0gqOTtORiGOJUl` | Merged `d6f36b09`; effective-lock and embedded-input review fixes verified |
| 04 | `04-style-contract` | `ses_f05081949ffd2btrvc76HWGHTa` | Merged `89ff7889`; protocol v7 and style/native tests passed |
| 05 | `05-native-performance` | `ses_f05081948ffexDinEwHi4cp7io` | Merged `9ed85d73`; observation replay and flex-basis equivalence passed |
| 06 | `06-selection-search` | `ses_f05059a58ffesWtaKj9ntSXmgY` | Merged `5e19326c`; renderer envelope and selection lifecycle passed |
| 07 | `07-paint-media` | `ses_f05059a57ffeHzvbUN55kpP7mi` | Merged `3a0ab99`; overflow review fix and native raster tests passed |
| 08 | `08-signed-updater` | `ses_f05059a57ffdT9YjVlSHhDM0CB` | Recovery fixes merged `f513899d`; signed fixtures and restart tests passed |
| 09 | `09-reference-application` | `ses_f05059a4fffeVoD8VDxsQpVOpq` | Merged `3a02bd19`; combined native workload qualification in progress |
| 10 | `10-compiler-docs` | `ses_f05059a4fffdW434fUJ42kofBR` | Merged `91a6376`; compiler tarball qualification pending combined gate |

Implementation branches are prefixed `adopt/`. Agents must commit focused changes, update their delivery note, and merge the integration tip before reporting. Integration merges are serialized. No agents write implementation files in the main workspace.

## Shared seams to reconcile

- Ticket 04 owns canonical wire schema/style generation; regenerate the final protocol and goldens from its canonical source after merged consumers settle.
- Ticket 03 owns native contract/build identities; new modules/components from 06/07/08 must use that resulting contract rather than preserve old wrappers.
- Tickets 01/05/06 touch native renderer state. Reconcile invalidation/painted geometry/event metadata/selection lifetime as one design; do not accept mechanically merged independent state caches.
- Ticket 02 owns project/CLI/artifact delivery; ticket 10's compiler subpath must remain accessible in installed tarballs without starting Vite/native processes.
- Ticket 09's source-pinned application acceptance consumes the final native harness, selection and paint/media capabilities. Its existing semantic tests alone are not final native acceptance.
- Regenerate combined SDK and website native bindings from the merged host; hand edits of generated components are prohibited.
- Topical documentation and Chinese copies must agree. Final website catalog coverage requires examples/navigation for every new exported component.

## Final gate checklist

- [ ] All tickets have working implementations or an explicitly measured rejected optimization with preserved scope and evidence.
- [ ] Integration and public compiler/package typechecks, focused changed-path tests.
- [ ] Canonical wire/native generation checks and cross-language golden fixtures.
- [ ] Full workspace test gate once after integration; repeat only for new changes/failures.
- [ ] Standalone stock/custom consumer and extracted generic application checks.
- [ ] Native acceptance against real painted/hit-tested behavior, explicit unsupported-platform outputs.
- [ ] Recorded paint, live frame, cross-element selection/search resource/lifecycle checks.
- [ ] Signed update fixture verification, tamper/replay/platform rejection, cancellation, install/rollback in temporary app paths.
- [ ] Performance measurements serially after compilation, content verified before timing, exact binary/profile/features/viewport/monitor recorded.
- [ ] Website guide/canonical API/example/navigation synchronization, browser/content checks and native/WASM build where relevant.
- [ ] Standards and spec code review on integration branch; fixes integrated and checked.
- [ ] Tickets resolved with actual checks and platform limitations; clean commits and final delivery record.

Network publication, upstream messages, public release creation and modification of the user's running installed app are not part of this integration task. Release tooling and update behavior are made concrete and locally reviewable.

## Merge and check record

### Ticket 10 — canonical compiler and provenance

Merged `adopt/10-compiler-docs` commits `3c08361b`/`4045d953` with merge `91a63761081f330dde4decd6ee0326fb94f8b36b`; no conflicts. The merger ran three compiler checks and six website/content/runtime checks against source without rebuilding dist; all passed. The feature owner additionally ran package compilation, compiler tarball consumer without Vite, tooling types, and the native line-clamp cache regression; see `delivery-10.md`.

The standalone upstream patch contains standard blank context lines (one space) that `git diff --check` reports as trailing whitespace inside the patch artifact; preserve valid patch syntax rather than edit those lines. Production source/docs pass whitespace checks.

Preserve `./compiler`, optional Vite peer metadata and compiler pack-smoke wiring when ticket 02 lands. Final website typecheck/build requires generated WASM bindings. Extend the new `vendor/GPUI-SOURCES.md` inventory for later native vendor edits. Current machine has Bun 1.4.2, Rust 1.98.1, the required WASM nightly installed, wasm-bindgen available, and Xcode Metal compiler discoverable.

### Tickets 01 and 08 — native acceptance and signed updates

Merged ticket 01 tip `f7104b00` with `67e8c376de22455d890e18d4237d0e55c3048a4f`, then ticket 08 tip `41dbbab6` with `f1c3835b54f420cb4dd08761c8212d76bbd2c696`. Preserved both changelog/docs entries and Cargo gates. Eighteen focused helper/schema/compiler/website tests, core/desktop types, locked offline Cargo metadata and changed Rust formatting passed after merge; dependency notices regenerated identically. No full suite or native runtime requalification was repeated by merger.

Ticket 08 independent review found two concrete transitions needing correction: status reads the pre-swap transaction without installation serialization; cleanup removes the recovery record before candidate cleanup succeeds. Owner is implementing serialized status and durable cleanup-state recovery with failure-injection tests. The fixes must land before updater acceptance. macOS installation is implemented; Windows/Linux explicitly reject installation authority, and physical/relaunch/power-loss qualification is not claimed.

Ticket 03 must migrate updater `with_contract` to its canonical semantic-version interface, preserving the explicitly non-abortable blocking commit command. Ticket 05 must preserve native acceptance painted observations through region reuse. Ticket 09 owns the accessibility forwarding fix for core VirtualList.

### Project wording constraint

Maintained prose describes this project's capabilities, design and usage. Preserve required legal attribution and concise fixture provenance. Historical research is archived outside the repository at `/private/var/folders/sl/42r5kc756mj96rlxkglrpr900000gn/T/opencode/solid-gpui-research-20261001`; its tracked copy has been removed after archive-byte verification.

### Ticket 07 — recorded painting and live frames

Merged pinned tip `4222e41a` with `3a0ab997e172a1d75e9cab53ee3d8889366b20cf`, no conflicts. Website typecheck and five focused catalog/example/highlight/Markdown checks passed. The feature owner verified five resource/correctness tests, native macOS pixel assertions, actual WASM build and website production build. Independent resource/lifecycle review is running. Full integrated host catalogs will be regenerated after identity/style/selection land.

Two feature worktrees found the WASM script assumed `target/` despite custom Cargo target directories. Integration corrects the wasm-bindgen input to use Cargo metadata's actual target directory and synchronizes both Web guides. Final WASM build will exercise this through the shared target directory rather than a hand-invoked bindgen workaround.

## Resumed integration — October 2, 2026

All six remaining workstreams and updater recovery fixes are integrated; no merge is pending. Native builds remain serialized. The four shutdown stashes remain intact; selected required fixes from stashes 0/1/3 were inspected and applied without dropping history.

Shared seams now use v7 explicit lengths, strict build envelopes on renderer commands, active selection/search live geometry, state/axis-aware region eligibility, retained painted observation replay, and parent-owned flex basis/aspect ratio. The generated selection example owns its import without indexing the unrelated paint examples. Vite runtime fixtures use Cargo's actual executable and custom target directory.

Independent review found seven concrete defects: frame-size overflow; effective consumer-lock and Embedded Bun provenance omissions; retained flex-basis omission; portable tar dependence; incomplete extracted component admission; and Bun starter stdout contamination. Source fixes are integrated locally. Portable archive encoding/extraction uses Bun, preserving modes and long Unicode paths; the explicit macOS USTAR updater format remains separate. The native starter renders service results reactively.

Combined Rust gate: 459 passed, 0 failed, two existing ignored attribution/doc cases; Clippy all targets passed. A later explicit website acceptance-bin build exposed an unchecked Result, now handled with nonzero failure. Public native acceptance: four passed, including actual macOS Metal pixel capture, causal input/epoch rejection and cleanup. Final source/generation/package/website/native workload/performance gates are still being completed; do not treat the checklist as complete yet.

### Final lifecycle reconciliation

Acceptance v2 drives native resize and queued production frame observers. GPU
actions service AppKit's main run loop before acknowledging bounds changes.
Public native suite: seven passed, including device-pixel resize dimensions.
Command replies retain request Surface/epoch; the reused-request regression passes.
Surface and node commands share one deferred commit seam, permitting requests
inside the initial render and waiting for synchronous Solid batches. Popup
teardown enqueues closure before retiring its owner. Existing async Bun/QuickJS
and popup lifecycle regressions protect those boundaries.

Full final Rust suite: 459 passed, zero failed, two existing ignored cases.
All-target Clippy explicitly includes both acceptance bins and passed.
The signed-update fixture explicitly sets accepted macOS sockets to blocking
with a finite read timeout. Final package suites passed after lifecycle fixes,
as did eight website checks, package/tool/fixture typechecks, canonical protocol
generation and cross-language goldens. WASM, paired-consumer packaging,
generation rechecks and serialized performance evidence remain in progress.

Reference deterministic and Metal workloads passed real paint/input/selection/
search, responsive layout, clip business ownership, bounded rows and preview
staging cancellation/release. Peak live row owners: 20. Reports and screenshot
are in `qualification/reference/`; tightening frame-advance and minimum-width
lane regressions requires a final focused rerun. No physical input/display or
other-platform qualification is implied.
