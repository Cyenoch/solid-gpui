# Per-surface versus shared-runtime failure isolation

Status: resolved

## Evidence

- `packages/react-gpui/tests/renderer.test.tsx:868-888` proves a failed render on surface 95 does not prevent a sibling surface 97 from committing over the same transport, and the failed root can recover afterward.
- `packages/react-gpui/tests/surface-host.test.ts:126-145` retains command-result demultiplexing coverage across two routed roots.
- `packages/react-gpui/src/surface-host.ts:158-212` decodes one shared transport and terminates all routed roots on malformed/version-invalid frames.
- `crates/react-gpui-host/src/main.rs:572-596` routes commit-reader protocol/renderer termination to `fatal_runtime_failure` and `close_all`.
- `docs/adr/0008-error-handling-philosophy.md:50-62` specifies fail-fast shared runtime handling.

- Full isolated renderer rerun: `bun test packages/react-gpui/tests/renderer.test.tsx` → **66 pass, 0 fail, 288 expect calls**. The expected validation test diagnostics still print (`width must be a finite non-negative number`), but intentional invalid-style tests pass. The prior failure observed only during a combined/parallel invocation was test-runner contamination, not a product regression.
## Decision

No isolation bug: root-local reconciliation and shared-runtime fatal transport failure are intentionally distinct. Regression coverage remains in the existing renderer and surface-host tests.
