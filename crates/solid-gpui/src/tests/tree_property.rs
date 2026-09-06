use super::support::*;
use crate::protocol::UPDATE_SELECTABLE;
use crate::renderer::SolidRoot;
use gpui::AppContext as _;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

const PROPERTY_SEEDS: usize = 32;
const PROPERTY_STEPS: usize = 64;

#[derive(Clone, Copy)]
struct Rng {
    state: u64,
}

impl Rng {
    fn new(seed: u64) -> Self {
        Self {
            state: seed.wrapping_add(0x9e37_79b9_7f4a_7c15),
        }
    }

    fn next_u64(&mut self) -> u64 {
        let mut value = self.state;
        value ^= value >> 12;
        value ^= value << 25;
        value ^= value >> 27;
        self.state = value;
        value.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }

    fn below(&mut self, upper: usize) -> usize {
        debug_assert!(upper > 0);
        (self.next_u64() % upper as u64) as usize
    }
}

fn append_node(
    nodes: &mut Vec<Node>,
    next_indexes: &mut HashMap<u32, u32>,
    id: u32,
    parent_id: u32,
    kind: u32,
) -> usize {
    let index = next_indexes.entry(parent_id).or_default();
    let node = Node::new(id, parent_id, *index, kind);
    *index += 1;
    nodes.push(node);
    nodes.len() - 1
}

fn controlled_input(value: &str) -> TextInputProperties {
    TextInputProperties {
        value: value.to_owned(),
        placeholder: None,
        multiline: false,
        disabled: false,
        controlled: true,
        ack_edit_seq: 0,
        selection_start: 0,
        selection_end: 0,
        marked_start: None,
        marked_end: None,
        max_length: None,
        selection_reversed: false,
    }
}

fn medium_snapshot() -> Snapshot {
    let mut nodes = vec![Node::new(1, 0, 0, KIND_VIEW)];
    let mut next_indexes = HashMap::from([(1, 0)]);

    let paragraph_a = append_node(&mut nodes, &mut next_indexes, 2, 1, KIND_TEXT);
    nodes[paragraph_a].style = Some(Style {
        font_size: Some(14.0),
        ..Style::default()
    });
    let raw_a = append_node(&mut nodes, &mut next_indexes, 3, 2, KIND_RAW_TEXT);
    nodes[raw_a].text = Some("prefix ".to_owned());
    let run_a = append_node(&mut nodes, &mut next_indexes, 4, 2, KIND_TEXT);
    nodes[run_a].listener_id = 4;
    nodes[run_a].focusable = true;
    nodes[run_a].style = Some(Style {
        color_rgba: Some(0x3366ccff),
        ..Style::default()
    });
    let run_raw_a = append_node(&mut nodes, &mut next_indexes, 5, 4, KIND_RAW_TEXT);
    nodes[run_raw_a].text = Some("link".to_owned());

    let paragraph_b = append_node(&mut nodes, &mut next_indexes, 6, 1, KIND_TEXT);
    nodes[paragraph_b].selectable = true;
    let raw_b = append_node(&mut nodes, &mut next_indexes, 7, 6, KIND_RAW_TEXT);
    nodes[raw_b].text = Some("untouched paragraph".to_owned());

    let branch = append_node(&mut nodes, &mut next_indexes, 8, 1, KIND_VIEW);
    nodes[branch].style = Some(Style {
        opacity: Some(0.8),
        transition: Some(Transition {
            duration_ms: 80,
            delay_ms: 0,
            easing: Easing::Linear,
            properties: TRANSITION_OPACITY,
        }),
        ..Style::default()
    });
    append_node(&mut nodes, &mut next_indexes, 9, 8, KIND_VIEW);
    append_node(&mut nodes, &mut next_indexes, 10, 8, KIND_PRESSABLE);
    nodes.last_mut().expect("pressable inserted").listener_id = 10;
    append_node(&mut nodes, &mut next_indexes, 11, 9, KIND_VIEW);
    append_node(&mut nodes, &mut next_indexes, 12, 9, KIND_VIEW);

    let input = append_node(&mut nodes, &mut next_indexes, 13, 1, KIND_TEXT_INPUT);
    nodes[input].listener_id = 13;
    nodes[input].host_properties = Some(HostProperties::TextInput(controlled_input("input")));
    let list = append_node(&mut nodes, &mut next_indexes, 14, 1, KIND_VIRTUAL_LIST);
    nodes[list].listener_id = 14;
    nodes[list].host_properties = Some(HostProperties::VirtualList(VirtualListProperties {
        item_count: 32,
        range_start: 0,
        range_end: 4,
        estimated_item_size: 24.0,
        overscan: 2,
    }));
    let image = append_node(&mut nodes, &mut next_indexes, 15, 1, KIND_IMAGE);
    nodes[image].host_properties = Some(HostProperties::Image(ImageProperties {
        source: "/tmp/property-test-image.png".to_owned(),
        object_fit: 1,
        fallback_source: None,
    }));
    let pressable = append_node(&mut nodes, &mut next_indexes, 16, 1, KIND_PRESSABLE);
    nodes[pressable].listener_id = 16;
    nodes[pressable].focusable = true;
    let _paragraph_c = append_node(&mut nodes, &mut next_indexes, 17, 9, KIND_TEXT);

    let raw_c = append_node(&mut nodes, &mut next_indexes, 18, 17, KIND_RAW_TEXT);
    nodes[raw_c].text = Some("branch text".to_owned());
    append_node(&mut nodes, &mut next_indexes, 19, 11, KIND_VIEW);
    append_node(&mut nodes, &mut next_indexes, 20, 11, KIND_VIEW);
    append_node(&mut nodes, &mut next_indexes, 21, 12, KIND_VIEW);
    append_node(&mut nodes, &mut next_indexes, 22, 12, KIND_VIEW);
    append_node(&mut nodes, &mut next_indexes, 23, 20, KIND_VIEW);
    append_node(&mut nodes, &mut next_indexes, 24, 20, KIND_VIEW);

    Snapshot::new(7, 3, 0, 1, nodes)
}

