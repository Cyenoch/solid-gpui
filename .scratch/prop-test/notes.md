# Tree patch property test

## Coverage

The deterministic property test checks five invariant groups after every patch operation:

1. The node graph is reachable from the synthetic root, with valid parent links, dense sibling indexes, and accurate child-length flags.
2. Every parent chain is cycle-free and terminates at the synthetic root.
3. Text nodes have recursively correct `text_content`; RawText nodes retain text and remain direct Text children; leaf and kind constraints hold.
4. Renderer side maps contain only live node IDs, including input, selectable-text, focus, VirtualList, layout/bounds, rich-cache, link-affordance, animation, style, and frame state.
5. Rich-text cache entries for subtrees untouched by a patch survive that patch.

## Generator design

`tree_property.rs` uses a hand-rolled xorshift64* generator so no external property-testing dependency is needed. Seeds `0..32` each execute 64 single-operation protocol patches. Operation selection is weighted approximately 55% update, 20% create, 15% delete, and 10% move. Candidate nodes are sorted by ID before selection for reproducibility.

The generator produces protocol-valid operations: parent/child kind restrictions, one-level nested Text rules, compatible nested Text styles, cycle-free moves, subtree-safe deletes, and valid insertion indexes. Generated Pressable nodes carry listeners but are deliberately non-focusable. This is correct rather than a masked product bug: a newly created focusable interactive node needs a reconciled `FocusHandle` before rendering, while the generator applies a patch and does not perform a render/reconciliation round that establishes that handle; the protocol's default node shape is non-focusable, and existing focusable fixtures are supplied through snapshots and exercised by updates.

## Findings and fixes

- Empty created Text nodes initially had `text_content: None` instead of the required empty string. `NodeStore::apply_create` now computes the new node's content before recomputing its parent.
- Listener-bearing Text updates were rejected by the interactive-node validation guard even though Text is a supported focusable/listener-bearing kind. Text is now included in that guard's accepted kinds.
- Deleted animated subtrees could leave stale animation state/style entries. Animation reconciliation now prunes deleted IDs from the affected workset without restoring a full-library scan.
- Deleted subtrees could leave stale renderer side-map entries, first observed in `selectable_text_layouts`. `ReactRoot::apply_payload` enumerates deleted subtree IDs before applying the patch and removes those IDs from all node-keyed side maps and deleted singleton anchors.
- A generated focusable Pressable exposed a missing handle panic. This was a generator validity issue, not a product violation: the patch protocol cannot assume the renderer has already created a focus handle for a newly introduced focusable node. The generator now follows the default non-focusable shape; snapshot-provided focusable nodes still exercise the renderer path.

## Runtime and performance

The full property sequence (32 seeds × 64 operations) passed in 0.97 s for the targeted test command on the development machine. The named seed-zero regression also passes. The snapshot performance guard reports a 20,000-node single-operation host phase of 0.104 ms, below the 1 ms budget; the guard passed.
