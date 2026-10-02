# Integration ownership and acceptance record

Integration branch: `integrate/comparison-adoption`.
Starting implementation/spec commit: `e879b66`.
Source baseline: `e9bc4389c5394e5de1fecb6758d2d06e48877fdb`.

All ten workstreams are concurrent parts of one authorized delivery, not delivery phases. The user explicitly requested all comparative recommendations and high-quality fixes/refactors/adoption.

## Workstream registry

All worktrees are below `/private/var/folders/sl/42r5kc756mj96rlxkglrpr900000gn/T/opencode/solid-gpui-adoption-20261002/`.

| Ticket | Branch/worktree suffix | Agent session | State |
| --- | --- | --- | --- |
| 01 | `01-native-acceptance` | `ses_f05081953ffeN4aQ8DSP0ruwr5` | Implementing |
| 02 | `02-delivery` | `ses_f05081953ffdcg3qfb1jmdkR5L` | Implementing |
| 03 | `03-contract-identity` | `ses_f05081949ffe0gqOTtORiGOJUl` | Implementing |
| 04 | `04-style-contract` | `ses_f05081949ffd2btrvc76HWGHTa` | Implementing |
| 05 | `05-native-performance` | `ses_f05081948ffexDinEwHi4cp7io` | Implementing |
| 06 | `06-selection-search` | `ses_f05059a58ffesWtaKj9ntSXmgY` | Implementing |
| 07 | `07-paint-media` | `ses_f05059a57ffeHzvbUN55kpP7mi` | Implementing |
| 08 | `08-signed-updater` | `ses_f05059a57ffdT9YjVlSHhDM0CB` | Implementing |
| 09 | `09-reference-application` | `ses_f05059a4fffeVoD8VDxsQpVOpq` | Implementing |
| 10 | `10-compiler-docs` | `ses_f05059a4fffdW434fUJ42kofBR` | Implementing |

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