fn nested_text_style() -> Style {
    Style {
        color_rgba: Some(0x8855ccff),
        font_weight: Some(FontWeightCode::Medium),
        ..Style::default()
    }
}
fn valid_style(_node: &StoredNode) -> Style {
    Style {
        width: Some(12.0),
        opacity: Some(0.75),
        ..Style::default()
    }
}

fn node_is_nested_text(store: &NodeStore, node: &StoredNode) -> bool {
    node.kind == KIND_TEXT
        && store
            .get(node.parent_id)
            .is_some_and(|parent| parent.kind == KIND_TEXT)
}

fn createable_parent(node: &StoredNode) -> bool {
    matches!(node.kind, KIND_VIEW | KIND_PRESSABLE | KIND_TEXT)
}

fn allowed_child_kinds(store: &NodeStore, parent: &StoredNode) -> &'static [u32] {
    if parent.kind == KIND_TEXT {
        if store
            .get(parent.parent_id)
            .is_some_and(|ancestor| ancestor.kind == KIND_TEXT)
        {
            &[KIND_RAW_TEXT]
        } else {
            &[KIND_RAW_TEXT, KIND_TEXT]
        }
    } else {
        &[KIND_VIEW, KIND_PRESSABLE, KIND_TEXT, KIND_IMAGE]
    }
}

fn generated_node(id: u32, parent_id: u32, index: u32, kind: u32) -> Node {
    let mut node = Node::new(id, parent_id, index, kind);
    match kind {
        KIND_PRESSABLE => {
            node.listener_id = id;
            node.focusable = false;
        }
        KIND_RAW_TEXT => {
            node.text = Some(format!("generated-{id}"));
        }
        KIND_TEXT => {
            node.style = Some(nested_text_style());
        }
        KIND_IMAGE => {
            node.host_properties = Some(HostProperties::Image(ImageProperties {
                source: format!("/tmp/property-test-{id}.png"),
                object_fit: 1,
                fallback_source: None,
            }));
        }
        _ => {}
    }
    node
}

