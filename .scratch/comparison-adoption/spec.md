# Complete native capability and application delivery

Status: resolved

User authorization (2026-10-02): implement all required outcomes in one integrated delivery, with parallel agents, high-quality fixes, cleanup and refactoring.

This is one integrated delivery. Independent work runs concurrently; integration dependencies do not reduce scope. Baseline: `e9bc4389c5394e5de1fecb6758d2d06e48877fdb`. Historical research is archived outside the repository.

## Required outcomes

1. Correct Unicode TestHost offsets, remove duplicated stale release protocol versions, and fix misleading capability/provenance documentation.
2. Supply a version-paired standalone starter, an optional prebuilt stock host path, and generic application packaging with extracted-artifact verification. Custom Rust modules remain supported. No moving main/latest template selection or install-time native compilation.
3. Export our canonical universal compiler independently of Vite lifecycle; all consumers share its implementation and source-map semantics.
4. Separate canonical public native contract identity (including explicit behavioral contract version) from implementation/build identity. Keep strict mismatch rejection; no historical decoder/wrapper fallback.
5. Provide a public opt-in native acceptance seam: real native geometry/paint and hit-tested input, stable locators, typing/drag/wheel, screenshots where actually supported, cleanup and explicit platform limitations. Public TestHost remains a semantic protocol helper.
6. Introduce correctly owned native rendering regions/input invalidation where measurement supports improvement; verify visible content, event generations, native-only edits, scrolling, async images, animations, inherited styles and repeated resizes. Investigate immutable style sharing/reclamation against actual local structures and retain only evidence-supported changes.
7. Implement strict useful style authoring conveniences and native hover/active/focus-visible refinements, lowering to supported native contracts rather than a fake DOM/CSS runtime. Diagnose unsupported values and document precedence.
8. Add renderer-wide cross-element selection/search with native ownership, stable identity, copy and lifecycle rules; preserve existing per-node/native control behavior.
9. Add bounded recorded native paint and host-owned live-frame/media resources through generated NativeView/NativeModule contracts. No JS paint callbacks, raw pointers crossing protocol boundaries, per-frame data URLs, or unbounded resource queues. Use a real example consumer and lifecycle verification.
10. Add signed application-update functionality as an explicit application-owned native capability with trusted feed/key, bounded acquisition, verification before installation, safe staged replacement/rollback, cancellation and explicit relaunch. Verify with local fixtures, never update the user's active app or publish secrets/artifacts.
11. Add an exact-source realistic multi-pane/large-history application acceptance workload with attribution/provenance, explicit native adaptations and meaningful interactions. Integrate website examples/content and qualification evidence.
12. Make vendored-patch provenance accurate and reduce unnecessary source-build coupling; prepare generally useful upstream fixes as reviewable local artifacts when warranted. Do not post upstream communications without explicit authorization.

## Invariants

- Solid owns composition/reactive state; GPUI owns native state/render/input. Only bounded owned bytes cross runtime/process seams.
- Atomic commit validation, surface/epoch/revision ownership, callback-generation safety, controlled edit acknowledgements, list row-owner virtualization, cancellation and deterministic cleanup survive every change.
- No N-API rewrite, silent current-handler routing, permissive partial mutations, fake DOM, unsupported style no-ops, legacy API fallbacks, or pointer serialization.
- English maintained source/docs; explicit `.zh-CN.md` translations for website guides. Follow AGENTS documentation synchronization.
- Only key behavioral tests. No performance claims from stale/omitted content, TestAppContext timings misrepresented as physical presentation, skipped-platform tests misrepresented as support, or workflows misrepresented as executed release results.

## Concurrent task map

| Ticket | Owner scope | Integration dependencies |
| --- | --- | --- |
| 01 | Unicode helper + native acceptance/testing API | Reference app consumes the resulting seam |
| 02 | Starter/prebuilt stock-host/generic packaging/release assertions | Compiler/contract outputs merged before final packaging qualification |
| 03 | Canonical native contract/build identity | Regenerate combined bindings after all native additions |
| 04 | Style ergonomics + native pseudo styles/schema | Own all canonical wire schema/style changes |
| 05 | Native render regions + style-sharing measurement | Reconcile style/selection/automation ownership when merging |
| 06 | Cross-element selection/search | Native testing/reference app consume resulting behavior |
| 07 | Recorded paint + live media resources | Generated catalog and reference app integration |
| 08 | Signed application-owned updater | Generic packager integration and local sandbox qualification |
| 09 | Exact-source realistic application workload | Final acceptance consumes tickets 01/04/06/07 |
| 10 | Public compiler + documentation/provenance cleanup | Tooling integration with ticket 02 |

Each ticket may start now. If another ticket's interface is needed, implement against a documented local interface and record integration requirements rather than waiting to start or reducing scope.

## Delivery and verification

Use one integration branch with isolated worktrees. Commit implementation work before merging. Each owner records focused checks, behavior evidence, documentation synchronized, limitations and merge instructions under `.scratch/comparison-adoption/` in its own delivery note. Final integration runs generated protocol/native checks, relevant package checks, full tests once, website checks/build, native acceptance/performance checks serially, and a parallel standards/spec code review. Fix review findings before final delivery. No network publication/release deployment is implied.

## Integrated completion

All required outcomes are integrated and locally qualified. See `integration.md`
for final source identity, measured counterexamples, checks, documentation
synchronization and explicit release/platform/physical verification limits.
