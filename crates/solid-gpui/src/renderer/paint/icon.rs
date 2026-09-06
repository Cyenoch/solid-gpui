use gpui::{AnyElement, Element, Entity, InteractiveElement, ParentElement, Styled, px, rgba};
use gpui_iconify::{IconId, IconView};

use crate::protocol::{HostProperties, Style};
use crate::tree::StoredNode;

use super::super::SolidRoot;
use super::accessibility::apply_accessibility;
use super::style::apply_style;

pub(super) fn render(
    node: &StoredNode,
    entity: &Entity<SolidRoot>,
    style: Option<&Style>,
) -> AnyElement {
    let Some(HostProperties::Icon(icon)) = node.host_properties.as_ref() else {
        return gpui::div()
            .id(gpui::ElementId::Integer(node.id as u64))
            .into_any();
    };
    let id = IconId::from_name(&icon.name).expect("validated Icon name");
    let mut icon_view = IconView::new(id).with_size(px(icon.size));
    if let Some(color) = icon
        .color_rgba
        .or_else(|| style.and_then(|style| style.color_rgba))
    {
        icon_view = icon_view.text_color(rgba(color));
    }
    let element = gpui::div()
        .id(gpui::ElementId::Integer(node.id as u64))
        .flex_none()
        .child(icon_view);
    let element = apply_style(element, style);
    super::measure_node(node, apply_accessibility(element, node).into_any(), entity)
}