fn nested_style_compatible(style: Option<&Style>) -> bool {
    let Some(style) = style else {
        return true;
    };
    style.width.is_none()
        && style.height.is_none()
        && style.flex_direction.is_none()
        && style.flex_grow.is_none()
        && style.padding.is_none()
        && style.gap.is_none()
        && style.justify_content.is_none()
        && style.align_items.is_none()
        && style.border_radius.is_none()
        && style.border_width.is_none()
        && style.border_color_rgba.is_none()
        && style.font_size.is_none()
        && style.background_rgba.is_none()
        && style.opacity.is_none()
        && style.transition.is_none()
        && style.overflow.is_none()
        && style.line_clamp.is_none()
        && style.text_overflow.is_none()
        && style.margin_top.is_none()
        && style.margin_right.is_none()
        && style.margin_bottom.is_none()
        && style.margin_left.is_none()
        && style.line_height.is_none()
        && style.min_width.is_none()
        && style.max_width.is_none()
        && style.min_height.is_none()
        && style
            .position
            .is_none_or(|value| value == PositionCode::Relative)
        && style.left.is_none()
        && style.top.is_none()
        && style.right.is_none()
        && style.bottom.is_none()
        && style.cursor.is_none()
        && style.box_shadows.is_none()
}

fn can_move_to(store: &NodeStore, id: u32, parent_id: u32) -> bool {
    let Some(node) = store.get(id) else {
        return false;
    };
    let Some(parent) = store.get(parent_id) else {
        return false;
    };
    if !createable_parent(parent) {
        return false;
    }
    if !allowed_child_kinds(store, parent).contains(&node.kind) {
        return false;
    }
    if node.kind == KIND_TEXT && node.selectable && parent.kind == KIND_TEXT {
        return false;
    }
    if parent.kind == KIND_TEXT && parent.selectable && node.kind == KIND_TEXT {
        return false;
    }
    if parent.kind == KIND_TEXT && node.kind == KIND_TEXT {
        if !nested_style_compatible(node.style.as_ref()) {
            return false;
        }
        let mut stack = store.children.get(&node.id).cloned().unwrap_or_default();
        while let Some(descendant_id) = stack.pop() {
            let Some(descendant) = store.get(descendant_id) else {
                return false;
            };
            if descendant.kind == KIND_TEXT {
                return false;
            }
            stack.extend(
                store
                    .children
                    .get(&descendant.id)
                    .cloned()
                    .unwrap_or_default(),
            );
        }
    }
    let mut ancestor = parent_id;
    while let Some(current) = store.get(ancestor) {
        if current.id == id {
            return false;
        }
        if current.parent_id == 0 {
            break;
        }
        ancestor = current.parent_id;
    }
    true
}

