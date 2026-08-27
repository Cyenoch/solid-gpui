# ADR-0008: Apply fail-fast and bounded degradation at different seams

- **Status:** Accepted
- **Date:** 2026-08-25

## Context

React GPUI has four distinct failure seams. A React render exception is a
JavaScript application failure; a Snapshot/Patch decode or tree-validation
failure means the two sides no longer share a provable protocol state; an
Image load failure is an expected external-resource failure; and a GPUI paint
panic occurs inside the native rendering lifecycle. StdioTransport also has a
normal temporary backpressure signal that must not be confused with a dead
peer.

The existing implementation and tests already establish different behavior at
each seam:

- `Root.render` records an unhandled reconciler error and throws it after the
  synchronous update; the Error Boundary tests prove a boundary can recover
  without an invalid frame, while an unbounded render throws
  (`packages/react-gpui/src/renderer.ts:167-192`,
  `packages/react-gpui/tests/renderer.test.tsx:528-583`).
- `ReactRoot::apply_payload` validates before applying a Snapshot/Patch and
  `NodeStore::apply_patch` rolls back a failed patch; the commit reader sends
  the error to `fatal_runtime_failure`, which shuts down the runtime and exits
  (`crates/react-gpui/src/renderer.rs:174-214`,
  `crates/react-gpui/src/renderer/commit_reader.rs:70-95`,
  `crates/react-gpui/src/tree.rs:310-326`,
  `crates/react-gpui/src/transport.rs:66-77`).
- Image resources are loaded through GPUI's image path/cache path; a missing or
  undecodable primary resource renders `fallbackSource` while loading/after
  failure when supplied, and otherwise produces blank output without a
  JavaScript failure (`crates/react-gpui/src/renderer/paint/image.rs:31-51`,
  `packages/react-gpui/README.md:134-153`).
- GPUI's `Window::draw` runs the element layout/prepaint/paint lifecycle without
  a node-level unwind boundary; the host panic hook records a crash report but
  does not resume a damaged draw (`references/zed/crates/gpui/src/window.rs:2851-2991`,
  `crates/react-gpui-host/src/main.rs:78-140`).
- StdioTransport queues a `write=false` result until `drain`, bounds the queue,
  and terminates on synchronous write errors or stream close/error. The
  backpressure tests cover ordering, repeated drain, overflow, EPIPE, and
  one-shot termination (`packages/react-gpui/src/transport.ts:163-278`,
  `packages/react-gpui/tests/transport.test.ts:116-279`).

## Decision

Use the narrowest safe behavior at each seam:

| Seam | Contract |
| --- | --- |
| React render error | A consumer Error Boundary owns recovery UI. Without a boundary, `root.render()` throws synchronously and no invalid Commit Batch is submitted. |
| Snapshot/Patch decode or tree validation error | Fail fast. Reject before mutation, preserve the last-good tree only as an implementation detail of validation/rollback, shut down the Runtime Adapter, and terminate the host. Do not retry, silently drop, or invent resync. |
| Image resource load error | Degrade locally to `fallbackSource` when supplied, otherwise blank image output. The retained tree and other nodes continue; this protocol version does not send an `Image` error Native Event. |
| GPUI paint panic/internal invariant | Treat as host-fatal. Do not catch and continue through a partially executed GPUI draw; the panic hook records diagnostics, then process/window termination is the safe outcome. |
| Temporary transport backpressure | Treat `write=false`/EAGAIN-style backpressure as normal: queue in order, retry from `drain`, and enforce the pending-byte cap. Queue overflow is an immediate caller-visible `RangeError`; EPIPE/close/error is transport termination. |

For a shared Runtime Adapter, a fatal commit or transport error closes all
registered Surfaces because they share the reader/runtime and there is no
per-Surface protocol resynchronization channel. A consumer that needs fault
isolation must use separate runtime adapters, not expect one root to recover a
shared stream.

## Alternatives rejected

- **Request a resync after a bad Snapshot/Patch:** rejected because protocol v3
  has no resync message or authenticated revision checkpoint. Once a frame is
  rejected, consuming later frames cannot prove a matching base revision; a
  resync protocol would be a new wire contract, not a local recovery tweak.
- **Ignore a bad frame and continue reading:** rejected because the next frame
  may depend on the rejected revision/tree, turning one known error into silent
  state divergence and misleading Native Events.
- **Catch GPUI paint panics and continue with the rest of the window:** rejected
  because GPUI's draw lifecycle mutates arena/dispatch/layout state across
  request-layout, prepaint, and paint. There is no verified unwind-safe
  node-level seam; continuing could present a half-mutated frame and corrupt
  subsequent draws.
- **Swallow an unhandled React render error or render an implicit fallback:**
  rejected because recovery UI is application policy and React Error Boundaries
  already provide the explicit scope. An implicit fallback would hide consumer
  bugs and change the synchronous `root.render` contract.
- **Retry every transport write on a timer:** rejected because the ordered
  `drain` queue is already the stream's temporary retry seam. A second retry
  loop would complicate queue bounds, shutdown, and duplicate-write reasoning.
- **Promote Image load failure to a surface-fatal error:** rejected because an
  external decorative resource must not discard an otherwise valid retained
  tree; `fallbackSource` contains the failure when supplied and blank output is
  the intentionally contained default otherwise.

## Consequences

- The host has a deliberately strict protocol/runtime failure boundary and a
  deliberately forgiving external-resource boundary; these must not be
  conflated in user-facing diagnostics.
- Error Boundaries, crash reports, transport termination callbacks, and Image
  fallback/blank output are separate mechanisms with separate owners.
- Adding a future recovery or resync protocol requires a new ADR and wire
  contract. It must not be introduced as an exception path inside the current
  fail-fast reader.
- The existing focused tests are the regression evidence for each contract;
  documentation should point consumers to those semantics rather than promise
  generic recovery.
