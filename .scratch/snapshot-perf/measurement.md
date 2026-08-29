# Initial Snapshot and First-Frame Performance Audit

## Scope and method

The existing headless `#[gpui::test]` renderer harness builds synthetic mixed trees and measures eight runs per size. Each tree contains `View` nodes, occasional styled `Text`/`RawText` runs, a selectable paragraph, and (for the 20,000-node shape) a `TextInput`, `VirtualList`, and focusable view. The guard records decode+apply wall time and the following draw wall time separately. A one-operation `UPDATE_TEXT` patch targets a rich-text raw run in a 20,000-node tree. Percentiles use the established rank convention.

The stage probe measures the host work inside the apply callback independently: protocol decode, `NodeStore` apply, each reconciliation sweep, and patch bookkeeping/invalidation. This separation matters: a wall-clock around `Entity::update` plus a draw includes GPUI's frame pipeline and is not a host-apply measurement.

## Scaling measurements

| Tree | Apply p50 / p99 (ms) | First draw p50 / p99 (ms) | Apply + draw p50 (ms) |
| ---: | ---: | ---: | ---: |
| 1,000 nodes | 12.890 / 13.070 | 9.419 / 9.664 | 22.309 |
| 5,000 nodes | 62.532 / 63.073 | 47.597 / 47.759 | 110.129 |
| 20,000 nodes | 259.019 / 259.656 | 195.747 / 198.639 | 454.766 |

The end-to-end wall time is approximately linear in this synthetic shape: the 20k apply is about 4.0x the 5k apply and the draw is about 4.1x. It nevertheless exceeds the defensible native-boot target of 100 ms by roughly 4.5x. The target is a UX threshold, not a claim that every visible 20k tree should be cheap; a native first frame should not spend multiple frame budgets before appearing.

## Host stage attribution

| Stage | 20k snapshot (ms) | 20k one-op patch (ms) | Complexity / allocation behavior |
| --- | ---: | ---: | --- |
| Snapshot decode | 28.255 | — | O(n) protocol decode; allocates decoded `Snapshot` nodes and strings |
| NodeStore apply | 34.433 | 0.051 | Snapshot O(n) maps plus validation; patch operation is O(affected) for this update |
| Input reconciliation | 1.122 | 0.005 | Snapshot scans store; patch uses affected workset |
| Selectable-text reconciliation | 0.337 | 0.001 | Snapshot scans store; patch uses affected workset |
| Virtual-list reconciliation | 0.394 | 0.007 | Snapshot scans state maps/store; patch uses affected workset |
| Animation reconciliation | 6.161 | 0.004 | Snapshot scans store and initializes styled animation states; patch uses affected workset |
| Patch bookkeeping | — | 0.005 | O(operations), plus bounded ancestor workset |
| Cache/layout invalidation | — | 0.031 | O(touched × ancestor depth) plus map retention |
| **Measured host stage total** | **70.705** | **0.131** | — |

The earlier approximately 190 ms “patch apply” number was an outer measurement that included the resulting full GPUI draw. The isolated one-operation host apply is approximately 0.2 ms (0.131 ms in the stage probe), not 190 ms.

Snapshot `recompute_all_text_content` collects text for each `Text` node and allocates an `Arc<str>` content value; with the one-level rich-text invariant this is linear in the text subtree and linear overall for this shape. `validate_child_indexes` sorts each sibling vector, bounded by the sum of per-parent `k log k`; `validate_reachable` is linear. The snapshot renderer then performs a separate full render traversal. The measured host sweeps are linear constants, not superlinear reconciliation.

## Fix

Patch reconciliation now honors its existing affected set. Input-state cleanup, selectable-text selection/layout maintenance, virtual-list maintenance, and animation-state maintenance no longer rescan the 20k store for a one-node patch. Patch bookkeeping also carries affected ancestors so a changed descendant still updates its relevant parent state. Animation style history is lazy: nodes without a style do not allocate an `animation_styles` entry. Deleted subtrees are included in the pre-patch affected workset for cleanup.

The guard asserts that the isolated affected patch host stage remains below 1 ms at 20k nodes and emits p50/p99 for both host apply and draw. It does not assert an end-to-end 100 ms limit because the draw path is owned by GPUI's frame architecture and is intentionally not cached here.

## Verdict and boundary

**Guard plus targeted correctness/performance fix; no element-tree cache.** The host-controlled 20k snapshot stages are approximately 71–78 ms in repeated probes, and the one-operation patch host stage is sub-millisecond. The remaining approximately 196 ms first draw is GPUI's full `AnyElement` reconstruction/layout pipeline. A 20k fully visible tree is itself an anti-pattern; `VirtualList` exists to window large histories, file trees, and tables so only the visible range reaches layout. Applications should keep committed snapshots windowed rather than treating 20k simultaneously visible nodes as the normal boot shape.

This boundary is analogous to the rich-text shape-cache audit: the host removed proven redundant assembly/scans, but does not add speculative element-tree or platform-layout caching. GPUI's element identity, width-dependent layout, and frame lifecycle make such a cache a deeper architectural seam; no measured evidence in this work justifies changing it. Future optimization of that draw cost belongs in GPUI's frame/layout architecture, not in this host patch.

## Reproduction

```text
cargo test -p react-gpui --lib snapshot_apply_and_first_draw_scaling_guard -- --nocapture
```

The test prints the scaling table and stage attribution under `perf_snapshot*` and `perf_patch_stage` prefixes. Latest captured run: snapshot-stage total 70.705 ms; patch-stage total 0.131 ms; 20k apply p50/p99 259.019/259.656 ms; first draw p50/p99 195.747/198.639 ms.