fn update_for(store: &NodeStore, rng: &mut Rng) -> Option<PatchOperation> {
    let mut nodes: Vec<StoredNode> = store.iter().cloned().collect();
    nodes.sort_unstable_by_key(|node| node.id);
    if nodes.is_empty() {
        return None;
    }
    let node = &nodes[rng.below(nodes.len())];
    let options = match node.kind {
        KIND_RAW_TEXT => vec![UPDATE_TEXT, UPDATE_STYLE, UPDATE_ACCESSIBILITY],
        KIND_TEXT => vec![UPDATE_STYLE, UPDATE_SELECTABLE, UPDATE_ACCESSIBILITY],
        KIND_VIEW | KIND_PRESSABLE => vec![
            UPDATE_STYLE,
            UPDATE_LISTENER,
            UPDATE_FOCUSABLE,
            UPDATE_TOOLTIP,
            UPDATE_POINTER_MOVE,
            UPDATE_ACCESSIBILITY,
        ],
        KIND_TEXT_INPUT | KIND_VIRTUAL_LIST | KIND_IMAGE => {
            vec![
                UPDATE_STYLE,
                UPDATE_LISTENER,
                UPDATE_PROPERTIES,
                UPDATE_ACCESSIBILITY,
            ]
        }
        _ => vec![UPDATE_STYLE, UPDATE_ACCESSIBILITY],
    };
    let mask = options[rng.below(options.len())];
    let style = if mask == UPDATE_STYLE {
        Some(if node_is_nested_text(store, node) {
            nested_text_style()
        } else {
            valid_style(node)
        })
    } else {
        None
    };
    let text =
        (mask == UPDATE_TEXT).then(|| format!("updated-{}-{}", node.id, rng.next_u64() % 100));
    let host_properties = if mask == UPDATE_PROPERTIES {
        store
            .get(node.id)
            .and_then(|node| node.host_properties.clone())
            .or_else(|| match node.kind {
                KIND_TEXT_INPUT => Some(HostProperties::TextInput(controlled_input("updated"))),
                KIND_VIRTUAL_LIST => Some(HostProperties::VirtualList(VirtualListProperties {
                    item_count: 32,
                    range_start: 0,
                    range_end: 4,
                    estimated_item_size: 24.0,
                    overscan: 2,
                })),
                KIND_IMAGE => Some(HostProperties::Image(ImageProperties {
                    source: format!("/tmp/property-test-{}.png", node.id),
                    object_fit: 1,
                    fallback_source: None,
                })),
                _ => None,
            })
    } else {
        None
    };
    let listener_id = if mask == UPDATE_LISTENER {
        node.listener_id
    } else {
        0
    };
    let tooltip = None;
    let accepts_pointer_move = false;
    let selectable = false;
    let accessibility = (mask == UPDATE_ACCESSIBILITY).then(|| AccessibilityProperties {
        role: 1,
        label: Some(format!("node {}", node.id)),
        description: None,
        disabled: false,
        checked: None,
        selected: None,
        value: None,
        expanded: None,
        level: None,
    });
    Some(PatchOperation::Update {
        id: node.id,
        mask,
        style,
        text,
        listener_id,
        host_properties,
        accessibility,
        focusable: false,
        selectable,
        tooltip,
        accepts_pointer_move,
    })
}

fn next_operation(store: &NodeStore, rng: &mut Rng, next_id: &mut u32) -> PatchOperation {
    let roll = rng.below(100);
    if roll < 55 {
        if let Some(update) = update_for(store, rng) {
            return update;
        }
    } else if roll < 75 {
        let mut parents: Vec<StoredNode> = store
            .iter()
            .filter(|node| createable_parent(node))
            .cloned()
            .collect();
        parents.sort_unstable_by_key(|node| node.id);
        if !parents.is_empty() {
            let parent = &parents[rng.below(parents.len())];
            let kinds = allowed_child_kinds(store, parent);
            let kind = kinds[rng.below(kinds.len())];
            let child_count = store.children.get(&parent.id).map_or(0, Vec::len);
            let index = rng.below(child_count + 1) as u32;
            let node = generated_node(*next_id, parent.id, index, kind);
            *next_id += 1;
            return PatchOperation::Create(node);
        }
    } else if roll < 90 {
        let mut nodes: Vec<StoredNode> =
            store.iter().filter(|node| node.id != 1).cloned().collect();
        nodes.sort_unstable_by_key(|node| node.id);
        if !nodes.is_empty() {
            return PatchOperation::Delete {
                id: nodes[rng.below(nodes.len())].id,
            };
        }
    } else {
        let mut nodes: Vec<StoredNode> =
            store.iter().filter(|node| node.id != 1).cloned().collect();
        nodes.sort_unstable_by_key(|node| node.id);
        let mut parents: Vec<StoredNode> = store.iter().cloned().collect();
        parents.sort_unstable_by_key(|node| node.id);
        if !nodes.is_empty() {
            for _ in 0..parents.len().max(1) {
                let node = &nodes[rng.below(nodes.len())];
                let parent = &parents[rng.below(parents.len())];
                if can_move_to(store, node.id, parent.id) {
                    let old_parent = node.parent_id;
                    let old_len = store.children.get(&old_parent).map_or(0, Vec::len);
                    let new_len = if old_parent == parent.id {
                        old_len.saturating_sub(1)
                    } else {
                        store.children.get(&parent.id).map_or(0, Vec::len)
                    };
                    let index = rng.below(new_len + 1) as u32;
                    return PatchOperation::Move {
                        id: node.id,
                        parent_id: parent.id,
                        index,
                    };
                }
            }
        }
    }
    update_for(store, rng).unwrap_or(PatchOperation::Update {
        id: 1,
        mask: UPDATE_STYLE,
        style: Some(Style::default()),
        text: None,
        listener_id: 0,
        host_properties: None,
        accessibility: None,
        focusable: false,
        selectable: false,
        tooltip: None,
        accepts_pointer_move: false,
    })
}

