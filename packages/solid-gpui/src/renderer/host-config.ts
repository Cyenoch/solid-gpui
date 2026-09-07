import { getOwner } from "solid-js";
import type { RendererOptions } from "solid-js/universal";

import { UPDATE_TEXT } from "../protocol";
import { assertChildKind, propGroup, VALID_HOST_TYPES } from "./facts";
import { HostTree } from "./host-tree";
import type { HostKind, HostNodeInternal } from "./types";

type SolidOwner = { readonly owner?: SolidOwner | null };

const ownerTrees = new WeakMap<object, HostTree>();
const pendingCommits = new WeakMap<HostTree, object>();
let activeTree: HostTree | undefined;
const transactionDepths = new WeakMap<HostTree, number>();

function rememberOwner(tree: HostTree): void {
  const owner = getOwner() as SolidOwner | null;
  if (owner !== null) ownerTrees.set(owner as object, tree);
}

function ownerTree(): HostTree | undefined {
  let owner = getOwner() as SolidOwner | null;
  const currentOwner = owner;
  while (owner !== null) {
    const tree = ownerTrees.get(owner as object);
    if (tree !== undefined) {
      if (currentOwner !== null && currentOwner !== owner) {
        ownerTrees.set(currentOwner as object, tree);
      }
      return tree;
    }
    owner = owner.owner ?? null;
  }
  return undefined;
}

export function resolveTree(node?: HostNodeInternal): HostTree {
  if (node !== undefined) return node.root as HostTree;
  const owner = ownerTree();
  if (owner !== undefined) return owner;
  if (activeTree !== undefined) {
    rememberOwner(activeTree);
    return activeTree;
  }
  throw new Error("Solid GPUI host operation is not associated with a root");
}

export function withRoot<T>(tree: HostTree, callback: () => T): T {
  const previous = activeTree;
  activeTree = tree;
  rememberOwner(tree);
  try {
    return callback();
  } finally {
    activeTree = previous;
  }
}

export function withRootTransaction<T>(tree: HostTree, callback: () => T): T {
  return withRoot(tree, () => {
    transactionDepths.set(tree, (transactionDepths.get(tree) ?? 0) + 1);
    try {
      return callback();
    } finally {
      const depth = (transactionDepths.get(tree) ?? 1) - 1;
      if (depth === 0) transactionDepths.delete(tree);
      else transactionDepths.set(tree, depth);
    }
  });
}

export function cancelScheduledCommit(tree: HostTree): boolean {
  return pendingCommits.delete(tree);
}

/** Commands cross the same root transaction seam as renderer mutations. */
export function afterRootCommit<T>(tree: HostTree, submit: () => Promise<T>): Promise<T> {
  if (transactionDepths.has(tree)) return Promise.resolve().then(() => afterRootCommit(tree, submit));
  try {
    if (cancelScheduledCommit(tree)) tree.commit();
    if (tree.invalid || tree.validationError)
      throw tree.validationError ?? new Error("Native command follows an invalid commit");
    return submit();
  } catch (error) {
    return Promise.reject(error);
  }
}

function prepareMutation(tree: HostTree): void {
  if (tree.isDisposed() || pendingCommits.has(tree) || transactionDepths.has(tree)) return;
  tree.beginRender();
  const token = {};
  pendingCommits.set(tree, token);
  queueMicrotask(() => {
    if (pendingCommits.get(tree) !== token) return;
    pendingCommits.delete(tree);
    if (!tree.isDisposed()) tree.commit();
  });
}

function configure(node: HostNodeInternal): void {
  if (node.mounted) return;
  node.root.setNodeProps(node, node.props);
  node.mounted = true;
}

function parentDepth(parent: HostNodeInternal): number {
  return parent.kind === "Text" && parent.parent?.kind === "Text" ? 2 : 1;
}

function assertParentAndChild(parent: HostNodeInternal, child: HostNodeInternal): HostTree {
  const tree = parent.root as HostTree;
  if (child.root !== tree) {
    tree.invalid = true;
    throw new TypeError("cannot insert a host node from a different Solid GPUI root");
  }
  try {
    assertChildKind(parent.kind, child.kind, parentDepth(parent));
  } catch (error) {
    tree.invalid = true;
    throw error;
  }
  return tree;
}

function createElement(type: string): HostNodeInternal {
  const tree = resolveTree();
  prepareMutation(tree);
  if (!Object.hasOwn(VALID_HOST_TYPES, type)) {
    tree.invalid = true;
    throw new TypeError(`Unknown GPUI host type: ${type}`);
  }
  return tree.allocateNode(type as HostKind);
}

function createTextNode(value: string): HostNodeInternal {
  const tree = resolveTree();
  prepareMutation(tree);
  const node = tree.allocateNode("RawText");
  node.text = String(value);
  return node;
}

function replaceText(node: HostNodeInternal, value: string): void {
  const text = String(value);
  if (node.text === text) return;
  const tree = node.root as HostTree;
  if (node.attached) {
    prepareMutation(tree);
    tree.recordNodeMutation(node);
  }
  node.text = text;
  if (node.attached && node.mounted) tree.markUpdated(node, UPDATE_TEXT);
}

function setProperty<T>(node: HostNodeInternal, name: string, value: T): void {
  if (name === "key") return;
  const tree = node.root as HostTree;
  if (node.mounted) {
    prepareMutation(tree);
    tree.recordNodeMutation(node);
  }
  const props = node.props as Record<string, unknown>;
  if (value === undefined) delete props[name];
  else props[name] = value;
  if (node.mounted) tree.markPropsDirty(node, propGroup(node.kind, name));
}

function insertNode(parent: HostNodeInternal, node: HostNodeInternal, anchor?: HostNodeInternal): void {
  const tree = assertParentAndChild(parent, node);
  prepareMutation(tree);
  configure(node);
  tree.insertNode(parent, node, anchor);
}

function removeNode(parent: HostNodeInternal, node: HostNodeInternal): void {
  const tree = assertParentAndChild(parent, node);
  prepareMutation(tree);
  tree.removeNode(parent, node);
}

function isTextNode(node: HostNodeInternal): boolean {
  return node.kind === "RawText";
}

function getParentNode(node: HostNodeInternal): HostNodeInternal | undefined {
  if (!node.attached) return undefined;
  return node.parent ?? (node.root as HostTree).syntheticRoot;
}

function getFirstChild(node: HostNodeInternal): HostNodeInternal | undefined {
  return node.children[0];
}

function getNextSibling(node: HostNodeInternal): HostNodeInternal | undefined {
  const parent = getParentNode(node);
  if (parent === undefined) return undefined;
  return parent.children[node.index + 1];
}

export const hostConfig: RendererOptions<HostNodeInternal> = {
  createElement,
  createTextNode,
  replaceText,
  isTextNode,
  setProperty,
  insertNode,
  removeNode,
  getParentNode,
  getFirstChild,
  getNextSibling,
};
