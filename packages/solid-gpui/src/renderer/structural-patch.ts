import type { PatchOperation, SnapshotNode } from "../protocol";
import type { NodeGraph } from "./nodes";
import type { HostNodeInternal } from "./types";

/** Plan sequential wire indexes against the published tree, not final Solid indexes. */
export function structuralPatch(
  graph: NodeGraph,
  snapshotNode: (node: HostNodeInternal) => SnapshotNode,
): PatchOperation[] {
  const operations: PatchOperation[] = [];
  const children = new Map<number, number[]>();
  const parents = new Map<number, number>();
  const siblings = (parentId: number): number[] => {
    let value = children.get(parentId);
    if (value === undefined) {
      value = graph.publishedChildren(parentId).map((node) => node.id);
      children.set(parentId, value);
    }
    return value;
  };
  const parentOf = (id: number) => parents.get(id) ?? graph.publishedParentId(id);
  const detach = (id: number) => {
    const old = siblings(parentOf(id));
    const index = old.indexOf(id);
    if (index < 0) throw new Error(`Published parent does not contain node ${id}`);
    old.splice(index, 1);
  };
  const current = (ids: Set<number>) =>
    [...ids]
      .map((id) => graph.nodesById.get(id))
      .filter((node): node is HostNodeInternal => node !== undefined)
      .sort((a, b) => graph.nodeDepth(a) - graph.nodeDepth(b) || a.index - b.index || a.id - b.id);

  // Append first: final insertion indexes may depend on pending removals or moves.
  // Parents exist before children, including destinations for rescued descendants.
  for (const node of current(graph.createdIds)) {
    const parentId = graph.nativeParentId(node);
    const target = siblings(parentId);
    operations.push({ type: "create", node: { ...snapshotNode(node), index: target.length } });
    target.push(node.id);
    parents.set(node.id, parentId);
    children.set(node.id, []);
  }
  for (const node of current(graph.movedIds)) {
    if (graph.createdIds.has(node.id)) continue;
    const parentId = graph.nativeParentId(node);
    if (parentOf(node.id) !== parentId) {
      detach(node.id);
      const target = siblings(parentId);
      operations.push({ type: "move", id: node.id, parentId, index: target.length });
      target.push(node.id);
      parents.set(node.id, parentId);
    } else {
      siblings(parentId);
    }
  }
  for (const id of graph.deletedNodeIds()) {
    detach(id);
    operations.push({ type: "delete", id });
  }

  // Only touched parents are compared. Sliding windows keep overlapping rows in
  // order naturally; emit moves only when a remaining position actually differs.
  for (const [parentId, order] of children) {
    const parent = parentId === graph.syntheticRoot.id ? graph.syntheticRoot : graph.nodesById.get(parentId);
    if (parent === undefined) continue;
    for (let index = 0; index < parent.children.length; index++) {
      const id = parent.children[index]!.id;
      if (order[index] === id) continue;
      const from = order.indexOf(id, index + 1);
      if (from < 0) throw new Error(`Structural patch lost child ${id}`);
      order.splice(from, 1);
      order.splice(index, 0, id);
      operations.push({ type: "move", id, parentId, index });
    }
  }
  return operations;
}
