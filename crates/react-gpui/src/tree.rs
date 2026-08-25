use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use thiserror::Error;

use crate::protocol::{
    AccessibilityProperties, HostProperties, Node, PATCH_MESSAGE, PROTOCOL_VERSION, Patch,
    PatchOperation, SNAPSHOT_MESSAGE, Snapshot, Style, TRANSITION_BACKGROUND_COLOR,
    TRANSITION_HEIGHT, TRANSITION_OPACITY, TRANSITION_WIDTH, UPDATE_ACCESSIBILITY,
    UPDATE_FOCUSABLE, UPDATE_LISTENER, UPDATE_PROPERTIES, UPDATE_STYLE, UPDATE_TEXT,
};
pub const KIND_VIEW: u32 = 1;
pub const KIND_TEXT: u32 = 2;
pub const KIND_PRESSABLE: u32 = 3;
pub const KIND_RAW_TEXT: u32 = 4;
pub const KIND_TEXT_INPUT: u32 = 5;
pub const KIND_VIRTUAL_LIST: u32 = 6;
pub const KIND_IMAGE: u32 = 7;

#[derive(Debug, Error, Clone, PartialEq)]
pub enum TreeError {
    #[error("unsupported snapshot protocol {protocol} / message {message}")]
    InvalidProtocol { protocol: u32, message: u32 },
    #[error("snapshot revision {revision} must be greater than base revision {base_revision}")]
    InvalidRevisionOrder { base_revision: u32, revision: u32 },
    #[error(
        "snapshot base revision {base_revision} does not match current revision {current_revision}"
    )]
    BaseRevisionMismatch {
        base_revision: u32,
        current_revision: u32,
    },
    #[error("snapshot revision {revision} is not newer than current revision {current_revision}")]
    NonMonotonicRevision {
        revision: u32,
        current_revision: u32,
    },
    #[error(
        "surface {surface_id} epoch {epoch} does not match current surface {current_surface_id} epoch {current_epoch}"
    )]
    SurfaceMismatch {
        surface_id: u32,
        epoch: u32,
        current_surface_id: u32,
        current_epoch: u32,
    },
    #[error("node id 0 is reserved for the root parent sentinel")]
    ZeroNodeId,
    #[error("duplicate node id {0}")]
    DuplicateId(u32),
    #[error("node {node_id} refers to missing parent {parent_id}")]
    MissingParent { node_id: u32, parent_id: u32 },
    #[error("node {node_id} appears before its parent {parent_id}")]
    ParentAfterChild { node_id: u32, parent_id: u32 },
    #[error(
        "synthetic root must be View id 1 with parent 0 and index 0 (found id {node_id}, kind {kind})"
    )]
    InvalidSyntheticRoot { node_id: u32, kind: u32 },
    #[error("node id 1 is reserved for the permanent synthetic root")]
    ReservedRootId,
    #[error("user node ids must be greater than 1 (found {0})")]
    ReservedNodeId(u32),
    #[error("expected exactly one root, found {0}")]
    RootCount(usize),
    #[error("root node {node_id} must have child index 0, found {index}")]
    InvalidRootIndex { node_id: u32, index: u32 },
    #[error("node {node_id} has unsupported kind {kind}")]
    InvalidKind { node_id: u32, kind: u32 },
    #[error("node {node_id} has invalid text for kind {kind}")]
    InvalidText { node_id: u32, kind: u32 },
    #[error("node {node_id} has unsupported listener {listener_id}")]
    InvalidListener { node_id: u32, listener_id: u32 },
    #[error("node {node_id} has invalid child {child_id}: {reason}")]
    InvalidChild {
        node_id: u32,
        child_id: u32,
        reason: &'static str,
    },
    #[error("node {node_id} has non-contiguous child index {index}; expected {expected}")]
    NonContiguousChildIndex {
        node_id: u32,
        index: u32,
        expected: u32,
    },
    #[error("node tree is not reachable from root; first unreachable node is {0}")]
    Unreachable(u32),
    #[error("node {node_id} has invalid style: {reason}")]
    InvalidStyle { node_id: u32, reason: &'static str },
    #[error("patch protocol {protocol} / message {message} is invalid")]
    InvalidPatchProtocol { protocol: u32, message: u32 },
    #[error("patch requires a Snapshot bootstrap")]
    PatchRequiresSnapshot,
    #[error("patch operation {operation} is invalid: {reason}")]
    InvalidPatchOperation {
        operation: usize,
        reason: &'static str,
    },
    #[error("patch operation {operation} references missing node {node_id}")]
    MissingPatchNode { operation: usize, node_id: u32 },
    #[error("patch operation {operation} would create a cycle at node {node_id}")]
    PatchCycle { operation: usize, node_id: u32 },
    #[error("patch operation {operation} conflicts with node {node_id}")]
    PatchConflict { operation: usize, node_id: u32 },
    #[error("node {node_id} has invalid kind-specific properties: {reason}")]
    InvalidProperties { node_id: u32, reason: &'static str },
}

#[derive(Debug, Clone, PartialEq)]
pub struct StoredNode {
    pub id: u32,
    pub parent_id: u32,
    pub index: u32,
    pub kind: u32,
    pub style: Option<Style>,
    pub text: Option<Arc<str>>,
    pub text_content: Option<Arc<str>>,
    pub listener_id: u32,
    pub host_properties: Option<HostProperties>,
    pub accessibility: Option<AccessibilityProperties>,
    pub focusable: bool,
    pub accessibility_id: Arc<str>,
    child_len: usize,
}

impl StoredNode {
    pub fn children<'a>(&self, store: &'a NodeStore) -> impl Iterator<Item = &'a StoredNode> + 'a {
        store
            .children
            .get(&self.id)
            .into_iter()
            .flat_map(|ids| ids.iter())
            .filter_map(|id| store.nodes.get(id))
    }

    pub fn has_children(&self) -> bool {
        self.child_len != 0
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PatchStats {
    pub operation_count: u32,
    pub affected_nodes: u32,
    pub affected_parents: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NodeStore {
    surface_id: u32,
    epoch: u32,
    revision: u32,
    root_id: u32,
    pub(crate) nodes: HashMap<u32, StoredNode>,
    pub(crate) children: HashMap<u32, Vec<u32>>,
    last_patch_stats: PatchStats,
}

impl NodeStore {
    pub fn empty() -> Self {
        Self {
            surface_id: 0,
            epoch: 0,
            revision: 0,
            root_id: 1,
            nodes: HashMap::new(),
            children: HashMap::new(),
            last_patch_stats: PatchStats::default(),
        }
    }

    pub fn surface_id(&self) -> u32 {
        self.surface_id
    }
    pub fn epoch(&self) -> u32 {
        self.epoch
    }
    pub fn revision(&self) -> u32 {
        self.revision
    }
    pub fn len(&self) -> usize {
        self.nodes.len()
    }
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
    pub fn root(&self) -> Option<&StoredNode> {
        self.nodes.get(&self.root_id)
    }
    pub fn get(&self, id: u32) -> Option<&StoredNode> {
        self.nodes.get(&id)
    }
    pub fn get_child_at(&self, parent_id: u32, index: u32) -> Option<&StoredNode> {
        self.children
            .get(&parent_id)
            .and_then(|children| children.get(index as usize))
            .and_then(|id| self.nodes.get(id))
    }
    pub fn iter(&self) -> impl Iterator<Item = &StoredNode> {
        self.nodes.values()
    }
    pub fn last_patch_stats(&self) -> PatchStats {
        self.last_patch_stats
    }

    pub fn apply_snapshot(&mut self, snapshot: Snapshot) -> Result<(), TreeError> {
        let candidate = self.build_snapshot(snapshot)?;
        *self = candidate;
        Ok(())
    }

    fn build_snapshot(&self, snapshot: Snapshot) -> Result<Self, TreeError> {
        validate_snapshot_revision(self, &snapshot)?;
        if snapshot.protocol != PROTOCOL_VERSION || snapshot.message != SNAPSHOT_MESSAGE {
            return Err(TreeError::InvalidProtocol {
                protocol: snapshot.protocol,
                message: snapshot.message,
            });
        }
        let capacity = snapshot.nodes.len();
        let mut nodes = HashMap::with_capacity(capacity);
        let mut children: HashMap<u32, Vec<u32>> = HashMap::with_capacity(capacity);
        let mut root_id = None;
        for (position, node) in snapshot.nodes.into_iter().enumerate() {
            validate_node_shape(&node)?;
            if node.id == 0 {
                return Err(TreeError::ZeroNodeId);
            }
            if node.id == 1 && node.parent_id != 0 {
                return Err(TreeError::ReservedRootId);
            }
            if node.id != 1 && node.id < 2 {
                return Err(TreeError::ReservedNodeId(node.id));
            }
            if nodes.contains_key(&node.id) {
                return Err(TreeError::DuplicateId(node.id));
            }
            if node.parent_id == 0 {
                if node.index != 0 {
                    return Err(TreeError::InvalidRootIndex {
                        node_id: node.id,
                        index: node.index,
                    });
                }
                if root_id.replace(node.id).is_some() {
                    return Err(TreeError::RootCount(2));
                }
            } else {
                let Some(parent) = nodes.get(&node.parent_id) else {
                    return Err(TreeError::MissingParent {
                        node_id: node.id,
                        parent_id: node.parent_id,
                    });
                };
                if position == 0 || !nodes.contains_key(&node.parent_id) {
                    return Err(TreeError::ParentAfterChild {
                        node_id: node.id,
                        parent_id: node.parent_id,
                    });
                }
                validate_parent_child_stored(parent, &node)?;
                children.entry(node.parent_id).or_default().push(node.id);
            }
            validate_style(node.id, node.style.as_ref())?;
            nodes.insert(
                node.id,
                StoredNode {
                    id: node.id,
                    parent_id: node.parent_id,
                    index: node.index,
                    kind: node.kind,
                    style: node.style,
                    text: node.text.map(Arc::<str>::from),
                    text_content: None,
                    listener_id: node.listener_id,
                    host_properties: node.host_properties.clone(),
                    accessibility: node.accessibility.clone(),
                    focusable: node.focusable,
                    accessibility_id: Arc::<str>::from(format!("react-gpui-node-{}", node.id)),
                    child_len: 0,
                },
            );
        }
        let root_id = root_id.ok_or(TreeError::RootCount(0))?;
        let root = nodes.get(&root_id).expect("root inserted");
        if root.id != 1 || root.kind != KIND_VIEW {
            return Err(TreeError::InvalidSyntheticRoot {
                node_id: root.id,
                kind: root.kind,
            });
        }
        validate_child_indexes(&nodes, &mut children)?;
        for (parent_id, siblings) in &children {
            if let Some(parent) = nodes.get_mut(parent_id) {
                parent.child_len = siblings.len();
            }
        }
        recompute_all_text_content(&mut nodes, &children)?;
        validate_reachable(&nodes, &children, root_id)?;
        Ok(Self {
            surface_id: snapshot.surface_id,
            epoch: snapshot.epoch,
            revision: snapshot.revision,
            root_id,
            nodes,
            children,
            last_patch_stats: PatchStats::default(),
        })
    }

    pub fn apply_patch(&mut self, patch: Patch) -> Result<(), TreeError> {
        validate_patch_header(self, &patch)?;
        let mut undo = Vec::new();
        let mut stats = PatchStats {
            operation_count: patch.operations.len() as u32,
            ..PatchStats::default()
        };
        let result = self.apply_patch_inner(&patch.operations, &mut undo, &mut stats);
        if let Err(error) = result {
            rollback(self, undo);
            return Err(error);
        }
        self.revision = patch.revision;
        self.last_patch_stats = stats;
        Ok(())
    }

    fn apply_patch_inner(
        &mut self,
        operations: &[PatchOperation],
        undo: &mut Vec<Undo>,
        stats: &mut PatchStats,
    ) -> Result<(), TreeError> {
        let mut affected_parents = HashSet::new();
        for (operation_index, operation) in operations.iter().enumerate() {
            match operation {
                PatchOperation::Create(node) => {
                    self.apply_create(operation_index, node, undo, stats, &mut affected_parents)?
                }
                PatchOperation::Update {
                    id,
                    mask,
                    style,
                    text,
                    listener_id,
                    host_properties,
                    accessibility,
                    focusable,
                } => self.apply_update(
                    operation_index,
                    *id,
                    *mask,
                    style.clone(),
                    text.clone(),
                    *listener_id,
                    host_properties.clone(),
                    accessibility.clone(),
                    *focusable,
                    undo,
                    stats,
                    &mut affected_parents,
                )?,
                PatchOperation::Move {
                    id,
                    parent_id,
                    index,
                } => self.apply_move(
                    operation_index,
                    *id,
                    *parent_id,
                    *index,
                    undo,
                    stats,
                    &mut affected_parents,
                )?,
                PatchOperation::Delete { id } => {
                    self.apply_delete(operation_index, *id, undo, stats, &mut affected_parents)?
                }
            }
        }
        stats.affected_parents = affected_parents.len() as u32;
        Ok(())
    }

    fn apply_create(
        &mut self,
        operation: usize,
        node: &Node,
        undo: &mut Vec<Undo>,
        stats: &mut PatchStats,
        parents: &mut HashSet<u32>,
    ) -> Result<(), TreeError> {
        if node.id < 2 || self.nodes.contains_key(&node.id) {
            return Err(TreeError::PatchConflict {
                operation,
                node_id: node.id,
            });
        }
        let parent = self
            .nodes
            .get(&node.parent_id)
            .ok_or(TreeError::MissingPatchNode {
                operation,
                node_id: node.parent_id,
            })?;
        validate_node_shape(node).map_err(|_| TreeError::InvalidPatchOperation {
            operation,
            reason: "invalid created node",
        })?;
        validate_parent_child_stored(parent, node).map_err(|_| {
            TreeError::InvalidPatchOperation {
                operation,
                reason: "invalid parent/child relationship",
            }
        })?;
        validate_style(node.id, node.style.as_ref()).map_err(|_| {
            TreeError::InvalidPatchOperation {
                operation,
                reason: "invalid style",
            }
        })?;
        let siblings = self
            .children
            .get(&node.parent_id)
            .cloned()
            .unwrap_or_default();
        if node.index as usize > siblings.len() {
            return Err(TreeError::InvalidPatchOperation {
                operation,
                reason: "child index is out of bounds",
            });
        }
        undo.push(Undo::with_immediate(self, &[node.parent_id]));
        let stored = StoredNode {
            id: node.id,
            parent_id: node.parent_id,
            index: node.index,
            kind: node.kind,
            style: node.style.clone(),
            text: node.text.clone().map(Arc::<str>::from),
            text_content: None,
            listener_id: node.listener_id,
            host_properties: node.host_properties.clone(),
            accessibility: node.accessibility.clone(),
            focusable: node.focusable,
            accessibility_id: Arc::<str>::from(format!("react-gpui-node-{}", node.id)),
            child_len: 0,
        };
        self.nodes.insert(node.id, stored);
        self.children.insert(node.id, Vec::new());
        self.children
            .entry(node.parent_id)
            .or_default()
            .insert(node.index as usize, node.id);
        self.reindex_parent(node.parent_id);
        self.recompute_text_content(node.parent_id)?;
        stats.affected_nodes += 1;
        parents.insert(node.parent_id);
        Ok(())
    }

    // Patch fields stay positional to mirror protocol validation; bookkeeping
    // references are kept explicit for atomic rollback.
    #[allow(clippy::too_many_arguments)]
    fn apply_update(
        &mut self,
        operation: usize,
        id: u32,
        mask: u32,
        style: Option<Style>,
        text: Option<String>,
        listener_id: u32,
        host_properties: Option<HostProperties>,
        accessibility: Option<AccessibilityProperties>,
        focusable: bool,
        undo: &mut Vec<Undo>,
        stats: &mut PatchStats,
        parents: &mut HashSet<u32>,
    ) -> Result<(), TreeError> {
        if mask == 0
            || mask
                & !(UPDATE_STYLE
                    | UPDATE_TEXT
                    | UPDATE_LISTENER
                    | UPDATE_PROPERTIES
                    | UPDATE_ACCESSIBILITY
                    | UPDATE_FOCUSABLE)
                != 0
        {
            return Err(TreeError::InvalidPatchOperation {
                operation,
                reason: "invalid update mask",
            });
        }
        let node = self
            .nodes
            .get(&id)
            .ok_or(TreeError::MissingPatchNode {
                operation,
                node_id: id,
            })?
            .clone();
        if mask & UPDATE_TEXT != 0 && node.kind != KIND_RAW_TEXT {
            return Err(TreeError::InvalidPatchOperation {
                operation,
                reason: "text updates require RawText",
            });
        }
        if mask & UPDATE_FOCUSABLE != 0 && node.kind != KIND_VIEW {
            return Err(TreeError::InvalidPatchOperation {
                operation,
                reason: "focusable updates require View",
            });
        }
        let resulting_listener = if mask & UPDATE_LISTENER != 0 {
            listener_id
        } else {
            node.listener_id
        };
        let resulting_style = if mask & UPDATE_STYLE != 0 {
            style.as_ref()
        } else {
            node.style.as_ref()
        };
        if resulting_listener != 0
            && node.kind != KIND_PRESSABLE
            && node.kind != KIND_TEXT_INPUT
            && node.kind != KIND_VIRTUAL_LIST
            && node.kind != KIND_VIEW
            && resulting_style
                .and_then(|style| style.transition.as_ref())
                .is_none()
        {
            return Err(TreeError::InvalidPatchOperation {
                operation,
                reason: "listener updates require an interactive or animated node",
            });
        }
        if mask & UPDATE_PROPERTIES != 0 {
            validate_host_properties_shape(id, node.kind, host_properties.as_ref()).map_err(
                |_| TreeError::InvalidPatchOperation {
                    operation,
                    reason: "invalid host properties",
                },
            )?;
        }
        if mask & UPDATE_ACCESSIBILITY != 0 {
            validate_accessibility_shape(id, accessibility.as_ref()).map_err(|_| {
                TreeError::InvalidPatchOperation {
                    operation,
                    reason: "invalid accessibility properties",
                }
            })?;
        }
        if mask & UPDATE_ACCESSIBILITY != 0
            && accessibility.is_none()
            && node.accessibility.is_some()
        {
            return Err(TreeError::InvalidPatchOperation {
                operation,
                reason: "accessibility update cannot remove required role",
            });
        }
        if mask & UPDATE_STYLE != 0 {
            validate_style(id, style.as_ref()).map_err(|_| TreeError::InvalidPatchOperation {
                operation,
                reason: "invalid style",
            })?;
        }
        undo.push(Undo::state(self, &[id, node.parent_id]));
        let target = self.nodes.get_mut(&id).expect("validated node");
        if mask & UPDATE_STYLE != 0 {
            target.style = style;
        }
        if mask & UPDATE_FOCUSABLE != 0 {
            target.focusable = focusable;
        }
        if mask & UPDATE_LISTENER != 0 {
            target.listener_id = listener_id;
        }
        if mask & UPDATE_PROPERTIES != 0 {
            target.host_properties = host_properties;
        }
        if mask & UPDATE_ACCESSIBILITY != 0 {
            target.accessibility = accessibility;
        }
        if mask & UPDATE_TEXT != 0 {
            target.text = text.map(Arc::<str>::from);
        }
        if mask & UPDATE_TEXT != 0 {
            self.recompute_text_content(node.parent_id)?;
            parents.insert(node.parent_id);
        }
        stats.affected_nodes += 1;
        Ok(())
    }

    // Patch fields stay positional to mirror protocol validation; bookkeeping
    // references are kept explicit for atomic rollback.
    #[allow(clippy::too_many_arguments)]
    fn apply_move(
        &mut self,
        operation: usize,
        id: u32,
        parent_id: u32,
        index: u32,
        undo: &mut Vec<Undo>,
        stats: &mut PatchStats,
        parents: &mut HashSet<u32>,
    ) -> Result<(), TreeError> {
        if id == self.root_id {
            return Err(TreeError::PatchConflict {
                operation,
                node_id: id,
            });
        }
        let node = self
            .nodes
            .get(&id)
            .cloned()
            .ok_or(TreeError::MissingPatchNode {
                operation,
                node_id: id,
            })?;
        let parent = self
            .nodes
            .get(&parent_id)
            .cloned()
            .ok_or(TreeError::MissingPatchNode {
                operation,
                node_id: parent_id,
            })?;
        validate_parent_child_kinds(parent.id, parent.kind, id, node.kind).map_err(|_| {
            TreeError::InvalidPatchOperation {
                operation,
                reason: "invalid parent/child relationship",
            }
        })?;
        let mut ancestor = parent_id;
        while ancestor != 0 {
            if ancestor == id {
                return Err(TreeError::PatchCycle {
                    operation,
                    node_id: id,
                });
            }
            ancestor = self.nodes.get(&ancestor).map(|n| n.parent_id).unwrap_or(0);
        }
        undo.push(Undo::with_immediate(self, &[node.parent_id, parent_id]));
        let old_parent = node.parent_id;
        let old_siblings =
            self.children
                .get_mut(&old_parent)
                .ok_or(TreeError::MissingPatchNode {
                    operation,
                    node_id: old_parent,
                })?;
        let old_index =
            old_siblings
                .iter()
                .position(|child| *child == id)
                .ok_or(TreeError::PatchConflict {
                    operation,
                    node_id: id,
                })?;
        old_siblings.remove(old_index);
        if old_parent == parent_id && index as usize > old_siblings.len() {
            return Err(TreeError::InvalidPatchOperation {
                operation,
                reason: "child index is out of bounds",
            });
        }
        let target = self.children.entry(parent_id).or_default();
        let target_index = index as usize;
        if target_index > target.len() {
            return Err(TreeError::InvalidPatchOperation {
                operation,
                reason: "child index is out of bounds",
            });
        }
        target.insert(target_index, id);
        self.nodes.get_mut(&id).expect("validated node").parent_id = parent_id;
        self.reindex_parent(old_parent);
        self.reindex_parent(parent_id);
        self.recompute_text_content(old_parent)?;
        self.recompute_text_content(parent_id)?;
        parents.insert(old_parent);
        parents.insert(parent_id);
        stats.affected_nodes += 1;
        Ok(())
    }

    fn apply_delete(
        &mut self,
        operation: usize,
        id: u32,
        undo: &mut Vec<Undo>,
        stats: &mut PatchStats,
        parents: &mut HashSet<u32>,
    ) -> Result<(), TreeError> {
        if id == self.root_id {
            return Err(TreeError::PatchConflict {
                operation,
                node_id: id,
            });
        }
        let root = self
            .nodes
            .get(&id)
            .cloned()
            .ok_or(TreeError::MissingPatchNode {
                operation,
                node_id: id,
            })?;
        let mut subtree = Vec::new();
        let mut stack = vec![id];
        while let Some(current) = stack.pop() {
            subtree.push(current);
            stack.extend(self.children.get(&current).cloned().unwrap_or_default());
        }
        undo.push(Undo::with_subtree(self, &subtree, root.parent_id));
        let siblings = self
            .children
            .get_mut(&root.parent_id)
            .expect("parent exists");
        let Some(position) = siblings.iter().position(|child| *child == id) else {
            return Err(TreeError::PatchConflict {
                operation,
                node_id: id,
            });
        };
        siblings.remove(position);
        self.reindex_parent(root.parent_id);
        for child in &subtree {
            self.nodes.remove(child);
            self.children.remove(child);
        }
        self.recompute_text_content(root.parent_id)?;
        parents.insert(root.parent_id);
        stats.affected_nodes += subtree.len() as u32;
        Ok(())
    }

    fn reindex_parent(&mut self, parent_id: u32) {
        let children = self.children.get(&parent_id).cloned().unwrap_or_default();
        if let Some(parent) = self.nodes.get_mut(&parent_id) {
            parent.child_len = children.len();
        }
        for (index, child_id) in children.into_iter().enumerate() {
            if let Some(child) = self.nodes.get_mut(&child_id) {
                child.index = index as u32;
            }
        }
    }

    fn recompute_text_content(&mut self, node_id: u32) -> Result<(), TreeError> {
        let Some(node) = self.nodes.get(&node_id).cloned() else {
            return Ok(());
        };
        if node.kind != KIND_TEXT {
            return Ok(());
        }
        let mut content = String::new();
        for child_id in self.children.get(&node_id).cloned().unwrap_or_default() {
            let child = self.nodes.get(&child_id).ok_or(TreeError::MissingParent {
                node_id: child_id,
                parent_id: node_id,
            })?;
            let Some(text) = child.text.as_deref() else {
                return Err(TreeError::InvalidChild {
                    node_id,
                    child_id,
                    reason: "Text child must carry text",
                });
            };
            content.push_str(text);
        }
        self.nodes
            .get_mut(&node_id)
            .expect("validated node")
            .text_content = Some(Arc::<str>::from(content));
        Ok(())
    }
}

#[derive(Debug)]
enum Undo {
    State {
        nodes: Vec<(u32, Option<StoredNode>)>,
        children: Vec<(u32, Option<Vec<u32>>)>,
    },
}

impl Undo {
    fn state(store: &NodeStore, ids: &[u32]) -> Self {
        let mut unique = HashSet::new();
        let mut nodes = Vec::new();
        let mut children = Vec::new();
        for id in ids {
            if unique.insert(*id) {
                nodes.push((*id, store.nodes.get(id).cloned()));
                children.push((*id, store.children.get(id).cloned()));
            }
        }
        Self::State { nodes, children }
    }

    fn with_immediate(store: &NodeStore, parents: &[u32]) -> Self {
        let mut ids = parents.to_vec();
        for parent in parents {
            ids.extend(store.children.get(parent).cloned().unwrap_or_default());
        }
        Self::state(store, &ids)
    }

    fn with_subtree(store: &NodeStore, subtree: &[u32], parent: u32) -> Self {
        let mut ids = subtree.to_vec();
        ids.push(parent);
        let mut state = Self::with_immediate(store, &[parent]);
        let extra = Self::state(store, &ids);
        let Undo::State { nodes, children } = &mut state;
        let Undo::State {
            nodes: extra_nodes,
            children: extra_children,
        } = extra;
        let mut seen: HashSet<u32> = nodes.iter().map(|(id, _)| *id).collect();
        for entry in extra_nodes {
            if seen.insert(entry.0) {
                nodes.push(entry);
            }
        }
        let mut seen_children: HashSet<u32> = children.iter().map(|(id, _)| *id).collect();
        for entry in extra_children {
            if seen_children.insert(entry.0) {
                children.push(entry);
            }
        }
        state
    }
}

fn rollback(store: &mut NodeStore, undo: Vec<Undo>) {
    for entry in undo.into_iter().rev() {
        match entry {
            Undo::State { nodes, children } => {
                for (id, node) in nodes {
                    if let Some(node) = node {
                        store.nodes.insert(id, node);
                    } else {
                        store.nodes.remove(&id);
                    }
                }
                for (id, list) in children {
                    if let Some(list) = list {
                        store.children.insert(id, list);
                    } else {
                        store.children.remove(&id);
                    }
                }
            }
        }
    }
}

impl Default for NodeStore {
    fn default() -> Self {
        Self::empty()
    }
}

fn validate_snapshot_revision(current: &NodeStore, snapshot: &Snapshot) -> Result<(), TreeError> {
    if snapshot.revision <= snapshot.base_revision {
        return Err(TreeError::InvalidRevisionOrder {
            base_revision: snapshot.base_revision,
            revision: snapshot.revision,
        });
    }
    if current.is_empty() {
        if snapshot.base_revision != 0 {
            return Err(TreeError::BaseRevisionMismatch {
                base_revision: snapshot.base_revision,
                current_revision: 0,
            });
        }
        return Ok(());
    }
    if snapshot.surface_id != current.surface_id || snapshot.epoch != current.epoch {
        if snapshot.base_revision != 0 {
            return Err(TreeError::SurfaceMismatch {
                surface_id: snapshot.surface_id,
                epoch: snapshot.epoch,
                current_surface_id: current.surface_id,
                current_epoch: current.epoch,
            });
        }
        return Ok(());
    }
    if snapshot.base_revision != current.revision {
        return Err(TreeError::BaseRevisionMismatch {
            base_revision: snapshot.base_revision,
            current_revision: current.revision,
        });
    }
    if snapshot.revision <= current.revision {
        return Err(TreeError::NonMonotonicRevision {
            revision: snapshot.revision,
            current_revision: current.revision,
        });
    }
    Ok(())
}

fn validate_patch_header(current: &NodeStore, patch: &Patch) -> Result<(), TreeError> {
    if patch.protocol != PROTOCOL_VERSION || patch.message != PATCH_MESSAGE {
        return Err(TreeError::InvalidPatchProtocol {
            protocol: patch.protocol,
            message: patch.message,
        });
    }
    if current.is_empty() {
        return Err(TreeError::PatchRequiresSnapshot);
    }
    if patch.surface_id != current.surface_id || patch.epoch != current.epoch {
        return Err(TreeError::SurfaceMismatch {
            surface_id: patch.surface_id,
            epoch: patch.epoch,
            current_surface_id: current.surface_id,
            current_epoch: current.epoch,
        });
    }
    if patch.base_revision != current.revision {
        return Err(TreeError::BaseRevisionMismatch {
            base_revision: patch.base_revision,
            current_revision: current.revision,
        });
    }
    if patch.revision <= patch.base_revision {
        return Err(TreeError::InvalidRevisionOrder {
            base_revision: patch.base_revision,
            revision: patch.revision,
        });
    }
    Ok(())
}

fn validate_node_shape(node: &Node) -> Result<(), TreeError> {
    if !matches!(
        node.kind,
        KIND_VIEW
            | KIND_TEXT
            | KIND_PRESSABLE
            | KIND_TEXT_INPUT
            | KIND_RAW_TEXT
            | KIND_VIRTUAL_LIST
            | KIND_IMAGE
    ) {
        return Err(TreeError::InvalidKind {
            node_id: node.id,
            kind: node.kind,
        });
    }
    if (node.kind == KIND_RAW_TEXT) != node.text.is_some()
        || (node.kind != KIND_RAW_TEXT && node.text.is_some())
    {
        return Err(TreeError::InvalidText {
            node_id: node.id,
            kind: node.kind,
        });
    }
    if node.focusable && !matches!(node.kind, KIND_VIEW | KIND_PRESSABLE) {
        return Err(TreeError::InvalidProperties {
            node_id: node.id,
            reason: "only View and Pressable nodes may be focusable",
        });
    }
    if node.listener_id != 0
        && node.kind != KIND_PRESSABLE
        && node.kind != KIND_TEXT_INPUT
        && node.kind != KIND_VIRTUAL_LIST
        && node.kind != KIND_VIEW
        && node
            .style
            .as_ref()
            .and_then(|style| style.transition.as_ref())
            .is_none()
    {
        return Err(TreeError::InvalidListener {
            node_id: node.id,
            listener_id: node.listener_id,
        });
    }
    validate_host_properties_shape(node.id, node.kind, node.host_properties.as_ref())?;
    validate_accessibility_shape(node.id, node.accessibility.as_ref())?;
    Ok(())
}
fn validate_accessibility_shape(
    node_id: u32,
    accessibility: Option<&AccessibilityProperties>,
) -> Result<(), TreeError> {
    let Some(accessibility) = accessibility else {
        return Ok(());
    };
    if accessibility.role > 6 {
        return Err(TreeError::InvalidProperties {
            node_id,
            reason: "unsupported accessibility role",
        });
    }
    if accessibility.checked.is_some() && accessibility.role != 5 {
        return Err(TreeError::InvalidProperties {
            node_id,
            reason: "checked requires checkbox role",
        });
    }
    Ok(())
}
fn validate_host_properties_shape(
    node_id: u32,
    kind: u32,
    host_properties: Option<&HostProperties>,
) -> Result<(), TreeError> {
    match (host_properties, kind) {
        (Some(HostProperties::Drag(drag)), KIND_VIEW | KIND_PRESSABLE) => {
            if drag.drag_type.as_ref().is_some_and(|value| {
                value.is_empty()
                    || value.chars().count() > 128
                    || value.chars().any(char::is_control)
            }) {
                return Err(TreeError::InvalidProperties {
                    node_id,
                    reason: "invalid drag type",
                });
            }
        }
        (Some(HostProperties::TextInput(input)), KIND_TEXT_INPUT) => {
            if input.marked_start.is_some() != input.marked_end.is_some()
                || input
                    .marked_start
                    .zip(input.marked_end)
                    .is_some_and(|(start, end)| start > end)
                || input.selection_start > input.selection_end
            {
                return Err(TreeError::InvalidProperties {
                    node_id,
                    reason: "invalid TextInput selection",
                });
            }
        }
        (Some(HostProperties::VirtualList(list)), KIND_VIRTUAL_LIST) => {
            if list.range_start > list.range_end
                || list.range_end > list.item_count
                || !list.estimated_item_size.is_finite()
                || list.estimated_item_size <= 0.0
            {
                return Err(TreeError::InvalidProperties {
                    node_id,
                    reason: "invalid VirtualList range or item size",
                });
            }
        }
        (Some(HostProperties::Image(image)), KIND_IMAGE) => {
            if image.source.is_empty()
                || image.source.len() > 1024
                || image.source.chars().any(char::is_control)
                || !(1..=5).contains(&image.object_fit)
            {
                return Err(TreeError::InvalidProperties {
                    node_id,
                    reason: "invalid Image source or object fit",
                });
            }
        }
        (None, KIND_TEXT_INPUT | KIND_VIRTUAL_LIST | KIND_IMAGE) => {
            return Err(TreeError::InvalidProperties {
                node_id,
                reason: "host node requires kind-specific properties",
            });
        }
        (Some(_), _) => {
            return Err(TreeError::InvalidProperties {
                node_id,
                reason: "host properties do not match node kind",
            });
        }
        (None, _) => {}
    }
    Ok(())
}

fn validate_parent_child_stored(parent: &StoredNode, child: &Node) -> Result<(), TreeError> {
    validate_parent_child_kinds(parent.id, parent.kind, child.id, child.kind)
}

fn validate_parent_child_kinds(
    parent_id: u32,
    parent_kind: u32,
    child_id: u32,
    child_kind: u32,
) -> Result<(), TreeError> {
    if parent_kind == KIND_IMAGE {
        return Err(TreeError::InvalidChild {
            node_id: parent_id,
            child_id,
            reason: "Image cannot contain children",
        });
    }
    if parent_kind == KIND_RAW_TEXT {
        return Err(TreeError::InvalidChild {
            node_id: parent_id,
            child_id,
            reason: "RawText cannot contain children",
        });
    }
    if child_kind == KIND_RAW_TEXT && parent_kind != KIND_TEXT {
        return Err(TreeError::InvalidChild {
            node_id: parent_id,
            child_id,
            reason: "RawText must be directly under Text",
        });
    }
    if parent_kind == KIND_TEXT && child_kind != KIND_RAW_TEXT {
        return Err(TreeError::InvalidChild {
            node_id: parent_id,
            child_id,
            reason: "Text may contain only RawText",
        });
    }
    Ok(())
}

fn validate_child_indexes(
    nodes: &HashMap<u32, StoredNode>,
    children: &mut HashMap<u32, Vec<u32>>,
) -> Result<(), TreeError> {
    for (parent_id, siblings) in children.iter_mut() {
        siblings.sort_unstable_by_key(|child_id| {
            nodes
                .get(child_id)
                .map(|node| node.index)
                .unwrap_or(u32::MAX)
        });
        for (expected, child_id) in siblings.iter().enumerate() {
            let index = nodes.get(child_id).expect("child exists").index;
            if index != expected as u32 {
                return Err(TreeError::NonContiguousChildIndex {
                    node_id: *parent_id,
                    index,
                    expected: expected as u32,
                });
            }
        }
    }
    Ok(())
}

fn recompute_all_text_content(
    nodes: &mut HashMap<u32, StoredNode>,
    children: &HashMap<u32, Vec<u32>>,
) -> Result<(), TreeError> {
    let text_ids: Vec<u32> = nodes
        .values()
        .filter(|node| node.kind == KIND_TEXT)
        .map(|node| node.id)
        .collect();
    for id in text_ids {
        let mut content = String::new();
        for child_id in children.get(&id).cloned().unwrap_or_default() {
            let child = nodes.get(&child_id).expect("child exists");
            if child.kind != KIND_RAW_TEXT || child.text.is_none() {
                return Err(TreeError::InvalidChild {
                    node_id: id,
                    child_id,
                    reason: "Text child must carry text",
                });
            }
            content.push_str(child.text.as_deref().expect("checked text"));
        }
        nodes.get_mut(&id).expect("text node exists").text_content =
            Some(Arc::<str>::from(content));
    }
    Ok(())
}

fn validate_reachable(
    nodes: &HashMap<u32, StoredNode>,
    children: &HashMap<u32, Vec<u32>>,
    root_id: u32,
) -> Result<(), TreeError> {
    let mut reachable = HashSet::new();
    let mut stack = vec![root_id];
    while let Some(id) = stack.pop() {
        if reachable.insert(id) {
            stack.extend(children.get(&id).cloned().unwrap_or_default());
        }
    }
    if let Some(id) = nodes.keys().find(|id| !reachable.contains(id)) {
        return Err(TreeError::Unreachable(*id));
    }
    Ok(())
}

fn validate_style(node_id: u32, style: Option<&Style>) -> Result<(), TreeError> {
    let Some(style) = style else { return Ok(()) };
    if style.flex_direction.is_some_and(|direction| direction > 2) {
        return Err(TreeError::InvalidStyle {
            node_id,
            reason: "flexDirection must be 0 (unset), 1 (row), or 2 (column)",
        });
    }
    if style.position.is_some_and(|position| position > 1) {
        return Err(TreeError::InvalidStyle {
            node_id,
            reason: "position must be 0 (relative) or 1 (absolute)",
        });
    }
    if style
        .overflow
        .is_some_and(|overflow| !(1..=3).contains(&overflow))
    {
        return Err(TreeError::InvalidStyle {
            node_id,
            reason: "overflow must be 1 (visible), 2 (hidden), or 3 (scroll)",
        });
    }
    if style
        .line_clamp
        .is_some_and(|line_clamp| !(1..=100).contains(&line_clamp))
    {
        return Err(TreeError::InvalidStyle {
            node_id,
            reason: "lineClamp must be 1..=100",
        });
    }
    if style
        .text_overflow
        .is_some_and(|text_overflow| !matches!(text_overflow, 1 | 2))
    {
        return Err(TreeError::InvalidStyle {
            node_id,
            reason: "textOverflow must be 1 (clip) or 2 (ellipsis)",
        });
    }
    if style
        .justify_content
        .is_some_and(|justify| !(1..=6).contains(&justify))
    {
        return Err(TreeError::InvalidStyle {
            node_id,
            reason: "justifyContent must be 1..6",
        });
    }
    if style
        .align_items
        .is_some_and(|align| !(1..=5).contains(&align))
    {
        return Err(TreeError::InvalidStyle {
            node_id,
            reason: "alignItems must be 1..5",
        });
    }
    if style
        .font_style
        .is_some_and(|font_style| !matches!(font_style, 0 | 1))
    {
        return Err(TreeError::InvalidStyle {
            node_id,
            reason: "fontStyle must be 0 (normal) or 1 (italic)",
        });
    }
    if style
        .text_decoration
        .is_some_and(|decoration| !(0..=2).contains(&decoration))
    {
        return Err(TreeError::InvalidStyle {
            node_id,
            reason: "textDecoration must be 0 (none), 1 (underline), or 2 (lineThrough)",
        });
    }
    if style
        .align_self
        .is_some_and(|align| !(1..=7).contains(&align))
    {
        return Err(TreeError::InvalidStyle {
            node_id,
            reason: "alignSelf must be 1..7",
        });
    }
    for value in [
        style.width,
        style.height,
        style.flex_grow,
        style.padding,
        style.gap,
        style.margin_top,
        style.margin_right,
        style.margin_bottom,
        style.margin_left,
        style.line_height,
        style.min_width,
        style.max_width,
        style.min_height,
        style.max_height,
        style.flex_shrink,
        style.border_radius,
        style.border_width,
        style.opacity,
    ]
    .into_iter()
    .flatten()
    {
        if !value.is_finite() || value < 0.0 {
            return Err(TreeError::InvalidStyle {
                node_id,
                reason: "numeric style values must be finite and non-negative",
            });
        }
    }
    for value in [style.left, style.top, style.right, style.bottom]
        .into_iter()
        .flatten()
    {
        if !value.is_finite() {
            return Err(TreeError::InvalidStyle {
                node_id,
                reason: "position insets must be finite",
            });
        }
    }
    if style
        .font_size
        .is_some_and(|size| !size.is_finite() || size <= 0.0)
    {
        return Err(TreeError::InvalidStyle {
            node_id,
            reason: "fontSize must be finite and positive",
        });
    }
    if style
        .font_weight
        .is_some_and(|weight| !matches!(weight, 400 | 500 | 600 | 700 | 900))
    {
        return Err(TreeError::InvalidStyle {
            node_id,
            reason: "fontWeight must be 400, 500, 600, 700, or 900",
        });
    }
    if style.opacity.is_some_and(|opacity| opacity > 1.0) {
        return Err(TreeError::InvalidStyle {
            node_id,
            reason: "opacity must be between 0 and 1",
        });
    }
    if let Some(transition) = style.transition.as_ref()
        && (transition.properties == 0
            || transition.properties
                & !(TRANSITION_OPACITY
                    | TRANSITION_BACKGROUND_COLOR
                    | TRANSITION_WIDTH
                    | TRANSITION_HEIGHT)
                != 0
            || transition.easing as u32 > 3)
    {
        return Err(TreeError::InvalidStyle {
            node_id,
            reason: "transition properties or easing are invalid",
        });
    }
    Ok(())
}
