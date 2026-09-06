use std::path::PathBuf;

use gpui::{
    AnyElement, Element, Entity, ImageSource, InteractiveElement, ObjectFit, StyledImage, img,
};

use crate::protocol::{HostProperties, Style};
use crate::tree::StoredNode;

use super::super::SolidRoot;

use super::accessibility::apply_accessibility;
use super::style::apply_style;

fn object_fit_from_code(code: u32) -> ObjectFit {
    match code {
        1 => ObjectFit::Fill,
        2 => ObjectFit::Contain,
        3 => ObjectFit::Cover,
        4 => ObjectFit::ScaleDown,
        5 => ObjectFit::None,
        _ => unreachable!("validated image objectFit"),
    }
}

pub(super) fn render(
    node: &StoredNode,
    entity: &Entity<SolidRoot>,
    style: Option<&Style>,
) -> AnyElement {
    let Some(HostProperties::Image(image)) = node.host_properties.as_ref() else {
        return gpui::div()
            .id(gpui::ElementId::Integer(node.id as u64))
            .into_any();
    };
    let object_fit_code = image.object_fit;
    let image_id = gpui::ElementId::named_usize("solid-gpui-image", node.id as usize);
    let mut image_element = img(ImageSource::from(PathBuf::from(&image.source)))
        .id(image_id)
        .object_fit(object_fit_from_code(object_fit_code));
    if let Some(fallback_source) = image.fallback_source.as_ref() {
        let fallback_path = PathBuf::from(fallback_source);
        let loading_path = fallback_path.clone();
        let loading_id = gpui::ElementId::named_usize("solid-gpui-image-loading", node.id as usize);
        let fallback_id =
            gpui::ElementId::named_usize("solid-gpui-image-fallback", node.id as usize);
        image_element = image_element.with_loading(move || {
            img(ImageSource::from(loading_path.clone()))
                .id(loading_id.clone())
                .object_fit(object_fit_from_code(object_fit_code))
                .into_any()
        });
        image_element = image_element.with_fallback(move || {
            img(ImageSource::from(fallback_path.clone()))
                .id(fallback_id.clone())
                .object_fit(object_fit_from_code(object_fit_code))
                .into_any()
        });
    }
    let image_element = apply_style(image_element, style);
    super::measure_node(
        node,
        apply_accessibility(image_element, node).into_any(),
        entity,
    )
}
