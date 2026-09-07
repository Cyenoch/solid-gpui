# QuickJS cancellation composition

Date: 2026-09-07. Baseline: `052bf17`.

The production QuickJS platform previously omitted `AbortSignal.any`. An
application combining operation cleanup, user cancellation, and a timeout would
fail with `not a function`. The existing real-VM router fixture reproduced that
failure before implementation and now passes with the composed signals.

The implementation keeps signal state in a private WeakMap and flattens nested
combinations to their original sources. It commits dependent states before
dispatching source callbacks, preserving the first reason under reentrant abort
and stopped event propagation. This follows the
[DOM cancellation ordering](https://dom.spec.whatwg.org/#abortsignal-signal-abort).
Input validation completes before selecting an already-aborted source.

Sources own their pending combinations. Reverse source references are weak, and
the first cancellation removes a combination from every source before callbacks.
Application cleanup must abort operation-owned controllers: discarded pending
combinations are retained while a source remains live. This does not claim
aggressive garbage collection of unused, un-aborted combinations. Creation work
scales with the distinct original sources; cancellation visits affected
combinations and their source links. Neither path performs GPUI rendering work.

## Verification

- The existing QuickJS router/Gallery fixture failed before implementation with
  `Error: not a function` at the first `AbortSignal.any` call, then passed.
- `cargo test -p solid-gpui --lib --locked --features quickjs runtime::quickjs::tests`:
  all six tests passed, including actual Solid Snapshot/press/Patch, routing,
  timers, UTF-8, busy-loop shutdown, queue pressure, and explicit runtime errors.
- `bun run task package-ci`: passed; see [log](cancellation-package-ci.log).
  Includes package consumers, generated bindings, formatting, TypeScript checks,
  renderer/router tests, production bundling, and release task tests.
- `git diff --check`: passed.

No native rendering or Rust code changed. This pass did not rebuild a standalone
Gallery archive or repeat desktop qualification. Platform signing, complete
redistribution notices, and Windows/Linux desktop qualification remain the
release gaps recorded in the earlier production-readiness report.
