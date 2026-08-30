# TypeScript commit emission property test

## Emission path

- `createRoot().render()` calls `beginRender()`, reconciler `updateContainerSync`, and `resetAfterCommit`; `hostConfig.resetAfterCommit` calls `RootContainer.commit()` once per completed React commit.
- `hostConfig.createInstance` allocates a `NodeGraph` host node, validates the parent/kind relation, and assembles props; `createTextInstance` allocates a `RawText` child only under `Text`.
- `hostConfig.append*`/`insert*` mutate parent arrays, detach a prior parent, refresh sibling indexes, and mark moved nodes after bootstrap; remove operations mark deleted roots and recursively detach their subtrees.
- `NodeGraph.setNodeProps` derives style, host properties, accessibility, focusability, interaction callbacks, and listener presence; listener IDs are allocated once, indexed in `listeners`/`inputListeners`, and removed when no callbacks remain or a subtree detaches.
- `RootContainer.commit` increments the u32 revision, emits a Snapshot `[base=0, revision=1]` on the first commit, then emits a Patch `[base=previous, revision=next]`; creates are parent-before-child, moves are depth ordered, updates/deletes are deterministic by ID, and mutation sets are cleared after submission.
- `NodeGraph.snapshotNodes` and patch assembly encode parent IDs, dense indexes, kind codes, optional rich-text tails, listener IDs, style, properties, accessibility, and RawText content; `encodeFrame` adds the bounded little-endian frame length.

## Invariants and code justification

1. Every outbound frame has a matching little-endian length and decodes through the TypeScript MessagePack decoder; `framePayload` writes `payload.byteLength`, while `FrameDecoder` emits only complete declared payloads.
2. Snapshot/Patch headers have seven fields, protocol v3, the expected kind, u32 surface/epoch/revisions, and a strictly increasing revision; `commit` computes `nextU32(baseRevision)` and uses the prior revision as the patch base.
3. Node IDs are unique within a snapshot and across live patch state; `NodeGraph.nodesById` is keyed by ID and `allocateNode` advances `nextNodeId` monotonically.
4. Every create and move references an existing parent and every update/delete references an existing live node; commit obtains creates/updates from `nodesById`, and native parent IDs come from the node's current parent or synthetic root.
5. The synthetic root is id 1, `View`, parent 0, index 0; `NodeGraph` constructs it once with those fields.
6. Every live node is reachable through parent children, has a reciprocal parent link, and sibling indexes are dense from zero; append/insert/remove call `refreshChildIndexes`, and snapshot traversal uses child indexes.
7. `Image` and `RawText` are leaves; RawText is direct under Text; Text children are only RawText or one-level nested Text runs; `assertChildKind` enforces these kind rules and `hostConfig.createTextInstance` enforces direct RawText placement.
8. A nested Text run cannot itself contain a nested Text run; host context tracks Text depth and rejects deeper nesting.
9. RawText carries text and non-RawText nodes carry no raw text on the wire; snapshot/patch assembly only exports `node.text` for RawText.
10. Active listener IDs are positive, unique, and attached only to live callback-bearing nodes; `setNodeProps` allocates/reuses IDs and removes them when callbacks disappear, while `detachSubtree` removes listener map entries.
11. Removed subtree listener IDs are revoked and never become active again; IDs are monotonically allocated and the property model records every deleted listener ID and listener update replacement.
12. Rich-text tails are structurally valid and preserve selectable/tooltip/pointer-move defaults; snapshot/patch assembly emits the optional fields in the protocol's positional order.

## Generator and harness

`renderer-property.test.tsx` uses a hand-rolled deterministic xorshift64\* generator matching the Rust constants. Seeds `0..23` each run 40 commits. The React tree starts with two View branches, normal Text, one nested Text run, a Pressable subtree, and an Image. Weighted mutations are approximately 55% restyle/text/listener update, 20% add, 15% delete, and 10% move/reorder fallback. Keys keep reconciliation identity stable. Moves reject cycles, Image parents, invalid Text depth, and incompatible rich-text destinations. Each commit calls the real exported `createRoot().render()` surface and reads the `MemoryTransport.submitted` outbound frame.

After every mutation, the test uses `FrameDecoder` and `decodeWireForGolden`, then performs a TypeScript encode/decode round trip, applies the operation to an independent wire model, and asserts IDs, parent references, dense indexes, kind/RawText/rich-text rules, listener allocation/revocation, and monotonic revisions. No private renderer state is inspected.

## Findings

No TS emission violation was observed across 24 seeds × 40 mutations (960 post-bootstrap commits, plus 24 snapshots). No source fix or seed-specific regression was necessary.

## Runtime

Targeted property command: `bun test tests/renderer-property.test.tsx`; 1 test passed with 362,716 assertions in 1.75 seconds. The full required Bun and Rust make gates are run by the integration owner after this test lands.
