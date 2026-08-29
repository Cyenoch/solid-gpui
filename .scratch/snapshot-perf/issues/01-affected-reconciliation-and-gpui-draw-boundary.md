# Affected reconciliation and GPUI draw boundary

## Summary

The initial snapshot audit found two separate costs for a 20,000-node tree: host snapshot construction/reconciliation and GPUI's first element/layout frame. A one-operation patch's earlier approximately 190 ms wall-clock measurement was draw-inclusive; the isolated host stages are sub-millisecond.

## Resolution

Patch reconciliation now propagates an affected node workset (including relevant ancestors and deleted subtrees) to input, selectable-text, virtual-list, animation, cache, and invalidation maintenance. Animation history is lazy for unstyled nodes. The guard records p50/p99 scaling for 1,000, 5,000, and 20,000 nodes and isolates host stage attribution.

## Boundary

The full first draw remains approximately 196 ms at 20,000 visible nodes. This is the GPUI element reconstruction/layout pipeline, not host reconciliation. Element-tree caching is intentionally out of scope: GPUI element identity, width-dependent layout, and frame lifecycle require a deeper architectural seam, and the measured host fix does not justify speculative caching. Consumers should use `VirtualList` to window large histories, file trees, and tables.

## Verification

- `cargo test -p react-gpui --lib snapshot_apply_and_first_draw_scaling_guard -- --nocapture`: passed; latest 20k snapshot host stages 70.705 ms and one-op patch host stages 0.131 ms.
- `cargo test -p react-gpui --lib`: passed (128 tests).
- `make ci`: passed.
- `make embedded-bun`: passed.
