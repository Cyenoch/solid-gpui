# GPUI Kit adoption implementation

Status: complete

## Objective

Implement the adoption recommendations in [research](research.md), with upstream
revision `05433bd8e9e75af2f3aa508141b78ba21bfa6261` as the fixed source baseline.
Preserve the repository's native state, atomic publication, lifecycle, and
performance contracts. Replace obsolete local implementations rather than
retaining compatibility wrappers. All generated bindings come from Rust.

## Delivery checklist

- [x] Refresh vendored Base, Component, macros, and assets; reconcile every local
  patch; include Kit and FPS from the same revision.
- [x] Replace the local FPS display with the upstream HUD; preserve explicit
  enablement, actual cadence access, native profiling, and window lifecycle.
- [x] Use upstream headless UI helpers for key native interaction regressions.
- [x] Add retained Carousel with keyed children, selection, navigation, and events.
- [x] Add editor multiple selections, native auto-close/smart indent configuration,
  and protect controlled editing, IME, selection, and undo semantics.
- [x] Add opt-in frontmatter rendering and SVG-backed component icon slots.
- [x] Expose native Base motion/presence/springs and custom visual controls through
  retained native instances, with reduced motion and no per-frame JS updates.
- [x] Keep Shell out of the runtime and dependency graph, per the updated scope.
- [x] Separate Kit default control assets from application Iconify assets; use
  only the default Kit subset on desktop and Web, without cross-catalog fallback.
- [x] Regenerate SDK/native bindings and provenance; synchronize English and
  Chinese guides, catalog usage, previews, and website capability notes.
- [x] Complete native/SDK checks, website build and tests, actual UI inspection,
  and bounded monitor-enabled/disabled measurement. Record platform limits.

## Ownership and acceptance

Solid owns application data and routing. GPUI owns drawing, focus, transient
editing and animation state. Native component instances retain state until
unmounted. Shell is intentionally not integrated. Kit assets belong to native
controls; application icons belong to the existing bounded Iconify catalog and
application registration system. Neither namespace widens or falls back into
the other.

The FPS HUD's derived capacity, actual cadence, draw budget exceedance, and
resource readings have distinct documented definitions. Input-to-present and
invalidation-to-present instrumentation remain separately available. No native
GPU or physical-input guarantee follows from a headless test.

Only key tests are added: actual interaction and state/lifecycle invariants,
including rejected updates preserving existing state. Native library and
application tests run against the same patched GPUI graph. Website samples use
real generated contracts and cannot advertise unimplemented behavior.

## Progress

- Directory and upstream URL rename completed before this implementation.
- Baseline website build and six website tests passed before the upgrade.

## Scope correction

The user subsequently excluded GPUI Shell and asked for an explicit Kit-assets /
application-Iconify boundary. The experimental Shell implementation, tests,
example, dependency, and JIT migration were removed. Shell source remains available
only in the upstream reference checkout. Its earlier research is historical evidence,
not an advertised capability. Solid keeps its existing QuickJS version.

Final checks and qualification limits: [verification.md](verification.md).
