# ADR-0001: Batch renderer updates into GPUI

- **Status:** Accepted
- **Date:** 2026-08-24

## Context

SolidJS owns reactive behavior and component composition. GPUI owns main-thread native state and renders a validated retained tree. The runtime may live in another process or on an embedded Bun/JSC thread, so GPUI cannot observe in-progress JavaScript host mutations.

## Decision

Publish one atomic, versioned Commit Batch per completed host update. Protocol v5
encodes the first update for a surface epoch as a complete Snapshot and later
updates as incremental Patches.

The TypeScript `HostTree` mutates a private Host Node graph and finalizes a
coherent prop set once per transaction. Root renders, signal work, and batched
Native Event dispatch are explicit transactions; no partial host mutation is
published. Rust validates a complete Snapshot or Patch before replacing native
state.

## Consequences

- Process and embedded runtime adapters share the same byte contract.
- Revisions, stale-event rejection, and surface retirement are explicit.
- Validation failure leaves the last published native tree intact and terminates a protocol-divergent runtime.
- The host graph is an implementation boundary, not a browser tree or public compatibility layer.

- `SurfaceRouter` routes one decoded input chunk as one ordered semantic event
  batch per surface; `CommandClient` owns pending command completion.
- Rust's `NativeStateRegistry` and `CommitPump` keep native state and commit
  application on their owning foreground seam.
- Listener identities are selected by surface epoch, revision, and callback
  generation, with bounded previous-generation retention for in-flight events.
