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
        7 => Some(gpui::accesskit::Role::Link),
        8 => Some(gpui::accesskit::Role::Status),
        9 => Some(gpui::accesskit::Role::Alert),
        10 => Some(gpui::accesskit::Role::Group),
        11 => Some(gpui::accesskit::Role::List),
        12 => Some(gpui::accesskit::Role::ListItem),
        13 => Some(gpui::accesskit::Role::Dialog),
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
    if let Some(expanded) = accessibility.expanded {
        element = element.aria_expanded(expanded);
    }
    if let Some(level) = accessibility.level {
        element = element.aria_level(level as usize);
    }
    let disabled = accessibility.disabled;
    let live = accessibility.live;
    // Mutate the same AX node after prepaint. No extra layout/semantic container
    // is introduced, so native focus, actions and child identity stay intact.
    element.a11y_synthetic_children(move |builder| {
        apply_native_state(builder.parent_node(), disabled, live);
    })
}

fn apply_native_state(node: &mut gpui::accesskit::Node, disabled: bool, live: Option<u32>) {
    if disabled {
        node.set_disabled();
    } else {
        node.clear_disabled();
    }
    if let Some(live) = live {
        node.set_live(match live {
            1 => gpui::accesskit::Live::Polite,
            2 => gpui::accesskit::Live::Assertive,
            _ => gpui::accesskit::Live::Off,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_state_updates_preserve_identity_semantics_and_announcement_value() {
        let mut node = gpui::accesskit::Node::new(gpui::accesskit::Role::Status);
        node.set_label("Build status");
        node.set_value("Build complete");
        node.set_children(vec![gpui::accesskit::NodeId(7)]);
        apply_native_state(&mut node, true, Some(1));
        assert!(node.is_disabled());
        assert_eq!(node.live(), Some(gpui::accesskit::Live::Polite));
        apply_native_state(&mut node, false, Some(0));
        assert!(!node.is_disabled());
        assert_eq!(node.live(), Some(gpui::accesskit::Live::Off));
        assert_eq!(node.role(), gpui::accesskit::Role::Status);
        assert_eq!(node.value(), Some("Build complete"));
        assert_eq!(node.children(), &[gpui::accesskit::NodeId(7)]);
    }
}
