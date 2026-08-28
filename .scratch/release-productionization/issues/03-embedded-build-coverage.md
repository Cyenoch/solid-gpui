# Embedded Bun source build is outside ordinary candidate coverage

Status: ready-for-human
Type: task

The embedded Bun path clones and compiles the pinned Bun/JSC source graph. It is
not included in the process-runtime host candidate archive or ordinary `make
ci`; it is covered by the separate `make embedded-bun` gate and the independent
manual/path-filtered embedded workflow.

Evidence: `Makefile`, `README.md`, and `.github/workflows/embedded-bun.yml`.

## Comments

The current separation is intentional to prevent every ordinary commit from
performing the expensive embedded source build. The embedded gate must remain
available before treating an embedded change as release-ready.

The embedded-before-release gate is now substantively present: `Makefile:38-40`
runs both the host embedded feature check and the embedded Bun feature tests,
and `.github/workflows/embedded-bun.yml:3-16` provides automatic pull-request
path filtering for crates/packages/toolchain inputs plus `workflow_dispatch`.
The remaining status is `ready-for-human` for release approval, not a missing
gate.

2026-08-28 coverage update: the embedded release gate now runs the host-owned
`embedded_examples.rs` integration target in addition to the adapter crate
tests. The bounded matrix loads `gallery.tsx`, `text-input.tsx`,
`virtual-list.tsx`, and `notes.tsx` through `EmbeddedBunAdapter`; each case
must emit a protocol-v3 Snapshot on surface 1 with a nontrivial bounded tree,
known startup text, no embedded evaluation failure, and a clean status-0
shutdown. A separate counter refresh probe confirms a queued Fast Refresh
module keeps the runtime alive and emits another commit.

The first implementation exposed a real Bun/JSC boundary: multiple embedded
VMs in one process trigger Bun's `ScriptExecutionContext::initialIdentifier()`
single-use assertion. The matrix therefore isolates each example in a worker
test subprocess, preserving real adapter coverage while giving each VM a fresh
process. This is documented in `.scratch/embedded-coverage/spec.md`; no
fallback entry or skipped case is used. Native painting, IME candidate
placement, picker UI, and actual file-command completion remain their existing
Baseline before the matrix was `/usr/bin/time -p make embedded-bun` real
**2.16 s** on the warm local tree (host feature check plus one counter test).
After the matrix and protocol-image fixes, the same gate passed in **8.68 s**
real (**+6.52 s**, about **4.0x** baseline): host feature check, two host
embedded integration tests (four isolated example workers plus refresh), and
the adapter counter test all passed. The increase stays below the tenfold
budget; no gate trimming or parallel JSC VM execution is needed. Status remains
`ready-for-human`; this evidence does not change release ownership.