fn text_content(store: &NodeStore, id: u32) -> Result<String, String> {
    let mut result = String::new();
    let child_ids = store.children.get(&id).cloned().unwrap_or_default();
    for child_id in child_ids {
        let child = store
            .get(child_id)
            .ok_or_else(|| format!("child {child_id} missing from parent {id}"))?;
        match child.kind {
            KIND_RAW_TEXT => result.push_str(
                child
                    .text
                    .as_deref()
                    .ok_or_else(|| format!("RawText {child_id} has no text"))?,
            ),
            KIND_TEXT => result.push_str(&text_content(store, child_id)?),
            kind => return Err(format!("Text {id} contains invalid child kind {kind}")),
        }
    }
    Ok(result)
}

fn assert_tree_invariants(store: &NodeStore) -> Result<(), String> {
    let root = store.root().ok_or_else(|| "root is missing".to_owned())?;
    if root.id != 1 || root.parent_id != 0 || root.index != 0 || root.kind != KIND_VIEW {
        return Err(format!("invalid synthetic root: {root:?}"));
    }

    let mut reachable = HashSet::new();
    let mut stack = vec![1];
    while let Some(id) = stack.pop() {
        if !reachable.insert(id) {
            continue;
        }
        stack.extend(store.children.get(&id).cloned().unwrap_or_default());
    }
    if reachable.len() != store.len() {
        let orphans: Vec<u32> = store
            .iter()
            .filter_map(|node| (!reachable.contains(&node.id)).then_some(node.id))
            .collect();
        return Err(format!("unreachable nodes: {orphans:?}"));
    }

    for (parent_id, children) in &store.children {
        if store.get(*parent_id).is_none() {
            return Err(format!("children entry has missing parent {parent_id}"));
        }
        for (expected, child_id) in children.iter().enumerate() {
            let child = store
                .get(*child_id)
                .ok_or_else(|| format!("parent {parent_id} references missing child {child_id}"))?;
            if child.parent_id != *parent_id || child.index != expected as u32 {
                return Err(format!(
                    "sibling invariant failed for parent {parent_id}: child {child_id} has parent/index {}/{} expected {}/{}",
                    child.parent_id, child.index, parent_id, expected
                ));
            }
        }
    }
    for node in store.iter() {
        let has_children = store
            .children
            .get(&node.id)
            .is_some_and(|children| !children.is_empty());
        if node.has_children() != has_children {
            return Err(format!("child length flag is stale for node {}", node.id));
        }
        let mut chain = HashSet::new();
        let mut current = node.id;
        loop {
            if !chain.insert(current) {
                return Err(format!(
                    "parent cycle reaches node {current} from {}",
                    node.id
                ));
            }
            let current_node = store.get(current).ok_or_else(|| {
                format!("parent chain for {} references missing {current}", node.id)
            })?;
            if current_node.parent_id == 0 {
                if current_node.id != 1 {
                    return Err(format!(
                        "parent chain for {} terminates at {}",
                        node.id, current_node.id
                    ));
                }
                break;
            }
            current = current_node.parent_id;
        }

        if node.kind == KIND_RAW_TEXT && node.text.is_none() {
            return Err(format!("RawText {} lost its text", node.id));
        }
        if node.kind != KIND_RAW_TEXT && node.text.is_some() {
            return Err(format!("non-RawText {} carries raw text", node.id));
        }
        let children = store.children.get(&node.id).cloned().unwrap_or_default();
        if matches!(node.kind, KIND_IMAGE | KIND_RAW_TEXT) && !children.is_empty() {
            return Err(format!("leaf node {} has children", node.id));
        }
        if node.kind == KIND_RAW_TEXT
            && !store
                .get(node.parent_id)
                .is_some_and(|parent| parent.kind == KIND_TEXT)
        {
            return Err(format!("RawText {} is not directly under Text", node.id));
        }
        if node.kind == KIND_TEXT {
            let expected = text_content(store, node.id)?;
            if node.text_content.as_deref() != Some(expected.as_str()) {
                return Err(format!(
                    "Text {} content mismatch: stored {:?}, expected {:?}",
                    node.id, node.text_content, expected
                ));
            }
            if let Some(parent) = store.get(node.parent_id)
                && parent.kind == KIND_TEXT
            {
                if store
                    .get(parent.parent_id)
                    .is_some_and(|ancestor| ancestor.kind == KIND_TEXT)
                {
                    return Err(format!("Text {} is nested more than one level", node.id));
                }
                if children.iter().any(|child_id| {
                    store
                        .get(*child_id)
                        .is_some_and(|child| child.kind == KIND_TEXT)
                }) {
                    return Err(format!(
                        "nested Text {} contains a nested Text child",
                        node.id
                    ));
                }
            }
            if children.iter().any(|child_id| {
                store
                    .get(*child_id)
                    .is_some_and(|child| !matches!(child.kind, KIND_RAW_TEXT | KIND_TEXT))
            }) {
                return Err(format!("Text {} contains a non-text child", node.id));
            }
        }
        if node.kind == KIND_IMAGE && !children.is_empty() {
            return Err(format!("Image {} is not a leaf", node.id));
        }
    }
    Ok(())
}

