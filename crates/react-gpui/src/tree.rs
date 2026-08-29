use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use thiserror::Error;

use crate::protocol::{
    AccessibilityProperties, HostProperties, Node, PATCH_MESSAGE, PROTOCOL_VERSION, Patch,
    PatchOperation, SNAPSHOT_MESSAGE, Snapshot, Style, TRANSITION_BACKGROUND_COLOR,
    TRANSITION_HEIGHT, TRANSITION_OPACITY, TRANSITION_WIDTH, UPDATE_ACCESSIBILITY,
    UPDATE_FOCUSABLE, UPDATE_LISTENER, UPDATE_POINTER_MOVE, UPDATE_PROPERTIES, UPDATE_SELECTABLE,
    UPDATE_STYLE, UPDATE_TEXT, UPDATE_TOOLTIP,
};
mod validation;
use validation::*;
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
    #[error("nested Text node {node_id} has unsupported style field {field}")]
    InvalidNestedTextStyle { node_id: u32, field: &'static str },
    #[error("Text node {node_id} cannot be selectable with nested styled Text runs")]
    InvalidRichTextSelection { node_id: u32 },
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
    pub selectable: bool,
    pub tooltip: Option<Arc<str>>,
    pub accepts_pointer_move: bool,
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
        let mut nodes: HashMap<u32, StoredNode> = HashMap::with_capacity(capacity);
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
                let parent_is_nested_text = parent.kind == KIND_TEXT
                    && parent.parent_id != 0
                    && nodes
                        .get(&parent.parent_id)
                        .is_some_and(|ancestor| ancestor.kind == KIND_TEXT);
                validate_parent_child_stored(parent, &node, parent_is_nested_text)?;
                children.entry(node.parent_id).or_default().push(node.id);
            }
            validate_style(node.id, node.style.as_ref())?;
            if node.kind == KIND_TEXT
                && nodes
                    .get(&node.parent_id)
                    .is_some_and(|parent| parent.kind == KIND_TEXT)
            {
                validate_nested_text_style(node.id, node.style.as_ref())?;
            }
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
                    selectable: node.selectable,
                    tooltip: node.tooltip.map(Arc::<str>::from),
                    accepts_pointer_move: node.accepts_pointer_move,
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
                    selectable,
                    tooltip,
                    accepts_pointer_move,
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
                    *selectable,
                    tooltip.clone(),
                    *accepts_pointer_move,
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
        if node.accepts_pointer_move
            && (node.kind != KIND_VIEW && node.kind != KIND_PRESSABLE || node.listener_id == 0)
        {
            return Err(TreeError::InvalidPatchOperation {
                operation,
                reason: "pointer move capability requires View or Pressable listener",
            });
        }
        validate_node_shape(node).map_err(|_| TreeError::InvalidPatchOperation {
            operation,
            reason: "invalid created node",
        })?;
        let parent_is_nested_text = parent.kind == KIND_TEXT
            && parent.parent_id != 0
            && self
                .nodes
                .get(&parent.parent_id)
                .is_some_and(|ancestor| ancestor.kind == KIND_TEXT);
        validate_parent_child_stored(parent, node, parent_is_nested_text).map_err(|_| {
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
            selectable: node.selectable,
            tooltip: node.tooltip.clone().map(Arc::<str>::from),
            accepts_pointer_move: node.accepts_pointer_move,
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
        selectable: bool,
        tooltip: Option<String>,
        accepts_pointer_move: bool,
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
                    | UPDATE_FOCUSABLE
                    | UPDATE_SELECTABLE
                    | UPDATE_TOOLTIP
                    | UPDATE_POINTER_MOVE)
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
        if mask & UPDATE_SELECTABLE != 0 && node.kind != KIND_TEXT {
            return Err(TreeError::InvalidPatchOperation {
                operation,
                reason: "selectable updates require Text",
            });
        }
        let parent = self.nodes.get(&node.parent_id);
        let is_nested_text =
            node.kind == KIND_TEXT && parent.is_some_and(|parent| parent.kind == KIND_TEXT);
        if is_nested_text {
            if mask & UPDATE_STYLE != 0 {
                validate_nested_text_style(id, style.as_ref()).map_err(|_| {
                    TreeError::InvalidPatchOperation {
                        operation,
                        reason: "nested Text style contains unsupported field",
                    }
                })?;
            }
            if mask & UPDATE_SELECTABLE != 0 && selectable {
                return Err(TreeError::InvalidPatchOperation {
                    operation,
                    reason: "nested Text cannot be selectable",
                });
            }
        } else if node.kind == KIND_TEXT
            && mask & UPDATE_SELECTABLE != 0
            && selectable
            && self
                .children
                .get(&id)
                .into_iter()
                .flatten()
                .any(|child_id| {
                    self.nodes
                        .get(child_id)
                        .is_some_and(|child| child.kind == KIND_TEXT)
                })
        {
            return Err(TreeError::InvalidPatchOperation {
                operation,
                reason: "Text with nested styled runs cannot be selectable",
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
        if mask & UPDATE_TOOLTIP != 0 && node.kind != KIND_VIEW && node.kind != KIND_PRESSABLE {
            return Err(TreeError::InvalidPatchOperation {
                operation,
                reason: "tooltip updates require View or Pressable",
            });
        }
        if mask & UPDATE_POINTER_MOVE != 0
            && (node.kind != KIND_VIEW && node.kind != KIND_PRESSABLE
                || accepts_pointer_move && resulting_listener == 0)
        {
            return Err(TreeError::InvalidPatchOperation {
                operation,
                reason: "pointer move capability requires View or Pressable listener",
            });
        }
        if mask & UPDATE_TOOLTIP != 0
            && tooltip.as_ref().is_some_and(|value| {
                value.is_empty() || value.len() > 256 || value.chars().any(char::is_control)
            })
        {
            return Err(TreeError::InvalidPatchOperation {
                operation,
                reason: "invalid tooltip",
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
        if mask & UPDATE_SELECTABLE != 0 {
            target.selectable = selectable;
        }
        if mask & UPDATE_LISTENER != 0 {
            target.listener_id = listener_id;
        }
        if mask & UPDATE_PROPERTIES != 0 {
            target.host_properties = host_properties;
        }
        if mask & UPDATE_TOOLTIP != 0 {
            target.tooltip = tooltip.map(Arc::<str>::from);
        }
        if mask & UPDATE_POINTER_MOVE != 0 {
            target.accepts_pointer_move = accepts_pointer_move;
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
        let parent_is_nested_text = parent.kind == KIND_TEXT
            && parent.parent_id != 0
            && self
                .nodes
                .get(&parent.parent_id)
                .is_some_and(|ancestor| ancestor.kind == KIND_TEXT);
        validate_parent_child_kinds(parent.id, parent.kind, id, node.kind).map_err(|_| {
            TreeError::InvalidPatchOperation {
                operation,
                reason: "invalid parent/child relationship",
            }
        })?;
        validate_nested_text_edge(
            &parent,
            id,
            node.kind,
            node.style.as_ref(),
            node.selectable,
            parent_is_nested_text,
        )
        .map_err(|_| TreeError::InvalidPatchOperation {
            operation,
            reason: "invalid parent/child relationship",
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
        let mut current = node_id;
        loop {
            let Some(node) = self.nodes.get(&current).cloned() else {
                return Ok(());
            };
            if node.kind == KIND_TEXT {
                let content = collect_text_content(current, &self.nodes, &self.children)?;
                self.nodes
                    .get_mut(&current)
                    .expect("validated text node")
                    .text_content = Some(Arc::<str>::from(content));
            }
            if node.parent_id == 0 {
                return Ok(());
            }
            current = node.parent_id;
        }
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
