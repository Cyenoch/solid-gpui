# ADR-0001: Batch React commits into GPUI

- **Status:** Accepted
- **Date:** 2026-08-24

## Decision

React/Bun own Fiber, hooks, context, fragments, JavaScript closures, and callback state. GPUI owns main-thread/native surface state and renders from a validated retained tree. The hard boundary is one atomic, versioned Commit Batch per completed React commit across `RuntimeAdapter`; protocol v3 encodes the first commit for an epoch as a complete Snapshot and later commits as incremental Patches, with tagged TextInput/VirtualList host properties, accessibility data, and native animation metadata.

The current adapter is `ProcessAdapter`, which transports framed MessagePack over a child process. Process and embedded runtimes remain Runtime Adapter choices; embedded Bun is a separate runtime implementation.

## Rationale

GPUI must receive and validate a coherent tree on its main thread, while React must retain ownership of JavaScript behavior and reconciliation. Atomic batching makes revisions, validation, stale-event checks, and unmount state explicit; per-prop synchronous FFI would expose intermediate mutation order, add call overhead, and couple GPUI state updates to Fiber's host mutation timing without improving the native rendering contract.
