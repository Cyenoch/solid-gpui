use std::sync::Arc;

use gpui::StatefulInteractiveElement;

use crate::tree::StoredNode;

pub(super) fn accessibility_role(role: u32) -> Option<gpui::accesskit::Role> {
    match role {
        1 => None,
        2 => Some(gpui::accesskit::Role::Button),
        3 => Some(gpui::accesskit::Role::Label),
        4 => Some(gpui::accesskit::Role::TextInput),
        5 => Some(gpui::accesskit::Role::CheckBox),
        6 => Some(gpui::accesskit::Role::Heading),
        _ => None,
    }
}

pub(super) fn apply_accessibility<E: StatefulInteractiveElement>(
    mut element: E,
    node: &StoredNode,
) -> E {
    let Some(accessibility) = node.accessibility.as_ref() else {
        return element;
    };
    let Some(role) = accessibility_role(accessibility.role) else {
        // GenericContainer is intentionally omitted by GPUI's AccessKit tree;
        // unknown roles are rejected for snapshots and ignored for safety here.
        return element;
    };
    element = element.role(role);
    element = element.accessibility_id(Arc::clone(&node.accessibility_id));
    if let Some(label) = &accessibility.label {
        element = element.aria_label(label.clone());
    }
    if let Some(description) = &accessibility.description {
        element = element.aria_description(description.clone());
    }
    if let Some(selected) = accessibility.selected {
        element = element.aria_selected(selected);
    }
    if let Some(checked) = accessibility.checked {
        // AccessKit 0.24 calls the checked/unchecked states True/False.
        element = element.aria_toggled(if checked {
            gpui::accesskit::Toggled::True
        } else {
            gpui::accesskit::Toggled::False
        });
    }
    if let Some(value) = &accessibility.value {
        element = element.aria_value(value.clone());
    }
    // GPUI 0.2.2 exposes no public aria-disabled builder, so the wire field
    // cannot yet be advertised in AX.
    element
}
