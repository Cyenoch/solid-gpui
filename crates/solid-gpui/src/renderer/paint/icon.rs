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
    let color = icon
        .color_rgba
        .or_else(|| style.and_then(|style| style.color_rgba));
    let icon_view = if let Some(id) = IconId::from_name(&icon.name) {
        let mut view = IconView::new(id).with_size(px(icon.size));
        if let Some(color) = color {
            view = view.text_color(rgba(color));
        }
        gpui::IntoElement::into_any_element(view)
    } else if let Some(image) = crate::icons::image(&icon.name) {
        gpui::IntoElement::into_any_element(gpui::img(image).size(px(icon.size)))
    } else {
        let mut view = gpui::svg()
            .size(px(icon.size))
            .path(format!("application-icons/{}", icon.name));
        if let Some(color) = color {
            view = view.text_color(rgba(color));
        }
        gpui::IntoElement::into_any_element(view)
    };
    let element = gpui::div()
        .id(gpui::ElementId::Integer(node.id as u64))
        .flex_none()
        .child(icon_view);
    let element = apply_style(element, style);
    super::measure_node(node, apply_accessibility(element, node).into_any(), entity)
}
