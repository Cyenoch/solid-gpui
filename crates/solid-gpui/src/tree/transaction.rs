//! One atomic patch and its dependency closure, shared by all native consumers.

use super::{HostProperties, NodeStore, StoredNode};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Default)]
pub(crate) struct PatchChanges {
    /// Journaled nodes, including derived text and reindexed siblings.
    pub(crate) changed: HashSet<u32>,
    /// Deleted identities, including a delete followed by recreation in one patch.
    pub(crate) removed: HashSet<u32>,
    /// Changed nodes and their original/final ancestors.
    pub(crate) affected: HashSet<u32>,
    /// Composition readers and children whose typed ownership may have changed.
    pub(crate) validation: HashSet<u32>,
}

pub(super) struct PatchTransaction<'a> {
    pub(super) store: &'a mut NodeStore,
    pub(super) undo: Option<PatchUndo>,
}

impl<'a> PatchTransaction<'a> {
    pub(super) fn new(store: &'a mut NodeStore) -> Self {
        Self {
            store,
            undo: Some(PatchUndo::default()),
        }
    }

    pub(super) fn changes(&self) -> PatchChanges {
        let undo = self.undo.as_ref().expect("active transaction");
        let changed: HashSet<_> = undo.nodes.keys().copied().collect();
        let mut before = HashSet::new();
        let mut affected = HashSet::new();
        for &id in &changed {
            let mut current = id;
            while current != 0 && before.insert(current) {
                current = undo
                    .nodes
                    .get(&current)
                    .map(|node| node.as_ref())
                    .unwrap_or_else(|| self.store.get(current))
                    .map_or(0, |node| node.parent_id);
            }
            let mut current = id;
            while current != 0 && affected.insert(current) {
                current = self.store.get(current).map_or(0, |node| node.parent_id);
            }
        }
        affected.extend(before);
        let mut validation = affected.clone();
        let mut ownership = HashSet::new();
        ownership.extend(undo.children.keys().copied());
        for (&id, previous) in &undo.nodes {
            let Some(node) = self.store.get(id) else {
                continue;
            };
            if previous
                .as_ref()
                .is_none_or(|old| old.parent_id != node.parent_id)
                || matches!(node.host_properties, Some(HostProperties::Extension(_)))
                    && previous
                        .as_ref()
                        .is_none_or(|old| extension_identity(old) != extension_identity(node))
            {
                ownership.insert(id);
            }
        }
        // Typed children may be direct or inside a named slot. Their contract
        // reads parent/grandparent identity and the slot's current child index.
        for id in ownership {
            if let Some(node) = self.store.get(id) {
                for child in node.children(self.store) {
                    validation.insert(child.id);
                    validation.extend(child.children(self.store).map(|node| node.id));
                }
            }
        }
        PatchChanges {
            changed,
            removed: undo.removed.clone(),
            affected,
            validation,
        }
    }

    pub(super) fn commit(mut self) {
        self.undo = None;
    }
}

impl Drop for PatchTransaction<'_> {
    fn drop(&mut self) {
        if let Some(undo) = self.undo.take() {
            undo.rollback(self.store);
        }
    }
}

/// Capture pre-transaction values on first mutation, including absent identities.
/// Repeated edits share one saved value, and derived Text caches participate in
/// the same journal as structural changes. Rollback never rebuilds the whole tree.
#[derive(Debug, Default)]
pub(super) struct PatchUndo {
    pub(super) removed: HashSet<u32>,
    nodes: HashMap<u32, Option<StoredNode>>,
    children: HashMap<u32, Option<Vec<u32>>>,
}

impl PatchUndo {
    pub(super) fn capture_node(&mut self, store: &NodeStore, id: u32) {
        self.nodes
            .entry(id)
            .or_insert_with(|| store.nodes.get(&id).cloned());
    }

    pub(super) fn capture(&mut self, store: &NodeStore, id: u32) {
        self.capture_node(store, id);
        self.children
            .entry(id)
            .or_insert_with(|| store.children.get(&id).cloned());
    }

    pub(super) fn capture_parent(&mut self, store: &NodeStore, parent: u32, first_changed: usize) {
        self.capture(store, parent);
        for &child in store
            .children
            .get(&parent)
            .into_iter()
            .flatten()
            .skip(first_changed)
        {
            // Only the shifted suffix needs saved indexes. Appending to a wide
            // parent does not visit or clone its existing sibling nodes.
            self.capture_node(store, child);
        }
    }

    fn rollback(self, store: &mut NodeStore) {
        for (id, node) in self.nodes {
            if let Some(node) = node {
                store.nodes.insert(id, node);
            } else {
                store.nodes.remove(&id);
            }
        }
        for (id, children) in self.children {
            if let Some(children) = children {
                store.children.insert(id, children);
            } else {
                store.children.remove(&id);
            }
        }
    }
}

fn extension_identity(node: &StoredNode) -> Option<([u8; 16], [u8; 32], u32, u32)> {
    match &node.host_properties {
        Some(HostProperties::Extension(props)) => Some((
            props.provider_id,
            props.catalog_digest,
            props.entry_id,
            props.entry_version,
        )),
        _ => None,
    }
}
