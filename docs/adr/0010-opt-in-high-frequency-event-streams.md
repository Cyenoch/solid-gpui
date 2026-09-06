# ADR-0010: Opt-in high-frequency event streams

- **Status:** Accepted
- **Date:** 2026-08-29

## Context

Pointer movement can arrive for every native mouse movement rather than only
at semantic interaction edges. Registering that listener and sending frames
for every View or Pressable would impose work on nodes that have no consumer,
then feed high-rate events through the synchronous renderer and commit path.
The event-storm audit in `.scratch/pointer-move/spec.md` and
`.scratch/perf-event-storm/spec.md` found the registered 143-node case within
its budget, while a 10,000-node stateful tree exceeds the 120 Hz frame interval
because of the Solid universal renderer/host commit boundary. The audit did not justify
implicit coalescing or a second delivery mechanism.

Pointer down/up coordinates, hover edges, and drag notifications have different
semantics. The implementation seams are
`packages/solid-gpui/src/renderer/nodes.ts`,
`packages/solid-gpui/src/protocol.ts`,
`crates/solid-gpui/src/tree.rs`,
`crates/solid-gpui/src/renderer/paint/mod.rs`,
`crates/solid-gpui/src/renderer/paint/drag.rs`, and
`crates/solid-gpui/src/renderer/events.rs`.

## Decision

- `onPointerMove` is an opt-in **Pointer-Move Capability**. Only a View or
  Pressable with that handler gets `acceptsPointerMove` in the retained node
  and a native move listener. Nodes without the capability register no native
  move listener and emit no move frames.
- Pointer-move data remains a tagged variant of the existing pointer event
  family: finite, viewport-clamped logical window coordinates and ordered
  modifier names, without button state. It uses the Surface's existing event
  sequence and listener routing.
- The host does not silently coalesce, drop, or reorder pointer-move events.
  Backpressure and the renderer/commit cost stay observable rather than being
  hidden behind a new policy that has not been requested by a consumer.
- Pointer down/up coordinates remain required event data, while hover remains
  an edge notification. Drag-over and drop use independent **Drag Capability
  Bits** (`acceptsDragOver` and `acceptsDrop`); active drag delivery is not
  reinterpreted as pointer movement.

## Alternatives rejected

- **Make pointer movement always-on:** rejected because every interactive node
  would pay native listener and transport cost even when no application reads
  the stream, and the 10,000-node stress evidence makes that cost material.
- **Reuse hover events for movement:** rejected because hover is an edge change
  with a null payload, whereas movement is an ordered coordinate stream with a
  different frequency and consumer contract.
- **Coalesce or drop movement in the host:** rejected because it would change
  the ordered Native Event contract and hide whether a renderer is keeping up;
  there is no consumer-specific sampling policy to preserve.
- **Use drag-over as the movement stream:** rejected because drag-over requires
  an active drag and a target capability, while ordinary pointer movement is
  useful without a drag and must carry its own listener identity.
- **Expose one unscoped per-Surface stream:** rejected because callbacks are
  node-scoped and capability registration is the cheap way to avoid routing
  unrelated high-rate traffic to JavaScript.

## Consequences

- Applications that need cursor tracking must opt in on each eligible View or
  Pressable and should treat the callback as a high-rate path. Removing the
  handler removes the native registration on the next commit.
- The protocol has an optional capability tail, but no second pointer event
  family or hidden sampling behavior. Pointer move, hover, and drag remain
  separately testable and separately documented.
- The capability validator must reject move registration on non-interactive
  nodes or without a listener. Drag targets can independently request drag-over
  and drop notifications without becoming drag sources.
- A future sampling, coalescing, or scheduler policy would alter observable
  event semantics and therefore requires a new ADR rather than an optimization
  hidden in the current path.
