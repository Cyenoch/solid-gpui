import type { PatchOperation, SnapshotNode } from "../protocol";
import type { NodeGraph } from "./nodes";
import type { HostNodeInternal } from "./types";

/**
 * Fenwick tree with exclusive-prefix queries: prefix(x) sums indexes < x.
 */
class RankIndex {
  private readonly tree: number[];

  constructor(size: number) {
    this.tree = new Array<number>(size + 1).fill(0);
  }
  add(index: number, delta: number): void {
    for (let i = index + 1; i < this.tree.length; i += i & -i) this.tree[i]! += delta;
  }
  prefix(index: number): number {
    let sum = 0;
    for (let i = Math.min(index, this.tree.length - 1); i > 0; i -= i & -i) sum += this.tree[i]!;
    return sum;
  }
}

/**
 * Plan sequential wire indexes against the published tree, not final Solid
 * indexes.
 *
 * Per touched parent: children whose published order already agrees with the
 * final order (one longest increasing subsequence of published positions) act
 * as fixed anchors and never move; every other child moves at most once,
 * immediately before the next anchor to its right (or the end). A right-to-left
 * walk over final slots with two Fenwick rank indexes — original positions that
 * still remain, and insertion buckets ahead of each anchor — derives each
 * emitted Move's exact sequential index with O(log n) rank arithmetic and no
 * rescans. A rotation collapses to a single Move and swapping the two endpoint
 * children costs two, regardless of parent size.
 */
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

  // Only touched parents are planned; each is planned once with two rank indexes.
  for (const [parentId, order] of children) {
    const parent = parentId === graph.syntheticRoot.id ? graph.syntheticRoot : graph.nodesById.get(parentId);
    if (parent === undefined) continue;
    const final = parent.children;
    const publishedPosition = new Map<number, number>();
    for (let index = 0; index < order.length; index++) publishedPosition.set(order[index]!, index);
    const positions = new Array<number>(final.length);
    for (let index = 0; index < final.length; index++) {
      const published = publishedPosition.get(final[index]!.id);
      if (published === undefined) throw new Error(`Structural patch lost child ${final[index]!.id}`);
      positions[index] = published;
    }
    let sorted = positions.length < 2;
    for (let index = 1; index < positions.length; index++) {
      if (positions[index]! <= positions[index - 1]!) {
        sorted = false;
        break;
      }
    }
    if (sorted) continue;

    // Patience LIS over `positions`; anchor slots keep their published relative order.
    const tails: number[] = [];
    const previous: number[] = new Array<number>(positions.length);
    for (let index = 0; index < positions.length; index++) {
      const value = positions[index]!;
      let low = 0;
      let high = tails.length;
      while (low < high) {
        const middle = (low + high) >> 1;
        if (positions[tails[middle]!]! < value) low = middle + 1;
        else high = middle;
      }
      previous[index] = low === 0 ? -1 : tails[low - 1]!;
      if (low === tails.length) tails.push(index);
      else tails[low] = index;
    }
    const anchored = new Array<boolean>(positions.length).fill(false);
    for (let index = tails[tails.length - 1]!; index !== -1; index = previous[index]!) anchored[index] = true;

    // Walk final slots right to left. `boundary` tracks the published position
    // of the anchor immediately right of the walk position; a moving child is
    // inserted immediately before it. `remaining` ranks untouched originals,
    // `inserted` counts children already moved into buckets ahead of anchors.
    const remaining = new RankIndex(order.length);
    const inserted = new RankIndex(order.length + 1);
    for (let index = 0; index < order.length; index++) remaining.add(index, 1);
    let boundary = order.length;
    for (let index = positions.length - 1; index >= 0; index--) {
      const published = positions[index]!;
      if (anchored[index]) {
        boundary = published;
        continue;
      }
      const source = remaining.prefix(published) + inserted.prefix(published + 1);
      remaining.add(published, -1);
      const destination = remaining.prefix(boundary) + inserted.prefix(boundary);
      inserted.add(boundary, 1);
      if (source !== destination) operations.push({ type: "move", id: final[index]!.id, parentId, index: destination });
    }
  }
  return operations;
}
