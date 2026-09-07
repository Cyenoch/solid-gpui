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
            | KIND_EXTENSION
            | KIND_ICON
    ) {
        return Err(TreeError::InvalidKind {
            node_id: node.id,
            kind: node.kind,
        });
    }
    if (node.kind == KIND_RAW_TEXT) != node.text.is_some() {
        return Err(TreeError::InvalidText {
            node_id: node.id,
            kind: node.kind,
        });
    }
    validate_interaction(
        node.id,
        node.kind,
        node.listener_id,
        node.focusable,
        node.accepts_pointer_move,
        node.tooltip.as_deref(),
    )?;
    if node.selectable && node.kind != KIND_TEXT {
        return Err(TreeError::InvalidProperties {
            node_id: node.id,
            reason: "only Text nodes may be selectable",
        });
    }
    if node.listener_id != 0 && !supports_listener(node.kind, node.style.as_ref()) {
        return Err(TreeError::InvalidListener {
            node_id: node.id,
            listener_id: node.listener_id,
        });
    }
    validate_host_properties_shape(node.id, node.kind, node.host_properties.as_ref())?;
    validate_accessibility_shape(node.id, node.accessibility.as_ref())?;
    Ok(())
}

pub(super) fn validate_interaction(
    node_id: u32,
    kind: u32,
    listener_id: u32,
    focusable: bool,
    accepts_pointer_move: bool,
    tooltip: Option<&str>,
) -> Result<(), TreeError> {
    if focusable && !matches!(kind, KIND_VIEW | KIND_PRESSABLE | KIND_TEXT) {
        return Err(TreeError::InvalidProperties {
            node_id,
            reason: "only View, Pressable, and interactive Text nodes may be focusable",
        });
    }
    if focusable && kind == KIND_TEXT && listener_id == 0 {
        return Err(TreeError::InvalidProperties {
            node_id,
            reason: "Text nodes may be focusable only when they have an onPress listener",
        });
    }
    if accepts_pointer_move && (!matches!(kind, KIND_VIEW | KIND_PRESSABLE) || listener_id == 0) {
        return Err(TreeError::InvalidProperties {
            node_id,
            reason: "pointer move capability requires View or Pressable listener",
        });
    }
    if tooltip.is_some() && !matches!(kind, KIND_VIEW | KIND_PRESSABLE) {
        return Err(TreeError::InvalidProperties {
            node_id,
            reason: "tooltips require View or Pressable",
        });
    }
    if tooltip.is_some_and(|value| {
        value.is_empty() || value.len() > 256 || value.chars().any(char::is_control)
    }) {
        return Err(TreeError::InvalidProperties {
            node_id,
            reason: "invalid tooltip",
        });
    }
    Ok(())
}
pub(super) fn validate_accessibility_shape(
    node_id: u32,
    accessibility: Option<&AccessibilityProperties>,
) -> Result<(), TreeError> {
    let Some(accessibility) = accessibility else {
        return Ok(());
    };
    if accessibility.role > 7 {
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
    if accessibility.level.is_some() && accessibility.role != 6 {
        return Err(TreeError::InvalidProperties {
            node_id,
            reason: "level requires heading role",
        });
    }
    if accessibility
        .level
        .is_some_and(|level| level == 0 || usize::try_from(level).is_err())
    {
        return Err(TreeError::InvalidProperties {
            node_id,
            reason: "level must be a positive usize",
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
                || image.fallback_source.as_ref().is_some_and(|source| {
                    source.is_empty() || source.len() > 1024 || source.chars().any(char::is_control)
                })
                || !(1..=5).contains(&image.object_fit)
            {
                return Err(TreeError::InvalidProperties {
                    node_id,
                    reason: "invalid Image source, fallback, or object fit",
                });
            }
        }
        (Some(HostProperties::Icon(icon)), KIND_ICON) => {
            if icon.name.is_empty()
                || icon.name.len() > 128
                || icon.name.chars().any(char::is_control)
                || gpui_iconify::IconId::from_name(&icon.name).is_none()
                || !icon.size.is_finite()
                || icon.size <= 0.0
            {
                return Err(TreeError::InvalidProperties {
                    node_id,
                    reason: "invalid Icon name or size",
                });
            }
        }
        (Some(HostProperties::Extension(extension)), KIND_EXTENSION) => {
            if extension.entry_id == 0
                || extension.entry_version == 0
                || extension.event_ids.first() == Some(&0)
                || extension.event_ids.windows(2).any(|ids| ids[0] >= ids[1])
                || extension.fields.first().is_some_and(|field| field.id == 0)
                || extension
                    .fields
                    .windows(2)
                    .any(|fields| fields[0].id >= fields[1].id)
            {
                return Err(TreeError::InvalidProperties {
                    node_id,
                    reason: "invalid Extension entry, field, or event IDs",
                });
            }
        }
        (None, KIND_EXTENSION | KIND_TEXT_INPUT | KIND_VIRTUAL_LIST | KIND_IMAGE | KIND_ICON) => {
            return Err(TreeError::InvalidProperties {
                node_id,
                reason: "host node requires kind-specific properties",
            });
        }
        (None, _) => {}
        (Some(_), _) => {
            return Err(TreeError::InvalidProperties {
                node_id,
                reason: "host properties do not match node kind",
            });
        }
    }
    Ok(())
}

pub(super) fn validate_parent_child_stored(
    parent: &StoredNode,
    child: &Node,
    parent_is_nested_text: bool,
) -> Result<(), TreeError> {
    validate_parent_child_kinds(parent.id, parent.kind, child.id, child.kind)?;
    validate_nested_text_edge(
        parent,
        child.id,
        child.kind,
        child.style.as_ref(),
        child.selectable,
        parent_is_nested_text,
    )
}

pub(super) fn validate_nested_text_edge(
    parent: &StoredNode,
    child_id: u32,
    child_kind: u32,
    child_style: Option<&Style>,
    child_selectable: bool,
    parent_is_nested_text: bool,
) -> Result<(), TreeError> {
    if parent.kind != KIND_TEXT || child_kind != KIND_TEXT {
        return Ok(());
    }
    if parent_is_nested_text {
        return Err(TreeError::InvalidChild {
            node_id: parent.id,
            child_id,
            reason: "nested Text may contain only RawText children",
        });
    }
    if child_selectable {
        return Err(TreeError::InvalidRichTextSelection { node_id: child_id });
    }
    validate_nested_text_style(child_id, child_style)
}

pub(super) fn validate_nested_text_style(
    node_id: u32,
    style: Option<&Style>,
) -> Result<(), TreeError> {
    let Some(style) = style else {
        return Ok(());
    };
    macro_rules! unsupported {
        ($field:ident, $name:literal) => {
            if style.$field.is_some() {
                return Err(TreeError::InvalidNestedTextStyle {
                    node_id,
                    field: $name,
                });
            }
        };
    }
    unsupported!(width, "width");
    unsupported!(height, "height");
    unsupported!(flex_direction, "flexDirection");
    unsupported!(padding, "padding");
    unsupported!(gap, "gap");
    unsupported!(justify_content, "justifyContent");
    unsupported!(align_items, "alignItems");
    unsupported!(border_radius, "borderRadius");
    unsupported!(border_width, "borderWidth");
    unsupported!(border_color_rgba, "borderColor");
    unsupported!(font_size, "fontSize");
    unsupported!(background_rgba, "backgroundColor");
    unsupported!(opacity, "opacity");
    unsupported!(transition, "transition");
    unsupported!(overflow, "overflow");
    unsupported!(line_clamp, "lineClamp");
    unsupported!(text_overflow, "textOverflow");
    unsupported!(margin_top, "marginTop");
    unsupported!(margin_right, "marginRight");
    unsupported!(margin_bottom, "marginBottom");
    unsupported!(line_height, "lineHeight");
    unsupported!(min_width, "minWidth");
    unsupported!(max_width, "maxWidth");
    unsupported!(min_height, "minHeight");
    if style
        .position
        .is_some_and(|value| value != PositionCode::Relative)
    {
        return Err(TreeError::InvalidNestedTextStyle {
            node_id,
            field: "position",
        });
    }
    unsupported!(left, "left");
    unsupported!(top, "top");
    unsupported!(right, "right");
    unsupported!(bottom, "bottom");
    unsupported!(cursor, "cursor");
    unsupported!(box_shadows, "boxShadow");
    Ok(())
}
pub(super) fn validate_parent_child_kinds(
    parent_id: u32,
    parent_kind: u32,
    child_id: u32,
    child_kind: u32,
) -> Result<(), TreeError> {
    if matches!(parent_kind, KIND_IMAGE | KIND_ICON) {
        return Err(TreeError::InvalidChild {
            node_id: parent_id,
            child_id,
            reason: "Image and Icon cannot contain children",
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
    if parent_kind == KIND_TEXT && !matches!(child_kind, KIND_RAW_TEXT | KIND_TEXT) {
        return Err(TreeError::InvalidChild {
            node_id: parent_id,
            child_id,
            reason: "Text may contain only RawText or one-level Text children",
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

pub(super) fn collect_text_content(
    node_id: u32,
    nodes: &HashMap<u32, StoredNode>,
    children: &HashMap<u32, Vec<u32>>,
) -> Result<String, TreeError> {
    let mut content = String::new();
    for child_id in children.get(&node_id).cloned().unwrap_or_default() {
        let child = nodes.get(&child_id).ok_or(TreeError::MissingParent {
            node_id: child_id,
            parent_id: node_id,
        })?;
        match child.kind {
            KIND_RAW_TEXT => {
                let Some(text) = child.text.as_deref() else {
                    return Err(TreeError::InvalidChild {
                        node_id,
                        child_id,
                        reason: "Text child must carry text",
                    });
                };
                content.push_str(text);
            }
            KIND_TEXT => content.push_str(&collect_text_content(child_id, nodes, children)?),
            _ => {
                return Err(TreeError::InvalidChild {
                    node_id,
                    child_id,
                    reason: "Text may contain only RawText or one-level Text children",
                });
            }
        }
    }
    Ok(content)
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
        let content = collect_text_content(id, nodes, children)?;
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
    if style.position == Some(PositionCode::Overlay)
        && (style.right.is_some() || style.bottom.is_some())
    {
        return Err(TreeError::InvalidStyle {
            node_id,
            reason: "overlay position supports left and top offsets only",
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
    if style.box_shadows.as_ref().is_some_and(|shadows| {
        shadows.is_empty()
            || shadows.len() > 2
            || shadows.iter().any(|shadow| {
                !shadow.offset_x.is_finite()
                    || !shadow.offset_y.is_finite()
                    || !shadow.blur_radius.is_finite()
                    || shadow.blur_radius < 0.0
                    || !shadow.spread_radius.is_finite()
                    || shadow.spread_radius < 0.0
            })
    }) {
        return Err(TreeError::InvalidStyle {
            node_id,
            reason: "boxShadow must contain one or two shadows with finite offsets and non-negative blur/spread",
        });
    }
    if style.font_family.as_ref().is_some_and(|family| {
        family.is_empty() || family.chars().count() > 64 || family.chars().any(char::is_control)
    }) {
        return Err(TreeError::InvalidStyle {
            node_id,
            reason: "fontFamily must be a non-empty string of at most 64 characters",
        });
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

// Snapshot, Create and Update share the same capability contract.
pub(super) fn supports_listener(kind: u32, style: Option<&Style>) -> bool {
    matches!(
        kind,
        KIND_PRESSABLE
            | KIND_TEXT_INPUT
            | KIND_VIRTUAL_LIST
            | KIND_VIEW
            | KIND_TEXT
            | KIND_IMAGE
            | KIND_EXTENSION
            | KIND_ICON
    ) || style.and_then(|style| style.transition.as_ref()).is_some()
}
