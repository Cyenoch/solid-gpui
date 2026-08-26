use super::*;

pub(super) fn validate_snapshot_revision(
    current: &NodeStore,
    snapshot: &Snapshot,
) -> Result<(), TreeError> {
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

pub(super) fn validate_patch_header(current: &NodeStore, patch: &Patch) -> Result<(), TreeError> {
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

pub(super) fn validate_node_shape(node: &Node) -> Result<(), TreeError> {
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
pub(super) fn validate_accessibility_shape(
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
pub(super) fn validate_host_properties_shape(
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

pub(super) fn validate_parent_child_stored(
    parent: &StoredNode,
    child: &Node,
) -> Result<(), TreeError> {
    validate_parent_child_kinds(parent.id, parent.kind, child.id, child.kind)
}

pub(super) fn validate_parent_child_kinds(
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

pub(super) fn validate_child_indexes(
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

pub(super) fn recompute_all_text_content(
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

pub(super) fn validate_reachable(
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

pub(super) fn validate_style(node_id: u32, style: Option<&Style>) -> Result<(), TreeError> {
    let Some(style) = style else { return Ok(()) };
    if style.flex_direction.is_some_and(|direction| direction > 4) {
        return Err(TreeError::InvalidStyle {
            node_id,
            reason: "flexDirection must be 0 (unset), 1 row, 2 column, 3 row-reverse, or 4 column-reverse",
        });
    }
    if style.position.is_some_and(|position| position > 1) {
        return Err(TreeError::InvalidStyle {
            node_id,
            reason: "position must be 0 (relative) or 1 (absolute)",
        });
    }
    if style.cursor.is_some_and(|cursor| cursor > 18) {
        return Err(TreeError::InvalidStyle {
            node_id,
            reason: "cursor must be a supported style code",
        });
    }
    if style.text_align.is_some_and(|align| align > 3) {
        return Err(TreeError::InvalidStyle {
            node_id,
            reason: "textAlign must be 0 (unset), 1 left, 2 center, or 3 right",
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