fn side_map<'a>(ids: &'a [(&'static str, HashSet<u32>)], name: &str) -> &'a HashSet<u32> {
    ids.iter()
        .find_map(|(entry_name, values)| (*entry_name == name).then_some(values))
        .expect("renderer audit helper omitted side map")
}

fn assert_renderer_side_maps(root: &SolidRoot) -> Result<(), String> {
    let ids = root.test_side_map_ids();
    let live: HashSet<u32> = root.store().iter().map(|node| node.id).collect();
    for (name, values) in &ids {
        if let Some(id) = values.iter().find(|id| !live.contains(id)) {
            return Err(format!(
                "renderer side map {name} retains deleted node {id}"
            ));
        }
    }
    Ok(())
}

fn cache_ids(root: &SolidRoot) -> HashSet<u32> {
    side_map(&root.test_side_map_ids(), "rich_text_parts_cache").clone()
}

fn ancestors(store: &NodeStore, start: u32, output: &mut HashSet<u32>) {
    let mut current = start;
    while let Some(node) = store.get(current) {
        if !output.insert(node.id) {
            break;
        }
        if node.parent_id == 0 {
            break;
        }
        current = node.parent_id;
    }
}

fn subtree(store: &NodeStore, start: u32, output: &mut HashSet<u32>) {
    let mut stack = vec![start];
    while let Some(id) = stack.pop() {
        if !output.insert(id) {
            continue;
        }
        stack.extend(store.children.get(&id).cloned().unwrap_or_default());
    }
}

fn cache_invalidated(
    pre: &NodeStore,
    post: &NodeStore,
    operation: &PatchOperation,
    id: u32,
) -> bool {
    let mut touched = HashSet::new();
    let mut pre_patch_roots = HashSet::new();
    match operation {
        PatchOperation::Create(node) => {
            touched.insert(node.id);
            touched.insert(node.parent_id);
        }
        PatchOperation::Update { id, .. } => {
            touched.insert(*id);
        }
        PatchOperation::Move { id, parent_id, .. } => {
            touched.insert(*id);
            touched.insert(*parent_id);
            if let Some(node) = pre.get(*id) {
                pre_patch_roots.insert(node.parent_id);
            }
        }
        PatchOperation::Delete { id } => {
            subtree(pre, *id, &mut touched);
            if let Some(node) = pre.get(*id) {
                pre_patch_roots.insert(node.parent_id);
            }
        }
    }
    let mut invalidated = HashSet::new();
    for root_id in pre_patch_roots {
        ancestors(pre, root_id, &mut invalidated);
    }
    for touched_id in touched {
        ancestors(post, touched_id, &mut invalidated);
    }
    post.get(id).is_none() || invalidated.contains(&id)
}

fn state_summary(store: &NodeStore) -> String {
    let mut nodes: Vec<(u32, u32, u32, u32)> = store
        .iter()
        .map(|node| (node.id, node.parent_id, node.index, node.kind))
        .collect();
    nodes.sort_unstable();
    format!("revision={} nodes={nodes:?}", store.revision())
}

fn run_property_sequences(cx: &mut gpui::TestAppContext, seeds: std::ops::Range<u64>) {
    let runtime = InMemoryAdapter::new();
    let window = cx.open_window(gpui::size(gpui::px(480.0), gpui::px(320.0)), {
        let runtime = Arc::clone(&runtime);
        move |_, _| SolidRoot::new(runtime)
    });
    let root = window.root(cx).expect("property test root");
    let snapshot = medium_snapshot();
    let snapshot_payload = snapshot.encode().expect("encode property snapshot");
    root.update(cx, |root, cx| root.apply_payload(&snapshot_payload, cx))
        .expect("apply property snapshot");
    cx.update_window(window.into(), |_, window, cx| {
        window.draw(cx).clear(cx);
    })
    .expect("warm property renderer state");
    cx.run_until_parked();

    let initial_side_maps = root.read_with(cx, |root, _| {
        assert_tree_invariants(root.store()).expect("initial tree invariants");
        assert_renderer_side_maps(root).expect("initial renderer side maps");
        root.test_side_map_ids()
    });
    assert!(
        !side_map(&initial_side_maps, "rich_text_parts_cache").is_empty(),
        "initial draw must warm at least one rich-text cache entry"
    );

    let mut next_id = 1000;
    let mut revision = 1;
    for seed in seeds {
        let mut rng = Rng::new(seed);
        let mut history = Vec::with_capacity(PROPERTY_STEPS);
        let reset_snapshot = Snapshot::new(7, 3, revision, revision + 1, medium_snapshot().nodes);
        let reset_payload = reset_snapshot
            .encode()
            .expect("encode reset property snapshot");
        root.update(cx, |root, cx| root.apply_payload(&reset_payload, cx))
            .expect("reset property snapshot");
        revision += 1;
        cx.update_window(window.into(), |_, window, cx| {
            window.draw(cx).clear(cx);
        })
        .expect("warm reset property renderer state");
        cx.run_until_parked();

        for step in 0..PROPERTY_STEPS {
            let before = root.read_with(cx, |root, _| (root.store().clone(), cache_ids(root)));
            let operation = next_operation(&before.0, &mut rng, &mut next_id);
            history.push(format!("{step}: {operation:?}"));
            let patch = Patch::new(7, 3, revision, revision + 1, vec![operation.clone()]);
            let payload = patch.encode().expect("encode generated property patch");
            let apply = root.update(cx, |root, cx| root.apply_payload(&payload, cx));
            if let Err(error) = apply {
                let state = root.read_with(cx, |root, _| state_summary(root.store()));
                panic!(
                    "seed {seed} step {step} generated an invalid patch: {error:?}\noperations:\n{}\nstate: {state}",
                    history.join("\n")
                );
            }
            revision += 1;
            let result = root.read_with(cx, |root, _| {
                assert_tree_invariants(root.store())?;
                assert_renderer_side_maps(root)?;
                let post_cache = cache_ids(root);
                for cached_id in &before.1 {
                    if !cache_invalidated(&before.0, root.store(), &operation, *cached_id)
                        && !post_cache.contains(cached_id)
                    {
                        return Err(format!(
                            "rich cache entry {cached_id} disappeared despite an untouched subtree"
                        ));
                    }
                }
                Ok::<(), String>(())
            });
            if let Err(error) = result {
                let state = root.read_with(cx, |root, _| state_summary(root.store()));
                panic!(
                    "seed {seed} step {step} violated an invariant: {error}\noperations:\n{}\nstate: {state}",
                    history.join("\n")
                );
            }
        }
    }
}

#[gpui::test]
fn seeded_random_valid_patch_sequences_preserve_tree_and_renderer_invariants(
    cx: &mut gpui::TestAppContext,
) {
    run_property_sequences(cx, 0..PROPERTY_SEEDS as u64);
}

#[gpui::test]
fn seed_zero_property_sequence_regression(cx: &mut gpui::TestAppContext) {
    run_property_sequences(cx, 0..1);
}
