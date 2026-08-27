use std::rc::Rc;

use gpui::{
    AnyElement, Element, ElementId, Entity, InteractiveElement, ParentElement, Styled, div, list,
    px,
};

use crate::protocol::{HostProperties, Style};
use crate::tree::StoredNode;

use super::super::ReactRoot;
use super::super::committed_child_index;
use super::accessibility::apply_accessibility;
use super::style::apply_style;

pub(super) fn render(
    root: &ReactRoot,
    node: &StoredNode,
    entity: &Entity<ReactRoot>,
    style: Option<&Style>,
) -> AnyElement {
    let Some(HostProperties::VirtualList(list_properties)) = node.host_properties.as_ref() else {
        return div().id(ElementId::Integer(node.id as u64)).into_any();
    };
    let state = root
        .virtual_lists
        .get(&node.id)
        .cloned()
        .expect("VirtualList state is reconciled before render");
    let list_id = node.id;
    let item_count = list_properties.item_count;
    let committed_start = list_properties.range_start;
    let committed_end = list_properties.range_end;
    let estimated = px(list_properties.estimated_item_size);
    let overscan = list_properties.overscan;
    let pending = Rc::clone(&root.pending_visible_ranges);
    let list_entity = entity.clone();
    let scroll_entity = entity.downgrade();
    state.set_scroll_handler(move |event, _, app| {
        let start = (event.visible_range.start as u32).saturating_sub(overscan);
        let end = (event.visible_range.end as u32)
            .saturating_add(overscan)
            .min(item_count);
        if let Some(entity) = scroll_entity.upgrade() {
            entity.update(app, |root, _| root.emit_visible_range(list_id, start, end));
        }
    });
    let mut list_element = list(state, move |absolute_index, window, app| {
        let should_schedule = {
            let mut pending = pending.borrow_mut();
            let should_schedule = !pending.contains_key(&list_id);
            let index = absolute_index as u32;
            let range = pending.entry(list_id).or_insert((index, index + 1));
            range.0 = range.0.min(index);
            range.1 = range.1.max(index + 1);
            should_schedule
        };
        if should_schedule {
            let pending_for_frame = Rc::clone(&pending);
            let entity_for_frame = list_entity.clone();
            window.on_next_frame(move |_, app| {
                let range = pending_for_frame.borrow_mut().remove(&list_id);
                if let Some((start, end)) = range {
                    entity_for_frame
                        .update(app, |root, _| root.emit_visible_range(list_id, start, end));
                }
            });
        }

        let row_id = ((list_id as u64) << 32) | absolute_index as u64;
        let child = list_entity.update(app, |root, _| {
            committed_child_index(absolute_index as u32, committed_start, committed_end)
                .and_then(|offset| root.store.get_child_at(list_id, offset))
                .cloned()
                .map(|child| {
                    div()
                        .id(ElementId::Integer(row_id))
                        .w_full()
                        .child(root.render_node(&child, &list_entity))
                        .into_any()
                })
        });
        child.unwrap_or_else(|| {
            div()
                .id(ElementId::Integer(row_id))
                .w_full()
                .h(estimated)
                .into_any()
        })
    });
    list_element = apply_style(list_element, style);
    // List registers its bubble listener while painting, after this boundary has
    // registered its listener. Bubble dispatch therefore lets ListState consume
    // the wheel first and then stops it from reaching an outer scroll container.
    let list_boundary = div()
        .id(ElementId::Integer(((node.id as u64) << 32) | u64::MAX))
        .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
        .child(list_element);
    if node.accessibility.is_none() {
        return super::measure_node(node, list_boundary.into_any(), entity);
    }
    super::measure_node(
        node,
        apply_accessibility(list_boundary, node).into_any(),
        entity,
    )
}
